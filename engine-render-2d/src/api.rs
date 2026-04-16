use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::effects::Region;
use engine_core::scene::Layer;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use std::collections::HashMap;

/// Render input for one layer pass.
pub struct Render2dInput<'a> {
    pub layer_idx: usize,
    pub layer: &'a Layer,
    pub scene_w: u16,
    pub scene_h: u16,
    pub asset_root: Option<&'a AssetRoot>,
    pub target_resolver: Option<&'a TargetResolver>,
    pub object_regions: &'a mut HashMap<String, Region>,
    pub root_origin_x: i32,
    pub root_origin_y: i32,
    pub object_states: &'a HashMap<String, ObjectRuntimeState>,
    pub scene_elapsed_ms: u64,
    pub current_stage: &'a SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub is_pixel_backend: bool,
    pub default_font: Option<&'a str>,
}

/// Seam between composition and 2D sprite rendering.
pub trait Render2dPipeline {
    fn render(&self, input: Render2dInput<'_>, target: &mut engine_core::buffer::Buffer);
}
