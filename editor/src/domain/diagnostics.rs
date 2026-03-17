//! Validation diagnostic messages collected during project indexing.

/// Accumulated warnings produced during project validation and indexing.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub warnings: Vec<String>,
}
