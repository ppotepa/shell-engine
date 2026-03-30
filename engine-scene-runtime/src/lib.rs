//! Runtime scene materialization and object graph helpers derived from the
//! authored scene model.
//!
//! # Module Organization
//!
//! - **`SceneRuntime`**: Main runtime struct managing scene state, objects, and behaviors
//! - **`behavior_runner`**: Behavior attachment and update lifecycle
//! - **`object_graph`**: Object lookup and target resolution (TargetResolver)
//! - **`lifecycle_controls`**: Terminal shell, object viewer, and size tester controls
//! - **`terminal_shell`**: Terminal UI state and command input handling
//! - **`construction`**: Scene runtime initialization and object materialization
//! - **`materialization`**: Sprite and text rendering helpers
//! - **`camera_3d`**: 3D camera state management for object sprites
//! - **`ui_focus`**: UI panel focus and navigation tracking
//! - **`access`**: Public trait interface for external access to scene runtime

pub mod access;
pub mod behavior_runner;
pub mod camera_3d;
pub mod construction;
pub mod lifecycle_controls;
pub mod materialization;
pub mod object_graph;
pub mod terminal_shell;
pub mod ui_focus;

pub use access::SceneRuntimeAccess;
use engine_animation::SceneStage;
use engine_behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, RhaiScriptBehavior,
    SceneAudioBehavior,
};
use engine_behavior_registry::ModBehaviorRegistry;
use engine_core::effects::Region;
use engine_core::game_object::{GameObject, GameObjectKind};
use engine_core::scene::{
    resolve_ui_theme_or_default, BehaviorSpec, Scene, SceneRenderedMode, Sprite, TermColour,
    TerminalShellControls, UiThemeStyle,
};
pub use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, RawKeyEvent, SidecarIoFrameState, TargetResolver,
};
use engine_events::{KeyCode, KeyEvent, KeyModifiers};
use engine_render_terminal::rasterizer::generic::GenericMode;
pub use lifecycle_controls::TerminalShellRoute;
pub(crate) use materialization::{find_text_layout_recursive, parse_term_colour};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, HashMap, HashSet};
#[cfg(test)]
pub(crate) use terminal_shell::wrap_text_to_width;
use tui_input::{Input, InputRequest};
pub(crate) use ui_focus::{find_panel_layout_recursive, set_panel_height_recursive};

