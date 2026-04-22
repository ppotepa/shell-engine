use crate::pipeline::GeneratedWorldSpriteSpec;
use crate::raster::take_last_obj_raster_stats;
use crate::scene::Renderable3D;
use crate::ObjRenderParams;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::{CameraSource, SpriteSizePreset, TonemapOperator};
use engine_core::scene_runtime_types::SceneCamera3D;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

const DEFAULT_WORLD_CLOUD_2_COLOR: Color = Color::Rgb {
    r: 0xd7,
    g: 0xe2,
    b: 0xec,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct GeneratedWorldPassMetrics {
    pub sprites_rendered: u32,
    pub viewport_area_px: u32,
    pub triangles_processed: u32,
    pub faces_drawn: u32,
    pub surface_us: f32,
    pub cloud1_us: f32,
    pub cloud2_us: f32,
    pub convert_us: f32,
    pub composite_us: f32,
    pub blit_us: f32,
}

thread_local! {
    static GENERATED_WORLD_PASS_METRICS: RefCell<GeneratedWorldPassMetrics> =
        const { RefCell::new(GeneratedWorldPassMetrics {
            sprites_rendered: 0,
            viewport_area_px: 0,
            triangles_processed: 0,
            faces_drawn: 0,
            surface_us: 0.0,
            cloud1_us: 0.0,
            cloud2_us: 0.0,
            convert_us: 0.0,
            composite_us: 0.0,
            blit_us: 0.0,
        }) };
}

static GENERATED_WORLD_CLOUD_CACHE: OnceLock<Mutex<HashMap<String, CachedCloudLayer>>> =
    OnceLock::new();

fn cloud_cache() -> &'static Mutex<HashMap<String, CachedCloudLayer>> {
    GENERATED_WORLD_CLOUD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
struct CachedCloudLayer {
    rgba: Arc<Vec<Option<[u8; 4]>>>,
    last_update_ms: u64,
    last_yaw_deg: f32,
    last_pitch_deg: f32,
    signature: u64,
}

#[inline]
fn accumulate_generated_world_pass_metrics(delta: GeneratedWorldPassMetrics) {
    GENERATED_WORLD_PASS_METRICS.with(|cell| {
        let mut acc = cell.borrow_mut();
        acc.sprites_rendered = acc.sprites_rendered.saturating_add(delta.sprites_rendered);
        acc.viewport_area_px = acc.viewport_area_px.max(delta.viewport_area_px);
        acc.triangles_processed = acc
            .triangles_processed
            .saturating_add(delta.triangles_processed);
        acc.faces_drawn = acc.faces_drawn.saturating_add(delta.faces_drawn);
        acc.surface_us += delta.surface_us;
        acc.cloud1_us += delta.cloud1_us;
        acc.cloud2_us += delta.cloud2_us;
        acc.convert_us += delta.convert_us;
        acc.composite_us += delta.composite_us;
        acc.blit_us += delta.blit_us;
    });
}

pub fn reset_generated_world_pass_metrics() {
    GENERATED_WORLD_PASS_METRICS
        .with(|cell| *cell.borrow_mut() = GeneratedWorldPassMetrics::default());
}

pub fn take_generated_world_pass_metrics() -> GeneratedWorldPassMetrics {
    GENERATED_WORLD_PASS_METRICS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

#[derive(Debug, Clone)]
pub struct GeneratedWorldRenderProfile {
    pub ambient: f32,
    pub ambient_floor: f32,
    pub shadow_contrast: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub tonemap: TonemapOperator,
    pub night_glow_scale: f32,
    pub haze_night_leak: f32,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
    pub terrain_displacement: f32,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub terrain_noise_scale: f32,
    pub terrain_noise_octaves: u8,
    pub marble_depth: f32,
    pub terrain_relief: f32,
    pub polar_ice_color: Option<[u8; 3]>,
    pub polar_ice_start: f32,
    pub polar_ice_end: f32,
    pub desert_color: Option<[u8; 3]>,
    pub desert_strength: f32,
    pub atmo_strength: f32,
    pub atmo_color: Option<[u8; 3]>,
    pub night_light_color: Option<[u8; 3]>,
    pub night_light_threshold: f32,
    pub night_light_intensity: f32,
    pub shadow_color: Option<Color>,
    pub midtone_color: Option<Color>,
    pub highlight_color: Option<Color>,
    pub tone_mix: f32,
    pub cel_levels: u8,
    pub noise_seed: f32,
    pub generated_heightmap: Option<std::sync::Arc<Vec<f32>>>,
    pub generated_heightmap_w: u32,
    pub generated_heightmap_h: u32,
    pub heightmap_blend: f32,
    pub warp_strength: f32,
    pub warp_octaves: u8,
    pub noise_lacunarity: f32,
    pub noise_persistence: f32,
    pub normal_perturb_strength: f32,
    pub ocean_specular: f32,
    pub ocean_noise_scale: f32,
    pub crater_density: f32,
    pub crater_rim_height: f32,
    pub snow_line_altitude: f32,
    pub ocean_color: Color,
    pub cloud_color: Color,
    pub cloud_threshold: f32,
    pub cloud_ambient: f32,
    pub cloud_noise_scale: f32,
    pub cloud_noise_octaves: u8,
    pub cloud_scale: f32,
    pub cloud2_scale: f32,
    pub cloud_render_scale_1: f32,
    pub cloud_render_scale_2: f32,
    pub atmo_visibility: f32,
    pub sun_dir: [f32; 3],
}

pub type RenderObjToRgbaCanvasFn = fn(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 4]>>, u16, u16)>;

pub type CompositeRgbaOverFn = fn(&mut [Option<[u8; 4]>], &[Option<[u8; 4]>]);
pub type BlitRgbaCanvasFn = fn(&mut Buffer, &[Option<[u8; 4]>], u16, u16, u16, u16, i32, i32);

pub struct GeneratedWorldRenderCallbacks {
    pub render_obj_to_rgba_canvas: RenderObjToRgbaCanvasFn,
    pub composite_rgba_over: CompositeRgbaOverFn,
    pub blit_rgba_canvas: BlitRgbaCanvasFn,
}

/// Phase 1 generated-world split: the opaque surface can move into a shared
/// world3d depth pass later, while cloud layers remain overlay-friendly.
#[derive(Debug, Clone)]
pub struct GeneratedWorldSurfacePass {
    pub node_id: String,
    pub mesh_path: String,
    pub size: Option<SpriteSizePreset>,
    pub sprite_width: u16,
    pub sprite_height: u16,
    pub draw_x: i32,
    pub draw_y: i32,
    pub params: ObjRenderParams,
    pub fg: Color,
}

#[derive(Debug, Clone)]
pub struct GeneratedWorldSurfaceCanvas {
    pub rgba: Vec<Option<[u8; 4]>>,
    pub virtual_w: u16,
    pub virtual_h: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedWorldOverlayKind {
    Cloud1,
    Cloud2,
}

#[derive(Debug, Clone)]
pub struct GeneratedWorldOverlayPass {
    pub kind: GeneratedWorldOverlayKind,
    pub mesh_path: String,
    pub size: Option<SpriteSizePreset>,
    pub render_width: u16,
    pub render_height: u16,
    pub params: ObjRenderParams,
    pub fg: Color,
    pub cache_key: String,
    pub cache_signature: u64,
    pub cache_interval_ms: u64,
    pub allow_stale_when_expensive: bool,
    pub stale_max_age_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GeneratedWorldResolvedPasses {
    pub surface: GeneratedWorldSurfacePass,
    pub overlays: Vec<GeneratedWorldOverlayPass>,
}

#[derive(Debug, Clone)]
struct GeneratedWorldResolvedBase {
    node_id: String,
    size: Option<SpriteSizePreset>,
    mesh_path: String,
    surface_scale: f32,
    base_yaw_deg: f32,
    surface_yaw_deg: f32,
    cloud1_yaw_deg: f32,
    cloud2_yaw_deg: f32,
    pitch_deg: f32,
    roll_deg: f32,
    camera_distance: f32,
    fov_degrees: f32,
    near_clip: f32,
    use_scene_camera: bool,
    sun_dir: [f32; 3],
}

#[inline]
fn clamped_cloud_render_scale(value: f32) -> f32 {
    value.clamp(0.2, 1.0)
}

#[inline]
fn scaled_dim(dim: u16, scale: f32) -> u16 {
    ((dim as f32 * clamped_cloud_render_scale(scale)).round() as u16).max(1)
}

#[inline]
fn adaptive_cloud_render_scale(base_scale: f32, sprite_w: u16, sprite_h: u16, layer: u8) -> f32 {
    let area_px = sprite_w as u32 * sprite_h as u32;
    let area_factor = if area_px >= 230_000 {
        0.82
    } else if area_px >= 170_000 {
        0.88
    } else if area_px >= 120_000 {
        0.94
    } else {
        1.0
    };
    let layer_factor = if layer == 2 { 0.94 } else { 1.0 };
    clamped_cloud_render_scale(base_scale * area_factor * layer_factor)
}

fn cloud_mesh_source(mesh_path: &str, divisor: u32, min_subdivisions: u32) -> String {
    if !mesh_path.starts_with("world://") {
        return mesh_path.to_string();
    }
    let rest = mesh_path.trim_start_matches("world://");
    let rest = rest
        .split_once(";lod=")
        .map(|(head, _)| head)
        .unwrap_or(rest);
    let (subdiv_raw, query) = rest.split_once('?').unwrap_or((rest, ""));
    let current_subdiv = subdiv_raw.trim().parse::<u32>().unwrap_or(32);
    let target_subdiv = (current_subdiv / divisor.max(1)).max(min_subdivisions);
    if query.is_empty() {
        format!("world://{target_subdiv}")
    } else {
        format!("world://{target_subdiv}?{query}")
    }
}

fn surface_mesh_source(mesh_path: &str, sprite_elapsed_ms: u64) -> String {
    if sprite_elapsed_ms < 220 {
        return cloud_mesh_source(mesh_path, 2, 48);
    }
    mesh_path.to_string()
}

#[inline]
fn cloud_update_interval_ms(cloud_layer_index: u8, angular_motion_deg: f32) -> u64 {
    if angular_motion_deg >= 1.25 {
        return 0;
    }
    if angular_motion_deg >= 0.35 {
        if cloud_layer_index == 1 {
            58
        } else {
            84
        }
    } else {
        if cloud_layer_index == 1 {
            120
        } else {
            170
        }
    }
}

fn cloud_signature(
    mesh_path: &str,
    width: u16,
    height: u16,
    scale: f32,
    threshold: f32,
    noise_scale: f32,
    noise_octaves: u8,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mesh_path.hash(&mut hasher);
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    ((scale * 1000.0).round() as i32).hash(&mut hasher);
    ((threshold * 1000.0).round() as i32).hash(&mut hasher);
    ((noise_scale * 1000.0).round() as i32).hash(&mut hasher);
    noise_octaves.hash(&mut hasher);
    hasher.finish()
}

fn get_cached_cloud_layer(
    key: &str,
    now_ms: u64,
    yaw_deg: f32,
    pitch_deg: f32,
    interval_ms: u64,
    signature: u64,
) -> Option<CachedCloudLayer> {
    let map = cloud_cache().lock().ok()?;
    let cached = map.get(key)?.clone();
    if cached.signature != signature {
        return None;
    }
    let age_ms = now_ms.saturating_sub(cached.last_update_ms);
    if age_ms > interval_ms {
        return None;
    }
    let angular_motion =
        (yaw_deg - cached.last_yaw_deg).abs() + (pitch_deg - cached.last_pitch_deg).abs();
    if angular_motion > 1.25 {
        return None;
    }
    Some(cached)
}

fn get_stale_cached_cloud_layer(
    key: &str,
    now_ms: u64,
    yaw_deg: f32,
    pitch_deg: f32,
    max_stale_ms: u64,
    signature: u64,
) -> Option<CachedCloudLayer> {
    let map = cloud_cache().lock().ok()?;
    let cached = map.get(key)?.clone();
    if cached.signature != signature {
        return None;
    }
    let age_ms = now_ms.saturating_sub(cached.last_update_ms);
    if age_ms > max_stale_ms {
        return None;
    }
    let angular_motion =
        (yaw_deg - cached.last_yaw_deg).abs() + (pitch_deg - cached.last_pitch_deg).abs();
    if angular_motion > 2.5 {
        return None;
    }
    Some(cached)
}

fn store_cached_cloud_layer(key: String, layer: CachedCloudLayer) {
    if let Ok(mut map) = cloud_cache().lock() {
        map.insert(key, layer);
    }
}

fn composite_rgba_over_scaled(
    dst: &mut [Option<[u8; 4]>],
    dst_w: u16,
    dst_h: u16,
    src: &[Option<[u8; 4]>],
    src_w: u16,
    src_h: u16,
) {
    if dst_w == 0 || dst_h == 0 || src_w == 0 || src_h == 0 {
        return;
    }
    let dst_len = dst_w as usize * dst_h as usize;
    if dst.len() < dst_len || src.len() < src_w as usize * src_h as usize {
        return;
    }
    if src_w == dst_w && src_h == dst_h {
        for i in 0..dst_len {
            let Some([sr, sg, sb, sa_u8]) = src[i] else {
                continue;
            };
            let sa = sa_u8 as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }
            let drgba = dst[i].unwrap_or([0, 0, 0, 0]);
            let da = drgba[3] as f32 / 255.0;
            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                continue;
            }
            let blend = |s: u8, d: u8| -> u8 {
                let s = s as f32 / 255.0;
                let d = d as f32 / 255.0;
                (((s * sa + d * da * (1.0 - sa)) / out_a).clamp(0.0, 1.0) * 255.0).round() as u8
            };
            dst[i] = Some([
                blend(sr, drgba[0]),
                blend(sg, drgba[1]),
                blend(sb, drgba[2]),
                (out_a * 255.0).round() as u8,
            ]);
        }
        return;
    }

    let dst_w_us = dst_w as usize;
    let dst_h_us = dst_h as usize;
    let src_w_us = src_w as usize;
    let src_h_us = src_h as usize;
    let sx_step = src_w as f32 / dst_w as f32;
    let sy_step = src_h as f32 / dst_h as f32;
    for dy in 0..dst_h_us {
        let sy = ((dy as f32 + 0.5) * sy_step - 0.5)
            .round()
            .clamp(0.0, (src_h_us - 1) as f32) as usize;
        for dx in 0..dst_w_us {
            let sx = ((dx as f32 + 0.5) * sx_step - 0.5)
                .round()
                .clamp(0.0, (src_w_us - 1) as f32) as usize;
            let src_idx = sy * src_w_us + sx;
            let Some([sr, sg, sb, sa_u8]) = src[src_idx] else {
                continue;
            };
            let sa = sa_u8 as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }
            let dst_idx = dy * dst_w_us + dx;
            let drgba = dst[dst_idx].unwrap_or([0, 0, 0, 0]);
            let da = drgba[3] as f32 / 255.0;
            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                continue;
            }
            let blend = |s: u8, d: u8| -> u8 {
                let s = s as f32 / 255.0;
                let d = d as f32 / 255.0;
                (((s * sa + d * da * (1.0 - sa)) / out_a).clamp(0.0, 1.0) * 255.0).round() as u8
            };
            dst[dst_idx] = Some([
                blend(sr, drgba[0]),
                blend(sg, drgba[1]),
                blend(sb, drgba[2]),
                (out_a * 255.0).round() as u8,
            ]);
        }
    }
}

