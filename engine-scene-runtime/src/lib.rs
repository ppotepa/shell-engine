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
pub mod dirty_tracking;
pub mod lifecycle_controls;
pub mod materialization;
pub mod mutations;
pub mod object_graph;
pub mod render3d_state;
pub mod request_adapter;
pub mod ui_focus;

pub use access::SceneRuntimeAccess;
pub use dirty_tracking::{dirty_for_render3d_mutation, dirty_for_scene_mutation};
use engine_animation::SceneStage;
use engine_behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, RhaiScriptBehavior,
    SceneAudioBehavior,
};
use engine_behavior_registry::ModBehaviorRegistry;
use engine_core::effects::Region;
use engine_core::game_object::{GameObject, GameObjectKind};
use engine_core::render_types::DirtyMask3D;
use engine_core::scene::{
    resolve_ui_theme_or_default, BehaviorSpec, FreeLookCameraControls, LightingProfile,
    ObjOrbitCameraControls, ResolvedViewProfile, Scene, SpaceEnvironmentProfile, Sprite,
    TermColour, UiThemeStyle,
};
pub use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, RawKeyEvent, SceneCamera3D, SidecarIoFrameState,
    TargetResolver,
};
use engine_core::spatial::SpatialContext;
use engine_events::{KeyCode, KeyEvent, KeyModifiers};
pub(crate) use materialization::{find_text_layout_recursive, parse_term_colour};
pub use mutations::{
    LightingProfileParam, Render3DGroupedParam, Render3DMutation, Render3DProfileParam,
    Render3DProfileSlot, SceneMutation, Set2DPropsMutation, SetCamera2DMutation,
    SetSpritePropertyMutation, SpaceEnvironmentParam,
};
pub use render3d_state::{scene_mutation_from_render_path, Render3dRebuildDiagnostics};
pub use request_adapter::{render3d_mutation_from_request, scene_mutation_from_request};
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
    /// Scene-wide spatial contract (units and axis convention).
    spatial_context: SpatialContext,
    scene_camera_3d: SceneCamera3D,
    resolved_view_profile: ResolvedViewProfile,
    runtime_lighting_profile_override: Option<LightingProfile>,
    runtime_space_environment_override: Option<SpaceEnvironmentProfile>,
    render3d_dirty_mask: DirtyMask3D,
    render3d_rebuild_diagnostics: Render3dRebuildDiagnostics,
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
    surface_mode: bool,
    surface_center: [f32; 3],
    surface_radius: f32,
    surface_altitude: f32,
    surface_min_altitude: f32,
    surface_max_altitude: f32,
    surface_vertical_speed: f32,
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
            surface_mode: controls.surface_mode,
            surface_center: [
                controls.surface_center_x,
                controls.surface_center_y,
                controls.surface_center_z,
            ],
            surface_radius: controls.surface_radius.max(0.001),
            surface_altitude: controls.surface_altitude.max(0.0),
            surface_min_altitude: controls.surface_min_altitude.max(0.0),
            surface_max_altitude: controls.surface_max_altitude.max(0.0),
            surface_vertical_speed: controls.surface_vertical_speed.max(0.001),
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
}