/// Materialized runtime view of a [`Scene`] with stable object ids, behavior
/// bindings, and per-frame mutable state.
pub struct SceneRuntime {
    scene: Scene,
    root_id: String,
    objects: HashMap<String, GameObject>,
    object_states: HashMap<String, ObjectRuntimeState>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: HashMap<String, String>,
    behaviors: Vec<ObjectBehaviorRuntime>,
    resolver_cache: std::sync::Arc<TargetResolver>,
    object_regions: std::sync::Arc<HashMap<String, Region>>,
    cached_object_kinds: std::sync::Arc<HashMap<String, String>>,
    object_mutation_gen: u64,
    cached_object_states_gen: u64,
    cached_effective_states_gen: u64,
    cached_object_props_gen: u64,
    cached_object_text_gen: u64,
    cached_object_states: Option<std::sync::Arc<HashMap<String, ObjectRuntimeState>>>,
    cached_effective_states: Option<std::sync::Arc<HashMap<String, ObjectRuntimeState>>>,
    effective_states_dirty: bool,
    cached_object_props: Option<std::sync::Arc<HashMap<String, serde_json::Value>>>,
    cached_object_text: Option<std::sync::Arc<HashMap<String, String>>>,
    cached_sidecar_io: Option<std::sync::Arc<SidecarIoFrameState>>,
    cached_object_regions: std::sync::Arc<HashMap<String, Region>>,
    obj_orbit_default_speed: HashMap<String, f32>,
    obj_camera_states: HashMap<String, ObjCameraState>,
    cached_obj_camera_states: Option<std::sync::Arc<HashMap<String, ObjCameraState>>>,
    terminal_shell_state: Option<TerminalShellState>,
    terminal_shell_scene_elapsed_ms: u64,
    ui_state: UiRuntimeState,
    pending_bindings: Vec<BehaviorBinding>,
    action_bindings: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct TerminalShellState {
    controls: TerminalShellControls,
    input: Input,
    input_masked: bool,
    sidecar_fullscreen_mode: bool,
    output_lines: Vec<String>,
    history: Vec<String>,
    history_cursor: Option<usize>,
    prompt_panel_height: Option<f32>,
    last_layout_sync_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct PanelLayoutSpec {
    width: u16,
    border_width: u16,
    padding: u16,
    height: u16,
}

#[derive(Debug, Clone)]
struct TextLayoutSpec {
    x: i32,
    y: i32,
    font: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ObjSpritePropertySnapshot {
    scale: Option<f32>,
    yaw: Option<f32>,
    pitch: Option<f32>,
    roll: Option<f32>,
    orbit_speed: Option<f32>,
    surface_mode: Option<String>,
    #[allow(dead_code)]
    clip_y_min: Option<f32>,
    #[allow(dead_code)]
    clip_y_max: Option<f32>,
}

#[derive(Debug, Clone)]
struct UiTextEvent {
    target_id: String,
    text: String,
}

#[derive(Debug, Clone, Default)]
struct UiRuntimeState {
    focus_order: Vec<String>,
    focused_index: usize,
    theme_id: Option<String>,
    theme_style: Option<UiThemeStyle>,
    last_submit: Option<UiTextEvent>,
    last_change: Option<UiTextEvent>,
    submit_seq: u64,
    change_seq: u64,
    pub last_raw_key: Option<RawKeyEvent>,
    pub keys_down: HashSet<String>,
    pub sidecar_io: SidecarIoFrameState,
}

struct ObjectBehaviorRuntime {
    object_id: String,
    behavior: Box<dyn Behavior + Send + Sync>,
}

struct BehaviorBinding {
    object_id: String,
    specs: Vec<BehaviorSpec>,
}

#[cfg(test)]
mod tests {
    use super::SceneRuntime;
    use engine_behavior::BehaviorCommand;
    use engine_core::game_object::GameObjectKind;
    use engine_core::scene::{Scene, SceneRenderedMode, Sprite, TermColour};

    fn intro_scene() -> Scene {
        serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: UI
    sprites:
      - type: grid
        id: root-grid
        width: 10
        height: 5
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse")
    }

    fn obj_scene(extra_fields: &str) -> Scene {
        serde_yaml::from_str(&format!(
            r#"
id: playground-3d-scene
title: 3D
bg_colour: black
layers:
  - name: obj
    sprites:
      - type: obj
        id: helsinki-uni-wireframe
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
{extra_fields}"#
        ))
        .expect("scene should parse")
    }

    #[test]
    fn builds_object_hierarchy_for_layers_and_nested_sprites() {
        let runtime = SceneRuntime::new(intro_scene());

        assert_eq!(runtime.object_count(), 4);
        let root = runtime
            .object(runtime.root_id())
            .expect("scene root should exist");
        assert_eq!(root.kind, GameObjectKind::Scene);
        assert_eq!(root.children.len(), 1);

        let grid = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::GridSprite)
            .expect("grid object");
        assert_eq!(grid.children.len(), 1);

        let text = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::TextSprite)
            .expect("text object");
        assert_eq!(text.parent_id.as_deref(), Some(grid.id.as_str()));
    }

    #[test]
    fn target_resolver_supports_alias_lookup_and_sprite_paths() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: HUD
    sprites:
      - type: grid
        id: root-grid
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();

        let title_id = resolver.resolve_alias("title").expect("title alias");
        assert_eq!(resolver.resolve_alias("HUD"), resolver.layer_object_id(0));
        assert_eq!(resolver.sprite_object_id(0, &[0, 0]), Some(title_id));
    }

    #[test]
    fn resolves_ui_theme_in_runtime_state() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: ui-theme-runtime
title: UI Theme Runtime
ui:
  theme: windows_98
layers: []
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        assert_eq!(runtime.ui_theme_id(), Some("win98"));
        let style = runtime.ui_theme_style().expect("theme style");
        assert_eq!(style.id, "win98");
    }

    #[test]
    fn falls_back_to_engine_default_theme_when_ui_theme_missing() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: ui-theme-runtime-default
