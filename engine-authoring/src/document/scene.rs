//! Authored scene document normalization before conversion into the runtime
//! [`Scene`] model.

use engine_core::scene::template::expand_scene_templates;
use engine_core::scene::{resolve_ui_theme_or_default, GameStateBinding, PaletteBinding, Scene};
use serde::de::Error as _;
use serde::Deserialize;
use serde_yaml::{Mapping, Number, Value};

use super::scene_helpers::*;

/// Authored scene document kept as raw YAML until scene-specific shorthands,
/// aliases, and templates are normalized.
///
/// # Purpose
///
/// `SceneDocument` is the authored-vs-runtime boundary for scenes. Repositories
/// and higher-level compilers can deserialize loose YAML into this type first,
/// then call [`SceneDocument::compile`] to produce the strict runtime [`Scene`].
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SceneDocument {
    pub raw: serde_yaml::Value,
}

impl SceneDocument {
    /// Normalizes authored YAML and materializes the runtime [`Scene`] model.
    ///
    /// In debug mode, validates sprite timeline and text-layout authoring and
    /// logs warnings for configurations that are structurally valid but likely
    /// ineffective at runtime.
    pub fn compile(self) -> Result<Scene, serde_yaml::Error> {
        let mut normalized = self.raw;
        normalize_scene_value(&mut normalized)?;
        let palette_bindings = extract_palette_bindings(&mut normalized);
        let game_state_bindings = extract_game_state_bindings(&mut normalized);
        let mut scene: Scene = serde_yaml::from_value(normalized)?;
        scene.palette_bindings = palette_bindings;
        scene.game_state_bindings = game_state_bindings;

        #[cfg(debug_assertions)]
        {
            use crate::validate::{validate_sprite_timeline, validate_text_layout_semantics};
            let diagnostics = validate_sprite_timeline(&scene);
            for diag in diagnostics {
                match diag {
                    crate::validate::TimelineDiagnostic::SpriteAppearsAfterSceneEnd {
                        layer_name,
                        sprite_index,
                        appear_at_ms,
                        scene_duration_ms,
                    } => {
                        eprintln!(
                            "⚠️  Scene '{}': sprite #{} in layer '{}' has appear_at_ms={} but on_enter ends at {}ms (sprite will never be visible)",
                            scene.id, sprite_index, layer_name, appear_at_ms, scene_duration_ms
                        );
                    }
                    crate::validate::TimelineDiagnostic::SpriteDisappearsBeforeAppear {
                        layer_name,
                        sprite_index,
                        appear_at_ms,
                        disappear_at_ms,
                    } => {
                        eprintln!(
                            "⚠️  Scene '{}': sprite #{} in layer '{}' disappears at {}ms before appearing at {}ms (always hidden)",
                            scene.id, sprite_index, layer_name, disappear_at_ms, appear_at_ms
                        );
                    }
                }
            }

            let text_layout_diagnostics = validate_text_layout_semantics(&scene);
            for diag in text_layout_diagnostics {
                match diag {
                    crate::validate::TextLayoutDiagnostic::EllipsisWithoutBounds {
                        layer_name,
                        sprite_index,
                    } => {
                        eprintln!(
                            "⚠️  Scene '{}': text sprite #{} in layer '{}' uses overflow-mode=ellipsis without max-width or line-clamp (ellipsis has no bounded layout contract)",
                            scene.id, sprite_index, layer_name
                        );
                    }
                    crate::validate::TextLayoutDiagnostic::LineClampWithoutWrap {
                        layer_name,
                        sprite_index,
                        line_clamp,
                    } => {
                        eprintln!(
                            "⚠️  Scene '{}': text sprite #{} in layer '{}' sets line-clamp={} with wrap-mode=none (clamp has no authored multi-line contract)",
                            scene.id, sprite_index, layer_name, line_clamp
                        );
                    }
                    crate::validate::TextLayoutDiagnostic::ReserveWidthTooSmall {
                        layer_name,
                        sprite_index,
                        reserve_width_ch,
                        max_width,
                    } => {
                        eprintln!(
                            "⚠️  Scene '{}': text sprite #{} in layer '{}' sets reserve-width-ch={} below max-width={} (reserved HUD footprint is smaller than the visible line budget)",
                            scene.id, sprite_index, layer_name, reserve_width_ch, max_width
                        );
                    }
                }
            }
        }

        Ok(scene)
    }
}

/// Walks the normalized YAML tree looking for `"@palette.<key>"` values in
/// `fg_colour` / `bg_colour` sprite fields. For each found:
/// - records a [`PaletteBinding`] (target sprite id → palette key),
/// - replaces the `"@palette.*"` string with `"white"` so `TermColour`
///   deserialization succeeds.
fn extract_palette_bindings(root: &mut Value) -> Vec<PaletteBinding> {
    let mut bindings = Vec::new();
    let Some(scene_map) = root.as_mapping_mut() else {
        return bindings;
    };
    let Some(layers) = scene_map.get_mut(Value::String("layers".to_string())) else {
        return bindings;
    };
    let Some(layer_seq) = layers.as_sequence_mut() else {
        return bindings;
    };
    for layer in layer_seq.iter_mut() {
        let Some(layer_map) = layer.as_mapping_mut() else {
            continue;
        };
        let Some(sprites) = layer_map.get_mut(Value::String("sprites".to_string())) else {
            continue;
        };
        collect_sprite_palette_bindings(sprites, &mut bindings);
    }
    bindings
}

fn collect_sprite_palette_bindings(sprites: &mut Value, bindings: &mut Vec<PaletteBinding>) {
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        return;
    };
    for sprite in sprite_seq.iter_mut() {
        let Some(sprite_map) = sprite.as_mapping_mut() else {
            continue;
        };
        let id = sprite_map
            .get(Value::String("id".to_string()))
            .and_then(Value::as_str)
            .map(str::to_owned);
        for (yaml_field, prop_path) in [
            ("fg_colour", "style.fg"),
            ("bg_colour", "style.bg"),
            ("border-colour", "style.border"),
            ("shadow-colour", "style.shadow"),
        ] {
            let field_key = Value::String(yaml_field.to_string());
            if let Some(val) = sprite_map.get(&field_key).and_then(Value::as_str) {
                if let Some(key) = val.strip_prefix("@palette.") {
                    if let Some(target) = &id {
                        bindings.push(PaletteBinding {
                            target: target.clone(),
                            prop: prop_path.to_owned(),
                            key: key.to_owned(),
                        });
                        sprite_map.insert(field_key, Value::String("white".to_string()));
                    }
                }
            }
        }
        if let Some(children) = sprite_map.get_mut(Value::String("children".to_string())) {
            collect_sprite_palette_bindings(children, bindings);
        }
    }
}

/// Walks the normalized YAML tree looking for `"@game_state.<path>"` values in sprite
/// `content` fields. For each found:
/// - records a [`GameStateBinding`] (target sprite id → game_state JSON pointer path),
/// - replaces the `"@game_state.*"` string with `""` so deserialization succeeds.
fn extract_game_state_bindings(root: &mut Value) -> Vec<GameStateBinding> {
    let mut bindings = Vec::new();
    let Some(scene_map) = root.as_mapping_mut() else {
        return bindings;
    };
    let Some(layers) = scene_map.get_mut(Value::String("layers".to_string())) else {
        return bindings;
    };
    let Some(layer_seq) = layers.as_sequence_mut() else {
        return bindings;
    };
    for layer in layer_seq.iter_mut() {
        let Some(layer_map) = layer.as_mapping_mut() else {
            continue;
        };
        let Some(sprites) = layer_map.get_mut(Value::String("sprites".to_string())) else {
            continue;
        };
        collect_sprite_game_state_bindings(sprites, &mut bindings);
    }
    bindings
}

