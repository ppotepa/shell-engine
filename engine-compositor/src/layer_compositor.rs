use super::effect_applicator::apply_layer_effects;
use super::provider::resolve_render_2d_pipeline;
use super::scene_compositor::PreparedLayerFrame;
#[cfg(feature = "render-3d")]
use crate::prepared_frame::PreparedLayerInput;
#[cfg(feature = "render-3d")]
use crate::render::check_visibility;
use crate::ObjPrerenderedFrames;
use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene::{ResolvedViewProfile, Sprite};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_core::spatial::SpatialContext;
use engine_pipeline::LayerCompositor;
use engine_render_2d::{Render2dInput, Render2dPipeline};
#[cfg(feature = "render-3d")]
use engine_render_3d::pipeline::obj_sprite_renderer::{
    render_prepared_obj_sprite_to_shared_rgba_buffers, PreparedObjSpriteRender,
};
#[cfg(feature = "render-3d")]
use engine_render_3d::pipeline::prepared_item_renderer::{
    prepare_prepared_mesh_item_render, PreparedRender3dRuntime,
};
#[cfg(feature = "render-3d")]
use engine_render_3d::pipeline::{prepare_render3d_item, resolve_view_lighting, SpriteRenderArea};
#[cfg(feature = "render-3d")]
use engine_render_3d::{blit_rgba_canvas, virtual_dimensions};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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
    pub resolved_view_profile: &'a ResolvedViewProfile,
    pub obj_camera_states: &'a HashMap<String, ObjCameraState>,
    pub scene_camera_3d: &'a SceneCamera3D,
    pub spatial_context: SpatialContext,
    pub celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub uses_pixel_output: bool,
    pub default_font: Option<&'a str>,
    pub ui_font_scale: f32,
    pub ui_layout_scale_x: f32,
    pub ui_layout_scale_y: f32,
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
    #[cfg(feature = "render-3d")]
    pub fully_batched_layers: &'a HashSet<usize>,
    #[cfg(feature = "render-3d")]
    pub prepared_layer_inputs: Option<&'a [PreparedLayerInput<'a>]>,
    pub render: PreparedLayerRenderInputs<'a>,
}

#[inline]
fn resolve_layer_origin(
    uses_2d_camera: bool,
    scene_w: u16,
    scene_h: u16,
    scene_origin_x: i32,
    scene_origin_y: i32,
    layer_state: &ObjectRuntimeState,
    camera_x: i32,
    camera_y: i32,
    camera_zoom: f32,
) -> (i32, i32) {
    let base_x = scene_origin_x.saturating_add(layer_state.offset_x);
    let base_y = scene_origin_y.saturating_add(layer_state.offset_y);
    if uses_2d_camera && (camera_zoom - 1.0).abs() > 0.001 {
        let half_w = scene_w as f32 * 0.5;
        let half_h = scene_h as f32 * 0.5;
        let world_x = (base_x - camera_x) as f32;
        let world_y = (base_y - camera_y) as f32;
        (
            (half_w + (world_x - half_w) * camera_zoom).round() as i32,
            (half_h + (world_y - half_h) * camera_zoom).round() as i32,
        )
    } else if uses_2d_camera {
        (
            base_x.saturating_sub(camera_x),
            base_y.saturating_sub(camera_y),
        )
    } else {
        (base_x, base_y)
    }
}

#[inline]
fn layer_is_culled(
    has_runtime_object: bool,
    total_origin_x: i32,
    total_origin_y: i32,
    scene_w: u16,
    scene_h: u16,
) -> bool {
    const CULL_MARGIN: i32 = 128;
    if has_runtime_object {
        total_origin_x < -CULL_MARGIN
            || total_origin_x > scene_w as i32 + CULL_MARGIN
            || total_origin_y < -CULL_MARGIN
            || total_origin_y > scene_h as i32 + CULL_MARGIN
    } else {
        total_origin_x + scene_w as i32 <= 0
            || total_origin_x >= scene_w as i32
            || total_origin_y + scene_h as i32 <= 0
            || total_origin_y >= scene_h as i32
    }
}

#[cfg(feature = "render-3d")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct World3dBatchCompatibility {
    fov_bits: u32,
    near_clip_bits: u32,
}