fn resolve_generated_world_base(
    spec: GeneratedWorldSpriteSpec<'_>,
    profile: &GeneratedWorldRenderProfile,
) -> Option<GeneratedWorldResolvedBase> {
    let GeneratedWorldSpriteSpec {
        node,
        size,
        spin_deg,
        cloud_spin_deg,
        cloud2_spin_deg,
        camera_distance,
        camera_source,
        fov_degrees,
        near_clip,
        sun_dir_x,
        sun_dir_y,
        sun_dir_z,
        ..
    } = spec;
    let Renderable3D::GeneratedWorld(generated_world) = node.renderable else {
        return None;
    };

    Some(GeneratedWorldResolvedBase {
        node_id: node.id.clone(),
        size,
        mesh_path: generated_world.mesh_key.as_str().to_string(),
        surface_scale: node.transform.scale[0],
        base_yaw_deg: node.transform.rotation_deg[1],
        surface_yaw_deg: node.transform.rotation_deg[1] + spin_deg.unwrap_or(0.0),
        cloud1_yaw_deg: node.transform.rotation_deg[1] + cloud_spin_deg.unwrap_or(0.0),
        cloud2_yaw_deg: node.transform.rotation_deg[1] + 180.0 + cloud2_spin_deg.unwrap_or(0.0),
        pitch_deg: node.transform.rotation_deg[0],
        roll_deg: node.transform.rotation_deg[2],
        camera_distance: camera_distance.unwrap_or(3.0),
        fov_degrees: fov_degrees.unwrap_or(60.0),
        near_clip: near_clip.unwrap_or(0.001),
        use_scene_camera: camera_source == CameraSource::Scene,
        sun_dir: [
            sun_dir_x.unwrap_or(profile.sun_dir[0]),
            sun_dir_y.unwrap_or(profile.sun_dir[1]),
            sun_dir_z.unwrap_or(profile.sun_dir[2]),
        ],
    })
}

