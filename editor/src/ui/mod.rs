//! Top-level UI rendering: dispatches frame drawing to the appropriate component tree.

pub mod components;
pub mod icons;
pub mod layout;
pub mod theme;

use ratatui::Frame;

use crate::state::{AppMode, AppState, SidebarItem};

/// Renders the complete application frame based on the current [`AppState`].
pub fn draw(frame: &mut Frame, app: &AppState) {
    if app.mode == AppMode::Start {
        components::start_screen::render(frame, frame.area(), app);
        return;
    }

    if app.mode == AppMode::Browser
        && app.sidebar_active == SidebarItem::Scenes
        && app.scene_preview_fullscreen_active()
    {
        components::scenes_preview::render_fullscreen(frame, frame.area(), app);
        return;
    }

    let chunks = layout::main_chunks(frame, app.sidebar_visible);

    // Always render sidebar icons (in Browser and EditMode)
    components::sidebar::icons::render(frame, chunks.sidebar_icons, app);

    // EditMode: render editor + sidebar
    if app.mode == AppMode::EditMode {
        if let Some(panel_rect) = chunks.sidebar_panel {
            match app.sidebar_active {
                SidebarItem::Explorer => {
                    components::sidebar::explorer::render(frame, panel_rect, app)
                }
                SidebarItem::Search => components::sidebar::effects::render(frame, panel_rect, app),
                SidebarItem::Scenes => components::sidebar::placeholder::render(
                    frame,
                    panel_rect,
                    "Scenes",
                    &[
                        "Scene browser lives in the center panel.",
                        "",
                        "Tip: hide this sidebar with T for full 50/50 scene layout.",
                    ],
                ),
                SidebarItem::Settings => components::sidebar::placeholder::render(
                    frame,
                    panel_rect,
                    "Settings",
                    &[
                        "Panel in progress",
                        "",
                        "Planned: theme, keybinds, runtime prefs",
                    ],
                ),
            }
        }
        components::editor::render(frame, chunks.center, app);
        components::status_bar::render(frame, chunks.status, app);
        return;
    }

    // Browser mode: render explorer panel + preview
    if let Some(panel_rect) = chunks.sidebar_panel {
        match app.sidebar_active {
            SidebarItem::Explorer => components::sidebar::explorer::render(frame, panel_rect, app),
            SidebarItem::Search => components::sidebar::effects::render(frame, panel_rect, app),
            SidebarItem::Scenes => components::sidebar::placeholder::render(
                frame,
                panel_rect,
                "Scenes",
                &[
                    "Scene browser lives in the center panel.",
                    "",
                    "Tip: hide this sidebar with T for full 50/50 scene layout.",
                ],
            ),
            SidebarItem::Settings => components::sidebar::placeholder::render(
                frame,
                panel_rect,
                "Settings",
                &[
                    "Panel in progress",
                    "",
                    "Planned: theme, keybinds, runtime prefs",
                ],
            ),
        }
    }
    if app.sidebar_active == SidebarItem::Search {
        components::effects_preview::render(frame, chunks.center, app);
    } else if app.sidebar_active == SidebarItem::Scenes {
        components::scenes_preview::render(frame, chunks.center, app);
    } else {
        components::preview::render(frame, chunks.center, app);
    }
    components::status_bar::render(frame, chunks.status, app);
}
