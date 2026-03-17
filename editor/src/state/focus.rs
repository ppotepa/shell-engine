//! Keyboard focus tracking across the three main UI panes.

/// Identifies which of the three main UI panes currently holds keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    ProjectTree,
    Browser,
    Inspector,
}

impl FocusPane {
    /// Returns the next pane in the Tab-cycle order.
    pub fn next(self) -> Self {
        match self {
            FocusPane::ProjectTree => FocusPane::Browser,
            FocusPane::Browser => FocusPane::Inspector,
            FocusPane::Inspector => FocusPane::ProjectTree,
        }
    }

    /// Returns the previous pane in the Tab-cycle order.
    pub fn prev(self) -> Self {
        match self {
            FocusPane::ProjectTree => FocusPane::Inspector,
            FocusPane::Browser => FocusPane::ProjectTree,
            FocusPane::Inspector => FocusPane::Browser,
        }
    }
}
