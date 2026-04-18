use std::cell::RefCell;
use std::time::Instant;

use crate::geom::math::dot3;
use crate::shading::mix_rgb;
use crate::ObjRenderParams;

#[derive(Debug, Clone)]
struct HaloTemporalCache {
    virtual_w: u16,
    virtual_h: u16,
    temporal_key: u64,
    material_key: u64,
    center_x: f32,
    center_y: f32,
    radius: f32,
    edge_count: usize,
    halo_pixels: Vec<(u32, [u8; 3])>,
}

thread_local! {
    static HALO_EDGE_PIXELS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
    static HALO_OCCUPIED_SCAN: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static HALO_NEAREST_SQ: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static HALO_TEMPORAL_CACHE: RefCell<Option<HaloTemporalCache>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Copy)]
pub struct HaloPassParams {
    pub ray_color: [u8; 3],
    pub haze_color: [u8; 3],
    pub absorption_color: [u8; 3],
    pub halo_strength: f32,
    pub halo_width: f32,
    pub halo_power: f32,
    pub rayleigh_amount: f32,
    pub haze_amount: f32,
    pub absorption_amount: f32,
    pub forward_scatter: f32,
    pub haze_night_leak: f32,
    pub night_glow: f32,
    pub night_glow_color: [u8; 3],
    pub light_intensity: f32,
    pub light_dir: [f32; 3],
    pub view_right: [f32; 3],
    pub view_up: [f32; 3],
    pub temporal_key: u64,
}

#[inline]
fn quantize_f32(value: f32, step: f32) -> i32 {
    (value / step.max(1e-6)).round() as i32
}

#[inline]
fn mix_hash(seed: &mut u64, value: i64) {
    *seed ^= value as u64;
    *seed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    *seed ^= *seed >> 29;
}

pub fn halo_temporal_key_from_obj_params(params: &ObjRenderParams) -> u64 {
    let mut key = 0xCBF2_9CE4_8422_2325u64;
    mix_hash(
        &mut key,
        quantize_f32(params.yaw_deg + params.rotation_y, 0.4) as i64,
    );
    mix_hash(
        &mut key,
        quantize_f32(params.pitch_deg + params.rotation_x, 0.4) as i64,
    );
    mix_hash(
        &mut key,
        quantize_f32(params.roll_deg + params.rotation_z, 0.5) as i64,
    );
    mix_hash(&mut key, quantize_f32(params.scale, 0.02) as i64);
    mix_hash(
        &mut key,
        quantize_f32(params.camera_distance.max(0.01), 0.05) as i64,
    );
    mix_hash(&mut key, quantize_f32(params.camera_look_yaw, 0.2) as i64);
    mix_hash(&mut key, quantize_f32(params.camera_look_pitch, 0.2) as i64);
    mix_hash(
        &mut key,
        quantize_f32(params.object_translate_x, 0.08) as i64,
    );
    mix_hash(
        &mut key,
        quantize_f32(params.object_translate_y, 0.08) as i64,
    );
    mix_hash(
        &mut key,
        quantize_f32(params.object_translate_z, 0.08) as i64,
    );
    key
}