fn collect_sprite_game_state_bindings(sprites: &mut Value, bindings: &mut Vec<GameStateBinding>) {
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        return;
    };
    for sprite in sprite_seq.iter_mut() {
        let Some(sprite_map) = sprite.as_mapping_mut() else {
            continue;
        };
        let id = sprite_map
            .get(Value::String("id".to_string()))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let content_key = Value::String("content".to_string());
        if let Some(val) = sprite_map.get(&content_key).and_then(Value::as_str) {
            if let Some(key) = val.strip_prefix("@game_state.") {
                if let Some(target) = &id {
                    let path = if key.starts_with('/') {
                        key.to_owned()
                    } else {
                        format!("/{key}")
                    };
                    bindings.push(GameStateBinding {
                        target: target.clone(),
                        path,
                    });
                    sprite_map.insert(content_key, Value::String(String::new()));
                }
            }
        }
        if let Some(children) = sprite_map.get_mut(Value::String("children".to_string())) {
            collect_sprite_game_state_bindings(children, bindings);
        }
    }
}

fn normalize_scene_value(root: &mut Value) -> Result<(), serde_yaml::Error> {
    let Some(scene) = root.as_mapping_mut() else {
        return Ok(());
    };

    apply_alias(scene, "bg", "bg_colour");
    if !scene.contains_key(Value::String("planet-spec".to_string())) {
        if let Some(value) = scene.remove(Value::String("planet_spec".to_string())) {
            scene.insert(Value::String("planet-spec".to_string()), value);
        }
    }
    if !scene.contains_key(Value::String("planet-spec-ref".to_string())) {
        if let Some(value) = scene.remove(Value::String("planet_spec_ref".to_string())) {
            scene.insert(Value::String("planet-spec-ref".to_string()), value);
        }
    }
    expand_scene_templates(scene);

    if let Some(stages) = scene.get_mut(Value::String("stages".to_string())) {
        normalize_stages(stages);
    }
    normalize_menu_options(scene);
    expand_menu_ui(scene);
    let scene_ui_theme = scene
        .get(Value::String("ui".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|ui| map_get_str(ui, &["theme"]))
        .map(str::trim)
        .filter(|theme| !theme.is_empty())
        .map(ToString::to_string);
    let scene_sprite_defaults = scene
        .get(Value::String("sprite-defaults".to_string()))
        .and_then(Value::as_mapping)
        .cloned();
    if let Some(layers) = scene.get_mut(Value::String("layers".to_string())) {
        normalize_layers(
            layers,
            scene_sprite_defaults.as_ref(),
            scene_ui_theme.as_deref(),
        )?;
    }
    scene.remove(Value::String("sprite-defaults".to_string()));
    Ok(())
}

fn normalize_stages(stages: &mut Value) {
    let Some(stages_map) = stages.as_mapping_mut() else {
        return;
    };
    for key in ["on_enter", "on_idle", "on_leave"] {
        if let Some(stage) = stages_map.get_mut(Value::String(key.to_string())) {
            normalize_stage(stage);
        }
    }
}

/// Converts `pause: duration` shorthand into `{duration, effects: []}`.
///
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
fn normalize_stage(stage: &mut Value) {
    let Some(stage_map) = stage.as_mapping_mut() else {
        return;
    };
    let Some(steps) = stage_map.get_mut(Value::String("steps".to_string())) else {
        return;
    };
    let Some(steps_seq) = steps.as_sequence_mut() else {
        return;
    };

    for step in steps_seq {
        let Some(step_map) = step.as_mapping_mut() else {
            continue;
        };
        let Some(pause) = step_map.remove(Value::String("pause".to_string())) else {
            continue;
        };
        if step_map.contains_key(Value::String("effects".to_string())) {
            continue;
        }
        let duration_ms = parse_duration_ms(&pause).unwrap_or(0);
        step_map.insert(
            Value::String("duration".to_string()),
            Value::Number(Number::from(duration_ms)),
        );
        step_map.insert(
            Value::String("effects".to_string()),
            Value::Sequence(Vec::new()),
        );
    }
}

/// Recursively normalizes all sprites in all layers.
///
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
fn normalize_layers(
    layers: &mut Value,
    inherited_defaults: Option<&Mapping>,
    scene_theme: Option<&str>,
) -> Result<(), serde_yaml::Error> {
    let Some(layer_seq) = layers.as_sequence_mut() else {
        return Ok(());
    };
    for layer in layer_seq {
        let Some(layer_map) = layer.as_mapping_mut() else {
            continue;
        };
        let layer_defaults = merge_defaults(
            inherited_defaults,
            layer_map
                .get(Value::String("sprite-defaults".to_string()))
                .and_then(Value::as_mapping),
        );
        let Some(sprites) = layer_map.get_mut(Value::String("sprites".to_string())) else {
            continue;
        };
        normalize_sprites(sprites, layer_defaults.as_ref(), scene_theme)?;
        layer_map.remove(Value::String("sprite-defaults".to_string()));
    }
    Ok(())
}

/// Applies aliases and anchor expansions to sprite fields.
///
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
fn normalize_sprites(
    sprites: &mut Value,
    inherited_defaults: Option<&Mapping>,
    scene_theme: Option<&str>,
) -> Result<(), serde_yaml::Error> {
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        return Ok(());
    };
    let mut out = Vec::with_capacity(sprite_seq.len());
    for mut sprite in std::mem::take(sprite_seq) {
        let Some(sprite_map) = sprite.as_mapping_mut() else {
            out.push(sprite);
            continue;
        };
        let local_defaults = sprite_map
            .get(Value::String("sprite-defaults".to_string()))
            .and_then(Value::as_mapping)
            .cloned();
        apply_defaults(sprite_map, inherited_defaults);
        if is_sprite_type(sprite_map, "frame-sequence") {
            let seq_defaults = merge_defaults(inherited_defaults, local_defaults.as_ref());
            let mut expanded = expand_frame_sequence(sprite_map, seq_defaults.as_ref())?;
            out.append(&mut expanded);
            continue;
        }
        if is_sprite_type(sprite_map, "window") {
            let window_defaults = merge_defaults(inherited_defaults, local_defaults.as_ref());
            let mut expanded =
                expand_window_sprite(sprite_map, window_defaults.as_ref(), scene_theme)?;
            out.append(&mut expanded);
            continue;
        }
        if is_sprite_type(sprite_map, "scroll-list") {
            let list_defaults = merge_defaults(inherited_defaults, local_defaults.as_ref());
            let mut expanded =
                expand_scroll_list_sprite(sprite_map, list_defaults.as_ref(), scene_theme)?;
            out.append(&mut expanded);
            continue;
        }

        apply_alias(sprite_map, "fg", "fg_colour");
        apply_alias(sprite_map, "bg", "bg_colour");
        apply_at_anchor(sprite_map);
        normalize_expression_fields(sprite_map);

        if matches!(
            sprite_map
                .get(Value::String("type".to_string()))
                .and_then(Value::as_str),
            Some("grid" | "flex" | "panel")
        ) {
            if let Some(children) = sprite_map.get_mut(Value::String("children".to_string())) {
                let child_defaults = merge_defaults(inherited_defaults, local_defaults.as_ref());
                normalize_sprites(children, child_defaults.as_ref(), scene_theme)?;
            }
        }

        sprite_map.remove(Value::String("sprite-defaults".to_string()));
        out.push(sprite);
    }
    *sprite_seq = out;
    Ok(())
}

/// Expands `to: scene_id` shorthand into `{scene, next}`.
///
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
fn normalize_menu_options(scene: &mut Mapping) {
    for key in ["menu-options", "menu_options"] {
        let Some(options) = scene.get_mut(Value::String(key.to_string())) else {
            continue;
        };
        let Some(seq) = options.as_sequence_mut() else {
            continue;
        };
        for option in seq {
            let Some(option_map) = option.as_mapping_mut() else {
                continue;
            };
            let Some(to_value) = option_map.get(Value::String("to".to_string())).cloned() else {
                continue;
            };
            option_map
                .entry(Value::String("scene".to_string()))
                .or_insert_with(|| to_value.clone());
            option_map
                .entry(Value::String("next".to_string()))
                .or_insert(to_value);
        }
    }
}

