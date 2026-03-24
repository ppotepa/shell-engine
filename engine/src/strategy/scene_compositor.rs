use std::collections::HashMap;
use crossterm::style::Color;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::scene::{Effect, Layer, SceneRenderedMode};
use crate::assets::AssetRoot;
use crate::scene_runtime::{ObjCameraState, ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;
use super::{HalfblockPacker, LayerCompositor};

/// All scene-invariant inputs to a single compositor invocation.
///
/// Groups the 14+ parameters that were previously threaded individually through
/// `composite_scene` / `composite_scene_halfblock` and their callers.
pub struct CompositeParams<'a> {
    pub bg: Color,
    pub layers: &'a [Layer],
    pub ui_enabled: bool,
    pub scene_rendered_mode: SceneRenderedMode,
    pub asset_root: Option<&'a AssetRoot>,
    pub target_resolver: &'a TargetResolver,
    pub object_states: &'a HashMap<String, ObjectRuntimeState>,
    pub obj_camera_states: &'a HashMap<String, ObjCameraState>,
    pub current_stage: &'a SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub scene_elapsed_ms: u64,
    pub scene_effects: &'a [Effect],
    pub scene_step_dur: u64,
}

/// Owns the rendered-mode-specific compositing path for a single frame.
///
/// Replaces the duplicate `match rendered_mode { ... }` blocks that previously
/// appeared in `compositor_system` for both the virtual and direct buffer paths.
/// Adding a new rendered mode is one new impl — `compositor_system` never changes.
pub trait SceneCompositor: Send + Sync {
    fn composite(
        &self,
        params: &CompositeParams<'_>,
        layer: &dyn LayerCompositor,
        halfblock: &dyn HalfblockPacker,
        buffer: &mut Buffer,
    ) -> HashMap<String, Region>;
}

/// Handles Cell, QuadBlock, and Braille rendered modes (direct layer compositing).
pub struct CellSceneCompositor;

/// Handles the HalfBlock rendered mode (renders at 2× height then packs to halfblocks).
pub struct HalfblockSceneCompositor;

/// Returns the correct `SceneCompositor` for the given rendered mode.
///
/// Uses static singletons — zero allocation per frame.
pub fn compositor_for(mode: SceneRenderedMode) -> &'static dyn SceneCompositor {
    static CELL: CellSceneCompositor = CellSceneCompositor;
    static HALFBLOCK: HalfblockSceneCompositor = HalfblockSceneCompositor;
    match mode {
        SceneRenderedMode::HalfBlock => &HALFBLOCK,
        _ => &CELL,
    }
}
