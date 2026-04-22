//! Authoring validation helpers.
//!
//! This module will contain reusable authoring checks shared by tests, editor
//! tooling, and future compile-time diagnostics.

mod render3d;

use engine_core::scene::{Scene, Sprite, TextOverflowMode, TextWrapMode};
pub use render3d::{validate_render_scene3d_document, Render3dDiagnostic};
use serde_yaml::{Mapping, Value};

/// Validation diagnostic for sprite timeline issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineDiagnostic {
    /// Sprite appear_at_ms is after on_enter stage duration (will never be visible during cutscene)
    SpriteAppearsAfterSceneEnd {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        scene_duration_ms: u64,
    },
    /// Sprite disappear_at_ms is before appear_at_ms (always hidden)
    SpriteDisappearsBeforeAppear {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        disappear_at_ms: u64,
    },
}

/// Validation diagnostic for text layout semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextLayoutDiagnostic {
    /// Ellipsis was requested without any authored width/line bound.
    EllipsisWithoutBounds {
        layer_name: String,
        sprite_index: usize,
    },
    /// A line clamp was set without a wrap contract to give it multi-line meaning.
    LineClampWithoutWrap {
        layer_name: String,
        sprite_index: usize,
        line_clamp: u16,
    },
    /// Reserved width smaller than the visible max width defeats the reserved layout footprint.
    ReserveWidthTooSmall {
        layer_name: String,
        sprite_index: usize,
        reserve_width_ch: u16,
        max_width: u16,
    },
}

/// Validation diagnostic for scene-level authored model/controller mismatches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneDocumentDiagnostic {
    /// Celestial bindings are only valid for celestial world models.
    CelestialRequiresCelestialWorldModel { world_model: String },
    /// Planar scenes cannot author 3D scene camera control profiles.
    PlanarWorldModelDisallowsCameraInput { input_profile: String },
    /// Planar scenes cannot select known 3D camera presets.
    PlanarWorldModelDisallowsCameraPreset { preset: String },
    /// Controller preset hooks should never be empty/whitespace.
    EmptyControllerPreset { field: String },
    /// Known camera presets still require a matching input profile.
    KnownCameraPresetMissingLegacyBinding {
        preset: String,
        required_input: String,
    },
    /// Runtime-object bridge nodes may carry an explicit kind marker.
    RuntimeObjectKindMismatch { path: String, kind: String },
}

