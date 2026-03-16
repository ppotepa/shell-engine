#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupIssueLevel {
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct StartupIssue {
    pub check: &'static str,
    pub level: StartupIssueLevel,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct StartupReport {
    issues: Vec<StartupIssue>,
}

impl StartupReport {
    pub fn add_info(&mut self, check: &'static str, message: impl Into<String>) {
        self.issues.push(StartupIssue {
            check,
            level: StartupIssueLevel::Info,
            message: message.into(),
        });
    }

    pub fn add_warning(&mut self, check: &'static str, message: impl Into<String>) {
        self.issues.push(StartupIssue {
            check,
            level: StartupIssueLevel::Warning,
            message: message.into(),
        });
    }

    pub fn issues(&self) -> &[StartupIssue] {
        &self.issues
    }
}
