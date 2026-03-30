//! Overlay and conditional patch builders for schema generation.

use engine_core::authoring::catalog::behavior_catalog;
use engine_core::authoring::metadata::FieldMetadata;
use serde_yaml::{Mapping, Value};

use super::helpers::{
    array_items_ref, field_metadata_to_schema, mapping_with, non_empty_string_schema, null_schema,
    nullable_suggested_string_refs, object_additional_properties_ref, object_schema, schema_ref,
    suggested_enum_strings, suggested_string_refs,
};

pub(super) fn scene_overlay_patch() -> Mapping {
    let mut patches = Vec::new();
    patches.push(conditional_property_overlay(
        "cutscene-ref",
        suggested_string_refs(&["./catalog.yaml#/$defs/cutscene_refs"]),
    ));
    patches.push(conditional_property_overlay(
        "cutscene_ref",
        suggested_string_refs(&["./catalog.yaml#/$defs/cutscene_refs"]),
    ));
    patches.push(conditional_property_overlay(
        "next",
        nullable_suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    ));
    patches.push(conditional_property_overlay(
        "menu-options",
        menu_options_overlay(),
    ));
    patches.push(conditional_property_overlay(
        "menu_options",
        menu_options_overlay(),
    ));
    patches.push(conditional_property_overlay("objects", objects_overlay()));
    patches.push(conditional_property_overlay("input", scene_input_overlay()));
    patches.push(conditional_property_overlay(
        "behaviors",
        array_items_ref("#/$defs/behavior_overlay"),
    ));
    patches.push(conditional_property_overlay(
        "logic",
        schema_ref("#/$defs/scene_logic_overlay"),
    ));
    patches.push(conditional_property_overlay(
        "layers",
        scene_layers_overlay(),
    ));
    patches.push(conditional_property_overlay(
        "templates",
        object_additional_properties_ref("#/$defs/sprite_overlay"),
    ));
    patches.push(conditional_property_overlay(
        "stages",
        schema_ref("#/$defs/scene_stages_overlay"),
    ));
    patches.push(conditional_property_overlay(
        "effect-presets",
        object_additional_properties_ref("./effects.yaml#/items"),
    ));
    patches.push(conditional_property_overlay(
        "effect_presets",
        object_additional_properties_ref("./effects.yaml#/items"),
    ));
    patches.push(conditional_property_overlay(
        "effect-presets-ref",
        suggested_string_refs(&["./catalog.yaml#/$defs/effect_refs"]),
    ));
    patches.push(conditional_property_overlay(
        "effect_presets_ref",
        suggested_string_refs(&["./catalog.yaml#/$defs/effect_refs"]),
    ));
    patches.push(conditional_property_overlay(
        "postfx",
        array_items_ref("./effects.yaml#/items"),
    ));

    let mut root = Mapping::new();
    root.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    root.insert(Value::String("allOf".to_string()), Value::Sequence(patches));
    root
}

pub(super) fn conditional_property_overlay(property_name: &str, property_schema: Value) -> Value {
    let mut if_block = Mapping::new();
    if_block.insert(
        Value::String("required".to_string()),
        Value::Sequence(vec![Value::String(property_name.to_string())]),
    );

    let mut props = Mapping::new();
    props.insert(Value::String(property_name.to_string()), property_schema);

    let mut then_block = Mapping::new();
    then_block.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );

    let mut block = Mapping::new();
    block.insert(Value::String("if".to_string()), Value::Mapping(if_block));
    block.insert(
        Value::String("then".to_string()),
        Value::Mapping(then_block),
    );
    Value::Mapping(block)
}