#[cfg(feature = "render-3d")]
impl World3dBatchCompatibility {
    fn from_prepared(prepared: &PreparedObjSpriteRender<'_>) -> Self {
        Self {
            fov_bits: prepared.params.fov_degrees.to_bits(),
            near_clip_bits: prepared.params.near_clip.to_bits(),
        }
    }
}

#[cfg(feature = "render-3d")]
#[inline]
fn clamp_region(x: i32, y: i32, width: u16, height: u16, scene_w: u16, scene_h: u16) -> Region {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + width as i32).min(scene_w as i32).max(x0);
    let y1 = (y + height as i32).min(scene_h as i32).max(y0);
    Region {
        x: x0 as u16,
        y: y0 as u16,
        width: (x1 - x0) as u16,
        height: (y1 - y0) as u16,
    }
}

#[cfg(feature = "render-3d")]
fn merge_world3d_local_canvas(
    scene_canvas: &mut [Option<[u8; 4]>],
    scene_depth: &mut [f32],
    scene_w: u16,
    scene_h: u16,
    local_canvas: &[Option<[u8; 4]>],
    local_depth: &[f32],
    local_w: u16,
    local_h: u16,
    draw_x: i32,
    draw_y: i32,
) {
    for local_y in 0..local_h {
        for local_x in 0..local_w {
            let local_idx = local_y as usize * local_w as usize + local_x as usize;
            let Some(rgba) = local_canvas.get(local_idx).copied().flatten() else {
                continue;
            };
            let depth = *local_depth.get(local_idx).unwrap_or(&f32::INFINITY);
            if !depth.is_finite() {
                continue;
            }
            let scene_x = draw_x + local_x as i32;
            let scene_y = draw_y + local_y as i32;
            if scene_x < 0 || scene_y < 0 || scene_x >= scene_w as i32 || scene_y >= scene_h as i32
            {
                continue;
            }
            let scene_idx = scene_y as usize * scene_w as usize + scene_x as usize;
            if depth <= scene_depth[scene_idx] {
                scene_depth[scene_idx] = depth;
                scene_canvas[scene_idx] = Some(rgba);
            }
        }
    }
}