fn expand_menu_ui(scene: &mut Mapping) {
    let Some(menu_ui_cfg) = scene
        .get(Value::String("menu-ui".to_string()))
        .and_then(Value::as_mapping)
        .cloned()
    else {
        return;
    };
    let options = collect_menu_options(scene);
    if options.is_empty() {
        scene.remove(Value::String("menu-ui".to_string()));
        return;
    }
    let Some(layer_map) = resolve_menu_ui_target_layer(scene, &menu_ui_cfg) else {
        scene.remove(Value::String("menu-ui".to_string()));
        return;
    };
    let sprites = layer_map
        .entry(Value::String("sprites".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        scene.remove(Value::String("menu-ui".to_string()));
        return;
    };

    let grid_id = cfg_str(&menu_ui_cfg, &["grid-id", "grid_id"]).unwrap_or("menu-grid");
    let item_prefix =
        cfg_str(&menu_ui_cfg, &["item-prefix", "item_prefix"]).unwrap_or("menu-item-");
    let font = cfg_str(&menu_ui_cfg, &["font"]).unwrap_or("generic:1");
    let at = cfg_str(&menu_ui_cfg, &["at"]).unwrap_or("cc");
    let window = cfg_u64(&menu_ui_cfg, &["window"]).unwrap_or(5).max(1);
    let step_y = cfg_u64(&menu_ui_cfg, &["step-y", "step_y"])
        .unwrap_or(2)
        .max(1);
    let endless = cfg_bool(&menu_ui_cfg, &["endless"]).unwrap_or(true);
    let arrows = cfg_bool(&menu_ui_cfg, &["arrows"]).unwrap_or(true);
    let width = cfg_u64(&menu_ui_cfg, &["width"]).unwrap_or(56);
    let height = cfg_u64(&menu_ui_cfg, &["height"]).unwrap_or(10);
    let gap_y = cfg_u64(&menu_ui_cfg, &["gap-y", "gap_y"]).unwrap_or(2);
    let fg_selected = cfg_str(&menu_ui_cfg, &["fg-selected", "fg_selected"]).unwrap_or("white");
    let fg_alt_a = cfg_str(&menu_ui_cfg, &["fg-alt-a", "fg_alt_a"]).unwrap_or("silver");
    let fg_alt_b = cfg_str(&menu_ui_cfg, &["fg-alt-b", "fg_alt_b"]).unwrap_or("gray");

    let mut rows = Vec::with_capacity(options.len());
    let mut children = Vec::with_capacity(options.len());
    for (idx, option) in options.iter().enumerate() {
        rows.push(Value::String("auto".to_string()));
        let item_id = format!("{item_prefix}{idx}");
        let fg = if idx == 0 {
            fg_selected
        } else if idx % 2 == 0 {
            fg_alt_b
        } else {
            fg_alt_a
        };
        let content = format!("[{}] {}", option.key, option.label);
        children.push(Value::Mapping(menu_item_sprite(
            &item_id, &content, idx, grid_id, window, step_y, endless, font, at, fg,
        )));
    }

    let mut grid = Mapping::new();
    grid.insert(
        Value::String("type".to_string()),
        Value::String("grid".to_string()),
    );
    grid.insert(
        Value::String("id".to_string()),
        Value::String(grid_id.to_string()),
    );
    grid.insert(
        Value::String("at".to_string()),
        Value::String(at.to_string()),
    );
    grid.insert(
        Value::String("width".to_string()),
        Value::Number(Number::from(width)),
    );
    grid.insert(
        Value::String("height".to_string()),
        Value::Number(Number::from(height)),
    );
    grid.insert(
        Value::String("columns".to_string()),
        Value::Sequence(vec![Value::String("1fr".to_string())]),
    );
    grid.insert(Value::String("rows".to_string()), Value::Sequence(rows));
    grid.insert(
        Value::String("gap-y".to_string()),
        Value::Number(Number::from(gap_y)),
    );
    grid.insert(
        Value::String("children".to_string()),
        Value::Sequence(children),
    );
    sprite_seq.push(Value::Mapping(grid));

    if arrows {
        for (idx, _) in options.iter().enumerate() {
            let item_id = format!("{item_prefix}{idx}");
            sprite_seq.push(Value::Mapping(arrow_sprite(
                &format!("{item_id}-left-arrow"),
                ">",
                "left",
                &item_id,
                idx,
                font,
                at,
            )));
            sprite_seq.push(Value::Mapping(arrow_sprite(
                &format!("{item_id}-right-arrow"),
                "<",
                "right",
                &item_id,
                idx,
                font,
                at,
            )));
        }
    }
    scene.remove(Value::String("menu-ui".to_string()));
}

#[derive(Clone)]
struct MenuUiOption {
    key: String,
    label: String,
}

fn collect_menu_options(scene: &Mapping) -> Vec<MenuUiOption> {
    for key in ["menu-options", "menu_options"] {
        let Some(options) = scene.get(Value::String(key.to_string())) else {
            continue;
        };
        let Some(seq) = options.as_sequence() else {
            continue;
        };
        let mut out = Vec::with_capacity(seq.len());
        for (idx, option) in seq.iter().enumerate() {
            let Some(option_map) = option.as_mapping() else {
                continue;
            };
            let key_value = option_map
                .get(Value::String("key".to_string()))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| (idx + 1).to_string());
            let label_value = option_map
                .get(Value::String("label".to_string()))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| key_value.clone());
            out.push(MenuUiOption {
                key: key_value,
                label: label_value,
            });
        }
        return out;
    }
    Vec::new()
}

fn resolve_menu_ui_target_layer<'a>(
    scene: &'a mut Mapping,
    cfg: &Mapping,
) -> Option<&'a mut Mapping> {
    let layer_name =
        cfg_str(cfg, &["layer", "target-layer", "target_layer"]).map(|s| s.to_string());
    let layers = scene.get_mut(Value::String("layers".to_string()))?;
    let layer_seq = layers.as_sequence_mut()?;
    if let Some(name) = layer_name {
        for layer in layer_seq {
            let Some(layer_map) = layer.as_mapping_mut() else {
                continue;
            };
            if layer_map
                .get(Value::String("name".to_string()))
                .and_then(Value::as_str)
                == Some(name.as_str())
            {
                return Some(layer_map);
            }
        }
        return None;
    }
    layer_seq.first_mut()?.as_mapping_mut()
}

#[allow(clippy::too_many_arguments)]
fn menu_item_sprite(
    id: &str,
    content: &str,
    index: usize,
    grid_id: &str,
    window: u64,
    step_y: u64,
    endless: bool,
    font: &str,
    at: &str,
    fg: &str,
) -> Mapping {
    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("type".to_string()),
        Value::String("text".to_string()),
    );
    sprite.insert(
        Value::String("id".to_string()),
        Value::String(id.to_string()),
    );
    sprite.insert(
        Value::String("content".to_string()),
        Value::String(content.to_string()),
    );
    sprite.insert(
        Value::String("grid-col".to_string()),
        Value::Number(Number::from(1)),
    );
    sprite.insert(
        Value::String("grid-row".to_string()),
        Value::Number(Number::from(index + 1)),
    );
    sprite.insert(
        Value::String("at".to_string()),
        Value::String(at.to_string()),
    );
    sprite.insert(
        Value::String("font".to_string()),
        Value::String(font.to_string()),
    );
    sprite.insert(
        Value::String("fg".to_string()),
        Value::String(fg.to_string()),
    );
    let mut params = Mapping::new();
    params.insert(
        Value::String("target".to_string()),
        Value::String(grid_id.to_string()),
    );
    params.insert(
        Value::String("index".to_string()),
        Value::Number(Number::from(index)),
    );
    params.insert(
        Value::String("window".to_string()),
        Value::Number(Number::from(window)),
    );
    params.insert(
        Value::String("step_y".to_string()),
        Value::Number(Number::from(step_y)),
    );
    params.insert(Value::String("endless".to_string()), Value::Bool(endless));
    let mut behavior = Mapping::new();
    behavior.insert(
        Value::String("name".to_string()),
        Value::String("menu-carousel".to_string()),
    );
    behavior.insert(Value::String("params".to_string()), Value::Mapping(params));
    sprite.insert(
        Value::String("behaviors".to_string()),
        Value::Sequence(vec![Value::Mapping(behavior)]),
    );
    sprite
}