pub(super) fn menu_options_overlay() -> Value {
    let mut option_props = Mapping::new();
    option_props.insert(
        Value::String("next".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    option_props.insert(
        Value::String("scene".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    option_props.insert(
        Value::String("to".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    let mut items = Mapping::new();
    items.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    items.insert(
        Value::String("properties".to_string()),
        Value::Mapping(option_props),
    );
    Value::Mapping(mapping_with("items", Value::Mapping(items)))
}

pub(super) fn objects_overlay() -> Value {
    Value::Mapping(mapping_with(
        "items",
        Value::Mapping(object_instance_overlay_patch()),
    ))
}

pub(super) fn scene_layers_overlay() -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    map.insert(
        Value::String("items".to_string()),
        Value::Mapping(mapping_with(
            "anyOf",
            Value::Sequence(vec![
                schema_ref("#/$defs/layer_overlay"),
                schema_ref("#/$defs/layer_ref_instance_overlay"),
            ]),
        )),
    );
    Value::Mapping(map)
}

pub(super) fn scene_input_overlay() -> Value {
    let mut obj_viewer_props = Mapping::new();
    obj_viewer_props.insert(
        Value::String("sprite_id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    let mut obj_viewer = Mapping::new();
    obj_viewer.insert(
        Value::String("properties".to_string()),
        Value::Mapping(obj_viewer_props),
    );

    let mut terminal_shell_props = Mapping::new();
    terminal_shell_props.insert(
        Value::String("prompt-sprite-id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    terminal_shell_props.insert(
        Value::String("prompt_sprite_id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    terminal_shell_props.insert(
        Value::String("output-sprite-id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    terminal_shell_props.insert(
        Value::String("output_sprite_id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    let mut terminal_shell = Mapping::new();
    terminal_shell.insert(
        Value::String("properties".to_string()),
        Value::Mapping(terminal_shell_props),
    );

    let mut input_props = Mapping::new();
    input_props.insert(
        Value::String("obj-viewer".to_string()),
        Value::Mapping(obj_viewer),
    );
    input_props.insert(
        Value::String("terminal-shell".to_string()),
        Value::Mapping(terminal_shell),
    );

    let mut input = Mapping::new();
    input.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    input.insert(
        Value::String("properties".to_string()),
        Value::Mapping(input_props),
    );
    Value::Mapping(input)
}

pub(super) fn object_instance_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("use".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/object_refs"]),
    );
    props.insert(
        Value::String("ref".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/object_refs"]),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch
}

pub(super) fn object_doc_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("logic".to_string()),
        schema_ref("#/$defs/object_logic_overlay"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch
}

pub(super) fn object_logic_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("behavior".to_string()),
        suggested_enum_strings(
            behavior_catalog()
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect(),
        ),
    );

    let conditional_blocks: Vec<Value> = behavior_catalog()
        .into_iter()
        .map(|(behavior_name, fields)| {
            let mut if_props = Mapping::new();
            if_props.insert(
                Value::String("behavior".to_string()),
                Value::Mapping(mapping_with(
                    "const",
                    Value::String(behavior_name.to_string()),
                )),
            );
            let mut if_block = Mapping::new();
            if_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(if_props),
            );

            let mut then_props = Mapping::new();
            then_props.insert(
                Value::String("params".to_string()),
                behavior_params_schema(&fields),
            );
            let mut then_block = Mapping::new();
            then_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(then_props),
            );

            let mut block = Mapping::new();
            block.insert(Value::String("if".to_string()), Value::Mapping(if_block));
            block.insert(
                Value::String("then".to_string()),
                Value::Mapping(then_block),
            );
            Value::Mapping(block)
        })
        .collect();

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    if !conditional_blocks.is_empty() {
        patch.insert(
            Value::String("allOf".to_string()),
            Value::Sequence(conditional_blocks),
        );
    }

    let mut overlay = Mapping::new();
    overlay.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/object.schema.yaml#/properties/logic"),
            Value::Mapping(patch),
        ]),
    );
    overlay
}

pub(super) fn scene_logic_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("behavior".to_string()),
        suggested_enum_strings(
            behavior_catalog()
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect(),
        ),
    );
    props.insert(
        Value::String("src".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/yaml_paths"]),
    );

    let conditional_blocks: Vec<Value> = behavior_catalog()
        .into_iter()
        .map(|(behavior_name, fields)| {
            let mut if_props = Mapping::new();
            if_props.insert(
                Value::String("behavior".to_string()),
                Value::Mapping(mapping_with(
                    "const",
                    Value::String(behavior_name.to_string()),
                )),
            );
            let mut if_block = Mapping::new();
            if_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(if_props),
            );

            let mut then_props = Mapping::new();
            then_props.insert(
                Value::String("params".to_string()),
                behavior_params_schema(&fields),
            );
            let mut then_block = Mapping::new();
            then_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(then_props),
            );

            let mut block = Mapping::new();
            block.insert(Value::String("if".to_string()), Value::Mapping(if_block));
            block.insert(
                Value::String("then".to_string()),
                Value::Mapping(then_block),
            );
            Value::Mapping(block)
        })
        .collect();

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    if !conditional_blocks.is_empty() {
        patch.insert(
            Value::String("allOf".to_string()),
            Value::Sequence(conditional_blocks),
        );
    }

    let mut overlay = Mapping::new();
    overlay.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml#/properties/logic"),
            Value::Mapping(patch),
        ]),
    );
    overlay
}