fn build_surface_pass(
    base: &GeneratedWorldResolvedBase,
    profile: &GeneratedWorldRenderProfile,
    sprite_width: u16,
    sprite_height: u16,
    draw_x: i32,
    draw_y: i32,
    sprite_elapsed: u64,
    scene_camera_3d: &SceneCamera3D,
) -> GeneratedWorldSurfacePass {
    let mut params = build_generated_world_base_params(
        base.surface_scale,
        base.surface_yaw_deg,
        base.pitch_deg,
        base.roll_deg,
        base.camera_distance,
        base.fov_degrees,
        base.near_clip,
        sprite_elapsed,
        base.use_scene_camera,
        scene_camera_3d,
        base.sun_dir,
        profile.ambient_floor,
        profile.shadow_contrast,
        profile.exposure,
        profile.gamma,
        profile.tonemap,
    );
    params.ambient = profile.ambient;
    params.ambient_floor = profile.ambient_floor;
    params.smooth_shading = true;
    params.latitude_bands = profile.latitude_bands;
    params.latitude_band_depth = profile.latitude_band_depth;
    params.terrain_displacement = profile.terrain_displacement;
    params.terrain_color = profile.terrain_color;
    params.terrain_threshold = profile.terrain_threshold;
    params.terrain_noise_scale = profile.terrain_noise_scale;
    params.terrain_noise_octaves = profile.terrain_noise_octaves;
    params.marble_depth = profile.marble_depth;
    params.terrain_relief = profile.terrain_relief;
    params.polar_ice_color = profile.polar_ice_color;
    params.polar_ice_start = profile.polar_ice_start;
    params.polar_ice_end = profile.polar_ice_end;
    params.desert_color = profile.desert_color;
    params.desert_strength = profile.desert_strength;
    params.atmo_color = None;
    params.atmo_height = 0.12;
    params.atmo_density = (profile.atmo_strength * profile.atmo_visibility).clamp(0.0, 1.0);
    params.atmo_strength = 0.0;
    params.atmo_rayleigh_amount = (profile.atmo_strength * profile.atmo_visibility).clamp(0.0, 1.0);
    params.atmo_rayleigh_color = profile.atmo_color;
    params.atmo_rayleigh_falloff = 0.32;
    params.atmo_haze_amount =
        (profile.atmo_strength * 0.45 * profile.atmo_visibility).clamp(0.0, 1.0);
    params.atmo_haze_color = params.atmo_rayleigh_color;
    params.atmo_haze_falloff = 0.18;
    params.atmo_absorption_amount = 0.0;
    params.atmo_absorption_color = None;
    params.atmo_absorption_height = 0.55;
    params.atmo_absorption_width = 0.18;
    params.atmo_forward_scatter = 0.72;
    params.atmo_limb_boost = 1.35;
    params.atmo_terminator_softness = 1.05;
    params.atmo_night_glow =
        (profile.atmo_strength * 0.08 * profile.atmo_visibility * profile.night_glow_scale)
            .clamp(0.0, 1.0);
    params.atmo_night_glow_color = None;
    params.atmo_haze_night_leak = profile.haze_night_leak;
    params.atmo_rim_power = 4.5;
    params.atmo_haze_strength = 0.0;
    params.atmo_haze_power = 1.8;
    params.atmo_veil_strength = 0.0;
    params.atmo_veil_power = 1.6;
    params.atmo_halo_strength = 0.0;
    params.atmo_halo_width = 0.12;
    params.atmo_halo_power = 2.2;
    params.night_light_color = profile.night_light_color;
    params.night_light_threshold = profile.night_light_threshold;
    params.night_light_intensity = profile.night_light_intensity;
    params.shadow_colour = profile.shadow_color;
    params.midtone_colour = profile.midtone_color;
    params.highlight_colour = profile.highlight_color;
    params.tone_mix = profile.tone_mix;
    params.cel_levels = profile.cel_levels;
    params.noise_seed = profile.noise_seed;
    params.heightmap = profile.generated_heightmap.clone();
    params.heightmap_w = profile.generated_heightmap_w;
    params.heightmap_h = profile.generated_heightmap_h;
    params.heightmap_blend = profile.heightmap_blend;
    params.warp_strength = profile.warp_strength;
    params.warp_octaves = profile.warp_octaves;
    params.noise_lacunarity = profile.noise_lacunarity;
    params.noise_persistence = profile.noise_persistence;
    params.normal_perturb_strength = profile.normal_perturb_strength;
    params.ocean_specular = profile.ocean_specular;
    params.ocean_noise_scale = profile.ocean_noise_scale;
    params.crater_density = profile.crater_density;
    params.crater_rim_height = profile.crater_rim_height;
    params.snow_line_altitude = profile.snow_line_altitude;

    let (ocean_r, ocean_g, ocean_b) = profile.ocean_color.to_rgb();
    params.ocean_color_rgb = Some([ocean_r, ocean_g, ocean_b]);

    GeneratedWorldSurfacePass {
        node_id: base.node_id.clone(),
        mesh_path: surface_mesh_source(base.mesh_path.as_str(), sprite_elapsed),
        size: base.size,
        sprite_width,
        sprite_height,
        draw_x,
        draw_y,
        params,
        fg: profile.ocean_color,
    }
}