/// Validates raw scene-document semantics that do not require full `Scene`
/// materialization.
pub fn validate_scene_document_semantics(scene: &Value) -> Vec<SceneDocumentDiagnostic> {
    let Some(scene_map) = scene.as_mapping() else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    let effective_world_model =
        mapping_get_str(scene_map, &["world-model", "world_model"]).unwrap_or("planar-2d");
    let input = scene_map
        .get(Value::String("input".to_string()))
        .and_then(Value::as_mapping);
    let camera_rig = scene_map
        .get(Value::String("camera-rig".to_string()))
        .or_else(|| scene_map.get(Value::String("camera_rig".to_string())))
        .and_then(Value::as_mapping);
    let controller_defaults = scene_map
        .get(Value::String("controller-defaults".to_string()))
        .or_else(|| scene_map.get(Value::String("controller_defaults".to_string())))
        .and_then(Value::as_mapping);
    let camera_preset = controller_defaults
        .and_then(|defaults| mapping_get_str(defaults, &["camera-preset", "camera_preset"]))
        .or_else(|| infer_camera_rig_preset(camera_rig));

    if mapping_has_non_null(scene_map, &["celestial"]) && effective_world_model != "celestial-3d" {
        diagnostics.push(
            SceneDocumentDiagnostic::CelestialRequiresCelestialWorldModel {
                world_model: effective_world_model.to_string(),
            },
        );
    }

    if effective_world_model == "planar-2d" {
        if let Some(input) = input {
            for profile in [
                ("obj-viewer", "obj_viewer"),
                ("free-look-camera", "free_look_camera"),
                ("orbit-camera", "orbit_camera"),
            ] {
                if mapping_has_non_null(input, &[profile.0, profile.1]) {
                    diagnostics.push(
                        SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraInput {
                            input_profile: profile.0.to_string(),
                        },
                    );
                }
            }
        }

        if let Some(camera_rig) = camera_rig {
            for profile in [
                ("obj-viewer", "obj_viewer"),
                ("free-look-camera", "free_look_camera"),
                ("orbit-camera", "orbit_camera"),
            ] {
                if mapping_has_non_null(camera_rig, &[profile.0, profile.1]) {
                    diagnostics.push(
                        SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraInput {
                            input_profile: profile.0.to_string(),
                        },
                    );
                }
            }
        }

        if let Some(preset) = camera_preset.filter(|preset| is_known_3d_camera_preset(preset)) {
            diagnostics.push(
                SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraPreset {
                    preset: preset.to_string(),
                },
            );
        }
    }

    if let Some(controller_defaults) = controller_defaults {
        for (field, aliases) in [
            ("camera-preset", &["camera-preset", "camera_preset"][..]),
            ("player-preset", &["player-preset", "player_preset"][..]),
            ("ui-preset", &["ui-preset", "ui_preset"][..]),
            ("spawn-preset", &["spawn-preset", "spawn_preset"][..]),
            ("gravity-preset", &["gravity-preset", "gravity_preset"][..]),
            ("surface-preset", &["surface-preset", "surface_preset"][..]),
        ] {
            if let Some(value) = mapping_get_str(controller_defaults, aliases) {
                if value.trim().is_empty() {
                    diagnostics.push(SceneDocumentDiagnostic::EmptyControllerPreset {
                        field: field.to_string(),
                    });
                }
            }
        }
    }

    diagnostics.extend(validate_runtime_object_kind_markers(scene_map));

    if let Some(preset) = camera_preset {
        let required_input = match preset {
            "obj-viewer"
                if !has_camera_rig_profile(input, camera_rig, &["obj-viewer", "obj_viewer"]) =>
            {
                Some("input.obj-viewer or camera-rig.obj-viewer")
            }
            "orbit-camera"
                if !has_camera_rig_profile(
                    input,
                    camera_rig,
                    &["orbit-camera", "orbit_camera"],
                ) =>
            {
                Some("input.orbit-camera or camera-rig.orbit-camera")
            }
            "free-look-camera"
                if !has_camera_rig_profile(
                    input,
                    camera_rig,
                    &["free-look-camera", "free_look_camera"],
                ) =>
            {
                Some("input.free-look-camera or camera-rig.free-look-camera")
            }
            "surface-free-look" if !surface_free_look_compat(input, camera_rig) => {
                Some("input.free-look-camera (surface-mode=true) or camera-rig.surface.mode=locked")
            }
            _ => None,
        };

        if let Some(required_input) = required_input {
            diagnostics.push(
                SceneDocumentDiagnostic::KnownCameraPresetMissingLegacyBinding {
                    preset: preset.to_string(),
                    required_input: required_input.to_string(),
                },
            );
        }
    }

    diagnostics
}

fn validate_runtime_object_kind_markers(scene_map: &Mapping) -> Vec<SceneDocumentDiagnostic> {
    let Some(runtime_objects) = scene_map
        .get(Value::String("runtime-objects".to_string()))
        .or_else(|| scene_map.get(Value::String("runtime_objects".to_string())))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    for (index, node) in runtime_objects.iter().enumerate() {
        validate_runtime_object_kind_node(
            node,
            format!("runtime-objects[{index}]"),
            &mut diagnostics,
        );
    }
    diagnostics
}

fn validate_runtime_object_kind_node(
    node: &Value,
    path: String,
    diagnostics: &mut Vec<SceneDocumentDiagnostic>,
) {
    let Some(node_map) = node.as_mapping() else {
        return;
    };

    if let Some(kind) = node_map
        .get(Value::String("kind".to_string()))
        .and_then(Value::as_str)
        .filter(|kind| *kind != "runtime-object")
    {
        diagnostics.push(SceneDocumentDiagnostic::RuntimeObjectKindMismatch {
            path: path.clone(),
            kind: kind.to_string(),
        });
    }

    if let Some(children) = node_map
        .get(Value::String("children".to_string()))
        .and_then(Value::as_sequence)
    {
        for (index, child) in children.iter().enumerate() {
            validate_runtime_object_kind_node(
                child,
                format!("{path}.children[{index}]"),
                diagnostics,
            );
        }
    }
}

fn mapping_get_str<'a>(map: &'a Mapping, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        map.get(Value::String((*key).to_string()))
            .and_then(Value::as_str)
    })
}

fn mapping_get_bool(map: &Mapping, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|key| {
        map.get(Value::String((*key).to_string()))
            .and_then(Value::as_bool)
    })
}

fn mapping_has_non_null(map: &Mapping, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        map.get(Value::String((*key).to_string()))
            .is_some_and(|value| !matches!(value, Value::Null))
    })
}

fn is_known_3d_camera_preset(preset: &str) -> bool {
    matches!(
        preset,
        "obj-viewer" | "free-look-camera" | "orbit-camera" | "surface-free-look"
    )
}

