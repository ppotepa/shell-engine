use std::collections::HashMap;
#[cfg(feature = "render-3d")]
use std::collections::HashSet;

use crate::layer_compositor::{composite_layers, LayerCompositeInputs, PreparedLayerRenderInputs};
#[cfg(feature = "render-3d")]
use crate::prepared_frame::{
    collect_world3d_batch_plan, layer_frames_from_prepared, PreparedLayerInput,
};
use crate::{prepare_layer_frames, CompositeParams};
use engine_core::buffer::Buffer;
#[cfg(not(feature = "render-3d"))]
use engine_core::buffer::PixelCanvas;
#[cfg(test)]
use engine_core::color::Color;
use engine_core::effects::Region;
#[cfg(not(feature = "render-3d"))]
use engine_core::scene::{environment_policy_renders_space_environment, ResolvedViewProfile};
use engine_effects::apply_effect;
use engine_pipeline::LayerCompositor;
use engine_render_2d::Render2dPipeline;
#[cfg(feature = "render-3d")]
use engine_render_3d::scene::render_space_environment as render_scene_environment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerPassKind {
    #[default]
    All,
    WorldOnly,
    UiOnly,
}

impl LayerPassKind {
    #[inline]
    fn includes_prepared(self, prepared: &crate::scene_compositor::PreparedLayerFrame<'_>) -> bool {
        match self {
            Self::All => true,
            // 3D content always belongs to the world pass, even if a layer is
            // accidentally marked as UI.
            Self::WorldOnly => !prepared.layer.ui || prepared.has_3d,
            Self::UiOnly => prepared.layer.ui && !prepared.has_3d,
        }
    }

    #[cfg(feature = "render-3d")]
    pub(crate) fn includes_layer_input(self, prepared: &PreparedLayerInput<'_>) -> bool {
        match self {
            Self::All => true,
            Self::WorldOnly => !prepared.layer.ui || prepared.has_3d,
            Self::UiOnly => prepared.layer.ui && !prepared.has_3d,
        }
    }

    #[inline]
    fn clears_target(self) -> bool {
        !matches!(self, Self::UiOnly)
    }

    #[inline]
    fn renders_scene_environment(self) -> bool {
        !matches!(self, Self::UiOnly)
    }

    #[inline]
    fn applies_scene_effects(self) -> bool {
        !matches!(self, Self::UiOnly)
    }
}

