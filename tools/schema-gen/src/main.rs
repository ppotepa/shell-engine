use anyhow::{Context, Result};
use clap::Parser;
use engine_authoring::schema::{generate_mod_schema_files, render_schema_file};
use engine_effects::{shared_dispatcher, EffectDispatcher, ParamControl};
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, BTreeSet};
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
    sync_shared_effect_schemas(Path::new("."), cli.check)?;
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

fn sync_shared_effect_schemas(repo_root: &Path, check: bool) -> Result<()> {
    let effect_schema_path = repo_root.join("schemas/effect.schema.yaml");
    let effect_params_schema_path = repo_root.join("schemas/effect-params.schema.yaml");
    let effect_schema = generate_shared_effect_schema();
    let effect_params_schema = generate_shared_effect_params_schema()?;
    sync_schema_file(&effect_schema_path, &effect_schema, check)?;
    sync_schema_file(&effect_params_schema_path, &effect_params_schema, check)?;
    Ok(())
}

fn generate_shared_effect_schema() -> Value {
    let dispatcher = shared_dispatcher();
    let names = EffectDispatcher::builtin_names();
    let mut summary_lines = vec!["Effect identifier. Built-in effects:".to_string()];
    for name in names {
        let summary = dispatcher.metadata(name).summary;
        summary_lines.push(format!("- {name}: {summary}"));
    }

    let mut name_prop = Mapping::new();
    name_prop.insert(yaml_str("type"), yaml_str("string"));
    name_prop.insert(yaml_str("description"), yaml_str(&summary_lines.join("\n")));
    name_prop.insert(
        yaml_str("enum"),
        Value::Sequence(names.iter().map(|name| yaml_str(name)).collect()),
    );

    let mut duration_prop = Mapping::new();
    duration_prop.insert(yaml_str("type"), yaml_str("integer"));
    duration_prop.insert(yaml_str("minimum"), yaml_int(0));
    duration_prop.insert(
        yaml_str("description"),
        yaml_str("Duration in milliseconds."),
    );

    let mut looping_prop = Mapping::new();
    looping_prop.insert(yaml_str("type"), yaml_str("boolean"));
    looping_prop.insert(yaml_str("default"), Value::Bool(false));
    looping_prop.insert(
        yaml_str("description"),
        yaml_str("Whether the effect repeats indefinitely within its step."),
    );

    let mut properties = Mapping::new();
    properties.insert(yaml_str("name"), Value::Mapping(name_prop));
    properties.insert(yaml_str("duration"), Value::Mapping(duration_prop));
    properties.insert(yaml_str("looping"), Value::Mapping(looping_prop));
    properties.insert(
        yaml_str("params"),
        Value::Mapping(mapping_with_ref("./effect-params.schema.yaml")),
    );

    let mut root = Mapping::new();
    root.insert(
        yaml_str("$schema"),
        yaml_str("https://json-schema.org/draft/2020-12/schema"),
    );
    root.insert(
        yaml_str("$id"),
        yaml_str("https://shell-quest.local/schemas/effect.schema.yaml"),
    );
    root.insert(yaml_str("title"), yaml_str("Effect Schema"));
    root.insert(
        yaml_str("description"),
        yaml_str(
            "A single named visual effect step used inside scene/layer/sprite lifecycle stages.",
        ),
    );
    root.insert(yaml_str("type"), yaml_str("object"));
    root.insert(yaml_str("additionalProperties"), Value::Bool(false));
    root.insert(
        yaml_str("required"),
        Value::Sequence(vec![yaml_str("name"), yaml_str("duration")]),
    );
    root.insert(yaml_str("properties"), Value::Mapping(properties));
    Value::Mapping(root)
}

