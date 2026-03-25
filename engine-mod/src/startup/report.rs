//! Startup report types — [`StartupIssue`] and [`StartupReport`] accumulate diagnostics from each check.

/// Severity level of a [`StartupIssue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupIssueLevel {
    Info,
    Warning,
}

/// A single diagnostic produced by a [`StartupCheck`](super::check::StartupCheck).
#[derive(Debug, Clone)]
pub struct StartupIssue {
    pub check: &'static str,
    pub level: StartupIssueLevel,
    pub message: String,
}

/// Collects [`StartupIssue`]s from all checks run during the startup pipeline.
#[derive(Debug, Default)]
pub struct StartupReport {
    issues: Vec<StartupIssue>,
}

impl StartupReport {
    /// Records an informational diagnostic for `check`.
    pub fn add_info(&mut self, check: &'static str, message: impl Into<String>) {
        self.issues.push(StartupIssue {
            check,
            level: StartupIssueLevel::Info,
            message: message.into(),
        });
    }

    /// Records a warning diagnostic for `check`.
    pub fn add_warning(&mut self, check: &'static str, message: impl Into<String>) {
        self.issues.push(StartupIssue {
            check,
            level: StartupIssueLevel::Warning,
            message: message.into(),
        });
    }

    /// Returns all issues collected during the pipeline run.
    pub fn issues(&self) -> &[StartupIssue] {
        &self.issues
    }
}
