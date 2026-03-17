#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    ProjectTree,
    Browser,
    Inspector,
}

impl FocusPane {
    pub fn next(self) -> Self {
        match self {
            FocusPane::ProjectTree => FocusPane::Browser,
            FocusPane::Browser => FocusPane::Inspector,
            FocusPane::Inspector => FocusPane::ProjectTree,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FocusPane::ProjectTree => FocusPane::Inspector,
            FocusPane::Browser => FocusPane::ProjectTree,
            FocusPane::Inspector => FocusPane::Browser,
        }
    }
}