fn has_camera_rig_profile(
    input: Option<&Mapping>,
    camera_rig: Option<&Mapping>,
    aliases: &[&str],
) -> bool {
    input.is_some_and(|map| mapping_has_non_null(map, aliases))
        || camera_rig.is_some_and(|map| mapping_has_non_null(map, aliases))
}

fn surface_free_look_compat(input: Option<&Mapping>, camera_rig: Option<&Mapping>) -> bool {
    input.is_some_and(has_surface_free_look_compat)
        || camera_rig.is_some_and(has_surface_free_look_camera_rig_compat)
}

fn has_surface_free_look_compat(input: &Mapping) -> bool {
    let profile = input
        .get(Value::String("free-look-camera".to_string()))
        .or_else(|| input.get(Value::String("free_look_camera".to_string())))
        .and_then(Value::as_mapping);
    let Some(profile) = profile else {
        return false;
    };
    mapping_get_bool(profile, &["surface-mode", "surface_mode"]).unwrap_or(false)
}

fn has_surface_free_look_camera_rig_compat(camera_rig: &Mapping) -> bool {
    let surface_locked = camera_rig
        .get(Value::String("surface".to_string()))
        .and_then(Value::as_mapping)
        .and_then(|surface| mapping_get_str(surface, &["mode"]))
        .is_some_and(|mode| mode.eq_ignore_ascii_case("locked"));
    if surface_locked {
        return true;
    }
    camera_rig
        .get(Value::String("free-look-camera".to_string()))
        .or_else(|| camera_rig.get(Value::String("free_look_camera".to_string())))
        .and_then(Value::as_mapping)
        .and_then(|profile| mapping_get_bool(profile, &["surface-mode", "surface_mode"]))
        .unwrap_or(false)
}

fn infer_camera_rig_preset(camera_rig: Option<&Mapping>) -> Option<&str> {
    let camera_rig = camera_rig?;
    if let Some(preset) = mapping_get_str(camera_rig, &["preset"]) {
        return Some(preset);
    }
    if mapping_has_non_null(camera_rig, &["orbit-camera", "orbit_camera"]) {
        return Some("orbit-camera");
    }
    if mapping_has_non_null(camera_rig, &["free-look-camera", "free_look_camera"]) {
        if has_surface_free_look_camera_rig_compat(camera_rig) {
            return Some("surface-free-look");
        }
        return Some("free-look-camera");
    }
    if has_surface_free_look_camera_rig_compat(camera_rig) {
        return Some("surface-free-look");
    }
    if mapping_has_non_null(camera_rig, &["obj-viewer", "obj_viewer"]) {
        return Some("obj-viewer");
    }
    None
}

