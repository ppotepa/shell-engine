use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

pub struct MainChunks {
    pub sidebar_icons: Rect,
    pub sidebar_panel: Option<Rect>, // None when panel hidden
    pub center: Rect,
    pub status: Rect,
}

pub fn main_chunks(frame: &Frame, sidebar_visible: bool) -> MainChunks {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
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
            .split(vertical[0]);

        MainChunks {
            sidebar_icons: horizontal[0],
            sidebar_panel: Some(horizontal[1]),
            center: horizontal[2],
            status: vertical[1],
        }
    } else {
        // Panel hidden: [icons 7ch][center 93%]
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(7),      // Icons only
                Constraint::Percentage(93), // Center takes rest
            ])
            .split(vertical[0]);

        MainChunks {
            sidebar_icons: horizontal[0],
            sidebar_panel: None, // No panel
            center: horizontal[1],
            status: vertical[1],
        }
    }
}
