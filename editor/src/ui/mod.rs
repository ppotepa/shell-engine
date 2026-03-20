//! Top-level UI rendering: dispatches frame drawing to the appropriate component tree.

pub mod components;
pub mod icons;
pub mod layout;
pub mod theme;

use ratatui::Frame;

use crate::state::{AppMode, AppState, SidebarItem};

/// Renders the complete application frame based on the current [`AppState`].
pub fn draw(frame: &mut Frame, app: &AppState) {
    if app.mode == AppMode::SceneRun {
        components::scene_run::render(frame, frame.area(), app);
        return;
    }

    let sidebar_visible =
        matches!(app.mode, AppMode::Browser | AppMode::EditMode) && app.sidebar.visible;
    let chunks = layout::main_chunks(frame, sidebar_visible);
    components::header::render(frame, chunks.header, app);

    if app.mode == AppMode::Start {
        components::start_screen::render(frame, chunks.body, app);
        components::status_bar::render(frame, chunks.status, app);
        if app.help_overlay_active {
            components::help::render(frame, chunks.body, app);
        }
        return;
    }

    if app.mode == AppMode::Browser
        && app.sidebar.active == SidebarItem::Scenes
        && app.scene_preview_fullscreen_active()
    {
        components::scenes_preview::render_fullscreen(frame, chunks.body, app);
        components::status_bar::render(frame, chunks.status, app);
        if app.help_overlay_active {
            components::help::render(frame, chunks.body, app);
        }
        return;
    }

    // Always render sidebar icons (in Browser and EditMode)
    components::sidebar::icons::render(frame, chunks.sidebar_icons, app);

    // EditMode: render editor + sidebar
    if app.mode == AppMode::EditMode {
        if let Some(panel_rect) = chunks.sidebar_panel {
            match app.sidebar.active {
                SidebarItem::Explorer => {
                    components::sidebar::explorer::render(frame, panel_rect, app)
                }
                SidebarItem::Search => components::sidebar::effects::render(frame, panel_rect, app),
                SidebarItem::Scenes => components::sidebar::placeholder::render(
                    frame,
                    panel_rect,
                    app,
                    "Scenes",
                    &[
                        "Scene browser lives in the center panel.",
                        "",
                        "Tip: hide this sidebar with T for a wider scene layout.",
                    ],
                ),
                SidebarItem::Cutscene => components::sidebar::placeholder::render(
                    frame,
                    panel_rect,
                    app,
                    "Cutscene Maker",
                    &[
                        "Stop-action source: assets/raw",
                        "",
                        "Expected naming: 1.png, 2.png, 3.png ...",
                        "Press F5 to rescan source frames.",
                    ],
                ),
            }
        }
        components::editor::render(frame, chunks.center, app);
        components::status_bar::render(frame, chunks.status, app);
        if app.help_overlay_active {
            components::help::render(frame, chunks.body, app);
        }
        return;
    }

    // Browser mode: render explorer panel + preview
    if let Some(panel_rect) = chunks.sidebar_panel {
        match app.sidebar.active {
            SidebarItem::Explorer => components::sidebar::explorer::render(frame, panel_rect, app),
            SidebarItem::Search => components::sidebar::effects::render(frame, panel_rect, app),
            SidebarItem::Scenes => components::sidebar::placeholder::render(
                frame,
                panel_rect,
                app,
                "Scenes",
                &[
                    "Scene browser lives in the center panel.",
                    "",
                    "Tip: hide this sidebar with T for a wider scene layout.",
                ],
            ),
            SidebarItem::Cutscene => components::sidebar::placeholder::render(
                frame,
                panel_rect,
                app,
                "Cutscene Maker",
                &[
                    "Stop-action source: assets/raw",
                    "",
                    "Expected naming: 1.png, 2.png, 3.png ...",
                    "Press F5 to rescan source frames.",
                ],
            ),
        }
    }
    if app.sidebar.active == SidebarItem::Search {
        components::effects_preview::render(frame, chunks.center, app);
    } else if app.sidebar.active == SidebarItem::Scenes {
        components::scenes_preview::render(frame, chunks.center, app);
    } else if app.sidebar.active == SidebarItem::Cutscene {
        components::cutscene_preview::render(frame, chunks.center, app);
    } else {
        components::preview::render(frame, chunks.center, app);
    }
    components::status_bar::render(frame, chunks.status, app);
    if app.help_overlay_active {
        components::help::render(frame, chunks.body, app);
    }
}