fn arrow_sprite(
    id: &str,
    content: &str,
    side: &str,
    target: &str,
    index: usize,
    font: &str,
    at: &str,
) -> Mapping {
    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("type".to_string()),
        Value::String("text".to_string()),
    );
    sprite.insert(
        Value::String("id".to_string()),
        Value::String(id.to_string()),
    );
    sprite.insert(
        Value::String("content".to_string()),
        Value::String(content.to_string()),
    );
    sprite.insert(
        Value::String("at".to_string()),
        Value::String(at.to_string()),
    );
    sprite.insert(
        Value::String("font".to_string()),
        Value::String(font.to_string()),
    );
    sprite.insert(
        Value::String("fg".to_string()),
        Value::String("yellow".to_string()),
    );
    let mut params = Mapping::new();
    params.insert(
        Value::String("target".to_string()),
        Value::String(target.to_string()),
    );
    params.insert(
        Value::String("index".to_string()),
        Value::Number(Number::from(index)),
    );
    params.insert(
        Value::String("side".to_string()),
        Value::String(side.to_string()),
    );
    params.insert(
        Value::String("padding".to_string()),
        Value::Number(Number::from(1)),
    );
    params.insert(
        Value::String("amplitude_x".to_string()),
        Value::Number(Number::from(1)),
    );
    params.insert(
        Value::String("period_ms".to_string()),
        Value::Number(Number::from(900)),
    );
    params.insert(
        Value::String("autoscale_height".to_string()),
        Value::Bool(true),
    );
    let mut behavior = Mapping::new();
    behavior.insert(
        Value::String("name".to_string()),
        Value::String("selected-arrows".to_string()),
    );
    behavior.insert(Value::String("params".to_string()), Value::Mapping(params));
    sprite.insert(
        Value::String("behaviors".to_string()),
        Value::Sequence(vec![Value::Mapping(behavior)]),
    );
    sprite
}

fn expand_window_sprite(
    sprite_map: &Mapping,
    inherited_defaults: Option<&Mapping>,
    scene_theme: Option<&str>,
) -> Result<Vec<Value>, serde_yaml::Error> {
    let base_id = map_get_str(sprite_map, &["id"]).unwrap_or("window");
    let title_id = map_get_str(sprite_map, &["title-id", "title_id"])
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{base_id}-title"));
    let body_id = map_get_str(sprite_map, &["body-id", "body_id"])
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{base_id}-body"));
    let footer_id = map_get_str(sprite_map, &["footer-id", "footer_id"])
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{base_id}-footer"));

    let title = map_get_str(sprite_map, &["title-bar", "title_bar", "title"]).unwrap_or_default();
    let body = map_get_str(
        sprite_map,
        &["body-content", "body_content", "body", "content"],
    )
    .unwrap_or_default();
    let footer = map_get_str(sprite_map, &["footer-content", "footer_content", "footer"])
        .unwrap_or_default();

    let theme_defaults = resolve_ui_theme_or_default(scene_theme);
    let border_fg = map_get_str(
        sprite_map,
        &[
            "border-fg",
            "border_fg",
            "border-colour",
            "border_colour",
            "frame-fg",
            "frame_fg",
            "fg",
            "fg_colour",
        ],
    )
    .or(Some(theme_defaults.window.border_fg))
    .unwrap_or("gray");
    let border_bg = map_get_str(
        sprite_map,
        &[
            "border-bg",
            "border_bg",
            "frame-bg",
            "frame_bg",
            "border-background",
            "border_background",
        ],
    )
    .or(Some(theme_defaults.window.border_bg))
    .unwrap_or("black");
    let panel_bg = map_get_str(
        sprite_map,
        &[
            "panel-bg",
            "panel_bg",
            "window-bg",
            "window_bg",
            "bg",
            "bg_colour",
        ],
    )
    .or(Some(theme_defaults.window.panel_bg))
    .unwrap_or("gray");
    let title_fg = map_get_str(sprite_map, &["title-fg", "title_fg"])
        .or(Some(theme_defaults.window.title_fg))
        .unwrap_or("white");
    let body_fg = map_get_str(sprite_map, &["body-fg", "body_fg"])
        .or(Some(theme_defaults.window.body_fg))
        .unwrap_or("silver");
    let footer_fg = map_get_str(sprite_map, &["footer-fg", "footer_fg"])
        .or(Some(theme_defaults.window.footer_fg))
        .unwrap_or("gray");
    let window_font = map_get_str(sprite_map, &["font"]).map(ToString::to_string);
    let slots_id = map_get_str(sprite_map, &["slots-id", "slots_id"])
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{base_id}-slots"));
    let mut panel = Mapping::new();
    for (key, value) in sprite_map {
        let Some(name) = key.as_str() else {
            panel.insert(key.clone(), value.clone());
            continue;
        };
        if WINDOW_RESERVED_KEYS.contains(&name) {
            continue;
        }
        panel.insert(key.clone(), value.clone());
    }
    panel.insert(
        Value::String("type".to_string()),
        Value::String("panel".to_string()),
    );
    if !panel.contains_key(Value::String("padding".to_string())) {
        panel.insert(
            Value::String("padding".to_string()),
            Value::Number(Number::from(0)),
        );
    }
    if !panel.contains_key(Value::String("border-width".to_string())) {
        panel.insert(
            Value::String("border-width".to_string()),
            Value::Number(Number::from(1)),
        );
    }
    if !panel.contains_key(Value::String("corner-radius".to_string())) {
        panel.insert(
            Value::String("corner-radius".to_string()),
            Value::Number(Number::from(1)),
        );
    }
    if !panel.contains_key(Value::String("shadow-x".to_string())) {
        panel.insert(
            Value::String("shadow-x".to_string()),
            Value::Number(Number::from(1)),
        );
    }
    if !panel.contains_key(Value::String("shadow-y".to_string())) {
        panel.insert(
            Value::String("shadow-y".to_string()),
            Value::Number(Number::from(1)),
        );
    }
    if !panel.contains_key(Value::String("bg".to_string())) {
        panel.insert(
            Value::String("bg".to_string()),
            Value::String(panel_bg.to_string()),
        );
    }
    if !panel.contains_key(Value::String("border-colour".to_string())) {
        panel.insert(
            Value::String("border-colour".to_string()),
            Value::String(border_fg.to_string()),
        );
    }
    if !panel.contains_key(Value::String("shadow-colour".to_string())) {
        panel.insert(
            Value::String("shadow-colour".to_string()),
            Value::String(border_bg.to_string()),
        );
    }

    let slot_children = vec![
        build_window_text_child(
            Some(title_id.as_str()),
            title,
            1,
            "ct",
            0,
            0,
            title_fg,
            None,
            window_font.as_deref(),
        ),
        build_window_text_child(
            Some(body_id.as_str()),
            body,
            1,
            "lt",
            0,
            0,
            body_fg,
            None,
            window_font.as_deref(),
        ),
        build_window_text_child(
            Some(footer_id.as_str()),
            footer,
            1,
            "lt",
            0,
            0,
            footer_fg,
            None,
            window_font.as_deref(),
        ),
    ];
    let mut children = vec![build_slots_container_sprite(
        &slots_id,
        Value::Sequence(slot_children),
    )];

    if let Some(extra_children) = sprite_map
        .get(Value::String("children".to_string()))
        .and_then(Value::as_sequence)
    {
        for child in extra_children {
            let mut child_value = child.clone();
            if let Some(child_map) = child_value.as_mapping_mut() {
                child_map
                    .entry(Value::String("at".to_string()))
                    .or_insert_with(|| Value::String("lt".to_string()));
                if let Some(font) = window_font.as_deref() {
                    child_map
                        .entry(Value::String("font".to_string()))
                        .or_insert_with(|| Value::String(font.to_string()));
                }
            }
            children.push(child_value);
        }
    }

    panel.insert(
        Value::String("children".to_string()),
        Value::Sequence(children),
    );
    apply_alias(&mut panel, "fg", "fg_colour");
    apply_alias(&mut panel, "bg", "bg_colour");
    apply_at_anchor(&mut panel);
    normalize_expression_fields(&mut panel);
    if let Some(children) = panel.get_mut(Value::String("children".to_string())) {
        normalize_sprites(children, inherited_defaults, scene_theme)?;
    }

    Ok(vec![Value::Mapping(panel)])
}