fn generate_shared_effect_params_schema() -> Result<Value> {
    #[derive(Clone, Copy)]
    struct ParamEntry {
        description: &'static str,
        control: ParamControl,
        /// True when multiple effects share this param name with differing defaults.
        ambiguous_default: bool,
    }

    let dispatcher = shared_dispatcher();
    let mut params: BTreeMap<&'static str, ParamEntry> = BTreeMap::new();
    let mut used_by: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();

    for effect_name in EffectDispatcher::builtin_names() {
        for param in dispatcher.metadata(effect_name).params {
            used_by.entry(param.name).or_default().insert(effect_name);
            match params.get(param.name) {
                None => {
                    params.insert(
                        param.name,
                        ParamEntry {
                            description: param.description,
                            control: param.control,
                            ambiguous_default: false,
                        },
                    );
                }
                Some(existing) if existing.control == param.control => {}
                Some(_existing)
                    if std::mem::discriminant(&_existing.control)
                        == std::mem::discriminant(&param.control) =>
                {
                    // Same control type, different defaults — mark as ambiguous so we
                    // omit the default from the shared schema.  Per-effect defaults are
                    // documented in each effect's own oneOf variant instead.
                    params.get_mut(param.name).unwrap().ambiguous_default = true;
                }
                Some(existing) => {
                    anyhow::bail!(
                        "parameter '{}' has incompatible control types across effects: {:?} vs {:?}",
                        param.name,
                        existing.control,
                        param.control
                    );
                }
            }
        }
    }

    let mut properties = Mapping::new();
    for (param_name, entry) in params {
        let users = used_by
            .get(param_name)
            .map(|set| set.iter().copied().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        let description = if users.is_empty() {
            entry.description.to_string()
        } else {
            format!("{} Used by: {}.", entry.description, users)
        };
        properties.insert(
            yaml_str(param_name),
            param_control_schema(entry.control, &description, entry.ambiguous_default),
        );
    }

    let mut defs = Mapping::new();
    defs.insert(yaml_str("colour"), colour_value_schema());

    let mut root = Mapping::new();
    root.insert(
        yaml_str("$schema"),
        yaml_str("https://json-schema.org/draft/2020-12/schema"),
    );
    root.insert(
        yaml_str("$id"),
        yaml_str("https://shell-quest.local/schemas/effect-params.schema.yaml"),
    );
    root.insert(yaml_str("title"), yaml_str("Effect Params Schema"));
    root.insert(
        yaml_str("description"),
        yaml_str("Parameter overrides for a visual effect step. Omitted parameters fall back to per-effect defaults."),
    );
    root.insert(yaml_str("type"), yaml_str("object"));
    root.insert(yaml_str("additionalProperties"), Value::Bool(false));
    root.insert(yaml_str("properties"), Value::Mapping(properties));
    root.insert(yaml_str("$defs"), Value::Mapping(defs));
    Ok(Value::Mapping(root))
}

fn param_control_schema(control: ParamControl, description: &str, omit_default: bool) -> Value {
    let mut prop = Mapping::new();
    match control {
        ParamControl::Slider {
            min,
            max,
            step,
            unit: _,
        } => {
            prop.insert(yaml_str("type"), yaml_str("number"));
            prop.insert(yaml_str("minimum"), yaml_float(min));
            prop.insert(yaml_str("maximum"), yaml_float(max));
            if step > 0.0 {
                prop.insert(yaml_str("multipleOf"), yaml_float(step));
            }
        }
        ParamControl::Select { options, default } => {
            prop.insert(yaml_str("type"), yaml_str("string"));
            prop.insert(
                yaml_str("enum"),
                Value::Sequence(options.iter().map(|v| yaml_str(v)).collect()),
            );
            if !omit_default {
                prop.insert(yaml_str("default"), yaml_str(default));
            }
        }
        ParamControl::Toggle { default } => {
            prop.insert(yaml_str("type"), yaml_str("boolean"));
            if !omit_default {
                prop.insert(yaml_str("default"), Value::Bool(default));
            }
        }
        ParamControl::Text { default } => {
            prop.insert(yaml_str("type"), yaml_str("string"));
            if !omit_default {
                prop.insert(yaml_str("default"), yaml_str(default));
            }
        }
        ParamControl::Colour { default } => {
            prop.insert(yaml_str("$ref"), yaml_str("#/$defs/colour"));
            if !omit_default {
                prop.insert(yaml_str("default"), yaml_str(default));
            }
        }
    }
    prop.insert(yaml_str("description"), yaml_str(description));
    Value::Mapping(prop)
}

fn colour_value_schema() -> Value {
    let mut hex = Mapping::new();
    hex.insert(yaml_str("type"), yaml_str("string"));
    hex.insert(yaml_str("pattern"), yaml_str("^#[0-9A-Fa-f]{6}$"));
    hex.insert(yaml_str("description"), yaml_str("Hex colour: #rrggbb"));

    let mut named = Mapping::new();
    named.insert(yaml_str("type"), yaml_str("string"));
    named.insert(
        yaml_str("enum"),
        Value::Sequence(
            [
                "black", "white", "gray", "grey", "silver", "red", "green", "blue", "yellow",
                "cyan", "magenta",
            ]
            .iter()
            .map(|name| yaml_str(name))
            .collect(),
        ),
    );
    named.insert(yaml_str("description"), yaml_str("Named colour."));

    let mut root = Mapping::new();
    root.insert(
        yaml_str("description"),
        yaml_str("A colour value. Either a named colour or a CSS-style hex string (#rrggbb)."),
    );
    root.insert(
        yaml_str("anyOf"),
        Value::Sequence(vec![Value::Mapping(hex), Value::Mapping(named)]),
    );
    Value::Mapping(root)
}

fn mapping_with_ref(reference: &str) -> Mapping {
    let mut map = Mapping::new();
    map.insert(yaml_str("$ref"), yaml_str(reference));
    map
}

fn yaml_str(value: &str) -> Value {
    Value::String(value.to_string())
}

fn yaml_int(value: i64) -> Value {
    Value::Number(value.into())
}

fn yaml_float(value: f32) -> Value {
    let rounded = ((value as f64) * 1000.0).round() / 1000.0;
    serde_yaml::to_value(rounded).expect("serialize float")
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
        assert!(layers_overlay.contains("type: array"));
        assert!(layers_overlay.contains("#/$defs/layer_overlay"));

        let templates_overlay = fs::read_to_string(mod_root.join("schemas/templates.yaml"))
            .expect("read templates overlay");
        assert!(templates_overlay.contains("type: object"));
        assert!(templates_overlay.contains("#/$defs/sprite_overlay"));

        let sprites_overlay = fs::read_to_string(mod_root.join("schemas/sprites.yaml"))
            .expect("read sprites overlay");
        assert!(sprites_overlay.contains("type: array"));
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

    #[test]
    fn shared_effect_schemas_cover_builtin_metadata() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        sync_shared_effect_schemas(&repo_root, true)
            .expect("shared effect schemas should be generated from builtin metadata");
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
