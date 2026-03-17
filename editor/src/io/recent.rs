//! Recent project list persistence: load, save, and push with deduplication.

use std::fs;
use std::path::PathBuf;

const MAX_RECENT: usize = 12;

fn recent_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("sq-editor")
        .join("recent_projects.txt")
}

/// Normalize path to canonical form for deduplication
fn normalize_path(path: &str) -> Option<String> {
    fs::canonicalize(path)
        .ok()
        .and_then(|p| p.to_str().map(ToOwned::to_owned))
}

/// Loads and deduplicates the recent project list from `~/.config/sq-editor/recent_projects.txt`.
pub fn load_recent() -> Vec<String> {
    let path = recent_file();
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };

    // Deduplicate on load by normalizing paths
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Try to normalize, use original if normalization fails
        let normalized = normalize_path(line).unwrap_or_else(|| line.to_string());

        if seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    // Save deduplicated list back to file if duplicates were found
    let original_count = raw.lines().filter(|l| !l.trim().is_empty()).count();
    if result.len() < original_count {
        save_recent(&result);
    }

    result
}

/// Persists up to `MAX_RECENT` project paths to disk.
pub fn save_recent(items: &[String]) {
    let path = recent_file();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = items
        .iter()
        .take(MAX_RECENT)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(path, content);
}

/// Prepends `path` to `items`, deduplicating on canonical form and capping at `MAX_RECENT`.
pub fn push_recent(items: &mut Vec<String>, path: &str) {
    // Normalize the incoming path
    let normalized = normalize_path(path).unwrap_or_else(|| path.to_string());

    // Remove any existing entries that normalize to the same path
    items.retain(|p| normalize_path(p).as_deref() != Some(normalized.as_str()));

    items.insert(0, normalized);
    if items.len() > MAX_RECENT {
        items.truncate(MAX_RECENT);
    }
}