fn expand_scroll_list_sprite(
    sprite_map: &Mapping,
    inherited_defaults: Option<&Mapping>,
    scene_theme: Option<&str>,
) -> Result<Vec<Value>, serde_yaml::Error> {
    let items = sprite_map
        .get(Value::String("items".to_string()))
        .and_then(Value::as_sequence)
        .ok_or_else(|| serde_yaml::Error::custom("scroll-list requires `items` array"))?;

    let list_id = map_get_str(sprite_map, &["id"]).unwrap_or("scroll-list");
    let default_prefix = format!("{list_id}-item-");
    let item_prefix =
        map_get_str(sprite_map, &["item-prefix", "item_prefix"]).unwrap_or(default_prefix.as_str());
    let bind_menu = map_get_bool(sprite_map, &["bind-menu", "bind_menu"]).unwrap_or(false);
    let endless = map_get_bool(sprite_map, &["endless"]).unwrap_or(true);
    let window = map_get_u64(sprite_map, &["window"]).unwrap_or(5).max(1);
    let step_y = map_get_u64(sprite_map, &["step-y", "step_y"])
        .unwrap_or(1)
        .max(1);
    let gap_y = map_get_u64(sprite_map, &["gap-y", "gap_y"]).unwrap_or(1);
    let theme_defaults = resolve_ui_theme_or_default(scene_theme);
    let selected_fg = map_get_str(sprite_map, &["fg-selected", "fg_selected"])
        .or(Some(theme_defaults.scroll_list.selected_fg))
        .unwrap_or("white");
    let fg_alt_a = map_get_str(sprite_map, &["fg-alt-a", "fg_alt_a"])
        .or(Some(theme_defaults.scroll_list.alt_a_fg))
        .unwrap_or("silver");
    let fg_alt_b = map_get_str(sprite_map, &["fg-alt-b", "fg_alt_b"])
        .or(Some(theme_defaults.scroll_list.alt_b_fg))
        .unwrap_or("gray");
    let list_font = map_get_str(sprite_map, &["font"]).map(ToString::to_string);

    let mut grid = Mapping::new();
    for (key, value) in sprite_map {
        let Some(name) = key.as_str() else {
            grid.insert(key.clone(), value.clone());
            continue;
        };
        if SCROLL_LIST_RESERVED_KEYS.contains(&name) {
            continue;
        }
        grid.insert(key.clone(), value.clone());
    }
    grid.insert(
        Value::String("type".to_string()),
        Value::String("grid".to_string()),
    );
    if !grid.contains_key(Value::String("columns".to_string())) {
        grid.insert(
            Value::String("columns".to_string()),
            Value::Sequence(vec![Value::String("1fr".to_string())]),
        );
    }
    if !grid.contains_key(Value::String("gap-y".to_string())) {
        grid.insert(
            Value::String("gap-y".to_string()),
            Value::Number(Number::from(gap_y)),
        );
    }

    let mut rows = Vec::with_capacity(items.len());
    let mut children = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        rows.push(Value::String("auto".to_string()));
        let (label, explicit_id, explicit_fg) = parse_scroll_list_item(item, idx);
        let item_id = explicit_id.unwrap_or_else(|| format!("{item_prefix}{idx}"));
        let fg = explicit_fg.unwrap_or_else(|| {
            if idx == 0 {
                selected_fg.to_string()
            } else if idx % 2 == 0 {
                fg_alt_b.to_string()
            } else {
                fg_alt_a.to_string()
            }
        });

        let mut sprite = Mapping::new();
        sprite.insert(
            Value::String("type".to_string()),
            Value::String("text".to_string()),
        );
        sprite.insert(Value::String("id".to_string()), Value::String(item_id));
        sprite.insert(Value::String("content".to_string()), Value::String(label));
        sprite.insert(
            Value::String("grid-col".to_string()),
            Value::Number(Number::from(1)),
        );
        sprite.insert(
            Value::String("grid-row".to_string()),
            Value::Number(Number::from(idx + 1)),
        );
        sprite.insert(
            Value::String("at".to_string()),
            Value::String("cc".to_string()),
        );
        sprite.insert(Value::String("fg".to_string()), Value::String(fg));
        if let Some(font) = list_font.as_deref() {
            sprite.insert(
                Value::String("font".to_string()),
                Value::String(font.to_string()),
            );
        }
        if bind_menu {
            let mut params = Mapping::new();
            params.insert(
                Value::String("target".to_string()),
                Value::String(list_id.to_string()),
            );
            params.insert(
                Value::String("index".to_string()),
                Value::Number(Number::from(idx)),
            );
            params.insert(
                Value::String("window".to_string()),
                Value::Number(Number::from(window)),
            );
            params.insert(
                Value::String("step_y".to_string()),
                Value::Number(Number::from(step_y)),
            );
            params.insert(Value::String("endless".to_string()), Value::Bool(endless));

            let mut behavior = Mapping::new();
            behavior.insert(
                Value::String("name".to_string()),
                Value::String("menu-carousel".to_string()),
            );
            behavior.insert(Value::String("params".to_string()), Value::Mapping(params));
            sprite.insert(
                Value::String("behaviors".to_string()),
                Value::Sequence(vec![Value::Mapping(behavior)]),
            );
        }
        children.push(Value::Mapping(sprite));
    }

    grid.insert(Value::String("rows".to_string()), Value::Sequence(rows));
    grid.insert(
        Value::String("children".to_string()),
        Value::Sequence(children),
    );
    apply_alias(&mut grid, "fg", "fg_colour");
    apply_alias(&mut grid, "bg", "bg_colour");
    apply_at_anchor(&mut grid);
    normalize_expression_fields(&mut grid);
    if let Some(children) = grid.get_mut(Value::String("children".to_string())) {
        normalize_sprites(children, inherited_defaults, scene_theme)?;
    }

    Ok(vec![Value::Mapping(grid)])
}

#[allow(clippy::too_many_arguments)]
fn build_window_text_child(
    id: Option<&str>,
    content: &str,
    row: u64,
    at: &str,
    x: i64,
    y: i64,
    fg: &str,
    bg: Option<&str>,
    font: Option<&str>,
) -> Value {
    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("type".to_string()),
        Value::String("text".to_string()),
    );
    if let Some(id) = id {
        sprite.insert(
            Value::String("id".to_string()),
            Value::String(id.to_string()),
        );
    }
    sprite.insert(
        Value::String("content".to_string()),
        Value::String(content.to_string()),
    );
    sprite.insert(
        Value::String("grid-col".to_string()),
        Value::Number(Number::from(1)),
    );
    sprite.insert(
        Value::String("grid-row".to_string()),
        Value::Number(Number::from(row)),
    );
    sprite.insert(
        Value::String("at".to_string()),
        Value::String(at.to_string()),
    );
    sprite.insert(
        Value::String("x".to_string()),
        Value::Number(Number::from(x)),
    );
    sprite.insert(
        Value::String("y".to_string()),
        Value::Number(Number::from(y)),
    );
    sprite.insert(
        Value::String("fg".to_string()),
        Value::String(fg.to_string()),
    );
    if let Some(bg) = bg {
        sprite.insert(
            Value::String("bg".to_string()),
            Value::String(bg.to_string()),
        );
    }
    if let Some(font) = font {
        sprite.insert(
            Value::String("font".to_string()),
            Value::String(font.to_string()),
        );
    }
    Value::Mapping(sprite)
}

