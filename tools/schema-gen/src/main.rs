use anyhow::{Context, Result};
use clap::Parser;
use engine_authoring::schema::{generate_mod_schema_files, render_schema_file};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "schema-gen")]
#[command(about = "Generate per-mod schema fragments for YAML authoring intellisense.")]
struct Cli {
    /// Path to one mod root (directory containing mod.yaml).
    #[arg(long, conflicts_with = "all_mods")]
    r#mod: Option<PathBuf>,

    /// Scan all mods under ./mods.
    #[arg(long, conflicts_with = "mod")]
    all_mods: bool,

    /// Output directory for generated schema fragments.
    #[arg(long, default_value = "schemas/generated")]
    out: PathBuf,

    /// Verify that generated schema fragments already match files on disk.
    #[arg(long)]
    check: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mods = resolve_mod_roots(&cli)?;
    if !cli.check {
        fs::create_dir_all(&cli.out)
            .with_context(|| format!("failed to create {}", cli.out.display()))?;
    }

    for mod_root in mods {
        sync_fragment_for_mod(&mod_root, &cli.out, cli.check)?;
    }
    Ok(())
}

fn resolve_mod_roots(cli: &Cli) -> Result<Vec<PathBuf>> {
    if let Some(mod_root) = &cli.r#mod {
        return Ok(vec![mod_root.clone()]);
    }
    if cli.all_mods {
        let base = PathBuf::from("mods");
        let entries =
            fs::read_dir(&base).with_context(|| format!("failed to read {}", base.display()))?;
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("mod.yaml").exists() {
                out.push(path);
            }
        }
        out.sort();
        return Ok(out);
    }
    anyhow::bail!("pass either --mod <path> or --all-mods");
}

fn sync_fragment_for_mod(mod_root: &Path, out_dir: &Path, check: bool) -> Result<()> {
    for file in generate_mod_schema_files(mod_root)? {
        sync_schema_file(&out_dir.join(&file.file_name), &file.value, check)?;
    }
    Ok(())
}

fn sync_schema_file(path: &Path, value: &Value, check: bool) -> Result<()> {
    let yaml = render_schema_file(value)?;
    if check {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("failed to read {} in --check mode", path.display()))?;
        if existing != yaml {
            anyhow::bail!(
                "generated schema is out of date: {} (run schema-gen without --check)",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_fragment_contains_dynamic_defs() {
        let temp_root = unique_temp_dir("schema-gen-test");
        let mod_root = temp_root.join("playground");
        fs::create_dir_all(mod_root.join("scenes/intro/layers")).expect("create layers");
        fs::create_dir_all(mod_root.join("scenes/intro/sprites")).expect("create sprites");
        fs::create_dir_all(mod_root.join("objects")).expect("create objects");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            "id: intro\neffects:\n  - name: fade-in\n    duration: 1.0\n",
        )
        .expect("write scene");
        fs::write(
            mod_root.join("scenes/intro/layers/bg.yml"),
            "name: background\n",
        )
        .expect("write layer partial");
        fs::write(mod_root.join("objects/npc.yml"), "name: npc\n").expect("write object");

        let out_dir = temp_root.join("out");
        fs::create_dir_all(&out_dir).expect("create out dir");
        sync_fragment_for_mod(&mod_root, &out_dir, false).expect("generate");

        let out_path = out_dir.join("playground.schema.yaml");
        let raw = fs::read_to_string(out_path).expect("read generated schema");
        let yaml: Value = serde_yaml::from_str(&raw).expect("parse generated schema");
        let defs = yaml
            .get("$defs")
            .and_then(Value::as_mapping)
            .expect("defs mapping");

        assert!(defs.contains_key(Value::String("scene_ids".to_string())));
        assert!(defs.contains_key(Value::String("object_names".to_string())));
        assert!(defs.contains_key(Value::String("effect_names".to_string())));
        assert!(defs.contains_key(Value::String("layer_refs".to_string())));

        let object_names = defs
            .get(Value::String("object_names".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("object_names enum");
        assert!(object_names.iter().any(|v| v.as_str() == Some("npc")));

        let layer_refs = defs
            .get(Value::String("layer_refs".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("layer_refs enum");
        assert!(layer_refs
            .iter()
            .any(|v| v.as_str() == Some("intro/layers/bg.yml")));

        let scene_overlay = fs::read_to_string(out_dir.join("playground.scene.schema.yaml"))
            .expect("read scene overlay");
        assert!(scene_overlay.contains("./playground.schema.yaml#/$defs/scene_ids"));

        let objects_overlay =
            fs::read_to_string(out_dir.join("playground.objects-file.schema.yaml"))
                .expect("read objects overlay");
        assert!(objects_overlay.contains("./playground.schema.yaml#/$defs/object_names"));

        let layers_overlay = fs::read_to_string(out_dir.join("playground.layers-file.schema.yaml"))
            .expect("read layers overlay");
        assert!(layers_overlay.contains("../layers-file.schema.yaml"));

        let templates_overlay =
            fs::read_to_string(out_dir.join("playground.templates-file.schema.yaml"))
                .expect("read templates overlay");
        assert!(templates_overlay.contains("../templates-file.schema.yaml"));

        let sprites_overlay =
            fs::read_to_string(out_dir.join("playground.sprites-file.schema.yaml"))
                .expect("read sprites overlay");
        assert!(sprites_overlay.contains("../sprites-file.schema.yaml"));

        let effect_overlay = fs::read_to_string(out_dir.join("playground.effect-file.schema.yaml"))
            .expect("read effect overlay");
        assert!(effect_overlay.contains("oneOf:"));
        assert!(effect_overlay.contains("const: fade-in"));
        assert!(effect_overlay.contains("easing:"));
    }

    #[test]
    fn check_mode_passes_when_generated_files_match() {
        let temp_root = unique_temp_dir("schema-gen-check-pass");
        let mod_root = temp_root.join("playground");
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/menu.yml"),
            "id: menu\ntitle: Menu\nnext: null\n",
        )
        .expect("write scene");

        let out_dir = temp_root.join("out");
        fs::create_dir_all(&out_dir).expect("create out dir");
        sync_fragment_for_mod(&mod_root, &out_dir, false).expect("generate");
        sync_fragment_for_mod(&mod_root, &out_dir, true).expect("check");
    }

    #[test]
    fn check_mode_detects_outdated_generated_files() {
        let temp_root = unique_temp_dir("schema-gen-check-fail");
        let mod_root = temp_root.join("playground");
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/menu.yml"),
            "id: menu\ntitle: Menu\nnext: null\n",
        )
        .expect("write scene");

        let out_dir = temp_root.join("out");
        fs::create_dir_all(&out_dir).expect("create out dir");
        sync_fragment_for_mod(&mod_root, &out_dir, false).expect("generate");
        fs::write(
            out_dir.join("playground.scene.schema.yaml"),
            "outdated: true\n",
        )
        .expect("mutate generated file");

        let err = sync_fragment_for_mod(&mod_root, &out_dir, true).expect_err("check should fail");
        assert!(err.to_string().contains("generated schema is out of date"));
    }

    #[test]
    fn committed_generated_schemas_are_current() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let out_dir = repo_root.join("schemas/generated");
        for mod_name in ["playground", "shell-quest"] {
            let mod_root = repo_root.join("mods").join(mod_name);
            sync_fragment_for_mod(&mod_root, &out_dir, true).unwrap_or_else(|err| {
                panic!("{mod_name} generated schemas should be current: {err}")
            });
        }
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{}-{now}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