title: UI Theme Runtime Default
layers: []
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        assert_eq!(runtime.ui_theme_id(), Some("engine-default"));
        let style = runtime.ui_theme_style().expect("theme style");
        assert_eq!(style.id, "engine-default");
    }

    #[test]
    fn effective_object_state_accumulates_parent_visibility_and_offsets() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetOffset {
                    target: "intro".to_string(),
                    dx: 1,
                    dy: 0,
                },
                BehaviorCommand::SetVisibility {
                    target: "UI".to_string(),
                    visible: false,
                },
                BehaviorCommand::SetOffset {
                    target: "UI".to_string(),
                    dx: 2,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "root-grid".to_string(),
                    dx: 3,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "title".to_string(),
                    dx: 4,
                    dy: 0,
                },
            ],
        );

        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .effective_object_state(title_id)
            .expect("effective state");

        assert!(!state.visible);
        assert_eq!(state.offset_x, 10);
        assert_eq!(state.offset_y, 0);
    }

    #[test]
    fn apply_behavior_commands_updates_text_content_from_set_text() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetText {
                target: "title".to_string(),
                text: "WORLD".to_string(),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("WORLD"));
    }

    #[test]
    fn apply_behavior_commands_updates_text_content_from_runtime_target_alias() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_runtime_id = resolver
            .resolve_alias("title")
            .expect("title runtime object id")
            .to_string();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetText {
                target: title_runtime_id,
                text: "UPDATED".to_string(),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("UPDATED"));
    }

    #[test]
    fn scene_spawn_clones_only_target_sprite_subtree() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: spawn-clone
title: Spawn Clone
layers:
  - name: main
    sprites:
      - type: vector
        id: rock-template
        points: [[0, 0], [2, 0], [1, 1]]
      - type: text
        id: hud
        content: HUD