fn build_slots_container_sprite(id: &str, children: Value) -> Value {
    let mut slots = Mapping::new();
    slots.insert(
        Value::String("type".to_string()),
        Value::String("flex".to_string()),
    );
    slots.insert(
        Value::String("id".to_string()),
        Value::String(id.to_string()),
    );
    slots.insert(
        Value::String("at".to_string()),
        Value::String("lt".to_string()),
    );
    slots.insert(
        Value::String("x".to_string()),
        Value::Number(Number::from(0)),
    );
    slots.insert(
        Value::String("y".to_string()),
        Value::Number(Number::from(0)),
    );
    slots.insert(
        Value::String("direction".to_string()),
        Value::String("column".to_string()),
    );
    slots.insert(
        Value::String("gap".to_string()),
        Value::Number(Number::from(0)),
    );
    slots.insert(Value::String("children".to_string()), children);
    Value::Mapping(slots)
}

fn parse_scroll_list_item(item: &Value, idx: usize) -> (String, Option<String>, Option<String>) {
    match item {
        Value::String(text) => (text.clone(), None, None),
        Value::Mapping(map) => {
            let label = map
                .get(Value::String("label".to_string()))
                .or_else(|| map.get(Value::String("content".to_string())))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("ITEM {}", idx + 1));
            let id = map
                .get(Value::String("id".to_string()))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let fg = map
                .get(Value::String("fg".to_string()))
                .or_else(|| map.get(Value::String("fg_colour".to_string())))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            (label, id, fg)
        }
        _ => (format!("ITEM {}", idx + 1), None, None),
    }
}

const WINDOW_RESERVED_KEYS: &[&str] = &[
    "type",
    "sprite-defaults",
    "title",
    "title-bar",
    "title_bar",
    "title-id",
    "title_id",
    "slots-id",
    "slots_id",
    "body",
    "body-content",
    "body_content",
    "footer",
    "footer-content",
    "footer_content",
    "title-fg",
    "title_fg",
    "body-fg",
    "body_fg",
    "footer-fg",
    "footer_fg",
    "border-fg",
    "border_fg",
    "border-bg",
    "border_bg",
    "border-background",
    "border_background",
    "border-colour",
    "border_colour",
    "frame-fg",
    "frame_fg",
    "frame-bg",
    "frame_bg",
    "panel-bg",
    "panel_bg",
    "window-bg",
    "window_bg",
    "border-style",
    "border_style",
    "frame-style",
    "frame_style",
    "children",
    "font",
];

const SCROLL_LIST_RESERVED_KEYS: &[&str] = &[
    "type",
    "sprite-defaults",
    "items",
    "item-prefix",
    "item_prefix",
    "bind-menu",
    "bind_menu",
    "window",
    "step-y",
    "step_y",
    "gap-y",
    "gap_y",
    "endless",
    "fg-selected",
    "fg_selected",
    "fg-alt-a",
    "fg_alt_a",
    "fg-alt-b",
    "fg_alt_b",
    "font",
];

fn expand_frame_sequence(
    sprite_map: &Mapping,
    inherited_defaults: Option<&Mapping>,
) -> Result<Vec<Value>, serde_yaml::Error> {
    let pattern = map_get_str(sprite_map, &["source-pattern", "source_pattern"])
        .ok_or_else(|| serde_yaml::Error::custom("frame-sequence requires `source-pattern`"))?;
    let from = map_get_u64(sprite_map, &["from"]).unwrap_or(1);
    let to = if let Some(to_value) = map_get_u64(sprite_map, &["to"]) {
        to_value
    } else if let Some(count) = map_get_u64(sprite_map, &["count"]) {
        from.saturating_add(count.saturating_sub(1))
    } else {
        return Err(serde_yaml::Error::custom(
            "frame-sequence requires `to` or `count`",
        ));
    };
    if to < from {
        return Err(serde_yaml::Error::custom(
            "frame-sequence requires `to >= from`",
        ));
    }
    let delay_ms = map_get_u64(sprite_map, &["delay-ms", "delay_ms"]).unwrap_or(100);
    if delay_ms == 0 {
        return Err(serde_yaml::Error::custom(
            "frame-sequence requires `delay-ms > 0`",
        ));
    }
    let last_delay_ms =
        map_get_u64(sprite_map, &["last-delay-ms", "last_delay_ms"]).unwrap_or(delay_ms);
    if last_delay_ms == 0 {
        return Err(serde_yaml::Error::custom(
            "frame-sequence requires `last-delay-ms > 0`",
        ));
    }
    let id_prefix = map_get_str(sprite_map, &["id-prefix", "id_prefix"]).unwrap_or("frame-");
    let mut base = Mapping::new();
    base.insert(
        Value::String("type".to_string()),
        Value::String("image".to_string()),
    );
    apply_defaults(&mut base, inherited_defaults);

    for (k, v) in sprite_map {
        let key = k.as_str().unwrap_or("");
        if matches!(
            key,
            "type"
                | "source-pattern"
                | "source_pattern"
                | "from"
                | "to"
                | "count"
                | "delay-ms"
                | "delay_ms"
                | "last-delay-ms"
                | "last_delay_ms"
                | "start-at-ms"
                | "start_at_ms"
                | "id-prefix"
                | "id_prefix"
                | "sprite-defaults"
        ) {
            continue;
        }
        base.insert(k.clone(), v.clone());
    }

    let mut out = Vec::with_capacity((to - from + 1) as usize);
    let mut elapsed = map_get_u64(sprite_map, &["start-at-ms", "start_at_ms"]).unwrap_or(0);
    for idx in from..=to {
        let mut frame = base.clone();
        let source = pattern
            .replace("{i}", &idx.to_string())
            .replace("{index}", &idx.to_string());
        frame.insert(
            Value::String("id".to_string()),
            Value::String(format!("{id_prefix}{idx}")),
        );
        frame.insert(Value::String("source".to_string()), Value::String(source));
        frame.insert(
            Value::String("appear_at_ms".to_string()),
            Value::Number(Number::from(elapsed)),
        );
        let duration = if idx == to { last_delay_ms } else { delay_ms };
        let disappear_at = elapsed.saturating_add(duration);
        frame.insert(
            Value::String("disappear_at_ms".to_string()),
            Value::Number(Number::from(disappear_at)),
        );
        apply_alias(&mut frame, "fg", "fg_colour");
        apply_alias(&mut frame, "bg", "bg_colour");
        apply_at_anchor(&mut frame);
        normalize_expression_fields(&mut frame);
        out.push(Value::Mapping(frame));
        elapsed = disappear_at;
    }
    Ok(out)
}

/// Renames field `from` to `to` if `to` is not already present.
///
/// Used for: bg→bg_colour, fg→fg_colour
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
/// Expands `at: anchor` shorthand into {align_x, align_y} pair.
///
/// Supported anchors: cc, ct, cb, lc, lt, lb, rc, rt, rb
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
#[cfg(test)]
mod tests {
    use super::SceneDocument;
    use engine_core::scene::{HorizontalAlign, Sprite, TermColour, VerticalAlign};

    fn first_flex_children(children: &[Sprite]) -> Option<&[Sprite]> {
        children.iter().find_map(|child| match child {
            Sprite::Flex { children, .. } => Some(children.as_slice()),
            _ => None,
        })
    }

