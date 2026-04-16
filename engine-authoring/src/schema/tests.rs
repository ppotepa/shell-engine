use super::builders::{
    build_animation_schema, build_behavior_schema, build_input_profile_schema, build_sugar_schema,
};
use super::collectors::{
    collect_cutscene_refs, collect_font_names, collect_image_paths, collect_model_paths,
    collect_sprite_ids, collect_template_names,
};
use super::{generate_mod_schema_files, render_schema_file};
use serde_yaml::Mapping;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn generate_fragment_contains_dynamic_defs() {
    let temp_root = unique_temp_dir("authoring-schema-test");
    let mod_root = temp_root.join("sample-mod");
    fs::create_dir_all(mod_root.join("scenes/intro/layers")).expect("create layers");
    fs::create_dir_all(mod_root.join("scenes/intro/sprites")).expect("create sprites");
    fs::create_dir_all(mod_root.join("scenes/intro/templates")).expect("create templates");
    fs::create_dir_all(mod_root.join("scenes/shared/objects")).expect("create shared objects");
    fs::create_dir_all(mod_root.join("objects")).expect("create objects");
    fs::create_dir_all(mod_root.join("assets/fonts/mono")).expect("create fonts");
    fs::create_dir_all(mod_root.join("assets/images")).expect("create images");
    fs::create_dir_all(mod_root.join("cutscenes")).expect("create cutscenes");
    fs::write(mod_root.join("mod.yaml"), "name: sample-mod\n").expect("write mod");
    fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            "id: intro\ntemplates:\n  title-card:\n    type: text\n    content: TEST\neffects:\n  - name: fade-in\n    duration: 1.0\n",
        )
        .expect("write scene");
    fs::write(
        mod_root.join("scenes/intro/layers/bg.yml"),
        "name: background\n",
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
    fs::write(
        mod_root.join("cutscenes/intro-main.yml"),
        "frames:\n  - source: /assets/images/logo.png\n    delay-ms: 100\n",
    )
    .expect("write cutscene");

    let files = generate_mod_schema_files(&mod_root).expect("generate schemas");
    let object_overlay = files
        .iter()
        .find(|file| file.file_name == "schemas/object.yaml")
        .expect("object overlay");
    let root = files
        .iter()
        .find(|file| file.file_name == "schemas/catalog.yaml")
        .expect("root schema");
    let yaml = root.value.as_mapping().expect("schema mapping");
    let defs = yaml
        .get(Value::String("$defs".to_string()))
        .and_then(Value::as_mapping)
        .expect("defs mapping");

    assert!(defs.contains_key(Value::String("scene_ids".to_string())));
    assert!(defs.contains_key(Value::String("scene_paths".to_string())));
    assert!(defs.contains_key(Value::String("scene_refs".to_string())));
    assert!(defs.contains_key(Value::String("object_names".to_string())));
    assert!(defs.contains_key(Value::String("object_refs".to_string())));
    assert!(defs.contains_key(Value::String("effect_names".to_string())));
    assert!(defs.contains_key(Value::String("layer_refs".to_string())));
    assert!(defs.contains_key(Value::String("font_names".to_string())));
    assert!(defs.contains_key(Value::String("font_specs".to_string())));
    assert!(defs.contains_key(Value::String("image_paths".to_string())));
    assert!(defs.contains_key(Value::String("model_paths".to_string())));
    assert!(defs.contains_key(Value::String("cutscene_refs".to_string())));
    assert!(defs.contains_key(Value::String("sprite_ids".to_string())));
    assert!(defs.contains_key(Value::String("template_names".to_string())));

    let scene_paths = defs
        .get(Value::String("scene_paths".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("scene_paths enum");
    assert!(scene_paths
        .iter()
        .any(|v| v.as_str() == Some("/scenes/intro/scene.yml")));

    let scene_refs = defs
        .get(Value::String("scene_refs".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("scene_refs enum");
    assert!(scene_refs.iter().any(|v| v.as_str() == Some("intro")));
    assert!(scene_refs
        .iter()
        .any(|v| v.as_str() == Some("/scenes/intro/scene.yml")));

    let object_refs = defs
        .get(Value::String("object_refs".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("object_refs enum");
    assert!(object_refs.iter().any(|v| v.as_str() == Some("npc")));
    assert!(object_refs
        .iter()
        .any(|v| v.as_str() == Some("/objects/npc.yml")));
    assert!(object_refs
        .iter()
        .any(|v| v.as_str() == Some("/scenes/shared/objects/banner.yml")));

    let font_specs = defs
        .get(Value::String("font_specs".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("font_specs enum");
    assert!(font_specs.iter().any(|v| v.as_str() == Some("generic:3")));
    assert!(font_specs
        .iter()
        .any(|v| v.as_str() == Some("Mono Display:raster")));

    let image_paths = defs
        .get(Value::String("image_paths".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("image_paths enum");
    assert!(image_paths
        .iter()
        .any(|v| v.as_str() == Some("/assets/images/logo.png")));

    let model_paths = defs
        .get(Value::String("model_paths".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("model_paths enum");
    assert!(model_paths
        .iter()
        .any(|v| v.as_str() == Some("/scenes/intro/cube.obj")));

    let cutscene_refs = defs
        .get(Value::String("cutscene_refs".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("cutscene_refs enum");
    assert!(cutscene_refs
        .iter()
        .any(|v| v.as_str() == Some("intro-main")));
    assert!(cutscene_refs
        .iter()
        .any(|v| v.as_str() == Some("/cutscenes/intro-main.yml")));

    let template_names = defs
        .get(Value::String("template_names".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("template_names enum");
    assert!(template_names
        .iter()
        .any(|v| v.as_str() == Some("title-card")));
    assert!(template_names
        .iter()
        .any(|v| v.as_str() == Some("menu-item")));

    let object_overlay_yaml =
        render_schema_file(&object_overlay.value).expect("render object overlay");
    assert!(object_overlay_yaml.contains("../../../schemas/object.schema.yaml"));
    assert!(object_overlay_yaml.contains("const: blink"));
    assert!(object_overlay_yaml.contains("behavior:"));
    assert!(object_overlay_yaml.contains("visible_ms:"));
}

#[test]
fn render_schema_file_quotes_description_scalars_with_inline_colons() {
    let mut root = Mapping::new();
    root.insert(
        Value::String("description".to_string()),
        Value::String("External sidecar process configuration used when `mode: sidecar`.".into()),
    );

    let yaml = render_schema_file(&Value::Mapping(root)).expect("render schema");
    assert!(yaml.contains(
        "description: 'External sidecar process configuration used when `mode: sidecar`.'"
    ));
    serde_yaml::from_str::<Value>(&yaml).expect("rendered yaml should parse");
}

#[test]
fn committed_generated_schemas_are_current() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repo root");
    for mod_name in ["playground", "asteroids"] {
        let mod_root = repo_root.join("mods").join(mod_name);
        let files = generate_mod_schema_files(&mod_root).expect("generate committed schemas");
        assert!(!files.is_empty(), "expected schema files for {mod_name}");
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

#[test]
fn test_collect_font_names() {
    let temp = unique_temp_dir("font-test");
    fs::create_dir_all(temp.join("assets/fonts/mono")).unwrap();
    fs::write(
        temp.join("assets/fonts/mono/manifest.yaml"),
        "name: mono-bold\n",
    )
    .unwrap();

    let names = collect_font_names(&temp).unwrap();
    assert!(names.contains("mono-bold"));
}

#[test]
fn test_collect_image_paths() {
    let temp = unique_temp_dir("image-test");
    fs::create_dir_all(temp.join("assets/images/ui")).unwrap();
    fs::write(temp.join("assets/images/logo.png"), b"").unwrap();
    fs::write(temp.join("assets/images/ui/button.png"), b"").unwrap();

    let paths = collect_image_paths(&temp).unwrap();
    assert!(paths.contains("/assets/images/logo.png"));
    assert!(paths.contains("/assets/images/ui/button.png"));
}

#[test]
fn test_collect_model_paths() {
    let temp = unique_temp_dir("model-test");
    fs::create_dir_all(temp.join("scenes/intro")).unwrap();
    fs::create_dir_all(temp.join("assets/models")).unwrap();
    fs::write(temp.join("scenes/intro/cube.obj"), "").unwrap();
    fs::write(temp.join("assets/models/sphere.obj"), "").unwrap();

    let paths = collect_model_paths(&temp).unwrap();
    assert!(paths.contains("/scenes/intro/cube.obj"));
    assert!(paths.contains("/assets/models/sphere.obj"));
}

#[test]
fn test_collect_cutscene_refs() {
    let temp = unique_temp_dir("cutscene-test");
    fs::create_dir_all(temp.join("cutscenes/intro")).unwrap();
    fs::write(
        temp.join("cutscenes/intro/opening.yml"),
        "frames:\n  - source: /assets/images/logo.png\n    delay-ms: 100\n",
    )
    .unwrap();

    let refs = collect_cutscene_refs(&temp).unwrap();
    assert!(refs.contains("intro/opening"));
    assert!(refs.contains("/cutscenes/intro/opening.yml"));
}

#[test]
fn test_collect_sprite_ids() {
    let temp = unique_temp_dir("sprite-id-test");
    fs::create_dir_all(temp.join("scenes")).unwrap();
    fs::write(
        temp.join("scenes/test.yml"),
        "layers:\n  - sprites:\n      - id: logo\n        type: text\n        content: Test\n",
    )
    .unwrap();

    let ids = collect_sprite_ids(&temp).unwrap();
    assert!(ids.contains("logo"));
}

#[test]
fn test_collect_template_names() {
    let temp = unique_temp_dir("template-test");
    fs::create_dir_all(temp.join("scenes/intro/templates")).unwrap();
    fs::write(
        temp.join("scenes/intro/templates/button.yml"),
        "menu-button:\n  type: text\n  content: START\n",
    )
    .unwrap();

    let names = collect_template_names(&temp).unwrap();
    assert!(names.contains("menu-button"));
}

#[test]
fn test_behavior_schema_generation() {
    let behavior_schema = build_behavior_schema();

    let defs = behavior_schema
        .as_mapping()
        .and_then(|m| m.get(Value::String("$defs".to_string())))
        .and_then(Value::as_mapping)
        .expect("$defs in behaviors schema");

    let behavior_def = defs
        .get(Value::String("behavior".to_string()))
        .and_then(Value::as_mapping)
        .expect("behavior def");

    let one_of = behavior_def
        .get(Value::String("oneOf".to_string()))
        .and_then(Value::as_sequence)
        .expect("oneOf variants");

    assert!(!one_of.is_empty(), "should have behavior variants");

    // Check that at least one known behavior exists
    let has_blink = one_of.iter().any(|variant| {
        variant
            .as_mapping()
            .and_then(|m| m.get(Value::String("properties".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|props| props.get(Value::String("name".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|name_prop| name_prop.get(Value::String("const".to_string())))
            .and_then(Value::as_str)
            == Some("blink")
    });
    assert!(has_blink, "blink behavior should be in schema");
}

#[test]
fn test_animation_schema_generation() {
    let animation_schema = build_animation_schema();

    let defs = animation_schema
        .as_mapping()
        .and_then(|m| m.get(Value::String("$defs".to_string())))
        .and_then(Value::as_mapping)
        .expect("$defs in animations schema");

    let animation_def = defs
        .get(Value::String("animation".to_string()))
        .and_then(Value::as_mapping)
        .expect("animation def");

    let one_of = animation_def
        .get(Value::String("oneOf".to_string()))
        .and_then(Value::as_sequence)
        .expect("oneOf variants");

    assert!(!one_of.is_empty(), "should have animation variants");

    // Check that float animation exists
    let has_float = one_of.iter().any(|variant| {
        variant
            .as_mapping()
            .and_then(|m| m.get(Value::String("properties".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|props| props.get(Value::String("name".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|name_prop| name_prop.get(Value::String("const".to_string())))
            .and_then(Value::as_str)
            == Some("float")
    });
    assert!(has_float, "float animation should be in schema");
}

#[test]
fn test_input_profile_schema_generation() {
    let profile_schema = build_input_profile_schema();

    let defs = profile_schema
        .as_mapping()
        .and_then(|m| m.get(Value::String("$defs".to_string())))
        .and_then(Value::as_mapping)
        .expect("$defs in input-profiles schema");

    let profile_def = defs
        .get(Value::String("input_profile".to_string()))
        .and_then(Value::as_mapping)
        .expect("input_profile def");

    let enum_values = profile_def
        .get(Value::String("enum".to_string()))
        .and_then(Value::as_sequence)
        .expect("enum values");

    assert!(!enum_values.is_empty(), "should have profile values");

    // Check that known profiles exist
    let has_obj_viewer = enum_values.iter().any(|v| v.as_str() == Some("obj-viewer"));
    assert!(has_obj_viewer, "obj-viewer profile should be in schema");
}

#[test]
fn test_sugar_schema_generation() {
    let sugar_schema = build_sugar_schema();

    let defs = sugar_schema
        .as_mapping()
        .and_then(|m| m.get(Value::String("$defs".to_string())))
        .and_then(Value::as_mapping)
        .expect("$defs in sugar schema");

    // Check aliases
    let aliases = defs
        .get(Value::String("aliases".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("items".to_string())))
        .and_then(Value::as_sequence)
        .expect("aliases items");
    assert!(!aliases.is_empty(), "should have alias definitions");

    // Check shorthands
    let shorthands = defs
        .get(Value::String("shorthands".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("items".to_string())))
        .and_then(Value::as_sequence)
        .expect("shorthands items");
    assert!(!shorthands.is_empty(), "should have shorthand definitions");

    // Check that pause shorthand exists
    let has_pause = shorthands.iter().any(|sh| {
        sh.as_mapping()
            .and_then(|m| m.get(Value::String("name".to_string())))
            .and_then(Value::as_str)
            == Some("pause")
    });
    assert!(has_pause, "pause shorthand should be in schema");

    // Check normalizers
    let normalizers = defs
        .get(Value::String("normalizers".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get(Value::String("enum".to_string())))
        .and_then(Value::as_sequence)
        .expect("normalizers enum");
    assert!(!normalizers.is_empty(), "should have normalizer names");
}

#[test]
fn test_no_schema_drift() {
    // Verify that generated schemas include all runtime behaviors and animations
    use engine_core::authoring::catalog::{animation_catalog, behavior_catalog};
    use serde_yaml::Value;

    let behavior_schema = build_behavior_schema();
    let animation_schema = build_animation_schema();

    // Check behaviors
    let behavior_catalog = behavior_catalog();
    let defs = behavior_schema.get("$defs").expect("$defs");
    let defs_map = defs.as_mapping().expect("$defs as mapping");
    let behavior = defs_map
        .get(&Value::String("behavior".to_string()))
        .expect("behavior");
    let behavior_map = behavior.as_mapping().expect("behavior as mapping");
    let oneof = behavior_map
        .get(&Value::String("oneOf".to_string()))
        .expect("oneOf");
    let behavior_oneof = oneof.as_sequence().expect("oneOf as sequence");

    assert_eq!(
        behavior_oneof.len(),
        behavior_catalog.len(),
        "Generated schema should have oneOf entry for each behavior in catalog"
    );

    // Check animations
    let animation_catalog = animation_catalog();
    let defs = animation_schema.get("$defs").expect("$defs");
    let defs_map = defs.as_mapping().expect("$defs as mapping");
    let animation = defs_map
        .get(&Value::String("animation".to_string()))
        .expect("animation");
    let animation_map = animation.as_mapping().expect("animation as mapping");
    let oneof = animation_map
        .get(&Value::String("oneOf".to_string()))
        .expect("oneOf");
    let animation_oneof = oneof.as_sequence().expect("oneOf as sequence");

    assert_eq!(
        animation_oneof.len(),
        animation_catalog.len(),
        "Generated schema should have oneOf entry for each animation in catalog"
    );
}

#[test]
fn test_metadata_coverage() {
    // Verify that every behavior/animation metadata has required fields
    use engine_core::authoring::catalog::{animation_catalog, behavior_catalog};

    for (name, fields) in behavior_catalog() {
        assert!(
            !fields.is_empty(),
            "Behavior '{}' should have metadata fields",
            name
        );

        // Verify each field has description
        for field in fields {
            assert!(
                !field.description.is_empty(),
                "Behavior '{}' field '{}' should have description",
                name,
                field.name
            );
        }
    }

    for (name, fields) in animation_catalog() {
        assert!(
            !fields.is_empty(),
            "Animation '{}' should have metadata fields",
            name
        );

        for field in fields {
            assert!(
                !field.description.is_empty(),
                "Animation '{}' field '{}' should have description",
                name,
                field.name
            );
        }
    }
}

#[test]
fn test_behavior_schema_preserves_required_and_stage_list_shapes() {
    let schema = build_behavior_schema();
    let defs = schema
        .get("$defs")
        .and_then(Value::as_mapping)
        .expect("$defs mapping");
    let behavior = defs
        .get(Value::String("behavior".to_string()))
        .and_then(Value::as_mapping)
        .expect("behavior def");
    let variants = behavior
        .get(Value::String("oneOf".to_string()))
        .and_then(Value::as_sequence)
        .expect("behavior oneOf");

    let follow = variants
        .iter()
        .find(|variant| {
            variant
                .as_mapping()
                .and_then(|variant| variant.get(Value::String("properties".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|props| props.get(Value::String("name".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|name| name.get(Value::String("const".to_string())))
                .and_then(Value::as_str)
                == Some("follow")
        })
        .and_then(Value::as_mapping)
        .expect("follow variant");
    let follow_params = follow
        .get(Value::String("properties".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|props| props.get(Value::String("params".to_string())))
        .and_then(Value::as_mapping)
        .expect("follow params");
    let follow_required = follow_params
        .get(Value::String("required".to_string()))
        .and_then(Value::as_sequence)
        .expect("follow required");
    assert!(follow_required
        .iter()
        .any(|value| value.as_str() == Some("target")));

    let stage_visibility = variants
        .iter()
        .find(|variant| {
            variant
                .as_mapping()
                .and_then(|variant| variant.get(Value::String("properties".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|props| props.get(Value::String("name".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|name| name.get(Value::String("const".to_string())))
                .and_then(Value::as_str)
                == Some("stage-visibility")
        })
        .and_then(Value::as_mapping)
        .expect("stage-visibility variant");
    let stages_schema = stage_visibility
        .get(Value::String("properties".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|props| props.get(Value::String("params".to_string())))
        .and_then(Value::as_mapping)
        .and_then(|params| params.get(Value::String("properties".to_string())))
        .and_then(Value::as_mapping)
        .and_then(|props| props.get(Value::String("stages".to_string())))
        .and_then(Value::as_mapping)
        .expect("stages schema");

    assert_eq!(
        stages_schema
            .get(Value::String("type".to_string()))
            .and_then(Value::as_str),
        Some("array")
    );
    let items = stages_schema
        .get(Value::String("items".to_string()))
        .and_then(Value::as_mapping)
        .expect("stages items");
    let enum_values = items
        .get(Value::String("enum".to_string()))
        .and_then(Value::as_sequence)
        .expect("stages enum");
    assert!(enum_values
        .iter()
        .any(|value| value.as_str() == Some("on-leave")));
    assert!(enum_values
        .iter()
        .any(|value| value.as_str() == Some("done")));
}