"#,
        )
        .expect("scene should parse");
        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SceneSpawn {
                template: "rock-template".to_string(),
                target: "rock-live".to_string(),
            }],
        );

        assert_eq!(runtime.scene().layers.len(), 1);
        assert_eq!(
            runtime
                .scene()
                .layers
                .iter()
                .map(|layer| layer.sprites.len())
                .sum::<usize>(),
            3
        );
        assert_eq!(
            runtime
                .objects()
                .filter(|object| matches!(object.kind, GameObjectKind::Layer))
                .count(),
            1
        );
        let live_id = runtime
            .target_resolver()
            .resolve_alias("rock-live")
            .expect("spawned sprite alias should resolve")
            .to_string();
        let live_object = runtime.object(&live_id).expect("spawned object");
        assert!(matches!(live_object.kind, GameObjectKind::VectorSprite));
    }

    #[test]
    fn apply_behavior_commands_set_props_updates_state_and_text() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetProps {
                target: "title".to_string(),
                visible: Some(false),
                dx: Some(3),
                dy: Some(-1),
                text: Some("PROPS".to_string()),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("PROPS"));
        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .object_state(title_id)
            .expect("object runtime state");
        assert!(!state.visible);
        assert_eq!(state.offset_x, 3);
        assert_eq!(state.offset_y, -1);
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_runtime_object_paths() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "visible".to_string(),
                    value: serde_json::json!(false),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.x".to_string(),
                    value: serde_json::json!(9),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.y".to_string(),
                    value: serde_json::json!(-2),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "text.content".to_string(),
                    value: serde_json::json!("PATH-SET"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "text.font".to_string(),
                    value: serde_json::json!("generic:half"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "style.fg".to_string(),
                    value: serde_json::json!("yellow"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "style.bg".to_string(),
                    value: serde_json::json!("#112233"),
                },
            ],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("PATH-SET"));
        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .object_state(title_id)
            .expect("object runtime state");
        assert!(!state.visible);
        assert_eq!(state.offset_x, 9);
        assert_eq!(state.offset_y, -2);
        let text_style = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Grid { children, .. } => children.iter().find_map(|child| match child {
                    Sprite::Text {
                        id,
                        font,
                        fg_colour,
                        bg_colour,
                        ..
                    } if id.as_deref() == Some("title") => {
                        Some((font.clone(), fg_colour.clone(), bg_colour.clone()))
                    }
                    _ => None,
                }),
                _ => None,
            })
            .expect("text style");
        assert_eq!(text_style.0.as_deref(), Some("generic:half"));
        assert_eq!(text_style.1, Some(TermColour::Yellow));
        assert_eq!(text_style.2, Some(TermColour::Rgb(0x11, 0x22, 0x33)));
    }

    #[test]
    fn apply_behavior_commands_set_property_accepts_float_positions() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.x".to_string(),
                    value: serde_json::json!(9.8),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.y".to_string(),
                    value: serde_json::json!(-2.4),
                },
            ],
        );
        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .object_state(title_id)
            .expect("object runtime state");
        assert_eq!(state.offset_x, 9);
        assert_eq!(state.offset_y, -2);
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_obj_paths() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.scale".to_string(),
                    value: serde_json::json!(1.5),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.yaw".to_string(),
                    value: serde_json::json!(15),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.pitch".to_string(),
                    value: serde_json::json!(-10),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.roll".to_string(),
                    value: serde_json::json!(2),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.orbit_speed".to_string(),
                    value: serde_json::json!(22),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.surface_mode".to_string(),
                    value: serde_json::json!("wireframe"),
                },
            ],
        );
        let obj_props = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    scale,
                    yaw_deg,
                    pitch_deg,
                    roll_deg,
                    rotate_y_deg_per_sec,
                    surface_mode,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => Some((
                    *scale,
                    *yaw_deg,
                    *pitch_deg,
                    *roll_deg,
                    *rotate_y_deg_per_sec,
                    surface_mode.clone(),
                )),
                _ => None,
            })
            .expect("obj properties");
        assert_eq!(obj_props.0, Some(1.5));
        assert_eq!(obj_props.1, Some(15.0));
        assert_eq!(obj_props.2, Some(-10.0));
        assert_eq!(obj_props.3, Some(2.0));
        assert_eq!(obj_props.4, Some(22.0));
        assert_eq!(obj_props.5.as_deref(), Some("wireframe"));
    }

    #[test]
    fn adjusts_obj_scale_for_target_sprite_id() {
        let mut runtime = SceneRuntime::new(obj_scene("        scale: 1.0"));
        runtime.set_scene_rendered_mode(SceneRenderedMode::Braille);
        assert!(runtime.adjust_obj_scale("helsinki-uni-wireframe", 0.2));

        let obj_scale = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj { id, scale, .. }
                    if id.as_deref() == Some("helsinki-uni-wireframe") =>
                {
                    *scale
                }
                _ => None,
            })
            .expect("obj scale");
        assert!((obj_scale - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn toggles_obj_surface_mode() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        assert!(runtime.toggle_obj_surface_mode("helsinki-uni-wireframe"));
        let mode = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id, surface_mode, ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => surface_mode.clone(),
                _ => None,
            })
            .expect("surface mode");
        assert_eq!(mode, "wireframe");
    }

    #[test]
    fn toggles_obj_orbit_speed_on_and_off() {
        let mut runtime = SceneRuntime::new(obj_scene("        rotate-y-deg-per-sec: 14"));
        assert!(runtime.toggle_obj_orbit("helsinki-uni-wireframe"));
        let speed_off = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    rotate_y_deg_per_sec,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *rotate_y_deg_per_sec,
                _ => None,
            })
            .expect("orbit speed");
        assert!((speed_off - 0.0).abs() < f32::EPSILON);

        assert!(runtime.toggle_obj_orbit("helsinki-uni-wireframe"));
        let speed_on = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    rotate_y_deg_per_sec,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *rotate_y_deg_per_sec,
                _ => None,
            })
            .expect("orbit speed");
        assert!((speed_on - 14.0).abs() < f32::EPSILON);
    }

    // ── wrap_text_to_width tests ─────────────────────────────────────

    #[test]
    fn wrap_plain_text_fits() {
        let result = super::wrap_text_to_width("hello", 10);
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn wrap_plain_text_exact() {
        let result = super::wrap_text_to_width("abcde", 5);
        assert_eq!(result, vec!["abcde"]);
    }

    #[test]
    fn wrap_word_boundary() {
        let result = super::wrap_text_to_width("hello world foo", 11);
        assert_eq!(result, vec!["hello world", "foo"]);
    }

    #[test]
    fn wrap_does_not_break_mid_word() {
        let result = super::wrap_text_to_width("the available memory", 10);
        assert_eq!(result, vec!["the", "available", "memory"]);
    }

    #[test]
    fn wrap_long_word_hard_break() {
        let result = super::wrap_text_to_width("abcdefghij", 4);
        assert_eq!(result, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn wrap_preserves_newlines() {
        let result = super::wrap_text_to_width("abc\ndefgh ij", 6);
        assert_eq!(result, vec!["abc", "defgh", "ij"]);
    }

    #[test]
    fn wrap_empty_line() {
        let result = super::wrap_text_to_width("", 10);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn wrap_markup_zero_width() {
        let result = super::wrap_text_to_width("[red]abcde[/]", 5);
        assert_eq!(result, vec!["[red]abcde[/]"]);
    }

    #[test]
    fn wrap_markup_overflow_carries_colour() {
        let result = super::wrap_text_to_width("[red]hello world[/]", 5);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "[red]hello[/]");
        assert_eq!(result[1], "[red]world[/]");
    }

    #[test]
    fn wrap_mixed_markup_and_plain() {
        // "xx " = 3 visible + "[green]yy[/]" = 2 visible = 5 total → fits on one line
        let result = super::wrap_text_to_width("xx [green]yy[/] zz", 5);
        assert_eq!(result, vec!["xx [green]yy[/]", "zz"]);
    }
}