fn composite_scene(
    params: &CompositeParams<'_>,
    render_2d_pipeline: Option<&dyn Render2dPipeline>,
    layer: &dyn LayerCompositor,
    pass: LayerPassKind,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    if pass.clears_target() {
        buffer.fill(params.bg);
    }
    if pass.renders_scene_environment() {
        render_scene_environment(buffer, params.prepared.resolved_view_profile);
    }
    let scene_state = params
        .prepared
        .object_states
        .get(params.prepared.target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return HashMap::new();
    }

    let mut object_regions = HashMap::with_capacity(params.frame.layers.len() + 4);
    object_regions.insert(
        params
            .prepared
            .target_resolver
            .scene_object_id()
            .to_string(),
        offset_region(
            buffer.width,
            buffer.height,
            scene_state.offset_x,
            scene_state.offset_y,
        ),
    );

    let scene_w = buffer.width;
    let scene_h = buffer.height;
    let ui_layout_scale_x = if params.prepared.ui_logical_width > 0 {
        scene_w as f32 / params.prepared.ui_logical_width as f32
    } else {
        1.0
    };
    let ui_layout_scale_y = if params.prepared.ui_logical_height > 0 {
        scene_h as f32 / params.prepared.ui_logical_height as f32
    } else {
        1.0
    };
    // Use pre-classified layer inputs when available; fall back to inline frame preparation.
    #[cfg(feature = "render-3d")]
    let (prepared_layers, fully_batched_layers): (Vec<_>, HashSet<usize>) =
        if let Some(inputs) = &params.frame.prepared_layer_inputs {
            let world3d_batch_plan = collect_world3d_batch_plan(inputs, pass);
            (
                layer_frames_from_prepared(inputs)
                    .into_iter()
                    .filter(|prepared| pass.includes_prepared(prepared))
                    .collect(),
                world3d_batch_plan.fully_batched_layers,
            )
        } else {
            (
                prepare_layer_frames(&params.frame, params.prepared.current_stage)
                    .into_iter()
                    .filter(|prepared| pass.includes_prepared(prepared))
                    .collect(),
                HashSet::new(),
            )
        };
    #[cfg(not(feature = "render-3d"))]
    let prepared_layers: Vec<_> =
        prepare_layer_frames(&params.frame, params.prepared.current_stage)
            .into_iter()
            .filter(|prepared| pass.includes_prepared(prepared))
            .collect();

    let mut layer_inputs = LayerCompositeInputs {
        prepared_layers: &prepared_layers,
        scene_w,
        scene_h,
        target_resolver: Some(params.prepared.target_resolver),
        object_regions: &mut object_regions,
        scene_origin_x: scene_state.offset_x,
        scene_origin_y: scene_state.offset_y,
        object_states: params.prepared.object_states,
        current_stage: params.prepared.current_stage,
        step_idx: params.prepared.step_idx,
        elapsed_ms: params.prepared.elapsed_ms,
        scene_elapsed_ms: params.prepared.scene_elapsed_ms,
        camera_x: params.prepared.camera.camera_x,
        camera_y: params.prepared.camera.camera_y,
        camera_zoom: params.prepared.camera.camera_zoom,
        #[cfg(feature = "render-3d")]
        fully_batched_layers: &fully_batched_layers,
        #[cfg(feature = "render-3d")]
        prepared_layer_inputs: params.frame.prepared_layer_inputs.as_deref(),
        render: PreparedLayerRenderInputs {
            render_2d_pipeline,
            asset_root: params.prepared.asset_root,
            resolved_view_profile: params.prepared.resolved_view_profile,
            obj_camera_states: params.prepared.obj_camera_states,
            scene_camera_3d: params.prepared.camera.scene_camera_3d,
            spatial_context: params.prepared.camera.spatial_context,
            celestial_catalogs: params.prepared.celestial_catalogs,
            is_pixel_backend: params.prepared.is_pixel_backend,
            default_font: params.prepared.default_font,
            ui_font_scale: params.prepared.ui_font_scale,
            ui_layout_scale_x,
            ui_layout_scale_y,
            prerender_frames: params.prepared.prerender_frames,
        },
    };
    composite_layers(&mut layer_inputs, layer, buffer);

    if params.frame.scene_effects.is_empty() || !pass.applies_scene_effects() {
        return object_regions;
    }
    let full_region = Region::full(buffer);
    for effect in params.frame.scene_effects {
        let region = params.prepared.target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(
            effect,
            params.prepared.scene_effect_progress,
            region,
            buffer,
        );
    }
    object_regions
}

#[cfg(not(feature = "render-3d"))]
fn render_scene_environment(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    if !environment_policy_renders_space_environment(view.environment_policy) {
        return;
    }
    render_starfield(buffer, view);
    render_primary_star_glare(buffer, view);
}

#[cfg(not(feature = "render-3d"))]
fn render_starfield(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    let env = &view.environment;
    let density = env.starfield_density.unwrap_or(0.0).clamp(0.0, 1.0);
    let brightness = env.starfield_brightness.unwrap_or(0.0).clamp(0.0, 1.5);
    if density <= 0.0 || brightness <= 0.0 || buffer.width == 0 || buffer.height == 0 {
        return;
    }

    let area = buffer.width as usize * buffer.height as usize;
    let star_count = ((area as f32 / 180.0) * density).round() as usize;
    if star_count == 0 {
        return;
    }

    let size_min = env.starfield_size_min.unwrap_or(1.0).clamp(0.5, 3.0);
    let size_max = env
        .starfield_size_max
        .unwrap_or(size_min.max(1.0))
        .clamp(size_min, 4.0);
    let (r, g, b) = star_rgb(brightness);
    let star_color = Color::rgb(r, g, b);
    let mut seed = starfield_seed(buffer.width, buffer.height, density, brightness);

    if let Some(canvas) = &mut buffer.pixel_canvas {
        for _ in 0..star_count {
            let x = (next_u32(&mut seed) % canvas.width as u32) as u16;
            let y = (next_u32(&mut seed) % canvas.height as u32) as u16;
            let size = lerp_size(
                size_min,
                size_max,
                next_u32(&mut seed) as f32 / u32::MAX as f32,
            );
            draw_star_pixels(canvas, x, y, size, r, g, b);
        }
        return;
    }

    for _ in 0..star_count {
        let x = (next_u32(&mut seed) % buffer.width as u32) as u16;
        let y = (next_u32(&mut seed) % buffer.height as u32) as u16;
        let size = lerp_size(
            size_min,
            size_max,
            next_u32(&mut seed) as f32 / u32::MAX as f32,
        );
        let glyph = if size >= 1.6 { '*' } else { '.' };
        buffer.set(x, y, glyph, star_color, Color::BLACK);
    }
}