fn build_cloud_overlay_passes(
    base: &GeneratedWorldResolvedBase,
    profile: &GeneratedWorldRenderProfile,
    sprite_width: u16,
    sprite_height: u16,
    sprite_elapsed: u64,
    scene_camera_3d: &SceneCamera3D,
) -> Vec<GeneratedWorldOverlayPass> {
    let mut cloud1_params = build_generated_world_base_params(
        profile.cloud_scale,
        base.cloud1_yaw_deg,
        base.pitch_deg,
        base.roll_deg,
        base.camera_distance,
        base.fov_degrees,
        base.near_clip,
        sprite_elapsed,
        base.use_scene_camera,
        scene_camera_3d,
        base.sun_dir,
        profile.ambient_floor,
        profile.shadow_contrast,
        profile.exposure,
        profile.gamma,
        profile.tonemap,
    );
    cloud1_params.ambient = profile.cloud_ambient;
    cloud1_params.ambient_floor = profile.ambient_floor;
    cloud1_params.smooth_shading = true;
    cloud1_params.terrain_color = Some(color_to_rgb(profile.cloud_color));
    cloud1_params.terrain_threshold = profile.cloud_threshold.clamp(0.0, 0.999);
    cloud1_params.terrain_noise_scale = profile.cloud_noise_scale;
    cloud1_params.terrain_noise_octaves = profile.cloud_noise_octaves.max(1);
    cloud1_params.marble_depth = (profile.marble_depth * 0.5).max(0.003);
    cloud1_params.below_threshold_transparent = true;
    cloud1_params.cloud_alpha_softness = 0.12;

    let cloud1_render_scale =
        adaptive_cloud_render_scale(profile.cloud_render_scale_1, sprite_width, sprite_height, 1);
    let cloud1_w = scaled_dim(sprite_width, cloud1_render_scale);
    let cloud1_h = scaled_dim(sprite_height, cloud1_render_scale);
    let cloud1_mesh_path = cloud_mesh_source(base.mesh_path.as_str(), 2, 24);
    let cloud1_interval_ms = cloud_update_interval_ms(
        1,
        (cloud1_params.yaw_deg - base.base_yaw_deg).abs()
            + (cloud1_params.pitch_deg - base.pitch_deg).abs(),
    );
    let cloud1_signature = cloud_signature(
        cloud1_mesh_path.as_str(),
        cloud1_w,
        cloud1_h,
        cloud1_params.scale,
        cloud1_params.terrain_threshold,
        cloud1_params.terrain_noise_scale,
        cloud1_params.terrain_noise_octaves,
    );

    let mut cloud2_params = build_generated_world_base_params(
        profile.cloud2_scale,
        base.cloud2_yaw_deg,
        base.pitch_deg,
        base.roll_deg,
        base.camera_distance,
        base.fov_degrees,
        base.near_clip,
        sprite_elapsed,
        base.use_scene_camera,
        scene_camera_3d,
        base.sun_dir,
        profile.ambient_floor,
        profile.shadow_contrast,
        profile.exposure,
        profile.gamma,
        profile.tonemap,
    );
    cloud2_params.ambient = 0.004;
    cloud2_params.ambient_floor = profile.ambient_floor;
    cloud2_params.smooth_shading = true;
    cloud2_params.terrain_color = Some(color_to_rgb(DEFAULT_WORLD_CLOUD_2_COLOR));
    cloud2_params.terrain_threshold = (profile.cloud_threshold + 0.12).min(0.992);
    cloud2_params.terrain_noise_scale = (profile.cloud_noise_scale * 0.35).max(1.1);
    cloud2_params.terrain_noise_octaves = profile.cloud_noise_octaves.clamp(1, 2);
    cloud2_params.marble_depth = (profile.marble_depth * 0.2).max(0.002);
    cloud2_params.below_threshold_transparent = true;
    cloud2_params.cloud_alpha_softness = 0.08;

    let cloud2_render_scale =
        adaptive_cloud_render_scale(profile.cloud_render_scale_2, sprite_width, sprite_height, 2);
    let cloud2_w = scaled_dim(sprite_width, cloud2_render_scale);
    let cloud2_h = scaled_dim(sprite_height, cloud2_render_scale);
    let cloud2_mesh_path = cloud_mesh_source(base.mesh_path.as_str(), 4, 16);
    let cloud2_interval_ms = cloud_update_interval_ms(
        2,
        (cloud2_params.yaw_deg - base.base_yaw_deg).abs()
            + (cloud2_params.pitch_deg - base.pitch_deg).abs(),
    );
    let cloud2_signature = cloud_signature(
        cloud2_mesh_path.as_str(),
        cloud2_w,
        cloud2_h,
        cloud2_params.scale,
        cloud2_params.terrain_threshold,
        cloud2_params.terrain_noise_scale,
        cloud2_params.terrain_noise_octaves,
    );
    let cloud2_stale_max_age_ms = cloud2_interval_ms.saturating_mul(2).saturating_add(80);

    vec![
        GeneratedWorldOverlayPass {
            kind: GeneratedWorldOverlayKind::Cloud1,
            mesh_path: cloud1_mesh_path.clone(),
            size: base.size,
            render_width: cloud1_w,
            render_height: cloud1_h,
            params: cloud1_params,
            fg: profile.cloud_color,
            cache_key: format!(
                "{}:cloud1:{}:{cloud1_w}x{cloud1_h}",
                base.node_id,
                cloud1_mesh_path.as_str()
            ),
            cache_signature: cloud1_signature,
            cache_interval_ms: cloud1_interval_ms,
            allow_stale_when_expensive: false,
            stale_max_age_ms: 0,
        },
        GeneratedWorldOverlayPass {
            kind: GeneratedWorldOverlayKind::Cloud2,
            mesh_path: cloud2_mesh_path.clone(),
            size: base.size,
            render_width: cloud2_w,
            render_height: cloud2_h,
            params: cloud2_params,
            fg: DEFAULT_WORLD_CLOUD_2_COLOR,
            cache_key: format!(
                "{}:cloud2:{}:{cloud2_w}x{cloud2_h}",
                base.node_id,
                cloud2_mesh_path.as_str()
            ),
            cache_signature: cloud2_signature,
            cache_interval_ms: cloud2_interval_ms,
            allow_stale_when_expensive: true,
            stale_max_age_ms: cloud2_stale_max_age_ms,
        },
    ]
}

