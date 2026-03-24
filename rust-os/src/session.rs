/// Per-login session state: current working directory, user info.
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user: String,
    pub hostname: String,
    pub home: String,
    cwd: String,
    pub last_exit_code: i32,
    pub command_history: Vec<String>,
}

impl UserSession {
    pub fn new(user: impl Into<String>, hostname: impl Into<String>) -> Self {
        let user = user.into();
        let hostname = hostname.into();
        let home = format!("/usr/{user}");
        Self {
            user,
            hostname,
            home: home.clone(),
            cwd: home,
            last_exit_code: 0,
            command_history: Vec::new(),
        }
    }

    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    pub fn set_cwd(&mut self, path: String) {
        self.cwd = normalize(&path);
    }

    /// Resolve a path relative to cwd. Handles ~, .., ., absolute paths.
    pub fn resolve_path(&self, path: Option<&str>) -> String {
        let p = match path {
            None | Some("") | Some("~") => return self.home.clone(),
            Some(p) if p.starts_with("~/") => {
                format!("{}/{}", self.home, &p[2..])
            }
            Some(p) if p.starts_with('/') => p.to_string(),
            Some(p) => format!("{}/{}", self.cwd, p),
        };
        normalize(&p)
    }

    /// User-friendly display of cwd (tilde expansion).
    pub fn display_cwd(&self) -> String {
        if self.cwd == self.home {
            "~".to_string()
        } else if self.cwd.starts_with(&self.home) {
            format!("~{}", &self.cwd[self.home.len()..])
        } else {
            self.cwd.clone()
        }
    }

    pub fn push_history(&mut self, cmd: &str) {
        if !cmd.trim().is_empty() {
            self.command_history.push(cmd.to_string());
        }
    }
}

/// Remove . and .. segments from an absolute path.
fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}
