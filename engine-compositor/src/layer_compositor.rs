use super::effect_applicator::apply_layer_effects;
use super::sprite_renderer::render_sprites;
use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene::{Layer, LayerSpace, SceneRenderedMode, SceneSpace};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_pipeline::LayerCompositor;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LAYER_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
    // OPT-39: Layer bounds cache for skip rendering layers entirely if outside viewport.
    static LAYER_BOUNDS_CACHE: RefCell<HashMap<usize, (i32, i32, i32, i32)>> = RefCell::new(HashMap::new());
    static NON_UI_JITTER_DIAG: RefCell<HashMap<String, (i32, i32)>> = RefCell::new(HashMap::new());
}

/// Composite all visible layers onto the scene framebuffer.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn composite_layers(
    layers: &[Layer],
    ui_enabled: bool,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    scene_space: SceneSpace,
    asset_root: Option<&AssetRoot>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    scene_origin_x: i32,
    scene_origin_y: i32,
    object_states: &HashMap<String, ObjectRuntimeState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    scene_camera_3d: &SceneCamera3D,
    is_pixel_backend: bool,
    default_font: Option<&str>,
    camera_x: i32,
    camera_y: i32,
    layer_compositor: &dyn LayerCompositor,
    buffer: &mut Buffer,
) {
    let resolve_layer_space = |layer: &Layer| match layer.space {
        LayerSpace::Inherit => {
            if layer.ui {
                LayerSpace::Screen
            } else {
                match scene_space {
                    SceneSpace::TwoD => LayerSpace::TwoD,
                    SceneSpace::ThreeD => LayerSpace::ThreeD,
                }
            }
        }
        other => other,
    };

    for (layer_idx, layer) in layers.iter().enumerate() {
        if layer.ui && !ui_enabled {
            continue;
        }
        let layer_object_id =
            target_resolver.and_then(|resolver| resolver.layer_object_id(layer_idx));
        let layer_state = layer_object_id
            .and_then(|object_id| object_states.get(object_id))
            .cloned()
            .unwrap_or_default();
        if !layer.visible {
            continue;
        }
        if !layer_state.visible {
            continue;
        }
        let base_x = scene_origin_x.saturating_add(layer_state.offset_x);
        let base_y = scene_origin_y.saturating_add(layer_state.offset_y);
        let layer_space = resolve_layer_space(layer);
        let use_2d_camera = matches!(layer_space, LayerSpace::TwoD);
        let total_origin_x = if use_2d_camera {
            base_x.saturating_sub(camera_x)
        } else {
            base_x
        };
        let total_origin_y = if use_2d_camera {
            base_y.saturating_sub(camera_y)
        } else {
            base_y
        };

        // ── Flicker diagnostic: detect non-UI layer position jitter ──────
        if !layer.ui {
            use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
            static JITTER_COUNT: AtomicU64 = AtomicU64::new(0);
            if layer_idx >= layers.len().saturating_sub(10) || layer.name.starts_with("ship") || layer.name.contains("ship") {
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
        let layer_has_active_effects = {
            let stage_ref = match current_stage {
                SceneStage::OnEnter => &layer.stages.on_enter,
                SceneStage::OnIdle => &layer.stages.on_idle,
                SceneStage::OnLeave => &layer.stages.on_leave,
                SceneStage::Done => &layer.stages.on_idle,
            };
            // Layer has effects OR layer has sprites with appear/disappear timing.
            // Sprites with timing need scratch path for proper dirty region tracking when they vanish.
            stage_ref.steps.iter().any(|s| !s.effects.is_empty())
                || layer
                    .sprites
                    .iter()
                    .any(|s| s.appear_at_ms().is_some() || s.disappear_at_ms().is_some())
        };
        let needs_scratch = layer_compositor.use_scratch(layer_has_active_effects);

        if needs_scratch {
            // Full scratch path: fill + render + effects + blit.
            LAYER_SCRATCH.with(|scratch| {
                let mut layer_buf = scratch.borrow_mut();
                if layer_buf.width != scene_w || layer_buf.height != scene_h {
                    layer_buf.resize(scene_w, scene_h);
                }
                layer_buf.fill(Color::Reset);

                render_sprites(
                    layer_idx,
                    layer,
                    scene_w,
                    scene_h,
                    scene_rendered_mode,
                    asset_root,
                    target_resolver,
                    object_regions,
                    total_origin_x,
                    total_origin_y,
                    object_states,
                    scene_elapsed_ms,
                    current_stage,
                    step_idx,
                    elapsed_ms,
                    obj_camera_states,
                    scene_camera_3d,
                    is_pixel_backend,
                    default_font,
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
            render_sprites(
                layer_idx,
                layer,
                scene_w,
                scene_h,
                scene_rendered_mode,
                asset_root,
                target_resolver,
                object_regions,
                total_origin_x,
                total_origin_y,
                object_states,
                scene_elapsed_ms,
                current_stage,
                step_idx,
                elapsed_ms,
                obj_camera_states,
                scene_camera_3d,
                is_pixel_backend,
                default_font,
                buffer,
            );
        }
    }
}
