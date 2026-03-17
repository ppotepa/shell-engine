use anyhow::{Context, Result};
use clap::Parser;
use engine_core::authoring::catalog::static_catalog;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mods = resolve_mod_roots(&cli)?;
    fs::create_dir_all(&cli.out)
        .with_context(|| format!("failed to create {}", cli.out.display()))?;

    for mod_root in mods {
        generate_fragment_for_mod(&mod_root, &cli.out)?;
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

fn generate_fragment_for_mod(mod_root: &Path, out_dir: &Path) -> Result<()> {
    let mod_name = mod_root
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid mod path: {}", mod_root.display()))?;

    let scene_ids = collect_scene_ids(mod_root)?;
    let object_names = collect_object_names(mod_root)?;
    let mut effect_names = collect_effect_names(mod_root)?;
    for name in static_catalog().effect_names {
        effect_names.insert((*name).to_string());
    }
    let layer_refs = collect_scene_partial_refs(mod_root, "layers")?;
    let sprite_refs = collect_scene_partial_refs(mod_root, "sprites")?;
    let template_refs = collect_scene_partial_refs(mod_root, "templates")?;
    let object_refs = collect_scene_partial_refs(mod_root, "objects")?;
    let effect_refs = collect_scene_partial_refs(mod_root, "effects")?;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/schemas/generated/{mod_name}.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} generated schema fragment")),
    );

    let mut defs = Mapping::new();
    defs.insert(
        Value::String("scene_ids".to_string()),
        enum_schema(scene_ids.into_iter().collect()),
    );
    defs.insert(
        Value::String("object_names".to_string()),
        enum_schema(object_names.into_iter().collect()),
    );
    defs.insert(
        Value::String("effect_names".to_string()),
        enum_schema(effect_names.into_iter().collect()),
    );
    defs.insert(
        Value::String("layer_refs".to_string()),
        enum_schema(layer_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("sprite_refs".to_string()),
        enum_schema(sprite_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("template_refs".to_string()),
        enum_schema(template_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("object_refs".to_string()),
        enum_schema(object_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("effect_refs".to_string()),
        enum_schema(effect_refs.into_iter().collect()),
    );
    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));

    let out_path = out_dir.join(format!("{mod_name}.schema.yaml"));
    let mut yaml = serde_yaml::to_string(&Value::Mapping(root))?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    fs::write(&out_path, yaml)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    println!("generated {}", out_path.display());
    Ok(())
}

fn enum_schema(values: Vec<String>) -> Value {
    let mut m = Mapping::new();
    m.insert(
        Value::String("type".to_string()),
        Value::String("string".to_string()),
    );
    m.insert(
        Value::String("enum".to_string()),
        Value::Sequence(values.into_iter().map(Value::String).collect()),
    );
    Value::Mapping(m)
}

fn collect_scene_ids(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("scenes"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                if let Some(id) = v.get("id").and_then(Value::as_str) {
                    ids.insert(id.to_string());
                }
            }
        }
    }
    Ok(ids)
}

fn collect_object_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("objects"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                if let Some(name) = v.get("name").and_then(Value::as_str) {
                    names.insert(name.to_string());
                }
            }
        }
    }
    Ok(names)
}

fn collect_effect_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("scenes"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                collect_effect_names_from_value(&v, &mut names);
            }
        }
    }
    Ok(names)
}

fn collect_effect_names_from_value(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Mapping(map) => {
            if let Some(name) = map
                .get(Value::String("name".to_string()))
                .and_then(Value::as_str)
            {
                if map.contains_key(Value::String("duration".to_string())) {
                    out.insert(name.to_string());
                }
            }
            for v in map.values() {
                collect_effect_names_from_value(v, out);
            }
        }
        Value::Sequence(seq) => {
            for entry in seq {
                collect_effect_names_from_value(entry, out);
            }
        }
        _ => {}
    }
}

fn collect_scene_partial_refs(mod_root: &Path, part_dir: &str) -> Result<BTreeSet<String>> {
    let scenes_root = mod_root.join("scenes");
    if !scenes_root.exists() {
        return Ok(BTreeSet::new());
    }
    let mut refs = BTreeSet::new();
    for scene_dir in fs::read_dir(&scenes_root)
        .with_context(|| format!("failed to read {}", scenes_root.display()))?
    {
        let scene_dir = scene_dir?;
        let scene_path = scene_dir.path();
        if !scene_path.is_dir() {
            continue;
        }
        let scene_name = match scene_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        let part_root = scene_path.join(part_dir);
        if !part_root.exists() {
            continue;
        }
        for file in yaml_files_under(&part_root)? {
            if let Ok(rel) = file.strip_prefix(&part_root) {
                refs.insert(format!("{scene_name}/{part_dir}/{}", rel.to_string_lossy()));
            }
        }
    }
    Ok(refs)
}

fn yaml_files_under(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_yaml(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_yaml(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_yaml(&p, out)?;
            continue;
        }
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if ext == "yml" || ext == "yaml" {
            out.push(p);
        }
    }
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
            "id: intro\neffects:\n  - name: fade\n    duration: 1.0\n",
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
        generate_fragment_for_mod(&mod_root, &out_dir).expect("generate");

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
