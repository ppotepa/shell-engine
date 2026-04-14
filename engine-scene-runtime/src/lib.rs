//! Runtime scene materialization and object graph helpers derived from the
//! authored scene model.
//!
//! # Module Organization
//!
//! - **`SceneRuntime`**: Main runtime struct managing scene state, objects, and behaviors
//! - **`behavior_runner`**: Behavior attachment and update lifecycle
//! - **`object_graph`**: Object lookup and target resolution (TargetResolver)
//! - **`lifecycle_controls`**: Object viewer controls and other runtime-only input helpers
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
    resolve_ui_theme_or_default, BehaviorSpec, FreeLookCameraControls, ObjOrbitCameraControls,
    Scene, Sprite, TermColour, UiThemeStyle,
};
pub use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, RawKeyEvent, SceneCamera3D, SidecarIoFrameState,
    TargetResolver,
};
use engine_events::{KeyCode, KeyEvent, KeyModifiers};
pub(crate) use materialization::{find_text_layout_recursive, parse_term_colour};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, HashMap, HashSet};

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
    free_look_camera: Option<FreeLookCameraState>,
    orbit_camera: Option<ObjOrbitCameraState>,
    ui_state: UiRuntimeState,
    pending_bindings: Vec<BehaviorBinding>,
    action_bindings: HashMap<String, Vec<String>>,
    cached_action_bindings: Option<std::sync::Arc<HashMap<String, Vec<String>>>>,
    /// Previous frame's normalized collision pairs for enter/stay/exit computation.
    prev_collision_pairs: std::collections::HashSet<(u64, u64)>,
    /// Previous frame's held-key set — used to compute `keys_just_pressed`.
    prev_keys_down: std::collections::HashSet<String>,
    /// Previous frame's scene_elapsed_ms — used to compute per-frame delta.
    prev_scene_elapsed_ms: u64,
    /// World-space camera origin (top-left of the visible viewport in world pixels).
    /// Non-UI layers are offset by (-camera_x, -camera_y) during compositing.
    camera_x: i32,
    camera_y: i32,
    /// 2D camera zoom factor (default 1.0). Values > 1.0 zoom in, < 1.0 zoom out.
    camera_zoom: f32,
    scene_camera_3d: SceneCamera3D,
    /// Palette version when bindings were last applied; 0 means not yet applied.
    palette_applied_version: u64,
    /// GameState version when text bindings were last applied; 0 means not yet applied.
    game_state_applied_version: u64,
    /// Sprite `id` attr → layer index for O(1) property mutation lookup.
    sprite_id_to_layer: HashMap<String, usize>,
    /// When > 0, `refresh_runtime_caches()` is deferred (batch spawn mode).
    spawn_batch_depth: u32,
    /// GUI widget definitions (from scene.gui.widgets) — trait-based controls.
    gui_widgets: Vec<Box<dyn engine_gui::GuiControl>>,
    /// GUI runtime state: per-widget hover/press/value, mouse position.
    gui_state: engine_gui::GuiRuntimeState,
    /// Cached Arc wrapping gui_state for sharing with BehaviorContext (rebuilt on change).
    cached_gui_state: Option<std::sync::Arc<engine_gui::GuiRuntimeState>>,
}

#[derive(Debug, Clone)]
struct FreeLookCameraState {
    active: bool,
    pending_activate: bool,
    position: [f32; 3],
    yaw_deg: f32,
    pitch_deg: f32,
    move_speed: f32,
    mouse_sensitivity: f32,
    last_mouse_pos: Option<(f32, f32)>,
    held_keys: HashSet<String>,
}

impl FreeLookCameraState {
    fn from_controls(controls: &FreeLookCameraControls) -> Self {
        Self {
            active: false,
            pending_activate: false,
            position: [0.0, 0.0, 0.0],
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            move_speed: controls.move_speed,
            mouse_sensitivity: controls.mouse_sensitivity,
            last_mouse_pos: None,
            held_keys: HashSet::new(),
        }
    }
}

