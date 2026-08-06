# Sidebar workspaces (fork redesign)

Goal: make this fork a **sidebar-first terminal** — Limux information architecture,
Warp agentic power, quieter chrome than the default top tab strip.

## Why

Warp’s horizontal top tabs waste vertical space and flatten hierarchy. Limux gets
the model right:

```
Sidebar
  Workspace (project / folder / named context)
    Terminal sessions (tabs)
      Multiplex (splits / panes)
Main surface
  Focused session only
```

This fork ships that model as the **default**, not an opt-in experiment.

## Mapping (Limux → Warp)

| Limux concept | Warp primitive (current) | Sidebar presentation |
|---------------|--------------------------|----------------------|
| Workspace     | Tab (session) or tab group | Top-level row / group |
| Terminal tab  | Pane or tab (by display granularity) | Nested under workspace when expanded |
| Split / mux   | PaneGroup splits         | In-tab layout; switch via pane focus |
| Sidebar       | Vertical tabs panel      | Always-on by default |
| Top bar       | Horizontal tab bar       | Hidden when vertical tabs enabled |

## Defaults (this fork)

| Setting | Value | Why |
|---------|-------|-----|
| `appearance.vertical_tabs.enabled` | `true` | Sidebar, not top strip |
| `appearance.vertical_tabs.show_panel_in_restored_windows` | `true` | Session restore keeps navigator |
| `appearance.vertical_tabs.hide_title_bar_search_bar` | `true` | Search lives in the sidebar |
| `appearance.vertical_tabs.view_mode` | `Expanded` | Room for cwd / command / agent metadata |
| `appearance.vertical_tabs.display_granularity` | `Tabs` | One row per session; panes stay as multiplex |

Cargo features `vertical_tabs` and `grouped_tabs` remain in the app default feature set.

## Visual direction

- Wider default panel, softer 8px radii, more breathing room than stock Warp vertical tabs
- Explicit **Workspaces** section label + pill search field
- Right/left edge hairline so the sidebar reads as chrome, not terminal content
- Empty state copy that teaches: add a workspace, then split/tab to multiplex

## Roadmap (next slices)

1. **True workspace objects** — first-class named workspace entities with folder binding
   (closer to Limux `session.json`), not only tab groups.
2. **Nested terminal rows** — under each workspace, list terminals; click switches;
   `+` adds a terminal to that workspace only.
3. **Auto-group by cwd** — new sessions land under the workspace matching their
   working directory / git root.
4. **Collapse animation** — sidebar width + content fade matching Limux’s motion.
5. **TUI parity** — `warp_tui` orchestration tab bar becomes a left column, not a top row.
6. **Branding** — optional fork product name once the shell feels distinct.

## Non-goals (for now)

- Replacing Ghostty/Limux as a lightweight mux for people who want that stack only
- Removing agent features, Drive, or code panes
- Upstreaming every default flip without discussion (this is a personal fork default)

## How to try

```bash
./script/bootstrap   # once
./script/run         # build + launch GUI
```

If you already have settings synced with vertical tabs off, set in `settings.toml`
or the UI:

```toml
[appearance.vertical_tabs]
enabled = true
show_panel_in_restored_windows = true
hide_title_bar_search_bar = true
view_mode = "Expanded"
display_granularity = "Tabs"
```

Toggle the panel with the left chrome control (tooltip: **Workspaces**).

## Related upstream code

- `app/src/workspace/view/vertical_tabs.rs` — panel chrome + rows
- `app/src/workspace/tab_settings.rs` — defaults
- `app/src/workspace/view.rs` — when the horizontal bar is skipped
- Specs: `specs/APP-3742`, `specs/APP-3825` (vertical tabs product work)
