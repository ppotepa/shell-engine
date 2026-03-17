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

    /// Verify that generated schema fragments already match files on disk.
    #[arg(long)]
    check: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mods = resolve_mod_roots(&cli)?;

    for mod_root in mods {
        sync_fragment_for_mod(&mod_root, cli.check)?;
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

fn sync_fragment_for_mod(mod_root: &Path, check: bool) -> Result<()> {
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
        fs::create_dir_all(mod_root.join("scenes/intro/templates")).expect("create templates");
        fs::create_dir_all(mod_root.join("scenes/shared/objects")).expect("create shared objects");
        fs::create_dir_all(mod_root.join("objects")).expect("create objects");
        fs::create_dir_all(mod_root.join("assets/fonts/mono")).expect("create fonts");
        fs::create_dir_all(mod_root.join("assets/images")).expect("create images");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            "id: intro\ntemplates:\n  title-card:\n    type: text\n    content: TEST\ninput:\n  obj-viewer:\n    sprite_id: logo\neffects:\n  - name: fade-in\n    duration: 1.0\n",
        )
        .expect("write scene");
        fs::write(
            mod_root.join("scenes/intro/layers/bg.yml"),
            "- name: background\n  sprites:\n    - id: logo\n      use: title-card\n      font: generic:1\n      source: /assets/images/logo.png\n",
        )
        .expect("write layer partial");
        fs::write(
            mod_root.join("scenes/intro/templates/common.yml"),
            "menu-item:\n  type: text\n  content: START\n",
        )
        .expect("write template partial");
        fs::write(mod_root.join("objects/npc.yml"), "name: npc\n").expect("write object");
        fs::write(
            mod_root.join("scenes/shared/objects/banner.yml"),
            "name: banner\nsprites:\n  - type: text\n    content: SHARED\n",
        )
        .expect("write shared object");
        fs::write(
            mod_root.join("assets/fonts/mono/manifest.yaml"),
            "name: Mono Display\n",
        )
        .expect("write font manifest");
        fs::write(mod_root.join("assets/images/logo.png"), b"").expect("write image");
        fs::write(mod_root.join("scenes/intro/cube.obj"), "").expect("write model");

        sync_fragment_for_mod(&mod_root, false).expect("generate");

        let out_path = mod_root.join("schemas/catalog.yaml");
        let raw = fs::read_to_string(out_path).expect("read generated schema");
        let yaml: Value = serde_yaml::from_str(&raw).expect("parse generated schema");
        let defs = yaml
            .get("$defs")
            .and_then(Value::as_mapping)
            .expect("defs mapping");

        assert!(defs.contains_key(Value::String("scene_ids".to_string())));
        assert!(defs.contains_key(Value::String("scene_paths".to_string())));
        assert!(defs.contains_key(Value::String("scene_refs".to_string())));
        assert!(defs.contains_key(Value::String("object_names".to_string())));
        assert!(defs.contains_key(Value::String("object_refs".to_string())));
        assert!(defs.contains_key(Value::String("effect_names".to_string())));
        assert!(defs.contains_key(Value::String("layer_refs".to_string())));
        assert!(defs.contains_key(Value::String("font_specs".to_string())));

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

        let mod_overlay =
            fs::read_to_string(mod_root.join("schemas/mod.yaml")).expect("read mod overlay");
        assert!(mod_overlay.contains("../../../schemas/mod.schema.yaml"));
        assert!(mod_overlay.contains("./catalog.yaml#/$defs/scene_paths"));

        let object_doc_overlay =
            fs::read_to_string(mod_root.join("schemas/object.yaml")).expect("read object overlay");
        assert!(object_doc_overlay.contains("../../../schemas/object.schema.yaml"));
        assert!(object_doc_overlay.contains("const: blink"));
        assert!(object_doc_overlay.contains("visible_ms:"));

        let scene_overlay =
            fs::read_to_string(mod_root.join("schemas/scenes.yaml")).expect("read scene overlay");
        assert!(scene_overlay.contains("./catalog.yaml#/$defs/scene_refs"));
        assert!(scene_overlay.contains("./catalog.yaml#/$defs/sprite_ids"));
        assert!(scene_overlay.contains("./catalog.yaml#/$defs/template_names"));
        assert!(scene_overlay.contains("./catalog.yaml#/$defs/font_specs"));
        assert!(scene_overlay.contains("./effects.yaml#/items"));
        assert!(scene_overlay.contains("#/$defs/behavior_overlay"));
        assert!(scene_overlay.contains("const: blink"));
        assert!(scene_overlay
            .contains("scene_stages_overlay:\n    type: object\n    additionalProperties: false"));
        assert!(scene_overlay.contains(
            "behavior_overlay:\n    oneOf:\n    - type: object\n      additionalProperties: false"
        ));
        assert!(scene_overlay.contains("#/$defs/sprite_overlay"));
        assert!(scene_overlay.contains("../../../schemas/scene.schema.yaml"));

        let objects_overlay = fs::read_to_string(mod_root.join("schemas/objects.yaml"))
            .expect("read objects overlay");
        assert!(objects_overlay.contains("./catalog.yaml#/$defs/object_refs"));
        assert!(objects_overlay.contains("../../../schemas/objects-file.schema.yaml#/items"));

        let layers_overlay =
            fs::read_to_string(mod_root.join("schemas/layers.yaml")).expect("read layers overlay");
        assert!(layers_overlay.contains("../../../schemas/layers-file.schema.yaml"));
        assert!(layers_overlay.contains("#/$defs/layer_overlay"));

        let templates_overlay = fs::read_to_string(mod_root.join("schemas/templates.yaml"))
            .expect("read templates overlay");
        assert!(templates_overlay.contains("../../../schemas/templates-file.schema.yaml"));
        assert!(templates_overlay.contains("#/$defs/sprite_overlay"));

        let sprites_overlay = fs::read_to_string(mod_root.join("schemas/sprites.yaml"))
            .expect("read sprites overlay");
        assert!(sprites_overlay.contains("../../../schemas/sprites-file.schema.yaml"));
        assert!(sprites_overlay.contains("#/$defs/sprite_overlay"));

        let effect_overlay =
            fs::read_to_string(mod_root.join("schemas/effects.yaml")).expect("read effect overlay");
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

        sync_fragment_for_mod(&mod_root, false).expect("generate");
        sync_fragment_for_mod(&mod_root, true).expect("check");
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

        sync_fragment_for_mod(&mod_root, false).expect("generate");
        fs::write(mod_root.join("schemas/scenes.yaml"), "outdated: true\n")
            .expect("mutate generated file");

        let err = sync_fragment_for_mod(&mod_root, true).expect_err("check should fail");
        assert!(err.to_string().contains("generated schema is out of date"));
    }

    #[test]
    fn committed_generated_schemas_are_current() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        for mod_name in ["playground", "shell-quest"] {
            let mod_root = repo_root.join("mods").join(mod_name);
            sync_fragment_for_mod(&mod_root, true).unwrap_or_else(|err| {
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