    fn find_text_by_id<'a>(sprites: &'a [Sprite], target_id: &str) -> Option<&'a Sprite> {
        for sprite in sprites {
            match sprite {
                Sprite::Text { id: Some(id), .. } if id == target_id => return Some(sprite),
                Sprite::Panel { children, .. }
                | Sprite::Grid { children, .. }
                | Sprite::Flex { children, .. } => {
                    if let Some(found) = find_text_by_id(children, target_id) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    #[test]
    fn compiles_scene_with_aliases_and_pause_shorthand() {
        let raw = r#"
id: menu
title: Menu
bg: black
stages:
  on_enter:
    steps:
      - pause: 2s
layers:
  - sprites:
      - type: text
        content: START
        at: cc
        fg: white
menu-options:
  - key: "1"
    to: next-scene
"#;

        let doc: SceneDocument = serde_yaml::from_str(raw).expect("document");
        let scene = doc.compile().expect("scene");
        assert!(scene.bg_colour.is_some());
        assert_eq!(scene.stages.on_enter.steps[0].duration, Some(2000));
        assert!(scene.stages.on_enter.steps[0].effects.is_empty());
        assert_eq!(scene.menu_options[0].scene.as_deref(), Some("next-scene"));
        assert_eq!(scene.menu_options[0].next, "next-scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Text {
                align_x,
                align_y,
                fg_colour,
                ..
            } => {
                assert!(matches!(align_x, Some(HorizontalAlign::Center)));
                assert!(matches!(align_y, Some(VerticalAlign::Center)));
                assert!(fg_colour.is_some());
            }
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn compiles_scene_with_planet_spec_and_ref_aliases() {
        let raw = r#"
id: planet-spec-doc
title: Planet Spec Doc
planet_spec:
  generator:
    preset: gas-giant
    seed: 99
  body:
    id: jove
planet_spec_ref: /planet-specs/jove.yml
layers: []
"#;

        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");

        assert_eq!(
            scene
                .planet_spec
                .as_ref()
                .and_then(|spec| spec.generator.as_ref())
                .and_then(|generator| generator.preset.as_deref()),
            Some("gas-giant")
        );
        assert_eq!(
            scene.planet_spec_ref.as_deref(),
            Some("/planet-specs/jove.yml")
        );
    }

    #[test]
    fn expands_scene_templates_with_args() {
        let raw = r#"
id: menu
title: Menu
templates:
  menu-item:
    type: text
    content: "$label"
    at: cc
layers:
  - sprites:
      - use: menu-item
        args:
          label: START
        y: 2
"#;

        let doc: SceneDocument = serde_yaml::from_str(raw).expect("document");
        let scene = doc.compile().expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Text {
                content,
                y,
                align_x,
                align_y,
                ..
            } => {
                assert_eq!(content, "START");
                assert_eq!(*y, 2);
                assert!(matches!(align_x, Some(HorizontalAlign::Center)));
                assert!(matches!(align_y, Some(VerticalAlign::Center)));
            }
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn normalizes_expression_oscillate_and_animate() {
        let raw = r#"
id: fx
title: FX
layers:
  - sprites:
      - type: text
        content: HELLO
        y: oscillate(-2, 2, 1800ms)
      - type: obj
        source: /scenes/3d/model.obj
        rotation-y: animate(180deg, 540deg, 12s, loop)
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Text { y, animations, .. } => {
                assert_eq!(*y, 0);
                assert_eq!(animations.len(), 1);
                assert_eq!(animations[0].name, "float");
                assert_eq!(animations[0].params.period_ms, 1800);
            }
            _ => panic!("expected text"),
        }
        match &scene.layers[0].sprites[1] {
            Sprite::Obj {
                rotation_y,
                rotate_y_deg_per_sec,
                ..
            } => {
                assert_eq!(rotation_y.unwrap_or_default().round() as i32, 180);
                assert_eq!(rotate_y_deg_per_sec.unwrap_or_default().round() as i32, 30);
            }
            _ => panic!("expected obj"),
        }
    }

    #[test]
    fn applies_sprite_defaults_with_child_override() {
        let raw = r#"
id: defaults
title: Defaults
sprite-defaults:
  at: cc
  font: "generic:1"
  fg: silver
layers:
  - sprites:
      - type: text
        content: A
      - type: text
        content: B
        fg: yellow
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Text {
                align_x,
                align_y,
                font,
                fg_colour,
                ..
            } => {
                assert!(matches!(align_x, Some(HorizontalAlign::Center)));
                assert!(matches!(align_y, Some(VerticalAlign::Center)));
                assert_eq!(font.as_deref(), Some("generic:1"));
                assert!(fg_colour.is_some());
            }
            _ => panic!("expected text"),
        }
        match &scene.layers[0].sprites[1] {
            Sprite::Text { fg_colour, .. } => {
                assert!(fg_colour.is_some());
            }
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn expands_frame_sequence_to_timed_images() {
        let raw = r#"
id: sequence
title: Sequence
layers:
  - sprites:
      - type: frame-sequence
        id-prefix: cut-
        source-pattern: /assets/seq/{i}.png
        from: 1
        to: 3
        delay-ms: 120
        last-delay-ms: 200
        at: cc
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        assert_eq!(scene.layers[0].sprites.len(), 3);
        match &scene.layers[0].sprites[0] {
            Sprite::Image {
                source,
                appear_at_ms,
                disappear_at_ms,
                ..
            } => {
                assert_eq!(source, "/assets/seq/1.png");
                assert_eq!(*appear_at_ms, Some(0));
                assert_eq!(*disappear_at_ms, Some(120));
            }
            _ => panic!("expected image"),
        }
        match &scene.layers[0].sprites[2] {
            Sprite::Image {
                source,
                appear_at_ms,
                disappear_at_ms,
                ..
            } => {
                assert_eq!(source, "/assets/seq/3.png");
                assert_eq!(*appear_at_ms, Some(240));
                assert_eq!(*disappear_at_ms, Some(440));
            }
            _ => panic!("expected image"),
        }
    }

    #[test]
    fn expands_window_sprite_to_panel_with_slot_children() {
        let raw = r#"
id: window-scene
title: Window
layers:
  - sprites:
      - type: window
        id: terminal-window
        at: cc
        width: 32
        height: 10
        title: TERMINAL
        body-content: output line
        footer-content: "> ready"
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        assert_eq!(scene.layers[0].sprites.len(), 1);
        match &scene.layers[0].sprites[0] {
            Sprite::Panel { id, children, .. } => {
                assert_eq!(id.as_deref(), Some("terminal-window"));
                let slots = first_flex_children(children).expect("generated slots flex");
                assert_eq!(slots.len(), 3);
                let title = find_text_by_id(slots, "terminal-window-title")
                    .expect("generated title text child");
                match title {
                    Sprite::Text { content, .. } => assert_eq!(content, "TERMINAL"),
                    _ => panic!("title should be text"),
                }
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_sprite_preserves_width_percent_without_injecting_fixed_width() {
        let raw = r#"
id: window-percent
title: Window Percent
layers:
  - sprites:
      - type: window
        id: terminal-window
        at: cc
        width-percent: 95
        height: 5
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel {
                width,
                width_percent,
                ..
            } => {
                assert_eq!(*width, None);
                assert_eq!(*width_percent, Some(95));
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_sprite_uses_zero_padding_by_default_for_three_slot_layout() {
        let raw = r#"
id: window-padding-default
title: Window Padding
layers:
  - sprites:
      - type: window
        id: terminal-window
        at: cc
        width: 32
        height: 5
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel { padding, .. } => {
                assert_eq!(*padding, 0);
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_supports_title_bar_alias() {
        let raw = r#"
id: window-title-bar
title: Window Title Bar
layers:
  - sprites:
      - type: window
        id: terminal-window
        title-bar: TERMINAL
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel { children, .. } => {
                let title = find_text_by_id(children, "terminal-window-title")
                    .expect("generated title text child");
                match title {
                    Sprite::Text {
                        content,
                        align_x,
                        align_y,
                        ..
                    } => {
                        assert_eq!(content, "TERMINAL");
                        assert!(matches!(align_x, Some(HorizontalAlign::Center)));
                        assert!(matches!(align_y, Some(VerticalAlign::Top)));
                    }
                    _ => panic!("title should be text"),
                }
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn expands_window_sprite_with_generic_font_forwards_font_to_slot_children() {
        let raw = r#"
id: window-ascii
title: Window Ascii
layers:
  - sprites:
      - type: window
        id: terminal-window
        at: cc
        width: 20
        font: "generic:2"
        title: TERMINAL
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel { children, .. } => {
                let title_font = find_text_by_id(children, "terminal-window-title").and_then(
                    |child| match child {
                        Sprite::Text { font, .. } => font.as_deref(),
                        _ => None,
                    },
                );
                assert_eq!(title_font, Some("generic:2"));
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_slots_are_grouped_under_flex_container() {
        let raw = r#"
id: window-slot-offsets
title: Window Slot Offsets
layers:
  - sprites:
      - type: window
        id: terminal-window
        font: "generic:2"
        title-bar: HINTS
        body-content: TYPE LS
        footer-content: READY
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel { children, .. } => {
                let slots = first_flex_children(children).expect("generated slots flex");
                assert!(
                    find_text_by_id(slots, "terminal-window-title").is_some(),
                    "title slot missing"
                );
                assert!(
                    find_text_by_id(slots, "terminal-window-body").is_some(),
                    "body slot missing"
                );
                assert!(
                    find_text_by_id(slots, "terminal-window-footer").is_some(),
                    "footer slot missing"
                );
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn applies_window_theme_defaults_from_scene_ui_theme() {
        let raw = r#"
id: window-theme
title: Window Theme
ui:
  theme: win98
layers:
  - sprites:
      - type: window
        id: terminal-window
        at: cc
        width: 20
        title: STATUS
        body-content: BOOTING
        footer-content: READY
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel {
                bg_colour,
                border_colour,
                shadow_colour,
                children,
                ..
            } => {
                assert_eq!(bg_colour.as_ref(), Some(&TermColour::Silver));
                assert_eq!(border_colour.as_ref(), Some(&TermColour::Silver));
                assert_eq!(shadow_colour.as_ref(), Some(&TermColour::Gray));
                let footer_fg = find_text_by_id(children, "terminal-window-footer")
                    .and_then(|child| match child {
                        Sprite::Text { fg_colour, .. } => fg_colour.as_ref(),
                        _ => None,
                    })
                    .expect("generated footer text child");
                assert_eq!(footer_fg, &TermColour::Silver);
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_sprite_explicit_style_overrides_scene_theme_defaults() {
        let raw = r#"
id: window-theme-override
title: Window Theme Override
ui:
  theme: win98
layers:
  - sprites:
      - type: window
        id: terminal-window
        width: 20
        border-style: unicode
        border-fg: yellow
        title-fg: cyan
        body-fg: magenta
        footer-fg: green
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel {
                border_colour,
                children,
                ..
            } => {
                assert_eq!(border_colour.as_ref(), Some(&TermColour::Yellow));
                let title_fg = find_text_by_id(children, "terminal-window-title")
                    .and_then(|child| match child {
                        Sprite::Text { fg_colour, .. } => fg_colour.as_ref(),
                        _ => None,
                    })
                    .expect("generated title text child");
                assert_eq!(title_fg, &TermColour::Cyan);
                let body_fg = find_text_by_id(children, "terminal-window-body")
                    .and_then(|child| match child {
                        Sprite::Text { fg_colour, .. } => fg_colour.as_ref(),
                        _ => None,
                    })
                    .expect("generated body text child");
                assert_eq!(body_fg, &TermColour::Magenta);
                let footer_fg = find_text_by_id(children, "terminal-window-footer")
                    .and_then(|child| match child {
                        Sprite::Text { fg_colour, .. } => fg_colour.as_ref(),
                        _ => None,
                    })
                    .expect("generated footer text child");
                assert_eq!(footer_fg, &TermColour::Green);
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn window_theme_applies_when_generic_font_is_used() {
        let raw = r#"
id: window-theme-generic-fallback
title: Window Theme Generic Fallback
ui:
  theme: xp
layers:
  - sprites:
      - type: window
        id: terminal-window
        width: 20
        font: "generic:2"
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Panel {
                bg_colour,
                border_colour,
                ..
            } => {
                assert_eq!(bg_colour.as_ref(), Some(&TermColour::Silver));
                assert_eq!(border_colour.as_ref(), Some(&TermColour::Silver));
            }
            _ => panic!("expected panel from window sugar"),
        }
    }

    #[test]
    fn expands_scroll_list_sprite_to_grid_items_with_menu_binding() {
        let raw = r#"
id: list-scene
title: List
layers:
  - sprites:
      - type: scroll-list
        id: actions
        bind-menu: true
        endless: true
        window: 3
        step-y: 2
        items:
          - "LOOK"
          - { id: item-open, label: OPEN, fg: yellow }
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        assert_eq!(scene.layers[0].sprites.len(), 1);
        match &scene.layers[0].sprites[0] {
            Sprite::Grid { children, rows, .. } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(children.len(), 2);
                match &children[0] {
                    Sprite::Text {
                        content, behaviors, ..
                    } => {
                        assert_eq!(content, "LOOK");
                        assert_eq!(behaviors.len(), 1);
                        assert_eq!(behaviors[0].name, "menu-carousel");
                    }
                    _ => panic!("expected generated list item text"),
                }
                match &children[1] {
                    Sprite::Text { id, content, .. } => {
                        assert_eq!(id.as_deref(), Some("item-open"));
                        assert_eq!(content, "OPEN");
                    }
                    _ => panic!("expected generated mapped list item"),
                }
            }
            _ => panic!("expected grid from scroll-list sugar"),
        }
    }

    #[test]
    fn applies_scroll_list_theme_defaults_from_scene_ui_theme() {
        let raw = r#"
id: list-theme
title: List Theme
ui:
  theme: xp
layers:
  - sprites:
      - type: scroll-list
        id: actions
        items:
          - "LOOK"
          - "OPEN"
          - "EXIT"
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Grid { children, .. } => {
                match &children[0] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::Cyan));
                    }
                    _ => panic!("expected generated list item text"),
                }
                match &children[1] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::White));
                    }
                    _ => panic!("expected generated list item text"),
                }
                match &children[2] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::Silver));
                    }
                    _ => panic!("expected generated list item text"),
                }
            }
            _ => panic!("expected grid from scroll-list sugar"),
        }
    }

    #[test]
    fn scroll_list_explicit_colors_override_scene_theme_defaults() {
        let raw = r#"
id: list-theme-override
title: List Theme Override
ui:
  theme: xp
layers:
  - sprites:
      - type: scroll-list
        id: actions
        fg-selected: red
        fg-alt-a: green
        fg-alt-b: magenta
        items:
          - "LOOK"
          - "OPEN"
          - "EXIT"
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        match &scene.layers[0].sprites[0] {
            Sprite::Grid { children, .. } => {
                match &children[0] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::Red));
                    }
                    _ => panic!("expected generated list item text"),
                }
                match &children[1] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::Green));
                    }
                    _ => panic!("expected generated list item text"),
                }
                match &children[2] {
                    Sprite::Text { fg_colour, .. } => {
                        assert_eq!(fg_colour.as_ref(), Some(&TermColour::Magenta));
                    }
                    _ => panic!("expected generated list item text"),
                }
            }
            _ => panic!("expected grid from scroll-list sugar"),
        }
    }

    #[test]
    fn expands_menu_ui_into_sprites() {
        let raw = r#"
id: menu
title: Menu
layers:
  - name: main
    sprites: []
menu-options:
  - key: "1"
    label: PLAY
    to: next-scene
  - key: "2"
    label: EXIT
    to: quit-scene
menu-ui:
  layer: main
  grid-id: test-grid
"#;
        let scene = serde_yaml::from_str::<SceneDocument>(raw)
            .expect("document")
            .compile()
            .expect("scene");
        assert!(scene.layers[0].sprites.len() >= 3);
    }
}