pub(super) fn behavior_params_schema(fields: &[engine_core::authoring::metadata::FieldMetadata]) -> Value {
    use engine_core::authoring::metadata::Requirement;

    let mut props = Mapping::new();
    let mut required = Vec::new();
    for field in fields {
        props.insert(
            Value::String(field.name.to_string()),
            field_metadata_to_schema(field),
        );
        if matches!(field.requirement, Requirement::Required) {
            required.push(Value::String(field.name.to_string()));
        }
    }

    let mut schema = Mapping::new();
    schema.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    schema.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    schema.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    if !required.is_empty() {
        schema.insert(
            Value::String("required".to_string()),
            Value::Sequence(required),
        );
    }
    Value::Mapping(schema)
}

pub(super) fn shared_overlay_defs() -> Mapping {
    let mut defs = Mapping::new();
    defs.insert(
        Value::String("step_overlay".to_string()),
        Value::Mapping(step_overlay_def()),
    );
    defs.insert(
        Value::String("stage_overlay".to_string()),
        Value::Mapping(stage_overlay_def()),
    );
    defs.insert(
        Value::String("scene_stages_overlay".to_string()),
        Value::Mapping(scene_stages_overlay_def()),
    );
    defs.insert(
        Value::String("lifecycle_stages_overlay".to_string()),
        Value::Mapping(lifecycle_stages_overlay_def()),
    );
    defs.insert(
        Value::String("sprite_overlay".to_string()),
        Value::Mapping(sprite_overlay_def()),
    );
    defs.insert(
        Value::String("behavior_overlay".to_string()),
        Value::Mapping(behavior_overlay_def()),
    );
    defs.insert(
        Value::String("scene_logic_overlay".to_string()),
        Value::Mapping(scene_logic_overlay_def()),
    );
    defs.insert(
        Value::String("layer_overlay".to_string()),
        Value::Mapping(layer_overlay_def()),
    );
    defs.insert(
        Value::String("layer_ref_instance_overlay".to_string()),
        Value::Mapping(layer_ref_instance_overlay_def()),
    );
    defs
}

pub(super) fn step_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("effects".to_string()),
        array_items_ref("./effects.yaml#/items"),
    );

    let mut step = Mapping::new();
    step.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    step.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    step.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/scene.schema.yaml#/$defs/step",
        )]),
    );
    step
}

pub(super) fn scene_stages_overlay_def() -> Mapping {
    lifecycle_stages_overlay_def_with_base("../../../schemas/scene.schema.yaml#/$defs/scene_stages")
}

pub(super) fn lifecycle_stages_overlay_def() -> Mapping {
    lifecycle_stages_overlay_def_with_base("../../../schemas/scene.schema.yaml#/$defs/layer_stages")
}

pub(super) fn lifecycle_stages_overlay_def_with_base(base_ref: &str) -> Mapping {
    let mut stage_props = Mapping::new();
    for stage in ["on_enter", "on_idle", "on_leave"] {
        stage_props.insert(
            Value::String(stage.to_string()),
            schema_ref("#/$defs/stage_overlay"),
        );
    }

    let mut stages = Mapping::new();
    stages.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    stages.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    stages.insert(
        Value::String("properties".to_string()),
        Value::Mapping(stage_props),
    );
    stages.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(base_ref)]),
    );
    stages
}

pub(super) fn stage_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("steps".to_string()),
        array_items_ref("#/$defs/step_overlay"),
    );

    let mut stage = Mapping::new();
    stage.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    stage.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    stage.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/scene.schema.yaml#/$defs/stage",
        )]),
    );
    stage
}

