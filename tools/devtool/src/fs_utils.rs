use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::SchemaTargetArgs;

pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        if dir.join("Cargo.toml").exists() && dir.join("mods").is_dir() {
            return Ok(dir.to_path_buf());
        }
    }
    bail!(
        "could not find repository root from {} (expected Cargo.toml + mods/)",
        start.display()
    )
}

pub fn ensure_mod_exists(mod_root: &Path) -> Result<()> {
    if mod_root.join("mod.yaml").exists() {
        return Ok(());
    }
    bail!(
        "mod root not found or missing mod.yaml: {}",
        mod_root.display()
    )
}

pub fn write_file(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "file already exists: {} (use --force to overwrite)",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn resolve_mod_roots(repo_root: &Path, args: &SchemaTargetArgs) -> Result<Vec<PathBuf>> {
    if let Some(mod_arg) = &args.r#mod {
        let mod_root = parse_mod_target(repo_root, mod_arg);
        ensure_mod_exists(&mod_root)?;
        return Ok(vec![mod_root]);
    }

    let mods_dir = repo_root.join("mods");
    let mut out = Vec::new();
    let entries = fs::read_dir(&mods_dir)
        .with_context(|| format!("failed to read {}", mods_dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("mod.yaml").exists() {
            out.push(path);
        }
    }
    out.sort();
    if out.is_empty() {
        bail!("no mods found under {}", mods_dir.display());
    }
    Ok(out)
}

fn parse_mod_target(repo_root: &Path, mod_arg: &str) -> PathBuf {
    let path = PathBuf::from(mod_arg);
    if path.is_absolute() {
        return path;
    }
    if mod_arg.contains('/') || mod_arg.contains('\\') {
        return repo_root.join(path);
    }
    repo_root.join("mods").join(mod_arg)
}