#[cfg(not(feature = "render-3d"))]
fn render_primary_star_glare(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    let env = &view.environment;
    let strength = env
        .primary_star_glare_strength
        .unwrap_or(0.0)
        .clamp(0.0, 1.5);
    if strength <= 0.0 || buffer.width == 0 || buffer.height == 0 {
        return;
    }
    let width = env
        .primary_star_glare_width
        .unwrap_or(0.18)
        .clamp(0.02, 1.0);
    let (r, g, b) = parse_hex_rgb(env.primary_star_color.as_deref().unwrap_or("#fff4d6"))
        .unwrap_or((255, 244, 214));

    if let Some(canvas) = &mut buffer.pixel_canvas {
        render_primary_star_glare_pixels(canvas, strength, width, r, g, b);
        return;
    }
    render_primary_star_glare_cells(buffer, strength, width, r, g, b);
}

#[cfg(not(feature = "render-3d"))]
fn star_rgb(brightness: f32) -> (u8, u8, u8) {
    let value = (180.0 + 75.0 * brightness.clamp(0.0, 1.0)).round() as u8;
    (value, value, (value as f32 * 0.98).round() as u8)
}

#[cfg(not(feature = "render-3d"))]
fn starfield_seed(width: u16, height: u16, density: f32, brightness: f32) -> u64 {
    let mut seed = 0xcbf29ce484222325_u64;
    seed ^= width as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= height as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= density.to_bits() as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= brightness.to_bits() as u64;
    seed
}

#[cfg(not(feature = "render-3d"))]
fn next_u32(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 32) as u32
}

#[cfg(not(feature = "render-3d"))]
fn lerp_size(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

#[cfg(not(feature = "render-3d"))]
fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if hex.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

#[cfg(not(feature = "render-3d"))]
fn glare_curve(t: f32, strength: f32) -> f32 {
    let core = t.clamp(0.0, 1.0).powf(2.2);
    (core * strength * 0.7).clamp(0.0, 1.0)
}

#[cfg(not(feature = "render-3d"))]
fn blend_channel(base: u8, tint: u8, amount: f32) -> u8 {
    (base as f32 + tint as f32 * amount.clamp(0.0, 1.0))
        .clamp(0.0, 255.0)
        .round() as u8
}

#[cfg(not(feature = "render-3d"))]
fn render_primary_star_glare_pixels(
    canvas: &mut PixelCanvas,
    strength: f32,
    width: f32,
    r: u8,
    g: u8,
    b: u8,
) {
    let cw = canvas.width as f32;
    let ch = canvas.height as f32;
    let cx = -cw * 0.18;
    let cy = ch * 0.16;
    let radius = cw.max(ch) * (0.22 + width * 0.48);
    let radius_sq = radius * radius;
    let max_x = (cx + radius).ceil().clamp(0.0, cw - 1.0) as u16;
    let min_y = (cy - radius).floor().clamp(0.0, ch - 1.0) as u16;
    let max_y = (cy + radius).ceil().clamp(0.0, ch - 1.0) as u16;

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in 0..=max_x {
            let px = x as f32 + 0.5;
            let dx = px - cx;
            let dy = py - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= radius_sq {
                continue;
            }
            let glow = glare_curve(1.0 - (dist_sq / radius_sq).sqrt(), strength);
            if glow <= 0.001 {
                continue;
            }
            let idx = (y as usize * canvas.width as usize + x as usize) * 4;
            canvas.data[idx] = blend_channel(canvas.data[idx], r, glow);
            canvas.data[idx + 1] = blend_channel(canvas.data[idx + 1], g, glow);
            canvas.data[idx + 2] = blend_channel(canvas.data[idx + 2], b, glow);
            canvas.data[idx + 3] = 255;
        }
    }
    canvas.dirty = true;
}