pub fn resolve_generated_world_render_passes(
    spec: GeneratedWorldSpriteSpec<'_>,
    profile: &GeneratedWorldRenderProfile,
    sprite_width: u16,
    sprite_height: u16,
    draw_x: i32,
    draw_y: i32,
    sprite_elapsed: u64,
    scene_camera_3d: &SceneCamera3D,
) -> Option<GeneratedWorldResolvedPasses> {
    let base = resolve_generated_world_base(spec, profile)?;
    Some(GeneratedWorldResolvedPasses {
        surface: build_surface_pass(
            &base,
            profile,
            sprite_width,
            sprite_height,
            draw_x,
            draw_y,
            sprite_elapsed,
            scene_camera_3d,
        ),
        overlays: build_cloud_overlay_passes(
            &base,
            profile,
            sprite_width,
            sprite_height,
            sprite_elapsed,
            scene_camera_3d,
        ),
    })
}

fn accumulate_raster_stats(metrics: &mut GeneratedWorldPassMetrics) {
    let stats = take_last_obj_raster_stats();
    metrics.triangles_processed = metrics
        .triangles_processed
        .saturating_add(stats.triangles_processed);
    metrics.faces_drawn = metrics.faces_drawn.saturating_add(stats.faces_drawn);
    metrics.viewport_area_px = metrics.viewport_area_px.max(stats.viewport_area_px);
}