pub(crate) fn apply_obj_halo_from_params(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    params: &ObjRenderParams,
) -> f32 {
    if params.atmo_density <= 0.0
        || (params.atmo_rayleigh_amount <= 0.0
            && params.atmo_haze_amount <= 0.0
            && params.atmo_absorption_amount <= 0.0)
    {
        return 0.0;
    }

    let ray_color = params
        .atmo_rayleigh_color
        .or(params.atmo_color)
        .unwrap_or([124, 200, 255]);
    let haze_color = params.atmo_haze_color.unwrap_or(ray_color);
    let absorption_color = params.atmo_absorption_color.unwrap_or([255, 170, 110]);
    let halo_strength = (params.atmo_density
        * (0.18
            + 0.46 * params.atmo_rayleigh_amount.clamp(0.0, 1.0)
            + 0.36 * params.atmo_haze_amount.clamp(0.0, 1.0))
        * params.atmo_limb_boost.max(0.0))
    .clamp(0.0, 0.98);
    let halo_width = (0.02
        + params.atmo_height * (0.58 + 1.05 * params.atmo_haze_amount.clamp(0.0, 1.0)))
    .clamp(0.02, 0.75);
    let halo_power = (2.4 - params.atmo_forward_scatter.clamp(0.0, 1.0) * 1.1
        + (1.0 - params.atmo_haze_amount.clamp(0.0, 1.0)) * 0.35)
        .clamp(0.55, 4.0);
    let light_vec = [
        params.light_direction_x,
        params.light_direction_y,
        params.light_direction_z,
    ];
    let light_mag =
        (light_vec[0] * light_vec[0] + light_vec[1] * light_vec[1] + light_vec[2] * light_vec[2])
            .sqrt()
            .clamp(0.0, 4.0);
    let light_dir = if light_mag > 1e-5 {
        [
            light_vec[0] / light_mag,
            light_vec[1] / light_mag,
            light_vec[2] / light_mag,
        ]
    } else {
        [0.0, -1.0, 0.0]
    };
    let temporal_key = halo_temporal_key_from_obj_params(params);

    let t_halo = Instant::now();
    apply_halo_pass(
        canvas,
        virtual_w,
        virtual_h,
        HaloPassParams {
            ray_color,
            haze_color,
            absorption_color,
            halo_strength,
            halo_width,
            halo_power,
            rayleigh_amount: params.atmo_rayleigh_amount,
            haze_amount: params.atmo_haze_amount,
            absorption_amount: params.atmo_absorption_amount,
            forward_scatter: params.atmo_forward_scatter,
            haze_night_leak: params.atmo_haze_night_leak,
            night_glow: params.atmo_night_glow,
            night_glow_color: params.atmo_night_glow_color.unwrap_or([90, 130, 255]),
            light_intensity: light_mag,
            light_dir,
            view_right: [
                params.view_right_x,
                params.view_right_y,
                params.view_right_z,
            ],
            view_up: [params.view_up_x, params.view_up_y, params.view_up_z],
            temporal_key,
        },
    );
    t_halo.elapsed().as_micros() as f32
}