/// Runtime state for the orbit camera — arcs around a single OBJ sprite target.
#[derive(Debug, Clone)]
struct ObjOrbitCameraState {
    target: String,
    active: bool,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pitch_min: f32,
    pitch_max: f32,
    distance_min: f32,
    distance_max: f32,
    distance_step: f32,
    drag_sensitivity: f32,
    last_mouse_pos: Option<(f32, f32)>,
    /// Auto-rotation speed saved when orbit activates, restored on deactivate.
    paused_orbit_speed: f32,
}

impl ObjOrbitCameraState {
    fn from_controls(controls: &ObjOrbitCameraControls) -> Self {
        Self {
            target: controls.target.clone(),
            active: false,
            yaw: controls.yaw,
            pitch: controls.pitch,
            distance: controls.distance,
            pitch_min: controls.pitch_min,
            pitch_max: controls.pitch_max,
            distance_min: controls.distance_min,
            distance_max: controls.distance_max,
            distance_step: controls.distance_step,
            drag_sensitivity: controls.drag_sensitivity,
            last_mouse_pos: None,
            paused_orbit_speed: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct PanelLayoutSpec {
    width: u16,
    border_width: u16,
    padding: u16,
    height: u16,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    use engine_core::scene::{Scene, Sprite, TermColour};

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

    fn planet_scene(extra_fields: &str) -> Scene {
        serde_yaml::from_str(&format!(
            r#"
id: planet-scene
title: Planet
layers:
  - name: planet
    sprites:
      - type: planet
        id: main-planet-view
        body-id: main-planet
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
    fn scene_despawn_removes_spawned_clone_from_scene_tree_and_runtime() {
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
        let baseline_objects = runtime.object_count();
        let baseline_sprites = runtime
            .scene()
            .layers
            .iter()
            .map(|layer| layer.sprites.len())
            .sum::<usize>();

        for _ in 0..8 {
            let resolver = runtime.target_resolver();
            runtime.apply_behavior_commands(
                &resolver,
                &[BehaviorCommand::SceneSpawn {
                    template: "rock-template".to_string(),
                    target: "rock-live".to_string(),
                }],
            );
            assert!(runtime
                .target_resolver()
                .resolve_alias("rock-live")
                .is_some());
            assert_eq!(
                runtime
                    .scene()
                    .layers
                    .iter()
                    .map(|layer| layer.sprites.len())
                    .sum::<usize>(),
                baseline_sprites + 1
            );

            let resolver = runtime.target_resolver();
            runtime.apply_behavior_commands(
                &resolver,
                &[BehaviorCommand::SceneDespawn {
                    target: "rock-live".to_string(),
                }],
            );
            assert!(runtime
                .target_resolver()
                .resolve_alias("rock-live")
                .is_none());
            assert_eq!(
                runtime
                    .scene()
                    .layers
                    .iter()
                    .map(|layer| layer.sprites.len())
                    .sum::<usize>(),
                baseline_sprites
            );
            assert_eq!(runtime.object_count(), baseline_objects);
        }
    }

    #[test]
    fn rebuild_keeps_runtime_clone_alias_reserved_for_layer() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: clone-rebuild
title: Clone Rebuild
layers:
  - name: ship-template
    visible: false
    sprites:
      - type: vector
        id: ship-body
        points: [[0, 0], [2, 0], [1, 1]]
  - name: fx-template
    visible: false
    sprites:
      - type: vector
        id: fx-body
        points: [[0, 0], [1, 0], [0, 1]]
"#,
        )
        .expect("scene should parse");
        let mut runtime = SceneRuntime::new(scene);

        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SceneSpawn {
                    template: "ship-template".to_string(),
                    target: "ship-1".to_string(),
                },
                BehaviorCommand::SceneSpawn {
                    template: "fx-template".to_string(),
                    target: "fx-1".to_string(),
                },
            ],
        );

        let ship_layer_id = runtime
            .target_resolver()
            .resolve_alias("ship-1")
            .expect("ship clone should resolve")
            .to_string();
        let ship_child_id = runtime
            .object(&ship_layer_id)
            .expect("ship clone layer")
            .children
            .first()
            .expect("ship clone child")
            .clone();
        assert_eq!(
            runtime
                .object(&ship_child_id)
                .expect("ship clone child object")
                .name,
            "ship-1"
        );
        assert!(matches!(
            runtime
                .object(&ship_layer_id)
                .expect("ship clone layer")
                .kind,
            GameObjectKind::Layer
        ));
        assert!(runtime
            .object(&ship_child_id)
            .expect("ship clone child object")
            .aliases
            .is_empty());

        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SceneDespawn {
                target: "fx-1".to_string(),
            }],
        );

        let ship_layer_id = runtime
            .target_resolver()
            .resolve_alias("ship-1")
            .expect("ship clone should still resolve after rebuild")
            .to_string();
        let ship_layer = runtime.object(&ship_layer_id).expect("ship clone layer");
        assert!(matches!(ship_layer.kind, GameObjectKind::Layer));
        let ship_child_id = ship_layer
            .children
            .first()
            .expect("ship clone child")
            .clone();
        assert_eq!(
            runtime
                .object(&ship_child_id)
                .expect("ship clone child object")
                .name,
            "ship-1"
        );
        assert!(runtime
            .object(&ship_child_id)
            .expect("ship clone child object")
            .aliases
            .is_empty());
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
                    value: serde_json::json!("generic:2"),
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
        assert_eq!(text_style.0.as_deref(), Some("generic:2"));
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
        assert_eq!(state.offset_x, 10);
        assert_eq!(state.offset_y, -2);
    }