fn add_overlay_render_time(
    kind: GeneratedWorldOverlayKind,
    metrics: &mut GeneratedWorldPassMetrics,
    micros: f32,
) {
    match kind {
        GeneratedWorldOverlayKind::Cloud1 => metrics.cloud1_us += micros,
        GeneratedWorldOverlayKind::Cloud2 => metrics.cloud2_us += micros,
    }
}

pub fn render_generated_world_surface_canvas_with(
    surface: &GeneratedWorldSurfacePass,
    asset_root: Option<&AssetRoot>,
    callbacks: &GeneratedWorldRenderCallbacks,
    metrics: &mut GeneratedWorldPassMetrics,
) -> Option<GeneratedWorldSurfaceCanvas> {
    let t_surface = Instant::now();
    let (rgba, virtual_w, virtual_h) = (callbacks.render_obj_to_rgba_canvas)(
        surface.mesh_path.as_str(),
        Some(surface.sprite_width),
        Some(surface.sprite_height),
        surface.size,
        surface.params.clone(),
        false,
        surface.fg,
        asset_root,
    )?;
    metrics.surface_us = t_surface.elapsed().as_micros() as f32;
    accumulate_raster_stats(metrics);
    Some(GeneratedWorldSurfaceCanvas {
        rgba,
        virtual_w,
        virtual_h,
    })
}

fn composite_overlay_into_surface_canvas(
    surface_canvas: &mut GeneratedWorldSurfaceCanvas,
    overlay_rgba: &[Option<[u8; 4]>],
    overlay_width: u16,
    overlay_height: u16,
    callbacks: &GeneratedWorldRenderCallbacks,
) {
    if overlay_width == surface_canvas.virtual_w && overlay_height == surface_canvas.virtual_h {
        (callbacks.composite_rgba_over)(&mut surface_canvas.rgba, overlay_rgba);
    } else {
        composite_rgba_over_scaled(
            &mut surface_canvas.rgba,
            surface_canvas.virtual_w,
            surface_canvas.virtual_h,
            overlay_rgba,
            overlay_width,
            overlay_height,
        );
    }
}