#[cfg(feature = "render-3d")]
fn render_world3d_batch_run(
    prepared_layers: &[PreparedLayerFrame<'_>],
    prepared_layer_inputs: Option<&[PreparedLayerInput<'_>]>,
    candidate_layers: &HashSet<usize>,
    start_idx: usize,
    scene_w: u16,
    scene_h: u16,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    scene_origin_x: i32,
    scene_origin_y: i32,
    object_states: &HashMap<String, ObjectRuntimeState>,
    current_stage: &SceneStage,
    scene_elapsed_ms: u64,
    camera_x: i32,
    camera_y: i32,
    camera_zoom: f32,
    resolved_view_profile: &ResolvedViewProfile,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    scene_camera_3d: &SceneCamera3D,
    spatial_context: SpatialContext,
    asset_root: Option<&AssetRoot>,
    prerender_frames: Option<&ObjPrerenderedFrames>,
    buffer: &mut Buffer,
) -> Option<HashSet<usize>> {
    let prepared_inputs = prepared_layer_inputs?;
    let start_layer = prepared_layers.get(start_idx)?;
    if !candidate_layers.contains(&start_layer.index) {
        return None;
    }
    if start_idx > 0 && candidate_layers.contains(&prepared_layers[start_idx - 1].index) {
        return None;
    }

    let mut batch_canvas = vec![None; scene_w as usize * scene_h as usize];
    let mut batch_depth = vec![f32::INFINITY; scene_w as usize * scene_h as usize];
    let mut executed_layers = HashSet::new();
    let mut compatibility = None;
    let view_lighting = resolve_view_lighting(resolved_view_profile);

    for prepared in prepared_layers.iter().skip(start_idx) {
        if !candidate_layers.contains(&prepared.index) {
            break;
        }

        let Some(layer_input) = prepared_inputs
            .iter()
            .find(|input| input.layer_index == prepared.index)
        else {
            break;
        };

        let layer_object_id =
            target_resolver.and_then(|resolver| resolver.layer_object_id(prepared.index));
        let layer_state = layer_object_id
            .and_then(|object_id| object_states.get(object_id))
            .cloned()
            .unwrap_or_default();
        if !prepared.authored_visible || !layer_state.visible {
            executed_layers.insert(prepared.index);
            continue;
        }

        let (total_origin_x, total_origin_y) = resolve_layer_origin(
            prepared.uses_2d_camera,
            scene_w,
            scene_h,
            scene_origin_x,
            scene_origin_y,
            &layer_state,
            camera_x,
            camera_y,
            camera_zoom,
        );
        if layer_is_culled(
            layer_object_id.is_some(),
            total_origin_x,
            total_origin_y,
            scene_w,
            scene_h,
        ) {
            executed_layers.insert(prepared.index);
            continue;
        }

        let area = SpriteRenderArea {
            origin_x: total_origin_x,
            origin_y: total_origin_y,
            width: scene_w,
            height: scene_h,
        };
        let mut layer_supported = true;

        for prepared_sprite in &layer_input.sprites_3d {
            let sprite: &Sprite = prepared_sprite.sprite;
            let sprite_path = [prepared_sprite.sprite_idx];
            let object_id = target_resolver
                .and_then(|resolver| resolver.sprite_object_id(prepared.index, &sprite_path));
            let object_state = object_id
                .and_then(|id| object_states.get(id))
                .cloned()
                .unwrap_or_default();
            let is_visible = if object_id.is_some() {
                object_state.visible
            } else {
                sprite.visible()
            };
            if !is_visible {
                continue;
            }
            let Some(appear_at) = check_visibility(
                sprite.hide_on_leave(),
                sprite.appear_at_ms(),
                sprite.disappear_at_ms(),
                current_stage,
                scene_elapsed_ms,
            ) else {
                continue;
            };
            let sprite_elapsed = scene_elapsed_ms.saturating_sub(appear_at);
            let obj_camera_state = sprite
                .id()
                .and_then(|sid| obj_camera_states.get(sid))
                .cloned();
            let Some(item) = prepare_render3d_item(sprite) else {
                layer_supported = false;
                break;
            };
            let Some(prepared_mesh) = prepare_prepared_mesh_item_render(
                item,
                area,
                PreparedRender3dRuntime {
                    scene_elapsed_ms,
                    sprite_elapsed_ms: sprite_elapsed,
                    object_offset_x: object_state.offset_x,
                    object_offset_y: object_state.offset_y,
                    obj_camera_state,
                    scene_camera_3d,
                    view_lighting,
                    spatial_context,
                    celestial_catalogs: None,
                    asset_root,
                    prerender_frames,
                },
            ) else {
                layer_supported = false;
                break;
            };

            let prepared_compat = World3dBatchCompatibility::from_prepared(&prepared_mesh);
            if let Some(existing) = compatibility {
                if existing != prepared_compat {
                    layer_supported = false;
                    break;
                }
            } else {
                compatibility = Some(prepared_compat);
            }

            let (local_w, local_h) =
                virtual_dimensions(prepared_mesh.target_w, prepared_mesh.target_h);
            let local_len = local_w as usize * local_h as usize;
            let mut local_canvas = vec![None; local_len];
            let mut local_depth = vec![f32::INFINITY; local_len];
            render_prepared_obj_sprite_to_shared_rgba_buffers(
                &prepared_mesh,
                asset_root,
                &mut local_canvas,
                &mut local_depth,
            );
            merge_world3d_local_canvas(
                &mut batch_canvas,
                &mut batch_depth,
                scene_w,
                scene_h,
                &local_canvas,
                &local_depth,
                local_w,
                local_h,
                prepared_mesh.draw_x,
                prepared_mesh.draw_y,
            );

            if let Some(id) = object_id {
                object_regions.insert(
                    id.to_string(),
                    clamp_region(
                        prepared_mesh.draw_x,
                        prepared_mesh.draw_y,
                        prepared_mesh.target_w,
                        prepared_mesh.target_h,
                        scene_w,
                        scene_h,
                    ),
                );
            }
        }

        if !layer_supported {
            break;
        }
        executed_layers.insert(prepared.index);
    }

    if executed_layers.is_empty() {
        return None;
    }

    blit_rgba_canvas(
        buffer,
        &batch_canvas,
        scene_w,
        scene_h,
        scene_w,
        scene_h,
        0,
        0,
    );
    Some(executed_layers)
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
    #[cfg(feature = "render-3d")]
    let fully_batched_layers = inputs.fully_batched_layers;
    #[cfg(feature = "render-3d")]
    let prepared_layer_inputs = inputs.prepared_layer_inputs;
    let asset_root = inputs.render.asset_root;
    let resolved_view_profile = inputs.render.resolved_view_profile;
    let obj_camera_states = inputs.render.obj_camera_states;
    let scene_camera_3d = inputs.render.scene_camera_3d;
    let spatial_context = inputs.render.spatial_context;
    let celestial_catalogs = inputs.render.celestial_catalogs;
    let uses_pixel_output = inputs.render.uses_pixel_output;
    let default_font = inputs.render.default_font;
    let resolved_render_pipeline = resolve_render_2d_pipeline(
        inputs.render.render_2d_pipeline,
        resolved_view_profile,
        obj_camera_states,
        scene_camera_3d,
        spatial_context,
        celestial_catalogs,
        inputs.render.prerender_frames,
    );
    let render_2d_pipeline: &dyn Render2dPipeline = resolved_render_pipeline.pipeline();
    #[cfg(feature = "render-3d")]
    let mut executed_world3d_layers = HashSet::new();

    for (prepared_idx, prepared) in prepared_layers.iter().enumerate() {
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
        let use_2d_camera = prepared.uses_2d_camera;
        let (total_origin_x, total_origin_y) = resolve_layer_origin(
            use_2d_camera,
            scene_w,
            scene_h,
            scene_origin_x,
            scene_origin_y,
            &layer_state,
            camera_x,
            camera_y,
            camera_zoom,
        );

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
        if layer_is_culled(
            layer_object_id.is_some(),
            total_origin_x,
            total_origin_y,
            scene_w,
            scene_h,
        ) {
            continue;
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

        #[cfg(feature = "render-3d")]
        if !executed_world3d_layers.contains(&layer_idx) {
            if let Some(executed_layers) = render_world3d_batch_run(
                prepared_layers,
                prepared_layer_inputs,
                fully_batched_layers,
                prepared_idx,
                scene_w,
                scene_h,
                target_resolver,
                object_regions,
                scene_origin_x,
                scene_origin_y,
                object_states,
                current_stage,
                scene_elapsed_ms,
                camera_x,
                camera_y,
                camera_zoom,
                resolved_view_profile,
                obj_camera_states,
                scene_camera_3d,
                spatial_context,
                asset_root,
                inputs.render.prerender_frames,
                buffer,
            ) {
                executed_world3d_layers.extend(executed_layers);
            }
        }

        #[cfg(feature = "render-3d")]
        if executed_world3d_layers.contains(&layer_idx) {
            continue;
        }

        // #4 opt-comp-layerscratch: LayerCompositor strategy — DirectLayerCompositor skips
        // scratch fill+blit for layers without active effects.
        // ScratchLayerCompositor (safe default) always uses the scratch path.
        let has_3d = prepared.has_3d;
        let needs_scratch = has_3d || layer_compositor.use_scratch(prepared.has_active_effects);

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
                        uses_pixel_output,
                        default_font,
                        ui_font_scale: if layer.ui {
                            inputs.render.ui_font_scale
                        } else {
                            1.0
                        },
                        ui_layout_scale_x: if layer.ui {
                            inputs.render.ui_layout_scale_x
                        } else {
                            1.0
                        },
                        ui_layout_scale_y: if layer.ui {
                            inputs.render.ui_layout_scale_y
                        } else {
                            1.0
                        },
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
                    uses_pixel_output,
                    default_font,
                    ui_font_scale: if layer.ui {
                        inputs.render.ui_font_scale
                    } else {
                        1.0
                    },
                    ui_layout_scale_x: if layer.ui {
                        inputs.render.ui_layout_scale_x
                    } else {
                        1.0
                    },
                    ui_layout_scale_y: if layer.ui {
                        inputs.render.ui_layout_scale_y
                    } else {
                        1.0
                    },
                },
                buffer,
            );
        }
    }
}
