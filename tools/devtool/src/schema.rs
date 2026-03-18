use anyhow::{bail, Context, Result};
use engine_authoring::schema::{generate_mod_schema_files, render_schema_file};
use serde_yaml::Value;
use std::fs;
use std::path::Path;

pub fn sync_fragment_for_mod(mod_root: &Path, check: bool) -> Result<()> {
    for file in generate_mod_schema_files(mod_root)? {
        let out_path = mod_root.join(&file.file_name);
        if !check {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
        }
        sync_schema_file(&out_path, &file.value, check)?;
    }
    Ok(())
}

fn sync_schema_file(path: &Path, value: &Value, check: bool) -> Result<()> {
    let yaml = render_schema_file(value)?;
    if check {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("failed to read {} in --check mode", path.display()))?;
        if existing != yaml {
            bail!(
                "generated schema is out of date: {} (run `devtool schema refresh`)",
                path.display()
            );
        }
        println!("checked {}", path.display());
        return Ok(());
    }
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))?;
    println!("generated {}", path.display());
    Ok(())
}