pub(super) fn layer_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("stages".to_string()),
        schema_ref("#/$defs/lifecycle_stages_overlay"),
    );
    props.insert(
        Value::String("behaviors".to_string()),
        array_items_ref("#/$defs/behavior_overlay"),
    );
    props.insert(Value::String("objects".to_string()), objects_overlay());
    props.insert(
        Value::String("sprites".to_string()),
        array_items_ref("#/$defs/sprite_overlay"),
    );

    let mut layer = Mapping::new();
    layer.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    layer.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    layer.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/scene.schema.yaml#/$defs/layer",
        )]),
    );
    layer
}

pub(super) fn layer_ref_instance_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("use".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/layer_refs"]),
    );
    props.insert(
        Value::String("ref".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/layer_refs"]),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/scene.schema.yaml#/$defs/layer_ref_instance",
        )]),
    );
    patch
}

pub(super) fn sprite_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("use".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/template_names"]),
    );
    props.insert(Value::String("args".to_string()), object_schema());
    props.insert(
        Value::String("source".to_string()),
        suggested_string_refs(&[
            "./catalog.yaml#/$defs/image_paths",
            "./catalog.yaml#/$defs/model_paths",
        ]),
    );
    props.insert(
        Value::String("src".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene3d_paths"]),
    );
    props.insert(
        Value::String("font".to_string()),
        nullable_suggested_string_refs(&["./catalog.yaml#/$defs/font_specs"]),
    );
    props.insert(
        Value::String("stages".to_string()),
        schema_ref("#/$defs/lifecycle_stages_overlay"),
    );
    props.insert(
        Value::String("behaviors".to_string()),
        array_items_ref("#/$defs/behavior_overlay"),
    );
    props.insert(
        Value::String("children".to_string()),
        array_items_ref("#/$defs/sprite_overlay"),
    );

    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    sprite.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    sprite.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/scene.schema.yaml#/$defs/sprite",
        )]),
    );
    sprite
}

pub(super) fn behavior_overlay_def() -> Mapping {
    let variants = behavior_catalog()
        .into_iter()
        .map(|(behavior_name, fields)| behavior_variant_overlay(behavior_name, &fields))
        .collect();

    let mut behavior = Mapping::new();
    behavior.insert(
        Value::String("oneOf".to_string()),
        Value::Sequence(variants),
    );
    behavior
}

pub(super) fn behavior_variant_overlay(
    behavior_name: &str,
    fields: &[engine_core::authoring::metadata::FieldMetadata],
) -> Value {
    use engine_core::authoring::metadata::Requirement;

    let mut props = Mapping::new();
    props.insert(
        Value::String("name".to_string()),
        Value::Mapping(mapping_with(
            "const",
            Value::String(behavior_name.to_string()),
        )),
    );

    let mut params_props = Mapping::new();
    let mut params_required = Vec::new();
    for field in fields {
        if matches!(field.name, "target" | "sprite_id") {
            let mut variants = vec![schema_ref("./catalog.yaml#/$defs/sprite_ids")];
            variants.push(field_metadata_to_schema(field));
            params_props.insert(
                Value::String(field.name.to_string()),
                Value::Mapping(mapping_with("anyOf", Value::Sequence(variants))),
            );
        } else {
            params_props.insert(
                Value::String(field.name.to_string()),
                field_metadata_to_schema(field),
            );
        }
        if matches!(field.requirement, Requirement::Required) {
            params_required.push(Value::String(field.name.to_string()));
        }
    }

    let mut params = Mapping::new();
    params.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    params.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    params.insert(
        Value::String("properties".to_string()),
        Value::Mapping(params_props),
    );
    if !params_required.is_empty() {
        params.insert(
            Value::String("required".to_string()),
            Value::Sequence(params_required),
        );
    }
    props.insert(Value::String("params".to_string()), Value::Mapping(params));

    let mut variant = Mapping::new();
    variant.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    variant.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    variant.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    variant.insert(
        Value::String("required".to_string()),
        Value::Sequence(vec![Value::String("name".to_string())]),
    );
    variant.insert(
        Value::String("title".to_string()),
        Value::String(format!("{behavior_name} behavior overlay")),
    );

    Value::Mapping(variant)
}