pub fn composite_generated_world_overlay_passes(
    surface_canvas: &mut GeneratedWorldSurfaceCanvas,
    overlays: &[GeneratedWorldOverlayPass],
    sprite_elapsed: u64,
    asset_root: Option<&AssetRoot>,
    callbacks: &GeneratedWorldRenderCallbacks,
    metrics: &mut GeneratedWorldPassMetrics,
) {
    let mut expensive_overlay_rendered = false;

    for overlay in overlays {
        let overlay_arc = if let Some(cached) = get_cached_cloud_layer(
            overlay.cache_key.as_str(),
            sprite_elapsed,
            overlay.params.yaw_deg,
            overlay.params.pitch_deg,
            overlay.cache_interval_ms,
            overlay.cache_signature,
        ) {
            cached.rgba
        } else {
            if overlay.allow_stale_when_expensive && expensive_overlay_rendered {
                if let Some(cached) = get_stale_cached_cloud_layer(
                    overlay.cache_key.as_str(),
                    sprite_elapsed,
                    overlay.params.yaw_deg,
                    overlay.params.pitch_deg,
                    overlay.stale_max_age_ms,
                    overlay.cache_signature,
                ) {
                    let t_composite = Instant::now();
                    composite_overlay_into_surface_canvas(
                        surface_canvas,
                        cached.rgba.as_ref().as_slice(),
                        overlay.render_width,
                        overlay.render_height,
                        callbacks,
                    );
                    metrics.composite_us += t_composite.elapsed().as_micros() as f32;
                    continue;
                }
            }

            expensive_overlay_rendered = true;
            let t_overlay = Instant::now();
            let Some((overlay_rgba, _, _)) = (callbacks.render_obj_to_rgba_canvas)(
                overlay.mesh_path.as_str(),
                Some(overlay.render_width),
                Some(overlay.render_height),
                overlay.size,
                overlay.params.clone(),
                false,
                overlay.fg,
                asset_root,
            ) else {
                add_overlay_render_time(
                    overlay.kind,
                    metrics,
                    t_overlay.elapsed().as_micros() as f32,
                );
                let _ = take_last_obj_raster_stats();
                continue;
            };
            add_overlay_render_time(
                overlay.kind,
                metrics,
                t_overlay.elapsed().as_micros() as f32,
            );
            accumulate_raster_stats(metrics);
            let overlay_arc = Arc::new(overlay_rgba);
            store_cached_cloud_layer(
                overlay.cache_key.clone(),
                CachedCloudLayer {
                    rgba: overlay_arc.clone(),
                    last_update_ms: sprite_elapsed,
                    last_yaw_deg: overlay.params.yaw_deg,
                    last_pitch_deg: overlay.params.pitch_deg,
                    signature: overlay.cache_signature,
                },
            );
            overlay_arc
        };

        let t_composite = Instant::now();
        composite_overlay_into_surface_canvas(
            surface_canvas,
            overlay_arc.as_ref().as_slice(),
            overlay.render_width,
            overlay.render_height,
            callbacks,
        );
        metrics.composite_us += t_composite.elapsed().as_micros() as f32;
    }
}

pub fn blit_generated_world_surface_canvas(
    target: &mut Buffer,
    surface: &GeneratedWorldSurfacePass,
    surface_canvas: &GeneratedWorldSurfaceCanvas,
    callbacks: &GeneratedWorldRenderCallbacks,
    metrics: &mut GeneratedWorldPassMetrics,
) {
    let t_blit = Instant::now();
    (callbacks.blit_rgba_canvas)(
        target,
        &surface_canvas.rgba,
        surface_canvas.virtual_w,
        surface_canvas.virtual_h,
        surface.sprite_width,
        surface.sprite_height,
        surface.draw_x,
        surface.draw_y,
    );
    metrics.blit_us = t_blit.elapsed().as_micros() as f32;
}

#[allow(clippy::too_many_arguments)]
pub fn render_generated_world_sprite_with(
    spec: GeneratedWorldSpriteSpec<'_>,
    profile: &GeneratedWorldRenderProfile,
    sprite_width: u16,
    sprite_height: u16,
    draw_x: i32,
    draw_y: i32,
    sprite_elapsed: u64,
    scene_camera_3d: &SceneCamera3D,
    asset_root: Option<&AssetRoot>,
    target: &mut Buffer,
    callbacks: GeneratedWorldRenderCallbacks,
) -> bool {
    let Some(resolved) = resolve_generated_world_render_passes(
        spec,
        profile,
        sprite_width,
        sprite_height,
        draw_x,
        draw_y,
        sprite_elapsed,
        scene_camera_3d,
    ) else {
        return false;
    };

    let mut metrics = GeneratedWorldPassMetrics {
        sprites_rendered: 1,
        viewport_area_px: sprite_width as u32 * sprite_height as u32,
        ..GeneratedWorldPassMetrics::default()
    };

    let Some(mut surface_canvas) = render_generated_world_surface_canvas_with(
        &resolved.surface,
        asset_root,
        &callbacks,
        &mut metrics,
    ) else {
        return false;
    };

    metrics.convert_us = 0.0;
    composite_generated_world_overlay_passes(
        &mut surface_canvas,
        &resolved.overlays,
        sprite_elapsed,
        asset_root,
        &callbacks,
        &mut metrics,
    );
    blit_generated_world_surface_canvas(
        target,
        &resolved.surface,
        &surface_canvas,
        &callbacks,
        &mut metrics,
    );
    accumulate_generated_world_pass_metrics(metrics);

    true
}

fn color_to_rgb(color: Color) -> [u8; 3] {
    let (r, g, b) = color.to_rgb();
    [r, g, b]
}

