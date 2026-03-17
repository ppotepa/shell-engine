//! Selection state for navigable list panels.

/// Tracks the selected item index in a navigable list.
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct SelectionState {
    pub index: usize,
}
