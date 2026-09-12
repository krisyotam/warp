//! Local startup bookmarks. Closing a pinned tab leaves its bookmark intact;
//! unpinning removes it. Only saved launch commands are replayed at startup.
use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use warpui::AppContext;

use super::launch_config::{PaneTemplateType, TabGroupTemplate, TabTemplate};
use crate::app_state::{AppState, LeafContents, PaneNodeSnapshot, WindowSnapshot};
use crate::workspace::Workspace;

fn path() -> PathBuf {
    warp_core::paths::config_local_dir().join("permanent-pins.json")
}

fn read() -> Result<Vec<TabTemplate>> {
    match std::fs::read(path()) {
        Ok(bytes) => serde_json::from_slice(&bytes).context("Invalid permanent-pins.json"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub fn load() -> Vec<TabTemplate> {
    match read() {
        Ok(mut pins) => {
            keep_group_members_together(&mut pins);
            pins
        }
        Err(error) => {
            log::error!("Could not load permanent startup pins: {error:#}");
            Vec::new()
        }
    }
}

fn keep_group_members_together(pins: &mut [TabTemplate]) {
    let mut positions = HashMap::new();
    for (index, pin) in pins.iter().enumerate() {
        if let Some(group) = &pin.group {
            positions.entry(group.id).or_insert(index);
        }
    }
    let mut indexed: Vec<_> = pins.iter().cloned().enumerate().collect();
    indexed.sort_by_key(|(index, pin)| {
        pin.group
            .as_ref()
            .map_or(*index, |group| positions[&group.id])
    });
    for (pin, (_, sorted)) in pins.iter_mut().zip(indexed) {
        *pin = sorted;
    }
}

fn first_terminal_id(root: &PaneNodeSnapshot) -> Option<Vec<u8>> {
    match root {
        PaneNodeSnapshot::Branch(branch) => branch
            .children
            .iter()
            .find_map(|(_, node)| first_terminal_id(node)),
        PaneNodeSnapshot::Leaf(leaf) => {
            if let LeafContents::Terminal(terminal) = &leaf.contents {
                Some(terminal.uuid.clone())
            } else {
                None
            }
        }
    }
}

pub fn remove_restored_pins(state: &mut AppState, pins: &[TabTemplate]) {
    let ids: HashSet<_> = pins
        .iter()
        .filter_map(|pin| pin.restored_pane_id.as_ref())
        .collect();
    for window in &mut state.windows {
        let active = window.tabs.get(window.active_tab_index).cloned();
        window
            .tabs
            .retain(|tab| !first_terminal_id(&tab.root).is_some_and(|id| ids.contains(&id)));
        window.active_tab_index = active
            .and_then(|active| window.tabs.iter().position(|tab| *tab == active))
            .unwrap_or_default();
        window
            .tab_groups
            .retain(|group| window.tabs.iter().any(|tab| tab.group_id == Some(group.id)));
    }
    let active = state
        .active_window_index
        .and_then(|index| state.windows.get(index))
        .cloned();
    state.windows.retain(|window| !window.tabs.is_empty());
    state.active_window_index =
        active.and_then(|active| state.windows.iter().position(|window| *window == active));
}

pub fn save_workspace(workspace: &Workspace, snapshot: &WindowSnapshot, app: &AppContext) {
    if let Err(error) = save(workspace, snapshot, app) {
        log::error!("Could not save permanent startup pins: {error:#}");
    }
}

fn save(workspace: &Workspace, snapshot: &WindowSnapshot, app: &AppContext) -> Result<()> {
    // Snapshotting skips transient drag placeholders. Never zip a partial snapshot.
    if workspace.tabs.len() != snapshot.tabs.len() {
        return Ok(());
    }
    let mut pins = read()?;
    let before = pins.clone();
    // Renames, colors, collapse and unpin apply to closed members as well.
    for group in &snapshot.tab_groups {
        pins.retain_mut(|pin| {
            if let Some(saved_group) = &mut pin.group
                && saved_group.id == group.id.0
            {
                saved_group.name = group.name.clone();
                saved_group.color = group.color.resolve(None);
                saved_group.collapsed = group.collapsed;
                saved_group.pinned = group.pinned;
                return pin.pinned || group.pinned;
            }
            true
        });
    }
    for (tab, saved) in workspace.tabs.iter().zip(&snapshot.tabs) {
        let group = saved
            .group_id
            .and_then(|id| snapshot.tab_groups.iter().find(|group| group.id == id));
        let pinned = saved.pinned || group.is_some_and(|group| group.pinned);
        let existing = pins
            .iter()
            .position(|pin| pin.startup_pin_id == Some(tab.startup_pin_id));
        if !pinned {
            if let Some(index) = existing {
                pins.remove(index);
            }
            continue;
        }
        let Ok(mut template) = TabTemplate::try_from(saved.clone()) else {
            continue;
        };
        template.startup_pin_id = Some(tab.startup_pin_id);
        template.restored_pane_id = first_terminal_id(&saved.root);
        if let Some(layout) = &tab.startup_layout {
            template.layout = layout.clone();
        } else if let Some(index) = existing {
            // Preserve the reconnect command even if the connection has dropped.
            template.layout = pins[index].layout.clone();
        }
        if let PaneTemplateType::PaneTemplate { cwd, commands, .. } = &mut template.layout {
            // For a manually opened single-terminal SSH tab, remember the connection.
            if let Some(command) = tab
                .pane_group
                .as_ref(app)
                .terminal_views(app)
                .first()
                .and_then(|terminal| terminal.as_ref(app).startup_ssh_command())
            {
                *cwd = dirs::home_dir().unwrap_or_default();
                *commands = vec![command.as_str().into()];
            }
        }
        template.group = group.map(|group| TabGroupTemplate {
            id: group.id.0,
            name: group.name.clone(),
            collapsed: group.collapsed,
            pinned: group.pinned,
            color: group.color.resolve(None),
        });
        if let Some(index) = existing {
            pins[index] = template;
        } else {
            pins.push(template);
        }
    }
    if pins == before {
        return Ok(());
    }
    let target = path();
    std::fs::create_dir_all(target.parent().context("Missing config directory")?)?;
    // Atomic replacement keeps an interrupted write from losing the startup set.
    let temporary = target.with_extension("json.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(&pins)?)?;
    file.sync_all()?;
    std::fs::rename(temporary, target)?;
    Ok(())
}

#[cfg(test)]
#[path = "permanent_pins_tests.rs"]
mod tests;
