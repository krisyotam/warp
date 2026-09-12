use super::*;

#[test]
fn startup_pin_roundtrip_preserves_remote_command_and_folder() {
    let json = r#"{"title":"Build","pinned":false,"startup_pin_id":"11111111-1111-4111-8111-111111111111","group":{"id":"22222222-2222-4222-8222-222222222222","name":"FLEET","pinned":true},"layout":{"cwd":"/tmp","commands":[{"exec":"ssh buildbox"}]}}"#;
    let pin: TabTemplate = serde_json::from_str(json).unwrap();
    let restored: TabTemplate = serde_json::from_slice(&serde_json::to_vec(&pin).unwrap()).unwrap();
    assert_eq!(pin, restored);
    assert!(restored.group.unwrap().pinned);
}

#[test]
fn old_launch_configs_remain_unpinned() {
    let pin: TabTemplate = serde_json::from_str(r#"{"layout":{"cwd":"/tmp"}}"#).unwrap();
    assert!(!pin.pinned);
    assert!(pin.group.is_none());
    assert!(pin.startup_pin_id.is_none());
}

fn tab(id: u8) -> crate::app_state::TabSnapshot {
    use crate::app_state::{LeafSnapshot, TerminalPaneSnapshot};
    crate::app_state::TabSnapshot {
        custom_title: Some("Same name".into()),
        root: PaneNodeSnapshot::Leaf(LeafSnapshot {
            is_focused: true,
            custom_vertical_tabs_title: None,
            contents: LeafContents::Terminal(TerminalPaneSnapshot {
                uuid: vec![id],
                cwd: Some("/tmp".into()),
                shell_launch_data: None,
                is_active: true,
                is_read_only: false,
                input_config: None,
                llm_model_override: None,
                active_profile_id: None,
                conversation_ids_to_restore: vec![],
                active_conversation_id: None,
            }),
        }),
        default_directory_color: None,
        selected_color: Default::default(),
        left_panel: None,
        right_panel: None,
        group_id: None,
        pinned: true,
    }
}

fn state() -> AppState {
    AppState {
        windows: vec![WindowSnapshot {
            tabs: vec![tab(1), tab(2)],
            active_tab_index: 1,
            team_uid: None,
            bounds: None,
            fullscreen_state: Default::default(),
            quake_mode: false,
            universal_search_width: None,
            warp_ai_width: None,
            voltron_width: None,
            warp_drive_index_width: None,
            left_panel_open: false,
            vertical_tabs_panel_open: true,
            left_panel_width: None,
            right_panel_width: None,
            agent_management_filters: None,
            tab_groups: vec![],
        }],
        active_window_index: Some(0),
        block_lists: Default::default(),
        running_mcp_servers: vec![],
    }
}

#[test]
fn startup_deduplicates_by_terminal_identity_not_title() {
    let mut state = state();
    let mut pin = TabTemplate::try_from(tab(1)).unwrap();
    pin.restored_pane_id = Some(vec![1]);
    remove_restored_pins(&mut state, &[pin]);
    assert_eq!(state.windows[0].tabs, vec![tab(2)]);
    assert_eq!(state.windows[0].active_tab_index, 0);
    assert_eq!(state.active_window_index, Some(0));
}

#[test]
fn removing_all_startup_pins_does_not_restore_an_empty_window() {
    let mut state = state();
    let pins: Vec<_> = [1, 2]
        .into_iter()
        .map(|id| {
            let mut pin = TabTemplate::try_from(tab(id)).unwrap();
            pin.restored_pane_id = Some(vec![id]);
            pin
        })
        .collect();
    remove_restored_pins(&mut state, &pins);
    assert!(state.windows.is_empty());
    assert_eq!(state.active_window_index, None);
}

#[test]
fn reopening_closed_members_keeps_each_folder_contiguous() {
    let mut first: TabTemplate = serde_json::from_str(r#"{"title":"First","group":{"id":"22222222-2222-4222-8222-222222222222","name":"FLEET","pinned":true},"layout":{"cwd":"/tmp"}}"#).unwrap();
    let mut last = first.clone();
    last.title = Some("Last".into());
    let mut other = first.clone();
    other.title = Some("Other".into());
    other.group = None;
    first.pinned = true;
    let mut pins = vec![first, other, last];
    keep_group_members_together(&mut pins);
    assert_eq!(
        pins.iter()
            .map(|pin| pin.title.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["First", "Last", "Other"]
    );
}