impl ObjOrbitCameraState {
    fn from_controls(controls: &ObjOrbitCameraControls) -> Self {
        Self {
            target: controls.target.clone(),
            active: true,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RuntimeMutationImpact {
    state: bool,
    props: bool,
    text: bool,
    layout: bool,
    graph: bool,
}

impl RuntimeMutationImpact {
    const NONE: Self = Self {
        state: false,
        props: false,
        text: false,
        layout: false,
        graph: false,
    };

    const fn state() -> Self {
        Self {
            state: true,
            props: false,
            text: false,
            layout: false,
            graph: false,
        }
    }

    const fn props() -> Self {
        Self {
            state: false,
            props: true,
            text: false,
            layout: false,
            graph: false,
        }
    }

    const fn text() -> Self {
        Self {
            state: false,
            props: false,
            text: true,
            layout: false,
            graph: false,
        }
    }

    const fn layout() -> Self {
        Self {
            state: false,
            props: false,
            text: false,
            layout: true,
            graph: false,
        }
    }

    const fn graph() -> Self {
        Self {
            state: false,
            props: false,
            text: false,
            layout: false,
            graph: true,
        }
    }

    fn with_layout(mut self) -> Self {
        self.layout = true;
        self
    }

    fn merge(&mut self, other: Self) {
        self.state |= other.state;
        self.props |= other.props;
        self.text |= other.text;
        self.layout |= other.layout;
        self.graph |= other.graph;
    }

    fn is_empty(self) -> bool {
        self == Self::NONE
    }

    fn bumps_object_mutation_gen(self) -> bool {
        self.state || self.props || self.text || self.graph
    }

    fn invalidates_object_states(self) -> bool {
        self.state || self.graph
    }

    fn invalidates_object_props(self) -> bool {
        self.props || self.graph
    }

    fn invalidates_object_text(self) -> bool {
        self.text || self.graph
    }

    fn invalidates_layout_regions(self) -> bool {
        self.layout || self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::{ObjOrbitCameraState, ObjectBehaviorRuntime, SceneRuntime};
    use engine_animation::SceneStage;
    use engine_api::commands::scene_mutation_request_from_set_path;
    use engine_api::scene::{Render3dMutationRequest, SceneMutationRequest};
    use engine_behavior::{Behavior, BehaviorCommand, BehaviorContext};
    use engine_core::effects::Region;
    use engine_core::game_object::{GameObject, GameObjectKind};
    use engine_core::render_types::DirtyMask3D;
    use engine_core::scene::{Scene, Sprite, TermColour};
    use engine_core::scene_runtime_types::ObjectRuntimeState;
    use engine_gui::{GuiControl, SliderControl, TextInputControl};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn set_path(
        target: &str,
        path: &str,
        value: serde_json::Value,
        current_state: Option<&ObjectRuntimeState>,
    ) -> BehaviorCommand {
        BehaviorCommand::ApplySceneMutation {
            request: scene_mutation_request_from_set_path(target, path, &value, current_state)
                .expect("typed mutation"),
        }
    }

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

    fn scene3d_scene(extra_fields: &str) -> Scene {
        serde_yaml::from_str(&format!(
            r#"
id: scene3d-scene
title: Scene3D
layers:
  - name: cutscene
    sprites:
      - type: scene3_d
        id: intro-view
        src: /assets/3d/sample.scene3d.yml
        frame: idle
{extra_fields}"#
        ))
        .expect("scene should parse")
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ObservedBehaviorSnapshot {
        text: Option<String>,
        font: Option<String>,
        offset_x: Option<i32>,
        region_width: Option<u16>,
        layout_regions_stale: bool,
    }

    struct MutateBehavior;

    impl Behavior for MutateBehavior {
        fn update(
            &mut self,
            _object: &GameObject,
            _scene: &Scene,
            _ctx: &BehaviorContext,
            commands: &mut Vec<BehaviorCommand>,
        ) {
            commands.push(BehaviorCommand::SetText {
                target: "title".to_string(),
                text: "UPDATED".to_string(),
            });
            commands.push(set_path(
                "title",
                "text.font",
                serde_json::json!("generic:2"),
                None,
            ));
            commands.push(BehaviorCommand::SetOffset {
                target: "title".to_string(),
                dx: 4,
                dy: 0,
            });
        }
    }

    struct VisibilityOnlyMutateBehavior;

    impl Behavior for VisibilityOnlyMutateBehavior {
        fn update(
            &mut self,
            _object: &GameObject,
            _scene: &Scene,
            _ctx: &BehaviorContext,
            commands: &mut Vec<BehaviorCommand>,
        ) {
            commands.push(BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: false,
            });
        }
    }

    struct ObserveBehavior {
        observed: Arc<Mutex<Option<ObservedBehaviorSnapshot>>>,
    }

    impl Behavior for ObserveBehavior {
        fn update(
            &mut self,
            _object: &GameObject,
            _scene: &Scene,
            ctx: &BehaviorContext,
            _commands: &mut Vec<BehaviorCommand>,
        ) {
            let Some(title_id) = ctx.resolve_target("title") else {
                return;
            };
            let font = ctx
                .object_props
                .get(title_id)
                .and_then(|props| props.get("text"))
                .and_then(|text| text.get("font"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            let snapshot = ObservedBehaviorSnapshot {
                text: ctx.object_text.get(title_id).cloned(),
                font,
                offset_x: ctx.object_state(title_id).map(|state| state.offset_x),
                region_width: ctx.object_region(title_id).map(|region| region.width),
                layout_regions_stale: ctx.layout_regions_stale,
            };
            *self.observed.lock().expect("observer lock") = Some(snapshot);
        }
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
            &[BehaviorCommand::ApplySceneMutation {
                request: engine_api::scene::SceneMutationRequest::SpawnObject {
                    template: "rock-template".to_string(),
                    target: "rock-live".to_string(),
                },
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
    fn typed_spawn_and_despawn_requests_follow_runtime_spawn_pipeline() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: spawn-clone-typed
title: Spawn Clone Typed
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
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SpawnObject {
                    template: "rock-template".to_string(),
                    target: "rock-live".to_string(),
                },
            }],
        );

        assert!(runtime
            .target_resolver()
            .resolve_alias("rock-live")
            .is_some());

        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::DespawnObject {
                    target: "rock-live".to_string(),
                },
            }],
        );

        assert!(runtime
            .target_resolver()
            .resolve_alias("rock-live")
            .is_none());
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
                &[BehaviorCommand::ApplySceneMutation {
                    request: engine_api::scene::SceneMutationRequest::SpawnObject {
                        template: "rock-template".to_string(),
                        target: "rock-live".to_string(),
                    },
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
                &[BehaviorCommand::ApplySceneMutation {
                    request: engine_api::scene::SceneMutationRequest::DespawnObject {
                        target: "rock-live".to_string(),
                    },
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
                BehaviorCommand::ApplySceneMutation {
                    request: engine_api::scene::SceneMutationRequest::SpawnObject {
                        template: "ship-template".to_string(),
                        target: "ship-1".to_string(),
                    },
                },
                BehaviorCommand::ApplySceneMutation {
                    request: engine_api::scene::SceneMutationRequest::SpawnObject {
                        template: "fx-template".to_string(),
                        target: "fx-1".to_string(),
                    },
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
            &[BehaviorCommand::ApplySceneMutation {
                request: engine_api::scene::SceneMutationRequest::DespawnObject {
                    target: "fx-1".to_string(),
                },
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
                set_path("title", "visible", serde_json::json!(false), None),
                set_path(
                    "title",
                    "position.x",
                    serde_json::json!(9),
                    Some(&ObjectRuntimeState::default()),
                ),
                set_path(
                    "title",
                    "position.y",
                    serde_json::json!(-2),
                    Some(&ObjectRuntimeState::default()),
                ),
                set_path("title", "text.content", serde_json::json!("PATH-SET"), None),
                set_path("title", "text.font", serde_json::json!("generic:2"), None),
                set_path("title", "style.fg", serde_json::json!("yellow"), None),
                set_path("title", "style.bg", serde_json::json!("#112233"), None),
            ],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("PATH-SET"));
        let title_id = resolver.resolve_alias("title").expect("title id");
        let object_props = runtime.object_props_snapshot();
        let title_props = object_props
            .get(title_id)
            .and_then(|value| value.as_object())
            .expect("title props");
        let text_props = title_props
            .get("text")
            .and_then(|value| value.as_object())
            .expect("text props");
        assert_eq!(
            text_props.get("font"),
            Some(&serde_json::json!("generic:2"))
        );
        assert_eq!(text_props.get("fg"), Some(&serde_json::json!("yellow")));
        assert_eq!(text_props.get("bg"), Some(&serde_json::json!("#112233")));
        assert!(runtime.layout_regions_stale());
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
                set_path(
                    "title",
                    "position.x",
                    serde_json::json!(9.8),
                    Some(&ObjectRuntimeState::default()),
                ),
                set_path(
                    "title",
                    "position.y",
                    serde_json::json!(-2.4),
                    Some(&ObjectRuntimeState::default()),
                ),
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
    fn apply_behavior_commands_tracks_transform_visibility_and_camera_dirty_masks() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetRender3d(
                        Render3dMutationRequest::SetNodeTransform {
                            target: "helsinki-uni-wireframe".to_string(),
                            translation: Some([3.0, -2.0, 0.0]),
                            rotation_deg: None,
                            scale: None,
                        },
                    ),
                },
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetCamera3d(
                        engine_api::scene::Camera3dMutationRequest::LookAt {
                            eye: [0.0, 0.0, 4.0],
                            look_at: [0.0, 0.0, 0.0],
                        },
                    ),
                },
            ],
        );

        let dirty = runtime.take_render3d_dirty_mask();
        assert!(dirty.contains(DirtyMask3D::TRANSFORM));
        assert!(dirty.contains(DirtyMask3D::CAMERA));
        assert_eq!(runtime.render3d_dirty_mask(), DirtyMask3D::empty());
    }

    #[test]
    fn apply_behavior_commands_tracks_material_atmosphere_and_worldgen_dirty_masks() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetRender3d(
                        Render3dMutationRequest::SetMaterialParam {
                            target: "helsinki-uni-wireframe".to_string(),
                            name: "surface_mode".to_string(),
                            value: serde_json::json!("wireframe"),
                        },
                    ),
                },
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetRender3d(
                        Render3dMutationRequest::SetAtmosphereParam {
                            target: "helsinki-uni-wireframe".to_string(),
                            name: "obj.atmo.halo_strength".to_string(),
                            value: serde_json::json!(1.4),
                        },
                    ),
                },
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetRender3d(
                        Render3dMutationRequest::SetWorldParam {
                            target: "helsinki-uni-wireframe".to_string(),
                            name: "world.seed".to_string(),
                            value: serde_json::json!(99),
                        },
                    ),
                },
            ],
        );

        let dirty = runtime.take_render3d_dirty_mask();
        assert!(dirty.contains(DirtyMask3D::MATERIAL));
        assert!(dirty.contains(DirtyMask3D::ATMOSPHERE));
        assert!(dirty.contains(DirtyMask3D::WORLDGEN));
    }

    #[test]
    fn apply_behavior_commands_tracks_render3d_rebuild_diagnostics() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[set_path(
                "helsinki-uni-wireframe",
                "world.seed",
                serde_json::json!(7),
                None,
            )],
        );

        let diagnostics = runtime.take_render3d_rebuild_diagnostics();
        assert_eq!(diagnostics.worldgen_dirty_events, 1);
        assert_eq!(diagnostics.mesh_dirty_events, 0);
        assert!(runtime.take_render3d_rebuild_diagnostics().is_empty());
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_obj_paths() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.scale",
                    serde_json::json!(1.5),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.yaw",
                    serde_json::json!(15),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.pitch",
                    serde_json::json!(-10),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.roll",
                    serde_json::json!(2),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.orbit_speed",
                    serde_json::json!(22),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.surface_mode",
                    serde_json::json!("wireframe"),
                    None,
                ),
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
    fn apply_behavior_commands_set_property_updates_obj_world_axes() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.world.x",
                    serde_json::json!(12.5),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.world.y",
                    serde_json::json!(-7.25),
                    None,
                ),
                set_path(
                    "helsinki-uni-wireframe",
                    "obj.world.z",
                    serde_json::json!(3.0),
                    None,
                ),
            ],
        );

        let obj_world = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    world_x,
                    world_y,
                    world_z,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => {
                    Some((*world_x, *world_y, *world_z))
                }
                _ => None,
            })
            .expect("obj world axes");

        assert_eq!(obj_world.0, Some(12.5));
        assert_eq!(obj_world.1, Some(-7.25));
        assert_eq!(obj_world.2, Some(3.0));
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_planet_paths() {
        let mut runtime = SceneRuntime::new(planet_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                set_path(
                    "main-planet-view",
                    "planet.spin_deg",
                    serde_json::json!(15),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.cloud_spin_deg",
                    serde_json::json!(21),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.cloud2_spin_deg",
                    serde_json::json!(33),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.observer_altitude_km",
                    serde_json::json!(420),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.sun_dir.x",
                    serde_json::json!(0.5),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.sun_dir.y",
                    serde_json::json!(-0.4),
                    None,
                ),
                set_path(
                    "main-planet-view",
                    "planet.sun_dir.z",
                    serde_json::json!(0.2),
                    None,
                ),
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
    fn apply_behavior_commands_set_property_updates_scene3d_frame() {
        let mut runtime = SceneRuntime::new(scene3d_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[set_path(
                "intro-view",
                "scene3d.frame",
                serde_json::json!("closeup"),
                None,
            )],
        );

        let scene3d_frame = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Scene3D { id, frame, .. } if id.as_deref() == Some("intro-view") => {
                    Some(frame.as_str())
                }
                _ => None,
            })
            .expect("scene3d frame");

        assert_eq!(scene3d_frame, "closeup");
    }

    #[test]
    fn render3d_obj_set_property_matches_typed_world_param_mutation() {
        let mut via_property = SceneRuntime::new(obj_scene(""));
        let mut via_typed = SceneRuntime::new(obj_scene(""));
        let resolver_a = via_property.target_resolver();
        let resolver_b = via_typed.target_resolver();

        via_property.apply_behavior_commands(
            &resolver_a,
            &[set_path(
                "helsinki-uni-wireframe",
                "obj.scale",
                serde_json::json!(1.5),
                None,
            )],
        );
        via_typed.apply_behavior_commands(
            &resolver_b,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetWorldParam {
                        target: "helsinki-uni-wireframe".to_string(),
                        name: "obj.scale".to_string(),
                        value: serde_json::json!(1.5),
                    },
                ),
            }],
        );

        let read_scale = |runtime: &SceneRuntime| {
            runtime
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
                .expect("obj scale")
        };
        assert_eq!(read_scale(&via_property), read_scale(&via_typed));
    }

    #[test]
    fn render3d_planet_set_property_matches_typed_world_param_mutation() {
        let mut via_property = SceneRuntime::new(planet_scene(""));
        let mut via_typed = SceneRuntime::new(planet_scene(""));
        let resolver_a = via_property.target_resolver();
        let resolver_b = via_typed.target_resolver();

        via_property.apply_behavior_commands(
            &resolver_a,
            &[set_path(
                "main-planet-view",
                "planet.spin_deg",
                serde_json::json!(15.0),
                None,
            )],
        );
        via_typed.apply_behavior_commands(
            &resolver_b,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetWorldParam {
                        target: "main-planet-view".to_string(),
                        name: "planet.spin_deg".to_string(),
                        value: serde_json::json!(15.0),
                    },
                ),
            }],
        );

        let read_spin = |runtime: &SceneRuntime| {
            runtime
                .scene()
                .layers
                .iter()
                .flat_map(|layer| layer.sprites.iter())
                .find_map(|sprite| match sprite {
                    Sprite::Planet { id, spin_deg, .. }
                        if id.as_deref() == Some("main-planet-view") =>
                    {
                        *spin_deg
                    }
                    _ => None,
                })
                .expect("planet spin")
        };
        assert_eq!(read_spin(&via_property), read_spin(&via_typed));
    }

    #[test]
    fn render3d_scene3d_frame_set_property_matches_typed_world_param_mutation() {
        let mut via_property = SceneRuntime::new(scene3d_scene(""));
        let mut via_typed = SceneRuntime::new(scene3d_scene(""));
        let resolver_a = via_property.target_resolver();
        let resolver_b = via_typed.target_resolver();

        via_property.apply_behavior_commands(
            &resolver_a,
            &[set_path(
                "intro-view",
                "scene3d.frame",
                serde_json::json!("closeup"),
                None,
            )],
        );
        via_typed.apply_behavior_commands(
            &resolver_b,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetWorldParam {
                        target: "intro-view".to_string(),
                        name: "scene3d.frame".to_string(),
                        value: serde_json::json!("closeup"),
                    },
                ),
            }],
        );

        let read_frame = |runtime: &SceneRuntime| {
            runtime
                .scene()
                .layers
                .iter()
                .flat_map(|layer| layer.sprites.iter())
                .find_map(|sprite| match sprite {
                    Sprite::Scene3D { id, frame, .. } if id.as_deref() == Some("intro-view") => {
                        Some(frame.clone())
                    }
                    _ => None,
                })
                .expect("scene3d frame")
        };
        assert_eq!(read_frame(&via_property), read_frame(&via_typed));
    }

    #[test]
    fn non_render_set_property_updates_state_through_typed_path_mapping() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[set_path(
                "title",
                "text.content",
                serde_json::json!("HELLO_TYPED"),
                None,
            )],
        );

        let title_id = resolver.resolve_alias("title").expect("title id");
        let object_text = runtime.object_text_snapshot();
        let title_text = object_text
            .get(title_id)
            .cloned()
            .expect("title text in snapshot");

        assert_eq!(title_text, "HELLO_TYPED");
        assert!(runtime.layout_regions_stale());
        assert_eq!(runtime.take_render3d_dirty_mask(), DirtyMask3D::empty());
    }

    #[test]
    fn apply_behavior_commands_emits_debug_log_for_unsupported_typed_request() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let diagnostics = runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty {
                    target: "title".to_string(),
                    path: "audio.pitch".to_string(),
                    value: serde_json::json!(2.0),
                },
            }],
        );

        assert!(matches!(
            diagnostics.as_slice(),
            [BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity: engine_api::commands::DebugLogSeverity::Warn,
                message,
            }] if scene_id == "intro"
                && source.as_deref() == Some("scene-mutation")
                && message.contains("unsupported sprite property path `audio.pitch`")
        ));
    }

    #[test]
    fn apply_behavior_commands_emits_debug_log_for_missing_mutation_target() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let diagnostics = runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "missing".to_string(),
                    visible: Some(false),
                    dx: None,
                    dy: None,
                    text: None,
                },
            }],
        );

        assert!(matches!(
            diagnostics.as_slice(),
            [BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity: engine_api::commands::DebugLogSeverity::Warn,
                message,
            }] if scene_id == "intro"
                && source.as_deref() == Some("scene-mutation")
                && message.contains("target `missing` was not found")
        ));
    }

    #[test]
    fn sync_widget_visuals_refreshes_object_text_snapshot_and_layout_staleness() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_id = resolver
            .resolve_alias("title")
            .expect("title id")
            .to_string();
        runtime.set_object_regions(HashMap::from([(
            title_id.clone(),
            Region {
                x: 0,
                y: 0,
                width: 5,
                height: 1,
            },
        )]));
        assert!(!runtime.layout_regions_stale());

        let initial_text = runtime.object_text_snapshot();
        assert_eq!(
            initial_text.get(&title_id).map(String::as_str),
            Some("HELLO")
        );

        let widget = TextInputControl {
            id: "edit".to_string(),
            sprite: "title".to_string(),
            x: 0,
            y: 0,
            w: 10,
            h: 1,
            text_sprite: "title".to_string(),
            placeholder: String::new(),
            value: "SYNCED".to_string(),
            max_length: 16,
            follow_layout: false,
        };
        let state = widget.initial_state();
        runtime.gui_widgets = vec![Box::new(widget)];
        runtime.gui_state.widgets.insert("edit".to_string(), state);

        runtime.sync_widget_visuals();

        let object_text = runtime.object_text_snapshot();
        assert_eq!(
            object_text.get(&title_id).map(String::as_str),
            Some("SYNCED")
        );
        assert!(runtime.layout_regions_stale());

        runtime.set_object_regions(HashMap::from([(
            title_id,
            Region {
                x: 0,
                y: 0,
                width: 6,
                height: 1,
            },
        )]));
        assert!(!runtime.layout_regions_stale());
    }

    #[test]
    fn sync_widget_visuals_refreshes_state_snapshots() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_id = resolver
            .resolve_alias("title")
            .expect("title id")
            .to_string();

        let initial_states = runtime.object_states_snapshot();
        assert_eq!(
            initial_states
                .get(&title_id)
                .expect("title state before sync")
                .offset_x,
            0
        );
        let initial_effective = runtime.effective_object_states_snapshot();
        assert_eq!(
            initial_effective
                .get(&title_id)
                .expect("title effective state before sync")
                .offset_x,
            0
        );

        let widget = SliderControl {
            id: "slider".to_string(),
            sprite: "title".to_string(),
            x: 0,
            y: 0,
            w: 40,
            h: 1,
            min: 0.0,
            max: 1.0,
            value: 0.5,
            hit_padding: 0,
            handle: "title".to_string(),
            follow_layout: false,
        };
        let mut state = widget.initial_state();
        state.value = 0.5;
        runtime.gui_widgets = vec![Box::new(widget)];
        runtime
            .gui_state
            .widgets
            .insert("slider".to_string(), state);

        runtime.sync_widget_visuals();

        let object_states = runtime.object_states_snapshot();
        assert_eq!(
            object_states
                .get(&title_id)
                .expect("title state after sync")
                .offset_x,
            20
        );
        let effective_states = runtime.effective_object_states_snapshot();
        assert_eq!(
            effective_states
                .get(&title_id)
                .expect("title effective state after sync")
                .offset_x,
            20
        );
        assert!(runtime.layout_regions_stale());
    }

    #[test]
    fn same_frame_behavior_refresh_exposes_last_known_regions_and_stale_flag() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_id = resolver
            .resolve_alias("title")
            .expect("title id")
            .to_string();
        runtime.set_object_regions(HashMap::from([(
            title_id,
            Region {
                x: 0,
                y: 0,
                width: 5,
                height: 1,
            },
        )]));

        let observed = Arc::new(Mutex::new(None));
        let root_id = runtime.root_id().to_string();
        runtime.behaviors = vec![
            ObjectBehaviorRuntime {
                object_id: root_id.clone(),
                behavior: Box::new(MutateBehavior),
            },
            ObjectBehaviorRuntime {
                object_id: root_id,
                behavior: Box::new(ObserveBehavior {
                    observed: Arc::clone(&observed),
                }),
            },
        ];

        runtime.update_behaviors(
            SceneStage::OnIdle,
            16,
            16,
            0,
            None,
            None,
            None,
            None,
            None,
            Arc::new(Vec::new()),
            Arc::new(engine_behavior::catalog::ModCatalogs::default()),
            Arc::new(engine_behavior::palette::PaletteStore::default()),
            None,
            false,
        );

        assert_eq!(
            *observed.lock().expect("observer lock"),
            Some(ObservedBehaviorSnapshot {
                text: Some("UPDATED".to_string()),
                font: Some("generic:2".to_string()),
                offset_x: Some(4),
                region_width: Some(5),
                layout_regions_stale: true,
            })
        );
    }

    #[test]
    fn same_frame_state_only_mutation_marks_layout_regions_stale() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_id = resolver
            .resolve_alias("title")
            .expect("title id")
            .to_string();
        runtime.set_object_regions(HashMap::from([(
            title_id,
            Region {
                x: 0,
                y: 0,
                width: 5,
                height: 1,
            },
        )]));

        let observed = Arc::new(Mutex::new(None));
        let root_id = runtime.root_id().to_string();
        runtime.behaviors = vec![
            ObjectBehaviorRuntime {
                object_id: root_id.clone(),
                behavior: Box::new(VisibilityOnlyMutateBehavior),
            },
            ObjectBehaviorRuntime {
                object_id: root_id,
                behavior: Box::new(ObserveBehavior {
                    observed: Arc::clone(&observed),
                }),
            },
        ];

        runtime.update_behaviors(
            SceneStage::OnIdle,
            16,
            16,
            0,
            None,
            None,
            None,
            None,
            None,
            Arc::new(Vec::new()),
            Arc::new(engine_behavior::catalog::ModCatalogs::default()),
            Arc::new(engine_behavior::palette::PaletteStore::default()),
            None,
            false,
        );

        let snapshot = observed
            .lock()
            .expect("observer lock")
            .clone()
            .expect("same-frame snapshot");
        assert_eq!(snapshot.text.as_deref(), Some("HELLO"));
        assert_eq!(snapshot.offset_x, Some(0));
        assert_eq!(snapshot.region_width, Some(5));
        assert!(snapshot.layout_regions_stale);

        let state = runtime
            .effective_object_states_snapshot()
            .get(
                resolver
                    .resolve_alias("title")
                    .expect("title id after update"),
            )
            .cloned()
            .expect("title state after update");
        assert!(!state.visible);
    }

    #[test]
    fn set_camera_marks_layout_regions_stale() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_id = resolver
            .resolve_alias("title")
            .expect("title id")
            .to_string();
        runtime.set_object_regions(HashMap::from([(
            title_id,
            Region {
                x: 0,
                y: 0,
                width: 5,
                height: 1,
            },
        )]));
        assert!(!runtime.layout_regions_stale());

        runtime
            .apply_behavior_commands(&resolver, &[BehaviorCommand::SetCamera { x: 12.0, y: 4.0 }]);

        assert!(runtime.layout_regions_stale());
    }

    #[test]
    fn render3d_set_property_with_invalid_value_does_not_apply_other_path_mapping() {
        let mut runtime = SceneRuntime::new(scene3d_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[set_path(
                "intro-view",
                "scene3d.frame",
                serde_json::json!(7),
                None,
            )],
        );

        let scene3d_frame = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Scene3D { id, frame, .. } if id.as_deref() == Some("intro-view") => {
                    Some(frame.clone())
                }
                _ => None,
            })
            .expect("scene3d frame");

        assert_eq!(scene3d_frame, "idle");
        assert_eq!(runtime.take_render3d_dirty_mask(), DirtyMask3D::empty());
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

    #[test]
    fn step_orbit_camera_applies_typed_obj_camera_fields() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        runtime.orbit_camera = Some(ObjOrbitCameraState {
            target: "helsinki-uni-wireframe".to_string(),
            active: true,
            yaw: 24.0,
            pitch: -15.0,
            distance: 42.0,
            pitch_min: -85.0,
            pitch_max: 85.0,
            distance_min: 0.3,
            distance_max: 10.0,
            distance_step: 0.5,
            drag_sensitivity: 0.5,
            last_mouse_pos: None,
        });

        assert!(runtime.step_orbit_camera());

        let obj_fields = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    yaw_deg,
                    pitch_deg,
                    camera_distance,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => {
                    Some((*yaw_deg, *pitch_deg, *camera_distance))
                }
                _ => None,
            })
            .expect("orbit camera fields");

        assert_eq!(obj_fields.0, Some(24.0));
        assert_eq!(obj_fields.1, Some(-15.0));
        assert_eq!(obj_fields.2, Some(10.0));
    }

    #[test]
    fn step_orbit_camera_enforces_safe_min_distance_for_large_atmosphere() {
        let mut runtime = SceneRuntime::new(obj_scene(
            r#"        fov-degrees: 40
        atmo-height: 0.20
        atmo-density: 0.60
        atmo-halo-strength: 1.0
        atmo-halo-width: 0.20"#,
        ));
        runtime.orbit_camera = Some(ObjOrbitCameraState {
            target: "helsinki-uni-wireframe".to_string(),
            active: true,
            yaw: 12.0,
            pitch: -8.0,
            distance: 1.0,
            pitch_min: -85.0,
            pitch_max: 85.0,
            distance_min: 0.3,
            distance_max: 10.0,
            distance_step: 0.25,
            drag_sensitivity: 0.5,
            last_mouse_pos: None,
        });

        assert!(runtime.step_orbit_camera());

        let obj_distance = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    camera_distance,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *camera_distance,
                _ => None,
            })
            .expect("camera distance after orbit step");

        assert!(
            obj_distance >= 3.5,
            "expected safe orbit clamp to prevent clipping, got {obj_distance}"
        );
    }

    #[test]
    fn step_orbit_camera_enforces_safe_min_distance_for_dense_haze_shell() {
        let mut runtime = SceneRuntime::new(obj_scene(
            r#"        fov-degrees: 40
        atmo-height: 0.78
        atmo-density: 0.90
        atmo-strength: 0.90
        atmo-rayleigh-amount: 0.80
        atmo-haze-amount: 0.92
        atmo-limb-boost: 2.10
        world-displacement-scale: 0.55"#,
        ));
        runtime.orbit_camera = Some(ObjOrbitCameraState {
            target: "helsinki-uni-wireframe".to_string(),
            active: true,
            yaw: 12.0,
            pitch: -8.0,
            distance: 1.0,
            pitch_min: -85.0,
            pitch_max: 85.0,
            distance_min: 0.3,
            distance_max: 10.0,
            distance_step: 0.25,
            drag_sensitivity: 0.5,
            last_mouse_pos: None,
        });

        assert!(runtime.step_orbit_camera());

        let obj_distance = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    camera_distance,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *camera_distance,
                _ => None,
            })
            .expect("camera distance after orbit step");

        assert!(
            obj_distance >= 5.0,
            "expected dense haze safe clamp to prevent viewport clipping, got {obj_distance}"
        );
    }

    #[test]
    fn scene_runtime_initializes_spatial_context_from_scene() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: spatial-runtime
title: Spatial Runtime
spatial:
  meters-per-world-unit: 25.0
  virtual-pixels-per-world-unit: 6.0
  handedness: right
  up-axis: z
layers: []
"#,
        )
        .expect("scene should parse");

        let runtime = SceneRuntime::new(scene);
        let spatial = runtime.spatial_context();
        assert_eq!(spatial.scale.meters_per_world_unit, 25.0);
        assert_eq!(spatial.scale.virtual_pixels_per_world_unit, Some(6.0));
        assert_eq!(
            spatial.axes.handedness,
            engine_core::spatial::Handedness::Right
        );
        assert_eq!(spatial.axes.up_axis, engine_core::spatial::UpAxis::Z);
    }
}
