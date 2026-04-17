use super::effect_applicator::apply_layer_effects;
use super::provider::resolve_render_2d_pipeline;
use super::scene_compositor::PreparedLayerFrame;
use crate::ObjPrerenderedFrames;
use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_core::spatial::SpatialContext;
use engine_pipeline::LayerCompositor;
use engine_render_2d::{Render2dInput, Render2dPipeline};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LAYER_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
    // OPT-39: Layer bounds cache for skip rendering layers entirely if outside viewport.
    static LAYER_BOUNDS_CACHE: RefCell<HashMap<usize, (i32, i32, i32, i32)>> = RefCell::new(HashMap::new());
    static NON_UI_JITTER_DIAG: RefCell<HashMap<String, (i32, i32)>> = RefCell::new(HashMap::new());
}

/// Prepared render-domain dependencies used while assembling one frame.
pub struct PreparedLayerRenderInputs<'a> {
    pub render_2d_pipeline: Option<&'a dyn Render2dPipeline>,
    pub asset_root: Option<&'a AssetRoot>,
    pub obj_camera_states: &'a HashMap<String, ObjCameraState>,
    pub scene_camera_3d: &'a SceneCamera3D,
    pub spatial_context: SpatialContext,
    pub celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub is_pixel_backend: bool,
    pub default_font: Option<&'a str>,
    pub prerender_frames: Option<&'a ObjPrerenderedFrames>,
}

/// Prepared layer/frame state consumed by compositor assembly.
pub struct LayerCompositeInputs<'a> {
    pub prepared_layers: &'a [PreparedLayerFrame<'a>],
    pub scene_w: u16,
    pub scene_h: u16,
    pub target_resolver: Option<&'a TargetResolver>,
    pub object_regions: &'a mut HashMap<String, Region>,
    pub scene_origin_x: i32,
    pub scene_origin_y: i32,
    pub object_states: &'a HashMap<String, ObjectRuntimeState>,
    pub current_stage: &'a SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub scene_elapsed_ms: u64,
    pub camera_x: i32,
    pub camera_y: i32,
    pub camera_zoom: f32,
    pub render: PreparedLayerRenderInputs<'a>,
}