fn build_generated_world_base_params(
    scale: f32,
    yaw_deg: f32,
    pitch_deg: f32,
    roll_deg: f32,
    camera_distance: f32,
    fov_degrees: f32,
    near_clip: f32,
    scene_elapsed_ms: u64,
    use_scene_camera: bool,
    scene_camera: &SceneCamera3D,
    sun_dir: [f32; 3],
    ambient_floor: f32,
    shadow_contrast: f32,
    exposure: f32,
    gamma: f32,
    tonemap: TonemapOperator,
) -> ObjRenderParams {
    ObjRenderParams {
        scale,
        yaw_deg,
        pitch_deg,
        roll_deg,
        rotation_x: 0.0,
        rotation_y: 0.0,
        rotation_z: 0.0,
        rotate_y_deg_per_sec: 0.0,
        camera_distance,
        fov_degrees,
        near_clip,
        light_direction_x: sun_dir[0],
        light_direction_y: sun_dir[1],
        light_direction_z: sun_dir[2],
        light_2_direction_x: 0.0,
        light_2_direction_y: 0.0,
        light_2_direction_z: -1.0,
        light_2_intensity: 0.0,
        light_point_x: 0.0,
        light_point_y: 2.0,
        light_point_z: 0.0,
        light_point_intensity: 0.0,
        light_point_colour: None,
        light_point_flicker_depth: 0.0,
        light_point_flicker_hz: 0.0,
        light_point_orbit_hz: 0.0,
        light_point_snap_hz: 0.0,
        light_point_2_x: 0.0,
        light_point_2_y: 0.0,
        light_point_2_z: 0.0,
        light_point_2_intensity: 0.0,
        light_point_2_colour: None,
        light_point_2_flicker_depth: 0.0,
        light_point_2_flicker_hz: 0.0,
        light_point_2_orbit_hz: 0.0,
        light_point_2_snap_hz: 0.0,
        cel_levels: 0,
        shadow_colour: None,
        midtone_colour: None,
        highlight_colour: None,
        tone_mix: 0.0,
        scene_elapsed_ms,
        camera_pan_x: 0.0,
        camera_pan_y: 0.0,
        camera_look_yaw: 0.0,
        camera_look_pitch: 0.0,
        object_translate_x: 0.0,
        object_translate_y: 0.0,
        object_translate_z: 0.0,
        clip_y_min: 0.0,
        clip_y_max: 1.0,
        camera_world_x: if use_scene_camera {
            scene_camera.eye[0]
        } else {
            0.0
        },
        camera_world_y: if use_scene_camera {
            scene_camera.eye[1]
        } else {
            0.0
        },
        camera_world_z: if use_scene_camera {
            scene_camera.eye[2]
        } else {
            -camera_distance
        },
        view_right_x: if use_scene_camera {
            scene_camera.right()[0]
        } else {
            1.0
        },
        view_right_y: if use_scene_camera {
            scene_camera.right()[1]
        } else {
            0.0
        },
        view_right_z: if use_scene_camera {
            scene_camera.right()[2]
        } else {
            0.0
        },
        view_up_x: if use_scene_camera {
            scene_camera.up[0]
        } else {
            0.0
        },
        view_up_y: if use_scene_camera {
            scene_camera.up[1]
        } else {
            1.0
        },
        view_up_z: if use_scene_camera {
            scene_camera.up[2]
        } else {
            0.0
        },
        view_forward_x: if use_scene_camera {
            scene_camera.forward()[0]
        } else {
            0.0
        },
        view_forward_y: if use_scene_camera {
            scene_camera.forward()[1]
        } else {
            0.0
        },
        view_forward_z: if use_scene_camera {
            scene_camera.forward()[2]
        } else {
            1.0
        },
        unlit: false,
        ambient: 0.05,
        ambient_floor,
        shadow_contrast,
        exposure,
        gamma,
        tonemap,
        light_point_falloff: 0.7,
        light_point_2_falloff: 0.7,
        smooth_shading: true,
        latitude_bands: 0,
        latitude_band_depth: 0.0,
        terrain_displacement: 0.0,
        terrain_color: None,
        terrain_threshold: 0.5,
        terrain_noise_scale: 2.5,
        terrain_noise_octaves: 2,
        marble_depth: 0.0,
        terrain_relief: 0.0,
        noise_seed: 0.0,
        warp_strength: 0.0,
        warp_octaves: 2,
        noise_lacunarity: 2.0,
        noise_persistence: 0.5,
        normal_perturb_strength: 0.0,
        ocean_specular: 0.0,
        crater_density: 0.0,
        crater_rim_height: 0.35,
        snow_line_altitude: 0.0,
        below_threshold_transparent: false,
        cloud_alpha_softness: 0.0,
        polar_ice_color: None,
        polar_ice_start: 0.78,
        polar_ice_end: 0.92,
        desert_color: None,
        desert_strength: 0.0,
        atmo_color: None,
        atmo_height: 0.12,
        atmo_density: 0.0,
        atmo_strength: 0.0,
        atmo_rayleigh_amount: 0.0,
        atmo_rayleigh_color: None,
        atmo_rayleigh_falloff: 0.32,
        atmo_haze_amount: 0.0,
        atmo_haze_color: None,
        atmo_haze_falloff: 0.18,
        atmo_absorption_amount: 0.0,
        atmo_absorption_color: None,
        atmo_absorption_height: 0.55,
        atmo_absorption_width: 0.18,
        atmo_forward_scatter: 0.72,
        atmo_limb_boost: 1.0,
        atmo_terminator_softness: 1.0,
        atmo_night_glow: 0.0,
        atmo_night_glow_color: None,
        atmo_haze_night_leak: 0.0,
        atmo_rim_power: 4.5,
        atmo_haze_strength: 0.0,
        atmo_haze_power: 1.8,
        atmo_veil_strength: 0.0,
        atmo_veil_power: 1.6,
        atmo_halo_strength: 0.0,
        atmo_halo_width: 0.12,
        atmo_halo_power: 2.2,
        ocean_noise_scale: 4.0,
        ocean_color_rgb: None,
        night_light_color: None,
        night_light_threshold: 0.82,
        night_light_intensity: 0.0,
        heightmap: None,
        heightmap_w: 0,
        heightmap_h: 0,
        heightmap_blend: 0.0,
        depth_sort_faces: false,
    }
}
