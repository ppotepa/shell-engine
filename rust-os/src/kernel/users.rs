#[derive(Debug, Clone)]
pub struct PasswdEntry {
    pub login: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home: String,
    pub shell: String,
}

#[derive(Debug, Clone)]
pub struct GroupEntry {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
}

pub struct UserDatabase {
    passwd: Vec<PasswdEntry>,
    groups: Vec<GroupEntry>,
}

impl Default for UserDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl UserDatabase {
    pub fn new() -> Self {
        // Hard-coded matching the VFS /etc/passwd and /etc/group
        Self {
            passwd: vec![
                PasswdEntry {
                    login: "root".into(),
                    uid: 0,
                    gid: 0,
                    gecos: "System Administrator".into(),
                    home: "/".into(),
                    shell: "/bin/sh".into(),
                },
                PasswdEntry {
                    login: "daemon".into(),
                    uid: 1,
                    gid: 1,
                    gecos: "Daemon".into(),
                    home: "/".into(),
                    shell: "/bin/false".into(),
                },
                PasswdEntry {
                    login: "ast".into(),
                    uid: 100,
                    gid: 10,
                    gecos: "Andrew S. Tanenbaum".into(),
                    home: "/usr/ast".into(),
                    shell: "/bin/sh".into(),
                },
                PasswdEntry {
                    login: "torvalds".into(),
                    uid: 1000,
                    gid: 10,
                    gecos: "Linus Torvalds".into(),
                    home: "/usr/torvalds".into(),
                    shell: "/bin/sh".into(),
                },
                PasswdEntry {
                    login: "nobody".into(),
                    uid: 65534,
                    gid: 65534,
                    gecos: "Nobody".into(),
                    home: "/".into(),
                    shell: "/bin/false".into(),
                },
            ],
            groups: vec![
                GroupEntry {
                    name: "root".into(),
                    gid: 0,
                    members: vec![],
                },
                GroupEntry {
                    name: "staff".into(),
                    gid: 10,
                    members: vec!["ast".into(), "torvalds".into()],
                },
                GroupEntry {
                    name: "wheel".into(),
                    gid: 11,
                    members: vec!["torvalds".into()],
                },
                GroupEntry {
                    name: "users".into(),
                    gid: 100,
                    members: vec!["ast".into(), "torvalds".into()],
                },
            ],
        }
    }

    pub fn get_user(&self, login: &str) -> Option<&PasswdEntry> {
        self.passwd.iter().find(|p| p.login == login)
    }

    pub fn all_users(&self) -> &[PasswdEntry] {
        &self.passwd
    }

    pub fn get_group(&self, gid: u32) -> Option<&GroupEntry> {
        self.groups.iter().find(|g| g.gid == gid)
    }

    pub fn get_group_by_name(&self, name: &str) -> Option<&GroupEntry> {
        self.groups.iter().find(|g| g.name == name)
    }

    pub fn groups_for_user(&self, login: &str) -> Vec<String> {
        self.groups
            .iter()
            .filter(|g| g.members.contains(&login.to_string()))
            .map(|g| g.name.clone())
            .collect()
    }
}