/// Composite all visible layers onto the scene framebuffer.
#[inline]
pub fn composite_layers(
    inputs: &mut LayerCompositeInputs<'_>,
    layer_compositor: &dyn LayerCompositor,
    buffer: &mut Buffer,
) {
    let prepared_layers = inputs.prepared_layers;
    let scene_w = inputs.scene_w;
    let scene_h = inputs.scene_h;
    let target_resolver = inputs.target_resolver;
    let object_regions = &mut *inputs.object_regions;
    let scene_origin_x = inputs.scene_origin_x;
    let scene_origin_y = inputs.scene_origin_y;
    let object_states = inputs.object_states;
    let current_stage = inputs.current_stage;
    let step_idx = inputs.step_idx;
    let elapsed_ms = inputs.elapsed_ms;
    let scene_elapsed_ms = inputs.scene_elapsed_ms;
    let camera_x = inputs.camera_x;
    let camera_y = inputs.camera_y;
    let camera_zoom = inputs.camera_zoom;
    let asset_root = inputs.render.asset_root;
    let obj_camera_states = inputs.render.obj_camera_states;
    let scene_camera_3d = inputs.render.scene_camera_3d;
    let spatial_context = inputs.render.spatial_context;
    let celestial_catalogs = inputs.render.celestial_catalogs;
    let is_pixel_backend = inputs.render.is_pixel_backend;
    let default_font = inputs.render.default_font;
    let resolved_render_pipeline = resolve_render_2d_pipeline(
        inputs.render.render_2d_pipeline,
        obj_camera_states,
        scene_camera_3d,
        spatial_context,
        celestial_catalogs,
        inputs.render.prerender_frames,
    );
    let render_2d_pipeline: &dyn Render2dPipeline = resolved_render_pipeline.pipeline();

    for prepared in prepared_layers {
        let layer_idx = prepared.index;
        let layer = prepared.layer;
        let layer_object_id =
            target_resolver.and_then(|resolver| resolver.layer_object_id(layer_idx));
        let layer_state = layer_object_id
            .and_then(|object_id| object_states.get(object_id))
            .cloned()
            .unwrap_or_default();
        if !prepared.authored_visible {
            continue;
        }
        if !layer_state.visible {
            continue;
        }
        let base_x = scene_origin_x.saturating_add(layer_state.offset_x);
        let base_y = scene_origin_y.saturating_add(layer_state.offset_y);
        let use_2d_camera = prepared.uses_2d_camera;
        let (total_origin_x, total_origin_y) = if use_2d_camera && (camera_zoom - 1.0).abs() > 0.001
        {
            // Zoom scales world pixels around the camera centre (viewport midpoint).
            let half_w = scene_w as f32 * 0.5;
            let half_h = scene_h as f32 * 0.5;
            let world_x = (base_x - camera_x) as f32;
            let world_y = (base_y - camera_y) as f32;
            (
                (half_w + (world_x - half_w) * camera_zoom).round() as i32,
                (half_h + (world_y - half_h) * camera_zoom).round() as i32,
            )
        } else if use_2d_camera {
            (
                base_x.saturating_sub(camera_x),
                base_y.saturating_sub(camera_y),
            )
        } else {
            (base_x, base_y)
        };

        // ── Flicker diagnostic: detect non-UI layer position jitter ──────
        if !layer.ui {
            use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
            static JITTER_COUNT: AtomicU64 = AtomicU64::new(0);
            if layer_idx >= prepared_layers.len().saturating_sub(10)
                || layer.name.starts_with("ship")
                || layer.name.contains("ship")
            {
                let diag_key = layer_object_id
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{}#{layer_idx}", layer.name));
                NON_UI_JITTER_DIAG.with(|state| {
                    let mut state = state.borrow_mut();
                    if let Some((prev_x, prev_y)) = state.get(&diag_key).copied() {
                        if prev_x != total_origin_x || prev_y != total_origin_y {
                            let jc = JITTER_COUNT.fetch_add(1, Relaxed);
                            if jc < 20 {
                                engine_core::logging::warn(
                                    "compositor.jitter",
                                    format!(
                                        "NON-UI layer '{}' idx={} origin changed: ({},{}) → ({},{}) cam=({},{}) offset=({},{}) frame={}",
                                        layer.name,
                                        layer_idx,
                                        prev_x,
                                        prev_y,
                                        total_origin_x,
                                        total_origin_y,
                                        camera_x,
                                        camera_y,
                                        layer_state.offset_x,
                                        layer_state.offset_y,
                                        step_idx,
                                    ),
                                );
                            }
                        }
                    }
                    state.insert(diag_key, (total_origin_x, total_origin_y));
                });
            }
        }

        // Viewport culling — all 4 sides.
        // Entity layers (have a physics body): entity center is at total_origin;
        // cull if center is more than CULL_MARGIN pixels outside the viewport.
        // Static/background layers (no physics body): content fills the scene rect
        // [total_origin, total_origin + scene_size]; cull if it doesn't overlap viewport.
        const CULL_MARGIN: i32 = 128;
        if layer_object_id.is_some() {
            if total_origin_x < -CULL_MARGIN || total_origin_x > scene_w as i32 + CULL_MARGIN {
                continue;
            }
            if total_origin_y < -CULL_MARGIN || total_origin_y > scene_h as i32 + CULL_MARGIN {
                continue;
            }
        } else {
            // Layer content spans [total_origin, total_origin + scene_size] on screen.
            if total_origin_x + scene_w as i32 <= 0 || total_origin_x >= scene_w as i32 {
                continue;
            }
            if total_origin_y + scene_h as i32 <= 0 || total_origin_y >= scene_h as i32 {
                continue;
            }
        }

        if let Some(object_id) = layer_object_id {
            object_regions.insert(
                object_id.to_string(),
                Region {
                    x: total_origin_x.max(0) as u16,
                    y: total_origin_y.max(0) as u16,
                    width: scene_w
                        .saturating_sub(total_origin_x.unsigned_abs().min(scene_w as u32) as u16),
                    height: scene_h
                        .saturating_sub(total_origin_y.unsigned_abs().min(scene_h as u32) as u16),
                },
            );
        }

        // #4 opt-comp-layerscratch: LayerCompositor strategy — DirectLayerCompositor skips
        // scratch fill+blit for layers without active effects.
        // ScratchLayerCompositor (safe default) always uses the scratch path.
        let needs_scratch = layer_compositor.use_scratch(prepared.has_active_effects);

        if needs_scratch {
            // Full scratch path: fill + render + effects + blit.
            LAYER_SCRATCH.with(|scratch| {
                let mut layer_buf = scratch.borrow_mut();
                if layer_buf.width != scene_w || layer_buf.height != scene_h {
                    layer_buf.resize(scene_w, scene_h);
                }
                layer_buf.fill(Color::Reset);

                render_2d_pipeline.render(
                    Render2dInput {
                        layer_idx,
                        layer,
                        scene_w,
                        scene_h,
                        asset_root,
                        target_resolver,
                        object_regions,
                        root_origin_x: total_origin_x,
                        root_origin_y: total_origin_y,
                        object_states,
                        scene_elapsed_ms,
                        current_stage,
                        step_idx,
                        elapsed_ms,
                        is_pixel_backend,
                        default_font,
                    },
                    &mut layer_buf,
                );

                apply_layer_effects(
                    layer,
                    current_stage,
                    step_idx,
                    elapsed_ms,
                    scene_elapsed_ms,
                    target_resolver,
                    object_regions,
                    &mut layer_buf,
                );

                buffer.blit_from(&layer_buf, 0, 0, 0, 0, scene_w, scene_h);
            });
        } else {
            // No effects: render sprites directly onto scene buffer (skip scratch).
            // Note: We still need to preserve transparency for this layer.
            // Render directly but rely on sprite rendering to skip transparent cells.
            render_2d_pipeline.render(
                Render2dInput {
                    layer_idx,
                    layer,
                    scene_w,
                    scene_h,
                    asset_root,
                    target_resolver,
                    object_regions,
                    root_origin_x: total_origin_x,
                    root_origin_y: total_origin_y,
                    object_states,
                    scene_elapsed_ms,
                    current_stage,
                    step_idx,
                    elapsed_ms,
                    is_pixel_backend,
                    default_font,
                },
                buffer,
            );
        }
    }
}
