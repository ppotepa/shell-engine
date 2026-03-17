//! Filter/search state for navigable list panels.

/// Holds the active search query string for filterable list views.
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct FilterState {
    pub query: String,
}
