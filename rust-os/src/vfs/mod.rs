use crate::state::FileStat;
use std::collections::BTreeMap;

pub mod seed;

/// In-memory virtual filesystem.
/// Directories stored as BTreeMap entries with None content.
/// Files stored with Some(content).
pub struct Vfs {
    files: BTreeMap<String, Option<String>>,
    stats: BTreeMap<String, FileStat>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    pub fn new() -> Self {
        let mut vfs = Self {
            files: BTreeMap::new(),
            stats: BTreeMap::new(),
        };
        seed::seed(&mut vfs);
        vfs
    }

    pub fn mkdir(&mut self, path: &str) {
        let p = normalize(path);
        self.files.entry(p.clone()).or_insert(None);
        self.stats.entry(p).or_insert_with(|| FileStat {
            permissions: "drwxr-xr-x".to_string(),
            links: 2,
            owner: "root".to_string(),
            group: "staff".to_string(),
            size: 512,
            modified: "Sep 17 21:12".to_string(),
        });
    }

    pub fn write_file(&mut self, path: &str, content: &str, owner: &str) {
        let p = normalize(path);
        // ensure parent exists
        if let Some(parent) = parent_of(&p) {
            self.mkdir(&parent);
        }
        let size = content.len() as u64;
        self.files.insert(p.clone(), Some(content.to_string()));
        self.stats.insert(
            p,
            FileStat {
                permissions: "-rw-r--r--".to_string(),
                links: 1,
                owner: owner.to_string(),
                group: "staff".to_string(),
                size,
                modified: "Sep 17 21:12".to_string(),
            },
        );
    }

    pub fn write_file_with_perms(&mut self, path: &str, content: &str, owner: &str, perms: &str) {
        self.write_file(path, content, owner);
        if let Some(stat) = self.stats.get_mut(&normalize(path)) {
            stat.permissions = perms.to_string();
        }
    }

    pub fn read_file(&self, path: &str) -> Option<&str> {
        let p = normalize(path);
        self.files.get(&p)?.as_deref()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.files.contains_key(&normalize(path))
    }

    pub fn is_dir(&self, path: &str) -> bool {
        let p = normalize(path);
        matches!(self.files.get(&p), Some(None))
    }

    pub fn is_file(&self, path: &str) -> bool {
        let p = normalize(path);
        matches!(self.files.get(&p), Some(Some(_)))
    }

    pub fn stat(&self, path: &str) -> Option<&FileStat> {
        self.stats.get(&normalize(path))
    }

    pub fn stat_mut(&mut self, path: &str) -> Option<&mut FileStat> {
        self.stats.get_mut(&normalize(path))
    }

    pub fn delete(&mut self, path: &str) {
        let p = normalize(path);
        self.files.remove(&p);
        self.stats.remove(&p);
    }

    /// List directory entries (immediate children only).
    pub fn readdir(&self, path: &str) -> Vec<String> {
        let p = normalize(path);
        let prefix = if p == "/" {
            "/".to_string()
        } else {
            format!("{p}/")
        };
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for key in self.files.keys() {
            if key == &p {
                continue;
            }
            if !key.starts_with(&prefix) {
                continue;
            }
            let rest = &key[prefix.len()..];
            let name = rest.split('/').next().unwrap_or(rest);
            if !name.is_empty() && seen.insert(name.to_string()) {
                result.push(name.to_string());
            }
        }
        result.sort();
        result
    }

    pub fn copy_file(&mut self, src: &str, dst: &str) -> bool {
        let content = match self.read_file(src) {
            Some(c) => c.to_string(),
            None => return false,
        };
        let owner = self
            .stat(src)
            .map(|s| s.owner.clone())
            .unwrap_or_else(|| "root".to_string());
        self.write_file(dst, &content, &owner);
        true
    }

    pub fn move_file(&mut self, src: &str, dst: &str) -> bool {
        if !self.copy_file(src, dst) {
            return false;
        }
        self.delete(src);
        true
    }

    /// Inject a subtle anomaly: append a stray line to a random file
    pub fn inject_data_bleed(&mut self, from_path: &str, to_path: &str) {
        let bleed: Option<String> = self
            .read_file(from_path)
            .and_then(|c| c.lines().next().map(|l| l.to_string()));
        if let Some(line) = bleed {
            if let Some(Some(content)) = self.files.get_mut(&normalize(to_path)) {
                content.push('\n');
                content.push_str(&line);
            }
        }
    }
}

pub fn normalize(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
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

fn parent_of(path: &str) -> Option<String> {
    let p = path.trim_end_matches('/');
    if let Some(pos) = p.rfind('/') {
        if pos == 0 {
            Some("/".to_string())
        } else {
            Some(p[..pos].to_string())
        }
    } else {
        None
    }
}