/// Validates sprite timeline against scene duration.
///
/// Returns warnings for sprites that will never be visible during on_enter stage
/// (the primary cutscene/intro timing for most scenes).
///
/// # Checks
/// - sprite `appear_at_ms` >= on_enter duration → sprite never visible
/// - sprite `disappear_at_ms` <= `appear_at_ms` → sprite always hidden
///
/// # Notes
/// This validation focuses on on_enter because that's where most authored
/// sprite timing lives. Sprites visible only during on_idle or on_leave
/// are uncommon and require runtime state control (layer.visible or Rhai).
pub fn validate_sprite_timeline(scene: &Scene) -> Vec<TimelineDiagnostic> {
    let mut diagnostics = Vec::new();
    let scene_duration = scene.on_enter_duration_ms();

    for layer in &scene.layers {
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            let (appear_at, disappear_at) = match sprite {
                Sprite::Text {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Image {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Obj {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Planet {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Vector {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                // Panel, Grid, Flex, Scene3D don't have disappear_at_ms timeline validation
                Sprite::Panel { .. }
                | Sprite::Grid { .. }
                | Sprite::Flex { .. }
                | Sprite::Scene3D { .. } => continue,
            };

            let appear = appear_at.unwrap_or(0);

            // Check if sprite appears after scene ends
            if scene_duration > 0 && appear >= scene_duration {
                diagnostics.push(TimelineDiagnostic::SpriteAppearsAfterSceneEnd {
                    layer_name: layer.name.clone(),
                    sprite_index: sprite_idx,
                    appear_at_ms: appear,
                    scene_duration_ms: scene_duration,
                });
            }

            // Check if sprite disappears before appearing
            if let Some(disappear) = disappear_at {
                if disappear <= appear {
                    diagnostics.push(TimelineDiagnostic::SpriteDisappearsBeforeAppear {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        appear_at_ms: appear,
                        disappear_at_ms: disappear,
                    });
                }
            }
        }
    }

    diagnostics
}

/// Validates authored text layout semantics for likely HUD mistakes.
///
/// These checks are warning-only and focus on contracts that authors are likely
/// to assume exist:
/// - `overflow-mode: ellipsis` needs `max-width` or `line-clamp`
/// - `line-clamp` expects `wrap-mode: word|char`
/// - `reserve-width-ch` should not be smaller than `max-width`
pub fn validate_text_layout_semantics(scene: &Scene) -> Vec<TextLayoutDiagnostic> {
    let mut diagnostics = Vec::new();

    for layer in &scene.layers {
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            let Sprite::Text {
                max_width,
                overflow_mode,
                wrap_mode,
                line_clamp,
                reserve_width_ch,
                ..
            } = sprite
            else {
                continue;
            };

            if matches!(overflow_mode, TextOverflowMode::Ellipsis)
                && max_width.is_none()
                && line_clamp.is_none()
            {
                diagnostics.push(TextLayoutDiagnostic::EllipsisWithoutBounds {
                    layer_name: layer.name.clone(),
                    sprite_index: sprite_idx,
                });
            }

            if let Some(clamp) = line_clamp {
                if matches!(wrap_mode, TextWrapMode::None) {
                    diagnostics.push(TextLayoutDiagnostic::LineClampWithoutWrap {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        line_clamp: *clamp,
                    });
                }
            }

            if let (Some(reserved), Some(max_width)) = (reserve_width_ch, max_width) {
                if reserved < max_width {
                    diagnostics.push(TextLayoutDiagnostic::ReserveWidthTooSmall {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        reserve_width_ch: *reserved,
                        max_width: *max_width,
                    });
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::scene::model::{SceneControllerDefaults, SceneWorldModel};
    use engine_core::scene::{Layer, Scene, SceneStages, Sprite, Stage, Step};
    use serde_yaml::Value;

    fn make_test_scene(on_enter_duration: u64) -> Scene {
        Scene {
            id: "test".into(),
            title: "Test".into(),
            cutscene: true,
            target_fps: None,
            space: Default::default(),
            world_model: SceneWorldModel::default(),
            controller_defaults: SceneControllerDefaults::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: None,
            view: None,
            planet_spec: None,
            planet_spec_ref: None,
            virtual_size_override: None,
            bg_colour: None,
            stages: SceneStages {
                on_enter: Stage {
                    trigger: Default::default(),
                    steps: vec![Step {
                        duration: Some(on_enter_duration),
                        effects: vec![],
                    }],
                    looping: false,
                },
                on_idle: Default::default(),
                on_leave: Default::default(),
            },
            behaviors: vec![],
            audio: Default::default(),
            gui: Default::default(),
            ui: Default::default(),
            layers: vec![],
            runtime_objects: vec![],
            menu_options: vec![],
            input: Default::default(),
            postfx: vec![],
            next: None,
            prerender: false,
            palette_bindings: vec![],
            game_state_bindings: vec![],
        }
    }

    fn make_text_sprite(appear_at_ms: Option<u64>, disappear_at_ms: Option<u64>) -> Sprite {
        Sprite::Text {
            id: Some("test".into()),
            content: "test".into(),
            x: 0,
            y: 0,
            z_index: 0,
            grid_row: 0,
            grid_col: 0,
            row_span: 1,
            col_span: 1,
            size: None,
            font: None,
            force_font_mode: None,
            align_x: None,
            align_y: None,
            fg_colour: None,
            bg_colour: None,
            appear_at_ms,
            disappear_at_ms,
            reveal_ms: None,
            hide_on_leave: false,
            visible: true,
            stages: Default::default(),
            animations: vec![],
            behaviors: vec![],
            glow: None,
            text_transform: Default::default(),
            max_width: None,
            overflow_mode: Default::default(),
            wrap_mode: Default::default(),
            line_clamp: None,
            reserve_width_ch: None,
            line_height: 1,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    #[test]
    fn valid_sprite_timeline_passes() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(100), Some(5000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert!(diags.is_empty(), "Valid timeline should pass");
    }

    #[test]
    fn sprite_appears_after_scene_end_warns() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(8200), Some(10000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TimelineDiagnostic::SpriteAppearsAfterSceneEnd { .. }
        ));
    }

    #[test]
    fn sprite_disappears_before_appear_warns() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(3000), Some(1000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TimelineDiagnostic::SpriteDisappearsBeforeAppear { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_ellipsis_without_bounds() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text { overflow_mode, .. } = &mut sprite {
            *overflow_mode = TextOverflowMode::Ellipsis;
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::EllipsisWithoutBounds { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_line_clamp_without_wrap() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text { line_clamp, .. } = &mut sprite {
            *line_clamp = Some(2);
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::LineClampWithoutWrap { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_reserved_width_smaller_than_max_width() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text {
            max_width,
            reserve_width_ch,
            wrap_mode,
            ..
        } = &mut sprite
        {
            *max_width = Some(12);
            *reserve_width_ch = Some(8);
            *wrap_mode = TextWrapMode::Word;
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::ReserveWidthTooSmall {
                reserve_width_ch: 8,
                max_width: 12,
                ..
            }
        ));
    }

    #[test]
    fn valid_text_layout_semantics_pass() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text {
            max_width,
            overflow_mode,
            wrap_mode,
            line_clamp,
            reserve_width_ch,
            ..
        } = &mut sprite
        {
            *max_width = Some(24);
            *overflow_mode = TextOverflowMode::Ellipsis;
            *wrap_mode = TextWrapMode::Word;
            *line_clamp = Some(2);
            *reserve_width_ch = Some(24);
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert!(diags.is_empty(), "Valid text layout semantics should pass");
    }

    #[test]
    fn document_semantics_require_celestial_world_model_for_celestial_block() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: euclidean-3d
celestial:
  focus-body: earth
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::CelestialRequiresCelestialWorldModel {
                    world_model: "euclidean-3d".to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_reject_planar_camera_profiles() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: planar-2d
input:
  orbit-camera:
    target: viewer
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraInput {
                    input_profile: "orbit-camera".to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_reject_planar_obj_viewer_profile() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: planar-2d
input:
  obj-viewer:
    sprite-id: probe
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraInput {
                    input_profile: "obj-viewer".to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_reject_planar_known_camera_preset() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: planar-2d
controller-defaults:
  camera-preset: obj-viewer
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::PlanarWorldModelDisallowsCameraPreset {
                    preset: "obj-viewer".to_string()
                },
                SceneDocumentDiagnostic::KnownCameraPresetMissingLegacyBinding {
                    preset: "obj-viewer".to_string(),
                    required_input: "input.obj-viewer or camera-rig.obj-viewer".to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_reject_empty_controller_presets() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
controller-defaults:
  camera-preset: "   "
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![SceneDocumentDiagnostic::EmptyControllerPreset {
                field: "camera-preset".to_string()
            }]
        );
    }

    #[test]
    fn document_semantics_require_matching_input_for_builtin_camera_preset() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: euclidean-3d
controller-defaults:
  camera-preset: orbit-camera
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::KnownCameraPresetMissingLegacyBinding {
                    preset: "orbit-camera".to_string(),
                    required_input: "input.orbit-camera or camera-rig.orbit-camera".to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_accept_surface_free_look_with_explicit_surface_mode() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
input:
  free-look-camera:
    surface-mode: true
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    }

    #[test]
    fn document_semantics_accept_runtime_object_kind_marker() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
runtime-objects:
  - name: root
    kind: runtime-object
    transform:
      space: 2d
      x: 1
      y: 2
    children:
      - name: child
        kind: runtime-object
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.0]
          rotation-deg: [0.0, 0.0, 0.0]
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    }

    #[test]
    fn document_semantics_reject_runtime_object_kind_mismatch() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
runtime-objects:
  - name: root
    kind: object
    transform:
      space: 2d
      x: 1
      y: 2
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![SceneDocumentDiagnostic::RuntimeObjectKindMismatch {
                path: "runtime-objects[0]".to_string(),
                kind: "object".to_string()
            }]
        );
    }

    #[test]
    fn document_semantics_reject_surface_free_look_without_explicit_surface_mode() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
input:
  free-look-camera: {}
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert_eq!(
            diags,
            vec![
                SceneDocumentDiagnostic::KnownCameraPresetMissingLegacyBinding {
                    preset: "surface-free-look".to_string(),
                    required_input:
                        "input.free-look-camera (surface-mode=true) or camera-rig.surface.mode=locked"
                            .to_string()
                }
            ]
        );
    }

    #[test]
    fn document_semantics_accept_surface_free_look_via_camera_rig_surface_contract() {
        let scene: Value = serde_yaml::from_str(
            r#"
id: test
title: Test
world-model: celestial-3d
camera-rig:
  surface:
    mode: locked
  free-look-camera: {}
"#,
        )
        .expect("scene doc");

        let diags = validate_scene_document_semantics(&scene);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    }
}