    #[test]
    fn apply_behavior_commands_set_camera_rounds_to_nearest_pixel() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime
            .apply_behavior_commands(&resolver, &[BehaviorCommand::SetCamera { x: 9.8, y: -2.4 }]);
        assert_eq!(runtime.camera(), (10, -2));
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
    fn apply_behavior_commands_set_property_updates_planet_paths() {
        let mut runtime = SceneRuntime::new(planet_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.spin_deg".to_string(),
                    value: serde_json::json!(15),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.cloud_spin_deg".to_string(),
                    value: serde_json::json!(21),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.cloud2_spin_deg".to_string(),
                    value: serde_json::json!(33),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.observer_altitude_km".to_string(),
                    value: serde_json::json!(420),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.sun_dir.x".to_string(),
                    value: serde_json::json!(0.5),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.sun_dir.y".to_string(),
                    value: serde_json::json!(-0.4),
                },
                BehaviorCommand::SetProperty {
                    target: "main-planet-view".to_string(),
                    path: "planet.sun_dir.z".to_string(),
                    value: serde_json::json!(0.2),
                },
            ],
        );
        let planet_props = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Planet {
                    id,
                    spin_deg,
                    cloud_spin_deg,
                    cloud2_spin_deg,
                    observer_altitude_km,
                    sun_dir_x,
                    sun_dir_y,
                    sun_dir_z,
                    ..
                } if id.as_deref() == Some("main-planet-view") => Some((
                    *spin_deg,
                    *cloud_spin_deg,
                    *cloud2_spin_deg,
                    *observer_altitude_km,
                    *sun_dir_x,
                    *sun_dir_y,
                    *sun_dir_z,
                )),
                _ => None,
            })
            .expect("planet properties");
        assert_eq!(planet_props.0, Some(15.0));
        assert_eq!(planet_props.1, Some(21.0));
        assert_eq!(planet_props.2, Some(33.0));
        assert_eq!(planet_props.3, Some(420.0));
        assert_eq!(planet_props.4, Some(0.5));
        assert_eq!(planet_props.5, Some(-0.4));
        assert_eq!(planet_props.6, Some(0.2));
    }

    #[test]
    fn adjusts_obj_scale_for_target_sprite_id() {
        let mut runtime = SceneRuntime::new(obj_scene("        scale: 1.0"));
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

}
