//! Terminal layout calculation for the main editor panes.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

/// Holds the computed [`Rect`] areas for each main UI region in the current frame.
pub struct MainChunks {
    pub header: Rect,
    pub body: Rect,
    pub sidebar_icons: Rect,
    pub sidebar_panel: Option<Rect>, // None when panel hidden
    pub center: Rect,
    pub status: Rect,
}

/// Computes layout rectangles for the current frame, accounting for sidebar visibility.
pub fn main_chunks(frame: &Frame, sidebar_visible: bool) -> MainChunks {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    if sidebar_visible {
        // Panel visible: [icons 7ch][panel 30%][center 63%]
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(7),      // Icons (wider)
                Constraint::Percentage(30), // Panel
                Constraint::Percentage(63), // Center
            ])
            .split(vertical[1]);

        MainChunks {
            header: vertical[0],
            body: vertical[1],
            sidebar_icons: horizontal[0],
            sidebar_panel: Some(horizontal[1]),
            center: horizontal[2],
            status: vertical[2],
        }
    } else {
        // Panel hidden: [icons 7ch][center 93%]
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(7),      // Icons only
                Constraint::Percentage(93), // Center takes rest
            ])
            .split(vertical[1]);

        MainChunks {
            header: vertical[0],
            body: vertical[1],
            sidebar_icons: horizontal[0],
            sidebar_panel: None, // No panel
            center: horizontal[1],
            status: vertical[2],
        }
    }
}