#[cfg(not(feature = "render-3d"))]
fn render_primary_star_glare_cells(
    buffer: &mut Buffer,
    strength: f32,
    width: f32,
    r: u8,
    g: u8,
    b: u8,
) {
    let bw = buffer.width as f32;
    let bh = buffer.height as f32;
    let cx = -bw * 0.18;
    let cy = bh * 0.16;
    let radius = bw.max(bh) * (0.22 + width * 0.48);
    let radius_sq = radius * radius;
    let max_x = (cx + radius).ceil().clamp(0.0, bw - 1.0) as u16;
    let min_y = (cy - radius).floor().clamp(0.0, bh - 1.0) as u16;
    let max_y = (cy + radius).ceil().clamp(0.0, bh - 1.0) as u16;
    let stride = buffer.width as usize;
    let cells = buffer.back_cells_mut();

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in 0..=max_x {
            let px = x as f32 + 0.5;
            let dx = px - cx;
            let dy = py - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= radius_sq {
                continue;
            }
            let glow = glare_curve(1.0 - (dist_sq / radius_sq).sqrt(), strength);
            if glow <= 0.001 {
                continue;
            }
            let cell = &mut cells[y as usize * stride + x as usize];
            let (bg_r, bg_g, bg_b) = cell.bg.to_rgb();
            cell.bg = Color::rgb(
                blend_channel(bg_r, r, glow),
                blend_channel(bg_g, g, glow),
                blend_channel(bg_b, b, glow),
            );
            if cell.symbol != ' ' {
                let (fg_r, fg_g, fg_b) = cell.fg.to_rgb();
                cell.fg = Color::rgb(
                    blend_channel(fg_r, r, glow * 0.7),
                    blend_channel(fg_g, g, glow * 0.7),
                    blend_channel(fg_b, b, glow * 0.7),
                );
            }
        }
    }
    buffer.mark_all_dirty();
}

#[cfg(not(feature = "render-3d"))]
fn draw_star_pixels(canvas: &mut PixelCanvas, x: u16, y: u16, size: f32, r: u8, g: u8, b: u8) {
    canvas.set_pixel(x, y, r, g, b);
    if size < 1.6 {
        return;
    }
    if x > 0 {
        canvas.set_pixel(x - 1, y, r, g, b);
    }
    if x + 1 < canvas.width {
        canvas.set_pixel(x + 1, y, r, g, b);
    }
    if y > 0 {
        canvas.set_pixel(x, y - 1, r, g, b);
    }
    if y + 1 < canvas.height {
        canvas.set_pixel(x, y + 1, r, g, b);
    }
}

#[inline]
fn offset_region(width: u16, height: u16, offset_x: i32, offset_y: i32) -> Region {
    let origin_x = offset_x.max(0) as u16;
    let origin_y = offset_y.max(0) as u16;
    let clipped_w = width.saturating_sub(offset_x.unsigned_abs().min(width as u32) as u16);
    let clipped_h = height.saturating_sub(offset_y.unsigned_abs().min(height as u32) as u16);
    Region {
        x: origin_x,
        y: origin_y,
        width: clipped_w,
        height: clipped_h,
    }
}

pub fn dispatch_composite(
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    composite_scene(params, None, layer, LayerPassKind::All, buffer)
}

/// Composite a frame using a caller-provided 2D pipeline.
///
/// This keeps compositor focused on frame assembly while allowing runtime wiring to
/// provide render-domain implementation details.
pub fn dispatch_composite_with_render_2d_pipeline(
    params: &CompositeParams<'_>,
    render_2d_pipeline: Option<&dyn Render2dPipeline>,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    composite_scene(
        params,
        render_2d_pipeline,
        layer,
        LayerPassKind::All,
        buffer,
    )
}

