//! Shared file-system utilities for startup checks.

use std::path::Path;

/// Normalize a full filesystem path into a mod-relative path string like "/audio/foo.yml".
pub(super) fn normalize_relative_asset_path(mod_source: &Path, full_path: &Path) -> String {
    let rel = full_path.strip_prefix(mod_source).unwrap_or(full_path);
    format!("/{}", rel.display().to_string().replace('\\', "/"))
}

/// Returns true if `path` is a file with a `.yml` or `.yaml` extension.
pub(super) fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            let ext = ext.to_ascii_lowercase();
            ext == "yml" || ext == "yaml"
        })
}

/// Returns true if `path` is a file with a `.zip` extension.
pub(super) fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}