#[allow(clippy::too_many_lines)]
pub fn apply_halo_pass(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    params: HaloPassParams,
) {
    if params.halo_strength <= 0.0 || params.halo_width <= 0.0 {
        return;
    }

    let w = virtual_w as usize;
    let h = virtual_h as usize;
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut count = 0usize;
    let mut edge_pixels = HALO_EDGE_PIXELS.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken
    });
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    for y in 0..h {
        for x in 0..w {
            if canvas[y * w + x].is_none() {
                continue;
            }
            sum_x += x as f32;
            sum_y += y as f32;
            count += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);

            let left_empty = x == 0 || canvas[y * w + (x - 1)].is_none();
            let right_empty = x + 1 >= w || canvas[y * w + (x + 1)].is_none();
            let up_empty = y == 0 || canvas[(y - 1) * w + x].is_none();
            let down_empty = y + 1 >= h || canvas[(y + 1) * w + x].is_none();
            if left_empty || right_empty || up_empty || down_empty {
                edge_pixels.push((x as i32, y as i32));
            }
        }
    }
    if count == 0 || edge_pixels.is_empty() {
        HALO_TEMPORAL_CACHE.with(|cell| *cell.borrow_mut() = None);
        HALO_EDGE_PIXELS.with(|v| *v.borrow_mut() = edge_pixels);
        return;
    }

    let cx = sum_x / count as f32;
    let cy = sum_y / count as f32;
    let bbox_radius_x = ((max_x.saturating_sub(min_x) + 1) as f32) * 0.5;
    let bbox_radius_y = ((max_y.saturating_sub(min_y) + 1) as f32) * 0.5;
    let area_radius = (count as f32 / std::f32::consts::PI).sqrt();
    let radius = bbox_radius_x.max(bbox_radius_y).max(area_radius).max(1.0);
    let halo_px = (radius * params.halo_width.clamp(0.0, 1.0)).max(1.0);
    let halo_px_sq = halo_px * halo_px;
    let search = halo_px.ceil() as i32;
    let scan_min_x = min_x.saturating_sub(search as usize);
    let scan_min_y = min_y.saturating_sub(search as usize);
    let scan_max_x = (max_x + search as usize).min(w.saturating_sub(1));
    let scan_max_y = (max_y + search as usize).min(h.saturating_sub(1));
    let edge_count = edge_pixels.len();

    let mut material_key = 0xA24B_6F13_91D4_EE99u64;
    mix_hash(&mut material_key, params.ray_color[0] as i64);
    mix_hash(&mut material_key, params.ray_color[1] as i64);
    mix_hash(&mut material_key, params.ray_color[2] as i64);
    mix_hash(&mut material_key, params.haze_color[0] as i64);
    mix_hash(&mut material_key, params.haze_color[1] as i64);
    mix_hash(&mut material_key, params.haze_color[2] as i64);
    mix_hash(&mut material_key, params.absorption_color[0] as i64);
    mix_hash(&mut material_key, params.absorption_color[1] as i64);
    mix_hash(&mut material_key, params.absorption_color[2] as i64);
    mix_hash(
        &mut material_key,
        quantize_f32(params.halo_strength, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.halo_width, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.halo_power, 0.03) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.rayleigh_amount, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.haze_amount, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.absorption_amount, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.forward_scatter, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.haze_night_leak, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.night_glow, 0.01) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.light_intensity, 0.05) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.light_dir[0], 0.04) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.light_dir[1], 0.04) as i64,
    );
    mix_hash(
        &mut material_key,
        quantize_f32(params.light_dir[2], 0.04) as i64,
    );

    let should_try_temporal_reuse = edge_count >= 1200;
    if should_try_temporal_reuse {
        let mut reused = false;
        HALO_TEMPORAL_CACHE.with(|cell| {
            let slot = cell.borrow();
            let Some(cache) = slot.as_ref() else {
                return;
            };
            if cache.virtual_w != virtual_w || cache.virtual_h != virtual_h {
                return;
            }
            if cache.temporal_key != params.temporal_key || cache.material_key != material_key {
                return;
            }
            if (cache.center_x - cx).abs() > 0.75
                || (cache.center_y - cy).abs() > 0.75
                || (cache.radius - radius).abs() > 0.75
            {
                return;
            }
            let edge_delta = cache.edge_count.abs_diff(edge_count);
            let edge_tolerance = (edge_count / 6).max(24);
            if edge_delta > edge_tolerance {
                return;
            }
            for &(idx, color) in &cache.halo_pixels {
                let i = idx as usize;
                if i < canvas.len() && canvas[i].is_none() {
                    canvas[i] = Some(color);
                }
            }
            reused = true;
        });
        if reused {
            HALO_EDGE_PIXELS.with(|v| *v.borrow_mut() = edge_pixels);
            return;
        }
    }

    let sx = dot3(params.light_dir, params.view_right);
    let sy = dot3(params.light_dir, params.view_up);
    let sl = (sx * sx + sy * sy).sqrt();
    let sun2d = if sl > 1e-5 {
        [sx / sl, -sy / sl]
    } else {
        [0.0, -1.0]
    };
    let haze_white = mix_rgb([255, 252, 246], params.haze_color, 0.20);
    let bright_ring = mix_rgb([255, 255, 252], haze_white, 0.42);
    let ray_tint = mix_rgb(haze_white, params.ray_color, 0.74);
    let sunset_tint = mix_rgb([255, 214, 156], params.absorption_color, 0.65);
    let haze_amount = params.haze_amount.clamp(0.0, 1.0);
    let rayleigh_amount = params.rayleigh_amount.clamp(0.0, 1.0);
    let absorption_amount = params.absorption_amount.clamp(0.0, 1.0);
    let forward_scatter = params.forward_scatter.clamp(0.0, 1.0);
    let haze_night_leak = params.haze_night_leak.clamp(0.0, 1.0);
    let night_glow = params.night_glow.clamp(0.0, 1.0);
    let light_intensity = params.light_intensity.clamp(0.0, 4.0);
    let day_gain = light_intensity.clamp(0.0, 1.6);

    const CORE_LUT_SIZE: usize = 256;
    const TWILIGHT_LUT_SIZE: usize = 512;
    const DIST_LUT_SIZE: usize = 512;
    let core_center = 0.08 + 0.04 * (1.0 - haze_amount);
    let core_width = 0.08 + params.halo_width * 0.10;
    let twilight_width = 0.28 + 0.30 * (1.0 - forward_scatter);
    let skirt_exp = (params.halo_power * 0.55).max(0.3);
    let mut core_lut = [0.0f32; CORE_LUT_SIZE];
    let mut twilight_lut = [0.0f32; TWILIGHT_LUT_SIZE];
    let mut dist01_lut = [0.0f32; DIST_LUT_SIZE];
    let mut skirt_lut = [0.0f32; DIST_LUT_SIZE];
    for (i, slot) in core_lut.iter_mut().enumerate() {
        let t = i as f32 / (CORE_LUT_SIZE - 1) as f32;
        *slot = gaussian(t, core_center, core_width);
    }
    for (i, slot) in twilight_lut.iter_mut().enumerate() {
        let t = i as f32 / (TWILIGHT_LUT_SIZE - 1) as f32;
        let sun_alignment = t * 2.0 - 1.0;
        *slot = gaussian(sun_alignment, 0.0, twilight_width);
    }
    for i in 0..DIST_LUT_SIZE {
        let q = i as f32 / (DIST_LUT_SIZE - 1) as f32;
        let dist01 = q.sqrt();
        dist01_lut[i] = dist01;
        skirt_lut[i] = (1.0 - dist01).clamp(0.0, 1.0).powf(skirt_exp);
    }

    let scan_w = scan_max_x.saturating_sub(scan_min_x) + 1;
    let scan_h = scan_max_y.saturating_sub(scan_min_y) + 1;
    let scan_size = scan_w * scan_h;

    let mut occupied_scan = HALO_OCCUPIED_SCAN.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.resize(scan_size, 0);
        taken
    });
    for y in scan_min_y..=scan_max_y {
        let row_offset = (y - scan_min_y) * scan_w;
        let canvas_row = y * w;
        for x in scan_min_x..=scan_max_x {
            if canvas[canvas_row + x].is_some() {
                occupied_scan[row_offset + (x - scan_min_x)] = 1;
            }
        }
    }

    let mut nearest_sq = HALO_NEAREST_SQ.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.resize(scan_size, f32::INFINITY);
        taken
    });
    let mut halo_pixels = HALO_TEMPORAL_CACHE.with(|cell| {
        let mut slot = cell.borrow_mut();
        if let Some(mut cache) = slot.take() {
            cache.halo_pixels.clear();
            cache.halo_pixels
        } else {
            Vec::new()
        }
    });

    let edge_stride = if edge_pixels.len() > 7000 {
        6
    } else if edge_pixels.len() > 5000 {
        5
    } else if edge_pixels.len() > 3200 {
        4
    } else if edge_pixels.len() > 1800 {
        3
    } else if edge_pixels.len() > 900 {
        2
    } else {
        1
    };

    for &(ex, ey) in edge_pixels.iter().step_by(edge_stride) {
        let local_min_x = ((ex - search).max(scan_min_x as i32)) as usize;
        let local_max_x = ((ex + search).min(scan_max_x as i32)) as usize;
        let local_min_y = ((ey - search).max(scan_min_y as i32)) as usize;
        let local_max_y = ((ey + search).min(scan_max_y as i32)) as usize;
        for y in local_min_y..=local_max_y {
            let dy = y as i32 - ey;
            let row_offset = (y - scan_min_y) * scan_w;
            for x in local_min_x..=local_max_x {
                let local_idx = row_offset + (x - scan_min_x);
                if occupied_scan[local_idx] != 0 {
                    continue;
                }
                let dx = x as i32 - ex;
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > halo_px_sq {
                    continue;
                }
                if dist_sq < nearest_sq[local_idx] {
                    nearest_sq[local_idx] = dist_sq;
                }
            }
        }
    }

    let radial_limit_sq = (radius + halo_px + 1.5).powi(2);
    for y in scan_min_y..=scan_max_y {
        let row_offset = (y - scan_min_y) * scan_w;
        for x in scan_min_x..=scan_max_x {
            if occupied_scan[row_offset + (x - scan_min_x)] != 0 {
                continue;
            }
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dl_sq = dx * dx + dy * dy;
            if dl_sq > radial_limit_sq {
                continue;
            }
            let nearest_sq = nearest_sq[row_offset + (x - scan_min_x)];
            if !nearest_sq.is_finite() || nearest_sq > halo_px_sq {
                continue;
            }

            let dl = dl_sq.sqrt().max(1e-5);
            let edge_dir = [dx / dl, dy / dl];
            let sun_alignment = edge_dir[0] * sun2d[0] + edge_dir[1] * sun2d[1];
            let day = smoothstep(-0.18, 0.92, sun_alignment);
            let q = (nearest_sq / halo_px_sq).clamp(0.0, 1.0);
            let dist_idx = ((q * (DIST_LUT_SIZE - 1) as f32) as usize).min(DIST_LUT_SIZE - 1);
            let dist01 = dist01_lut[dist_idx];
            let skirt = skirt_lut[dist_idx];
            let core_idx = ((dist01 * (CORE_LUT_SIZE - 1) as f32) as usize).min(CORE_LUT_SIZE - 1);
            let core_ring = core_lut[core_idx];
            let night = (1.0 - day).clamp(0.0, 1.0);
            let lit_visibility = (day * day_gain)
                .clamp(0.0, 1.0)
                .max(night * haze_night_leak);
            let shadowed_visibility = lit_visibility.powf(1.12);
            let wide_scatter = skirt * (0.04 + 0.96 * shadowed_visibility);
            let forward_lobe = skirt.powf(0.52)
                * smoothstep(0.10, 1.0, sun_alignment).powf(1.8)
                * forward_scatter
                * day_gain.min(1.0);
            let twilight_t = ((sun_alignment + 1.0) * 0.5).clamp(0.0, 1.0);
            let twilight_idx =
                ((twilight_t * (TWILIGHT_LUT_SIZE - 1) as f32) as usize).min(TWILIGHT_LUT_SIZE - 1);
            let twilight_arc = twilight_lut[twilight_idx];
            let haze_alpha = (params.halo_strength
                * (0.12 + 0.88 * haze_amount)
                * (0.06 + 0.94 * lit_visibility)
                * (core_ring * (0.22 + 0.78 * shadowed_visibility + 0.35 * forward_lobe)
                    + wide_scatter * 0.14))
                .clamp(0.0, 0.97);
            let ray_alpha = (params.halo_strength
                * (0.10 + 0.90 * rayleigh_amount)
                * (wide_scatter + forward_lobe)
                * (0.05 + 0.95 * shadowed_visibility))
                .clamp(0.0, 0.95);
            let sunset_alpha = (params.halo_strength
                * absorption_amount
                * twilight_arc
                * (0.10 + 0.90 * skirt)
                * (0.08 + 0.92 * day + 0.12 * forward_lobe))
                .clamp(0.0, 0.78);
            let night_glow_alpha = (params.halo_strength
                * night_glow
                * night
                * (0.08 + 0.64 * skirt)
                * (0.08 + 0.52 * haze_amount))
                .clamp(0.0, 0.45);
            if haze_alpha <= 0.01
                && ray_alpha <= 0.01
                && sunset_alpha <= 0.01
                && night_glow_alpha <= 0.01
            {
                continue;
            }

            let mut out = [0, 0, 0];
            if haze_alpha > 0.0 {
                out = mix_rgb(out, bright_ring, haze_alpha);
            }
            if ray_alpha > 0.0 {
                out = mix_rgb(out, ray_tint, ray_alpha);
            }
            if sunset_alpha > 0.0 {
                out = mix_rgb(out, sunset_tint, sunset_alpha);
            }
            if night_glow_alpha > 0.0 {
                out = mix_rgb(out, params.night_glow_color, night_glow_alpha);
            }
            let idx = y * w + x;
            canvas[idx] = Some(out);
            halo_pixels.push((idx as u32, out));
        }
    }

    HALO_TEMPORAL_CACHE.with(|cell| {
        *cell.borrow_mut() = Some(HaloTemporalCache {
            virtual_w,
            virtual_h,
            temporal_key: params.temporal_key,
            material_key,
            center_x: cx,
            center_y: cy,
            radius,
            edge_count,
            halo_pixels,
        });
    });
    HALO_EDGE_PIXELS.with(|v| *v.borrow_mut() = edge_pixels);
    HALO_OCCUPIED_SCAN.with(|v| *v.borrow_mut() = occupied_scan);
    HALO_NEAREST_SQ.with(|v| *v.borrow_mut() = nearest_sq);
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn gaussian(x: f32, center: f32, width: f32) -> f32 {
    let w = width.max(0.001);
    let z = (x - center) / w;
    (-0.5 * z * z).exp()
}