pub fn dispatch_composite_filtered(
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    pass: LayerPassKind,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    composite_scene(params, None, layer, pass, buffer)
}

pub fn dispatch_composite_with_render_2d_pipeline_filtered(
    params: &CompositeParams<'_>,
    render_2d_pipeline: Option<&dyn Render2dPipeline>,
    layer: &dyn LayerCompositor,
    pass: LayerPassKind,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    composite_scene(params, render_2d_pipeline, layer, pass, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::scene::{
        LightingProfile, ResolvedViewProfile, SpaceEnvironmentProfile, ViewEnvironmentPolicy,
    };
    use engine_render_3d::scene::render_space_environment;

    #[test]
    fn starfield_pass_writes_pixels_when_environment_requests_it() {
        let mut buffer = Buffer::new(64, 32);
        buffer.enable_pixel_canvas(64, 32);
        buffer.fill(Color::BLACK);

        render_space_environment(
            &mut buffer,
            &ResolvedViewProfile {
                environment_policy: ViewEnvironmentPolicy::ThreeDCelestial,
                lighting: LightingProfile::default(),
                environment: SpaceEnvironmentProfile {
                    starfield_density: Some(0.5),
                    starfield_brightness: Some(0.9),
                    starfield_size_min: Some(1.0),
                    starfield_size_max: Some(1.5),
                    ..Default::default()
                },
                overrides: Default::default(),
            },
        );

        let lit_pixels = buffer
            .pixel_canvas
            .as_ref()
            .expect("pixel canvas")
            .data
            .chunks_exact(4)
            .filter(|px| px[0] > 0 || px[1] > 0 || px[2] > 0)
            .count();
        assert!(lit_pixels > 0);
    }

    #[test]
    fn primary_star_glare_brightens_scene_background() {
        let mut buffer = Buffer::new(64, 32);
        buffer.enable_pixel_canvas(64, 32);
        buffer.fill(Color::BLACK);

        render_space_environment(
            &mut buffer,
            &ResolvedViewProfile {
                environment_policy: ViewEnvironmentPolicy::ThreeDCelestial,
                lighting: LightingProfile::default(),
                environment: SpaceEnvironmentProfile {
                    primary_star_color: Some("#fff4d6".to_string()),
                    primary_star_glare_strength: Some(0.45),
                    primary_star_glare_width: Some(0.32),
                    ..Default::default()
                },
                overrides: Default::default(),
            },
        );

        let canvas = buffer.pixel_canvas.as_ref().expect("pixel canvas");
        let top_left = &canvas.data[..4];
        let center_idx =
            ((canvas.height as usize / 2) * canvas.width as usize + canvas.width as usize / 2) * 4;
        let center = &canvas.data[center_idx..center_idx + 4];
        assert!(top_left[0] > center[0] || top_left[1] > center[1] || top_left[2] > center[2]);
    }

    #[test]
    fn euclidean_policy_suppresses_environment_in_local_fallback_path() {
        let mut buffer = Buffer::new(64, 32);
        buffer.enable_pixel_canvas(64, 32);
        buffer.fill(Color::BLACK);

        render_scene_environment(
            &mut buffer,
            &ResolvedViewProfile {
                environment_policy: ViewEnvironmentPolicy::ThreeDEuclidean,
                lighting: LightingProfile::default(),
                environment: SpaceEnvironmentProfile {
                    starfield_density: Some(0.5),
                    starfield_brightness: Some(0.9),
                    primary_star_glare_strength: Some(0.45),
                    primary_star_glare_width: Some(0.32),
                    ..Default::default()
                },
                overrides: Default::default(),
            },
        );

        let lit_pixels = buffer
            .pixel_canvas
            .as_ref()
            .expect("pixel canvas")
            .data
            .chunks_exact(4)
            .filter(|px| px[0] > 0 || px[1] > 0 || px[2] > 0)
            .count();
        assert_eq!(lit_pixels, 0);
    }
}
