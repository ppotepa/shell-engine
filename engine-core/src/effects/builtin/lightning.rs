use crate::buffer::{Buffer, TRUE_BLACK};
use crate::color::Color;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{
    select, text, EffectMetadata, P_EASING, P_GLOW, P_INTENSITY, P_OCTAVES, P_ORIENTATION, P_SPEED,
    P_STRIKES, P_THICKNESS,
};
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use std::f32::consts::TAU;

pub static METADATA_LIGHTNING_FLASH: EffectMetadata = EffectMetadata {
    name: "lightning-flash",
    display_name: "Lightning Flash",
    summary: "Short global lightning flash with a glow peak.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        P_INTENSITY,
        select(
            "coverage",
            "Coverage",
            "Scope hint for the flash.",
            &["full-screen", "region"],
            "full-screen",
        ),
        P_ORIENTATION,
        P_EASING,
    ],
    sample: "- name: lightning-flash\n  duration: 260\n  params:\n    intensity: 1.0",
};

pub static METADATA_LIGHTNING_BRANCH: EffectMetadata = EffectMetadata {
    name: "lightning-branch",
    display_name: "Lightning Branch",
    summary: "Procedural forked bolt between start/end anchors.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        P_STRIKES,
        P_THICKNESS,
        P_GLOW,
        P_SPEED,
        P_ORIENTATION,
        text("start_x", "Start X", "Start anchor (number or \"random\").", "random"),
        text("end_x", "End X", "End anchor (number or \"random\").", "random"),
        P_EASING,
    ],
    sample: "- name: lightning-branch\n  duration: 720\n  params:\n    strikes: 3\n    glow: true\n    start_x: random\n    end_x: random",
};

pub static METADATA_LIGHTNING_FBM: EffectMetadata = EffectMetadata {
    name: "lightning-fbm",
    display_name: "Lightning FBM",
    summary: "Fractal Brownian Motion plasma bolt.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[P_INTENSITY, P_OCTAVES, P_SPEED, P_ORIENTATION, P_EASING],
    sample:
        "- name: lightning-fbm\n  duration: 800\n  params:\n    intensity: 1.0\n    octave_count: 4",
};

pub static METADATA_LIGHTNING_OPTICAL_80S: EffectMetadata = EffectMetadata {
    name: "lightning-optical-80s",
    display_name: "Lightning 80s",
    summary: "Retro 80s neon-style optical lightning.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[P_INTENSITY, P_SPEED, P_EASING],
    sample: "- name: lightning-optical-80s\n  duration: 600\n  params:\n    intensity: 1.0",
};

pub static METADATA_LIGHTNING_GROWTH: EffectMetadata = EffectMetadata {
    name: "lightning-growth",
    display_name: "Lightning Growth",
    summary: "Slow-growing branched bolt spreading outward.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        P_STRIKES,
        P_THICKNESS,
        P_SPEED,
        P_GLOW,
        P_ORIENTATION,
        P_EASING,
    ],
    sample: "- name: lightning-growth\n  duration: 1200\n  params:\n    strikes: 4\n    glow: true",
};

pub static METADATA_LIGHTNING_AMBIENT: EffectMetadata = EffectMetadata {
    name: "lightning-ambient",
    display_name: "Lightning Ambient",
    summary: "Looping ambient electric atmosphere.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        P_INTENSITY,
        P_STRIKES,
        P_THICKNESS,
        P_SPEED,
        P_GLOW,
        P_ORIENTATION,
    ],
    sample:
        "- name: lightning-ambient\n  duration: 2000\n  loop: true\n  params:\n    intensity: 0.7",
};

pub static METADATA_LIGHTNING_NATURAL: EffectMetadata = EffectMetadata {
    name: "lightning-natural",
    display_name: "Lightning Natural",
    summary: "Naturalistic bolt with secondary arcs.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        P_INTENSITY,
        P_STRIKES,
        P_THICKNESS,
        P_SPEED,
        P_GLOW,
        P_ORIENTATION,
        P_EASING,
    ],
    sample: "- name: lightning-natural\n  duration: 900\n  params:\n    strikes: 2\n    glow: true",
};

pub static METADATA_TESLA_ORB: EffectMetadata = EffectMetadata {
    name: "tesla-orb",
    display_name: "Tesla Orb",
    summary: "Orbital electric arcs around a plasma core.",
    category: "lightning",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[P_INTENSITY, P_SPEED, P_OCTAVES, P_STRIKES, P_EASING],
    sample: "- name: tesla-orb\n  duration: 1000\n  loop: true\n  params:\n    speed: 1.0\n    intensity: 0.9",
};

fn n01(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

#[inline]
fn fract(v: f32) -> f32 {
    v - v.floor()
}

#[inline]
fn hash12f(p: (f32, f32)) -> f32 {
    fract(((p.0 * 13.9898 + p.1 * 8.141).cos()) * 43_758.547)
}

#[inline]
fn hash22f(p: (f32, f32)) -> (f32, f32) {
    // Inputs from noise2 grid corners are always integer-valued floats (floor results).
    // Replace expensive sin()/cos() with PCG-style integer bit mixing — ~25x faster.
    // Adds nonzero primes to avoid the (0,0) degenerate case.
    let ix = p.0 as i32 as u32;
    let iy = p.1 as i32 as u32;

    let mut h1 = ix.wrapping_add(2_654_435_761).wrapping_mul(374_761_393);
    h1 ^= iy.wrapping_mul(668_265_263);
    h1 ^= h1 >> 15;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;

    let mut h2 = iy.wrapping_add(2_246_822_519).wrapping_mul(374_761_393);
    h2 ^= ix.wrapping_mul(668_265_263);
    h2 ^= h2 >> 15;
    h2 = h2.wrapping_mul(0x85ebca6b);
    h2 ^= h2 >> 13;
    h2 = h2.wrapping_mul(0xc2b2ae35);
    h2 ^= h2 >> 16;

    let x = (h1 as i32 as f32) * (1.0 / 2_147_483_648.0);
    let y = (h2 as i32 as f32) * (1.0 / 2_147_483_648.0);
    (x, y)
}

#[inline]
fn dot2(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn noise2(uv: (f32, f32)) -> f32 {
    let iuv = (uv.0.floor(), uv.1.floor());
    let fuv = (fract(uv.0), fract(uv.1));
    let blur = (
        fuv.0 * fuv.0 * (3.0 - 2.0 * fuv.0),
        fuv.1 * fuv.1 * (3.0 - 2.0 * fuv.1),
    );

    let n00 = dot2(
        hash22f((iuv.0 + 0.0, iuv.1 + 0.0)),
        (fuv.0 - 0.0, fuv.1 - 0.0),
    );
    let n10 = dot2(
        hash22f((iuv.0 + 1.0, iuv.1 + 0.0)),
        (fuv.0 - 1.0, fuv.1 - 0.0),
    );
    let n01 = dot2(
        hash22f((iuv.0 + 0.0, iuv.1 + 1.0)),
        (fuv.0 - 0.0, fuv.1 - 1.0),
    );
    let n11 = dot2(
        hash22f((iuv.0 + 1.0, iuv.1 + 1.0)),
        (fuv.0 - 1.0, fuv.1 - 1.0),
    );

    let nx0 = n00 + (n10 - n00) * blur.0;
    let nx1 = n01 + (n11 - n01) * blur.0;
    (nx0 + (nx1 - nx0) * blur.1) + 0.5
}

fn fbm2(mut uv: (f32, f32), octaves: u8, amp_start: f32, amp_coeff: f32, freq_coeff: f32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = amp_start;
    for _ in 0..octaves {
        value += amplitude * noise2(uv);
        uv = (uv.0 * freq_coeff, uv.1 * freq_coeff);
        amplitude *= amp_coeff;
    }
    value
}

#[inline]
fn smoothstep01(v: f32) -> f32 {
    let t = v.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn band_hash(region: Region, lane: u32, salt: u32) -> f32 {
    crt_hash(
        region.x.wrapping_add((lane as u16).wrapping_mul(17)),
        region.y.wrapping_add((lane as u16).wrapping_mul(29)),
        salt ^ ((region.width as u32) << 16) ^ region.height as u32,
    ) as f32
        / u32::MAX as f32
}

fn ambient_pulse_descriptor(region: Region, pulse_idx: usize, pulse_count: usize) -> (f32, f32) {
    let segment = 1.0 / (pulse_count as f32 + 1.0);
    let lane = pulse_idx as u32 + 1;
    let center_jitter = (band_hash(region, lane, 0xA11D_1E) - 0.5) * segment * 0.55;
    let center = ((pulse_idx as f32 + 1.0) * segment + center_jitter).clamp(0.06, 0.94);
    let width = 0.035 + band_hash(region, lane, 0xA11D_EF) * 0.03;
    (center, width)
}

fn ambient_pulse_envelope(progress: f32, center: f32, width: f32) -> f32 {
    let dist = (progress - center).abs();
    if dist >= width {
        return 0.0;
    }

    smoothstep01(1.0 - dist / width)
}

#[inline]
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn parse_anchor(anchor: Option<&str>, w: u16, seed: u32) -> f32 {
    if w == 0 {
        return 0.0;
    }
    let max_x = (w - 1) as f32;
    match anchor.map(|s| s.trim().to_ascii_lowercase()) {
        Some(v) if v == "random" => (seed as f32 / u32::MAX as f32) * max_x,
        Some(v) => v
            .parse::<f32>()
            .map(|x| x.clamp(0.0, max_x))
            .unwrap_or(max_x * 0.5),
        None => max_x * 0.5,
    }
}

fn in_region_i32(x: i32, y: i32, region: Region) -> bool {
    x >= region.x as i32
        && y >= region.y as i32
        && x < (region.x + region.width) as i32
        && y < (region.y + region.height) as i32
}

fn in_region_i32_x(x: i32, region: Region) -> bool {
    x >= region.x as i32 && x < (region.x + region.width) as i32
}

fn in_region_u16_x(x: u16, region: Region) -> bool {
    x >= region.x && x < region.x + region.width
}

fn apply_to_neighborhood_3x3<F>(region: Region, cx: u16, cy: u16, mut f: F)
where
    F: FnMut(u16, u16),
{
    for gy in cy.saturating_sub(1)..=cy.saturating_add(1) {
        if gy < region.y || gy >= region.y + region.height {
            continue;
        }
        for gx in cx.saturating_sub(1)..=cx.saturating_add(1) {
            if gx < region.x || gx >= region.x + region.width {
                continue;
            }
            f(gx, gy);
        }
    }
}

fn get_effect_color(params: &EffectParams, default_r: u8, default_g: u8, default_b: u8) -> Color {
    params
        .colour
        .as_ref()
        .map(Color::from)
        .unwrap_or(Color::Rgb {
            r: default_r,
            g: default_g,
            b: default_b,
        })
}

#[derive(Clone, Copy)]
enum LightningOrientation {
    Vertical,
    Horizontal,
}

impl LightningOrientation {
    fn from_params(params: &EffectParams) -> Self {
        match params
            .orientation
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("horizontal") => Self::Horizontal,
            _ => Self::Vertical,
        }
    }
}

fn brighten_cell(buffer: &mut Buffer, x: u16, y: u16, amount: f32) {
    if let Some(cell) = buffer.get(x, y).cloned() {
        let a = amount.clamp(0.0, 1.0);
        let fg = lerp_colour(cell.fg, Color::White, a);
        let bg = lerp_colour(cell.bg, Color::White, (a * 0.85).clamp(0.0, 1.0));
        buffer.set(x, y, cell.symbol, fg, bg);
    }
}

fn bolt_symbol(intensity: f32) -> char {
    let v = intensity.clamp(0.0, 1.0);
    if v > 0.82 {
        '█'
    } else if v > 0.52 {
        '▓'
    } else if v > 0.25 {
        '▒'
    } else {
        '░'
    }
}

fn apply_glow(buffer: &mut Buffer, region: Region, x: u16, y: u16, decay: f32) {
    apply_to_neighborhood_3x3(region, x, y, |gx, gy| {
        if let Some(cell) = buffer.get(gx, gy).cloned() {
            let fg = lerp_colour(cell.fg, Color::White, 0.28 * decay);
            let sym = if cell.symbol == ' ' {
                '░'
            } else {
                cell.symbol
            };
            buffer.set(gx, gy, sym, fg, cell.bg);
        }
    });
}

fn blend_cell_to(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    to: Color,
    amount: f32,
    symbol: Option<char>,
    fallback_bg: Color,
) {
    let a = amount.clamp(0.0, 1.0);
    if let Some(cell) = buffer.get(x, y).cloned() {
        let fg = lerp_colour(cell.fg, to, a);
        let ch = symbol.unwrap_or(cell.symbol);
        buffer.set(x, y, ch, fg, cell.bg);
    } else {
        buffer.set(x, y, symbol.unwrap_or(' '), to, fallback_bg);
    }
}

fn render_growth_core(
    buffer: &mut Buffer,
    region: Region,
    x: i32,
    y: i32,
    thickness: f32,
    amount: f32,
    effect_color: Color,
    glow: bool,
) {
    if !in_region_i32(x, y, region) {
        return;
    }

    let xi = x as u16;
    let yi = y as u16;
    let core_radius = thickness.ceil() as i32;
    let halo_radius = (thickness * 2.6).ceil() as i32;

    for oy in -halo_radius..=halo_radius {
        for ox in -halo_radius..=halo_radius {
            let xx = x + ox;
            let yy = y + oy;
            if !in_region_i32(xx, yy, region) {
                continue;
            }

            let dist = ((ox * ox + oy * oy) as f32).sqrt();
            let core = (1.0 - dist / (0.35 + core_radius as f32))
                .clamp(0.0, 1.0)
                .powf(1.65);
            let halo = (1.0 - dist / (0.8 + halo_radius as f32))
                .clamp(0.0, 1.0)
                .powf(2.4);
            let local = (core * 0.84 + halo * 0.22) * amount;
            if local <= 0.02 {
                continue;
            }

            let symbol = if core > 0.68 {
                Some('█')
            } else if core > 0.34 {
                Some('▓')
            } else {
                Some('▒')
            };
            let tint = if core > 0.4 {
                Color::White
            } else {
                effect_color
            };
            blend_cell_to(
                buffer,
                xx as u16,
                yy as u16,
                tint,
                local.clamp(0.0, 1.0),
                symbol,
                TRUE_BLACK,
            );
        }
    }

    if glow {
        apply_glow(buffer, region, xi, yi, amount * 0.9);
    }
}

/// Short burst flash with multi-pulse behavior (closer to natural lightning).
pub struct LightningFlashEffect;

impl Effect for LightningFlashEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let intensity = params.intensity.unwrap_or(2.4).max(0.0);
        let frame = (p * 10_000.0) as u32;

        let pulse1 = (-((p - 0.16) * 14.0).powi(2)).exp();
        let pulse2 = (-((p - 0.44) * 17.0).powi(2)).exp() * 0.78;
        let pulse3 = (-((p - 0.79) * 22.0).powi(2)).exp() * 0.55;
        let envelope = (pulse1 + pulse2 + pulse3).clamp(0.0, 1.0);

        for dy in 0..region.height {
            let y = region.y + dy;
            for dx in 0..region.width {
                let x = region.x + dx;
                let grain = 0.82 + n01(x, y, frame) * 0.32;
                let row_bias = 0.88 + (1.0 - dy as f32 / region.height as f32) * 0.18;
                let amount = (envelope * intensity * grain * row_bias).clamp(0.0, 1.0);
                brighten_cell(buffer, x, y, amount);
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_FLASH
    }
}

/// Progressive branching lightning that grows across the frame before the return stroke.
pub struct LightningGrowthEffect;

impl Effect for LightningGrowthEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 12_000.0) as u32;
        let strikes = params.strikes.unwrap_or(2).clamp(1, 4) as usize;
        let thickness = params.thickness.unwrap_or(1.0).clamp(0.45, 2.8);
        let glow = params.glow.unwrap_or(true);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.1, 2.5);
        let octaves = params.octave_count.unwrap_or(6).clamp(2, 12);
        let amp_start = params.amp_start.unwrap_or(0.45).clamp(0.05, 1.0);
        let amp_coeff = params.amp_coeff.unwrap_or(0.5).clamp(0.1, 1.0);
        let freq_coeff = params.freq_coeff.unwrap_or(2.0).clamp(1.0, 5.0);
        let speed = params.speed.unwrap_or(0.6).clamp(0.0, 8.0);
        let orientation = LightningOrientation::from_params(params);
        let effect_color = get_effect_color(params, 132, 172, 255);

        let growth_progress = smoothstep01((p / 0.58).clamp(0.0, 1.0));
        let leader_intensity = (0.22 + 0.42 * smoothstep01((p / 0.42).clamp(0.0, 1.0))) * intensity;
        let return_pulse = (-((p - 0.58) * 9.0).powi(2)).exp() * 0.52 * intensity;
        let late_pulse = (-((p - 0.78) * 16.0).powi(2)).exp() * 0.18 * intensity;
        let sustain = if p < 0.72 {
            1.0
        } else {
            (1.0 - ((p - 0.72) / 0.28)).clamp(0.0, 1.0).powf(1.7)
        };
        let envelope = ((leader_intensity + return_pulse + late_pulse) * sustain).clamp(0.0, 1.0);
        if envelope <= 0.02 {
            return;
        }

        let (axis_len, cross_len) = match orientation {
            LightningOrientation::Vertical => (region.height.max(1), region.width.max(1)),
            LightningOrientation::Horizontal => (region.width.max(1), region.height.max(1)),
        };
        let grown_steps = ((axis_len as f32) * growth_progress)
            .ceil()
            .clamp(1.0, axis_len as f32) as usize;

        for s in 0..strikes {
            let seed = frame.wrapping_add((s as u32).wrapping_mul(117_223));
            let start = parse_anchor(
                params.start_x.as_deref(),
                cross_len,
                crt_hash(region.x.wrapping_add(s as u16), region.y, seed),
            );
            let end = parse_anchor(
                params.end_x.as_deref(),
                cross_len,
                crt_hash(
                    region.x,
                    region.y.wrapping_add(s as u16),
                    seed.wrapping_add(41),
                ),
            );

            let mut axis = start;
            let mut main_points: Vec<(i32, i32, f32)> = Vec::with_capacity(grown_steps);

            for idx in 0..grown_steps {
                let t = idx as f32 / axis_len.saturating_sub(1).max(1) as f32;
                let base_axis = lerp_f32(start, end, t);
                let fbm = fbm2(
                    (
                        t * 4.6 + p * speed * 2.2 + s as f32 * 0.31,
                        p * speed * 1.4 + t * 0.9 + s as f32 * 0.17,
                    ),
                    octaves,
                    amp_start,
                    amp_coeff,
                    freq_coeff,
                ) - 0.5;
                let side_energy = 1.0 - (t * 2.0 - 1.0).abs();
                let stepped = (n01(region.x.wrapping_add(idx as u16), region.y, seed) - 0.5)
                    * 2.0
                    * thickness
                    * (0.7 + side_energy * 0.9);
                axis += (base_axis + fbm * thickness * 4.4 - axis) * 0.52 + stepped * 0.62;
                axis = axis.clamp(0.0, cross_len.saturating_sub(1) as f32);

                let head_factor = if idx + 3 >= grown_steps { 0.86 } else { 1.0 };
                let local = (envelope
                    * (0.82
                        + n01(
                            region.x,
                            region.y.wrapping_add(idx as u16),
                            seed.wrapping_add(313),
                        ) * 0.18)
                    * head_factor)
                    .clamp(0.0, 1.0);

                let (x, y) = match orientation {
                    LightningOrientation::Vertical => (
                        region.x as i32 + axis.round() as i32,
                        region.y as i32 + idx as i32,
                    ),
                    LightningOrientation::Horizontal => (
                        region.x as i32 + idx as i32,
                        region.y as i32 + axis.round() as i32,
                    ),
                };
                render_growth_core(buffer, region, x, y, thickness, local, effect_color, glow);
                main_points.push((x, y, t));
            }

            for (idx, &(x, y, birth_t)) in main_points.iter().enumerate() {
                let gate = n01(
                    (x.max(0) as u16).wrapping_add(s as u16),
                    (y.max(0) as u16).wrapping_add(idx as u16),
                    seed.wrapping_add(911),
                );
                if gate <= 0.84 || idx < 2 || idx + 3 >= grown_steps {
                    continue;
                }

                let branch_progress =
                    ((growth_progress - birth_t) / (1.0 - birth_t).max(0.15)).clamp(0.0, 1.0);
                if branch_progress <= 0.08 {
                    continue;
                }

                let direction = if gate > 0.93 { 1.0 } else { -1.0 };
                let max_len = 2 + ((gate * 6.0) as usize);
                let visible_len = ((max_len as f32) * smoothstep01(branch_progress))
                    .ceil()
                    .max(1.0) as usize;

                for bi in 1..=visible_len.min(max_len) {
                    let bt = bi as f32 / max_len.max(1) as f32;
                    let branch_noise = fbm2(
                        (
                            birth_t * 6.0 + bt * 2.6 + s as f32 * 0.19,
                            p * speed * 1.6 + bt * 1.7 + s as f32 * 0.37,
                        ),
                        4,
                        0.34,
                        0.55,
                        2.0,
                    ) - 0.5;

                    let (bx, by) = match orientation {
                        LightningOrientation::Vertical => (
                            x + (direction * bi as f32 + branch_noise * 1.6).round() as i32,
                            y + ((bi as f32 * 0.55) + branch_noise.abs() * 0.8).round() as i32,
                        ),
                        LightningOrientation::Horizontal => (
                            x + ((bi as f32 * 0.55) + branch_noise.abs() * 0.8).round() as i32,
                            y + (direction * bi as f32 + branch_noise * 1.6).round() as i32,
                        ),
                    };

                    let local = (envelope
                        * (0.62 - bt * 0.34)
                        * (0.86 + gate * 0.14)
                        * (0.82 + branch_progress * 0.18))
                        .clamp(0.0, 1.0);
                    render_growth_core(
                        buffer,
                        region,
                        bx,
                        by,
                        (thickness * (0.52 - bt * 0.18)).clamp(0.22, 1.2),
                        local,
                        effect_color,
                        glow && bt < 0.75,
                    );
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_GROWTH
    }
}

/// Draws jagged lightning bolts with small branches and optional glow.
pub struct LightningBranchEffect;

impl Effect for LightningBranchEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 10_000.0) as u32;
        let strikes = params.strikes.unwrap_or(2).clamp(1, 6) as usize;
        let thickness = params.thickness.unwrap_or(1.0).clamp(1.0, 3.0);
        let glow = params.glow.unwrap_or(true);
        let decay = (1.0 - p * 0.45).clamp(0.25, 1.0);
        let orientation = LightningOrientation::from_params(params);

        let h = region.height.max(1);
        let w = region.width.max(1);
        for s in 0..strikes {
            let seed_base = frame.wrapping_add((s as u32).wrapping_mul(101_903));
            match orientation {
                LightningOrientation::Vertical => {
                    let start = parse_anchor(
                        params.start_x.as_deref(),
                        w,
                        crt_hash(region.x.wrapping_add(s as u16), region.y, seed_base),
                    );
                    let end = parse_anchor(
                        params.end_x.as_deref(),
                        w,
                        crt_hash(
                            region.x,
                            region.y.wrapping_add(s as u16),
                            seed_base.wrapping_add(17),
                        ),
                    );

                    let mut x = start;
                    for iy in 0..h {
                        let y = region.y + iy;
                        let t = iy as f32 / (h.saturating_sub(1).max(1) as f32);
                        let target = start + (end - start) * t;
                        let jitter =
                            (n01(region.x + (s as u16), y, seed_base) - 0.5) * 2.0 * thickness;
                        x += (target - x) * 0.38 + jitter * 0.85;
                        x = x.clamp(0.0, (w - 1) as f32);

                        let abs_x = region.x + x.round() as u16;
                        let core_intensity =
                            (0.82 + 0.18 * n01(abs_x, y, seed_base.wrapping_add(333))) * decay;

                        let mut bolt_fg = Color::White;
                        if let Some(cell) = buffer.get(abs_x, y) {
                            let (r, g, b) = colour_to_rgb(cell.fg);
                            let r = r.saturating_add((110.0 * decay) as u8);
                            let g = g.saturating_add((110.0 * decay) as u8);
                            let b = b.saturating_add((120.0 * decay) as u8);
                            bolt_fg = Color::Rgb { r, g, b };
                        }
                        let radius = thickness.round() as i32 - 1;
                        for tx in -radius..=radius {
                            let xx = abs_x as i32 + tx;
                            if !in_region_i32_x(xx, region) {
                                continue;
                            }
                            let x_cell = xx as u16;
                            let local_intensity =
                                (core_intensity - (tx.abs() as f32 * 0.22)).clamp(0.08, 1.0);
                            buffer.set(
                                x_cell,
                                y,
                                bolt_symbol(local_intensity),
                                bolt_fg,
                                TRUE_BLACK,
                            );
                        }

                        if glow {
                            apply_glow(buffer, region, abs_x, y, decay);
                        }

                        let branch_n = n01(abs_x, y, seed_base.wrapping_add(911));
                        if branch_n > 0.92 && iy > 2 && iy < h - 2 {
                            let dir: i32 = if branch_n > 0.96 { 1 } else { -1 };
                            let len = 2 + ((branch_n * 4.0) as i32);
                            for bi in 1..=len {
                                let bx_i = abs_x as i32 + dir * bi;
                                let by_i = y as i32 + (bi / 2);
                                if !in_region_i32(bx_i, by_i, region) {
                                    break;
                                }
                                let bx = bx_i as u16;
                                let by = by_i as u16;
                                let bi_decay = (decay - (bi as f32 * 0.08)).clamp(0.15, 1.0);
                                buffer.set(bx, by, bolt_symbol(bi_decay), Color::White, TRUE_BLACK);
                            }
                        }
                    }
                }
                LightningOrientation::Horizontal => {
                    let start = parse_anchor(
                        params.start_x.as_deref(),
                        h,
                        crt_hash(region.x.wrapping_add(s as u16), region.y, seed_base),
                    );
                    let end = parse_anchor(
                        params.end_x.as_deref(),
                        h,
                        crt_hash(
                            region.x,
                            region.y.wrapping_add(s as u16),
                            seed_base.wrapping_add(17),
                        ),
                    );

                    let mut y = start;
                    for ix in 0..w {
                        let x = region.x + ix;
                        let t = ix as f32 / (w.saturating_sub(1).max(1) as f32);
                        let target = start + (end - start) * t;
                        let jitter = (n01(x, region.y.wrapping_add(s as u16), seed_base) - 0.5)
                            * 2.0
                            * thickness;
                        y += (target - y) * 0.38 + jitter * 0.85;
                        y = y.clamp(0.0, (h - 1) as f32);

                        let abs_y = region.y + y.round() as u16;
                        let core_intensity =
                            (0.82 + 0.18 * n01(x, abs_y, seed_base.wrapping_add(333))) * decay;

                        let mut bolt_fg = Color::White;
                        if let Some(cell) = buffer.get(x, abs_y) {
                            let (r, g, b) = colour_to_rgb(cell.fg);
                            let r = r.saturating_add((110.0 * decay) as u8);
                            let g = g.saturating_add((110.0 * decay) as u8);
                            let b = b.saturating_add((120.0 * decay) as u8);
                            bolt_fg = Color::Rgb { r, g, b };
                        }
                        let radius = thickness.round() as i32 - 1;
                        for ty in -radius..=radius {
                            let yy = abs_y as i32 + ty;
                            if yy < region.y as i32 || yy >= (region.y + region.height) as i32 {
                                continue;
                            }
                            let y_cell = yy as u16;
                            let local_intensity =
                                (core_intensity - (ty.abs() as f32 * 0.22)).clamp(0.08, 1.0);
                            buffer.set(
                                x,
                                y_cell,
                                bolt_symbol(local_intensity),
                                bolt_fg,
                                TRUE_BLACK,
                            );
                        }

                        if glow {
                            apply_glow(buffer, region, x, abs_y, decay);
                        }

                        let branch_n = n01(x, abs_y, seed_base.wrapping_add(911));
                        if branch_n > 0.92 && ix > 2 && ix < w - 2 {
                            let dir: i32 = if branch_n > 0.96 { 1 } else { -1 };
                            let len = 2 + ((branch_n * 4.0) as i32);
                            for bi in 1..=len {
                                let bx_i = x as i32 + (bi / 2);
                                let by_i = abs_y as i32 + dir * bi;
                                if !in_region_i32(bx_i, by_i, region) {
                                    break;
                                }
                                let bx = bx_i as u16;
                                let by = by_i as u16;
                                let bi_decay = (decay - (bi as f32 * 0.08)).clamp(0.15, 1.0);
                                buffer.set(bx, by, bolt_symbol(bi_decay), Color::White, TRUE_BLACK);
                            }
                        }
                    }
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_BRANCH
    }
}

/// 80s optical lightning look:
/// - quantized frame jitter (hand-drawn cel feel),
/// - blue halation pass under white core,
/// - occasional ghost double-exposure offset.
pub struct LightningOptical80sEffect;

impl Effect for LightningOptical80sEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 12.0).floor() as u32;
        let strikes = params.strikes.unwrap_or(2).clamp(1, 4) as usize;
        let thickness = params.thickness.unwrap_or(1.2).clamp(1.0, 2.4);
        let decay = (1.0 - p * 0.35).clamp(0.45, 1.0);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.25, 2.0);
        let halo_colour = Color::Rgb {
            r: 80,
            g: 170,
            b: 255,
        };

        let h = region.height.max(1);
        let w = region.width.max(1);
        for s in 0..strikes {
            let seed = frame.wrapping_add((s as u32).wrapping_mul(151_337));
            let start = parse_anchor(
                params.start_x.as_deref(),
                w,
                crt_hash(region.x.wrapping_add(s as u16), region.y, seed),
            );
            let end = parse_anchor(
                params.end_x.as_deref(),
                w,
                crt_hash(
                    region.x,
                    region.y.wrapping_add(s as u16),
                    seed.wrapping_add(19),
                ),
            );

            let mut x = start;
            for iy in 0..h {
                let y = region.y + iy;
                let t = iy as f32 / (h.saturating_sub(1).max(1) as f32);
                let target = start + (end - start) * t;
                let jitter =
                    (n01(region.x.wrapping_add(s as u16), y, seed.wrapping_add(701)) - 0.5) * 2.0;
                x += (target - x) * 0.32 + jitter * (0.75 + thickness * 0.45);
                x = x.clamp(0.0, (w - 1) as f32);
                let abs_x = region.x + x.round() as u16;

                if !in_region_u16_x(abs_x, region) {
                    continue;
                }

                // Blue halation pass
                for gy in y.saturating_sub(1)..=y.saturating_add(1) {
                    if gy < region.y || gy >= region.y + region.height {
                        continue;
                    }
                    for gx in abs_x.saturating_sub(1)..=abs_x.saturating_add(1) {
                        if !in_region_u16_x(gx, region) {
                            continue;
                        }
                        let halation = (0.24 + 0.30 * intensity) * decay;
                        blend_cell_to(buffer, gx, gy, halo_colour, halation, Some('░'), TRUE_BLACK);
                    }
                }

                // White core pass
                let local =
                    ((0.84 + n01(abs_x, y, seed.wrapping_add(313)) * 0.16) * decay * intensity)
                        .clamp(0.0, 1.0);
                let core_ch = bolt_symbol(local);
                blend_cell_to(
                    buffer,
                    abs_x,
                    y,
                    Color::White,
                    (0.65 + 0.35 * local).clamp(0.0, 1.0),
                    Some(core_ch),
                    TRUE_BLACK,
                );
                apply_glow(buffer, region, abs_x, y, decay * 0.9);

                // Ghost offset pass (double exposure)
                let ghost_n = n01(abs_x, y, seed.wrapping_add(991));
                if ghost_n > 0.84 {
                    let dir = if ghost_n > 0.92 { 1_i32 } else { -1_i32 };
                    let gx = abs_x as i32 + dir;
                    if gx >= region.x as i32 && gx < (region.x + region.width) as i32 {
                        blend_cell_to(
                            buffer,
                            gx as u16,
                            y,
                            halo_colour,
                            (0.22 + 0.18 * decay).clamp(0.0, 1.0),
                            Some('▒'),
                            TRUE_BLACK,
                        );
                    }
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_OPTICAL_80S
    }
}

/// Shader-inspired procedural lightning (FBM/noise warp) adapted for terminal cells.
pub struct LightningFbmEffect;

impl Effect for LightningFbmEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let effect_color = get_effect_color(params, 51, 77, 204);
        let octaves = params.octave_count.unwrap_or(10).clamp(1, 20);
        let amp_start = params.amp_start.unwrap_or(0.5).clamp(0.0, 4.0);
        let amp_coeff = params.amp_coeff.unwrap_or(0.5).clamp(0.0, 1.0);
        let freq_coeff = params.freq_coeff.unwrap_or(2.0).clamp(1.0, 6.0);
        let speed = params.speed.unwrap_or(0.5).clamp(0.0, 8.0);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.1, 3.0);
        let orientation = LightningOrientation::from_params(params);
        let time = p * 6.0;
        let flicker = hash12f((time, 0.0)) * 0.05;

        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;

                let mut uv = (
                    (dx as f32 / region.width.max(1) as f32) * 2.0 - 1.0,
                    (dy as f32 / region.height.max(1) as f32) * 2.0 - 1.0,
                );
                let n = fbm2(
                    (uv.0 + time * speed, uv.1 + time * speed * 0.73),
                    octaves,
                    amp_start,
                    amp_coeff,
                    freq_coeff,
                );
                let distortion = (2.0 * n - 1.0) * 0.6;
                let dist = match orientation {
                    LightningOrientation::Vertical => {
                        uv.0 += distortion;
                        uv.0.abs().max(0.04)
                    }
                    LightningOrientation::Horizontal => {
                        uv.1 += distortion;
                        uv.1.abs().max(0.04)
                    }
                };
                let amount = ((flicker / dist) * intensity).clamp(0.0, 1.0);
                if amount <= 0.02 {
                    continue;
                }
                let sym = if amount > 0.75 {
                    Some('█')
                } else if amount > 0.45 {
                    Some('▓')
                } else {
                    Some('▒')
                };
                blend_cell_to(buffer, x, y, effect_color, amount, sym, TRUE_BLACK);
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_FBM
    }
}

/// More natural lightning band with a bright unstable core, soft halo, and secondary filaments.
pub struct LightningNaturalEffect;

impl Effect for LightningNaturalEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 12_000.0) as u32;
        let orientation = LightningOrientation::from_params(params);
        let effect_color = get_effect_color(params, 124, 170, 255);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.1, 2.5);
        let thickness = params.thickness.unwrap_or(1.2).clamp(0.35, 3.0);
        let strikes = params.strikes.unwrap_or(2).clamp(1, 4) as usize;
        let glow = params.glow.unwrap_or(true);
        let octaves = params.octave_count.unwrap_or(6).clamp(2, 12);
        let amp_start = params.amp_start.unwrap_or(0.45).clamp(0.05, 1.0);
        let amp_coeff = params.amp_coeff.unwrap_or(0.52).clamp(0.1, 1.0);
        let freq_coeff = params.freq_coeff.unwrap_or(2.0).clamp(1.0, 5.0);
        let speed = params.speed.unwrap_or(0.55).clamp(0.0, 8.0);

        let attack = smoothstep01((p / 0.16).clamp(0.0, 1.0));
        let sustain = if p < 0.56 {
            1.0
        } else {
            (1.0 - ((p - 0.56) / 0.44)).clamp(0.0, 1.0).powf(1.45)
        };
        let decay_flicker = 0.86 + hash12f((p * 14.0, 7.0)) * 0.14;
        let envelope = (attack * sustain * decay_flicker * intensity).clamp(0.0, 1.0);
        if envelope <= 0.02 {
            return;
        }

        match orientation {
            LightningOrientation::Horizontal => {
                let h = region.height.max(1);
                let w = region.width.max(1);
                let start = parse_anchor(
                    params.start_x.as_deref(),
                    h,
                    crt_hash(region.x, region.y, frame),
                );
                let end = parse_anchor(
                    params.end_x.as_deref(),
                    h,
                    crt_hash(region.x, region.y.saturating_add(1), frame.wrapping_add(19)),
                );
                let halo_radius = (thickness * 5.0).ceil() as i32;

                for ix in 0..w {
                    let x = region.x + ix;
                    let t = ix as f32 / w.saturating_sub(1).max(1) as f32;
                    let base_axis = start + (end - start) * t;
                    let noise = fbm2(
                        (t * 4.0 + p * speed * 2.3, p * speed * 1.7 + t * 0.75),
                        octaves,
                        amp_start,
                        amp_coeff,
                        freq_coeff,
                    ) - 0.5;
                    let jitter =
                        (n01(x, region.y, frame.wrapping_add(301)) - 0.5) * thickness * 1.1;
                    let core_axis = base_axis + noise * thickness * 3.2 + jitter;

                    for oy in -halo_radius..=halo_radius {
                        let yy = core_axis + oy as f32;
                        let yi = yy.round() as i32;
                        if yi < region.y as i32 || yi >= (region.y + region.height) as i32 {
                            continue;
                        }
                        let y = yi as u16;
                        let dist = oy.abs() as f32;
                        let core = (1.0 - dist / (0.55 + thickness)).clamp(0.0, 1.0).powf(1.6);
                        let halo = (1.0 - dist / (2.0 + thickness * 3.0))
                            .clamp(0.0, 1.0)
                            .powf(2.2);
                        let amount = (core * 0.78 + halo * 0.28) * envelope;
                        if amount <= 0.02 {
                            continue;
                        }
                        let symbol = if core > 0.72 {
                            Some('█')
                        } else if core > 0.38 {
                            Some('▓')
                        } else if halo > 0.12 {
                            Some('▒')
                        } else {
                            Some('░')
                        };
                        let tint = if core > 0.45 {
                            Color::White
                        } else {
                            effect_color
                        };
                        blend_cell_to(
                            buffer,
                            x,
                            y,
                            tint,
                            amount.clamp(0.0, 1.0),
                            symbol,
                            TRUE_BLACK,
                        );
                    }

                    if glow {
                        let glow_y = core_axis
                            .round()
                            .clamp(region.y as f32, (region.y + region.height - 1) as f32)
                            as u16;
                        apply_glow(buffer, region, x, glow_y, envelope * 0.85);
                    }

                    let branch_gate = n01(x, region.y.saturating_add(2), frame.wrapping_add(911));
                    if branch_gate > 0.885 {
                        let branch_dir = if branch_gate > 0.95 { 1_i32 } else { -1_i32 };
                        let branch_len = 1 + ((branch_gate * 4.0) as i32);
                        for branch_idx in 1..=branch_len.min((strikes as i32) + 2) {
                            let bx_i = x as i32 + branch_idx / 2;
                            let by_i = core_axis.round() as i32 + branch_dir * branch_idx;
                            if !in_region_i32(bx_i, by_i, region) {
                                break;
                            }
                            let local = (envelope - branch_idx as f32 * 0.11).clamp(0.12, 0.9);
                            blend_cell_to(
                                buffer,
                                bx_i as u16,
                                by_i as u16,
                                Color::White,
                                local,
                                Some(bolt_symbol(local)),
                                TRUE_BLACK,
                            );
                        }
                    }
                }
            }
            LightningOrientation::Vertical => {
                let h = region.height.max(1);
                let w = region.width.max(1);
                let start = parse_anchor(
                    params.start_x.as_deref(),
                    w,
                    crt_hash(region.x, region.y, frame),
                );
                let end = parse_anchor(
                    params.end_x.as_deref(),
                    w,
                    crt_hash(region.x.saturating_add(1), region.y, frame.wrapping_add(19)),
                );
                let halo_radius = (thickness * 5.0).ceil() as i32;

                for iy in 0..h {
                    let y = region.y + iy;
                    let t = iy as f32 / h.saturating_sub(1).max(1) as f32;
                    let base_axis = start + (end - start) * t;
                    let noise = fbm2(
                        (p * speed * 1.7 + t * 0.75, t * 4.0 + p * speed * 2.3),
                        octaves,
                        amp_start,
                        amp_coeff,
                        freq_coeff,
                    ) - 0.5;
                    let jitter =
                        (n01(region.x, y, frame.wrapping_add(301)) - 0.5) * thickness * 1.1;
                    let core_axis = base_axis + noise * thickness * 3.2 + jitter;

                    for ox in -halo_radius..=halo_radius {
                        let xx = core_axis + ox as f32;
                        let xi = xx.round() as i32;
                        if !in_region_i32_x(xi, region) {
                            continue;
                        }
                        let x = xi as u16;
                        let dist = ox.abs() as f32;
                        let core = (1.0 - dist / (0.55 + thickness)).clamp(0.0, 1.0).powf(1.6);
                        let halo = (1.0 - dist / (2.0 + thickness * 3.0))
                            .clamp(0.0, 1.0)
                            .powf(2.2);
                        let amount = (core * 0.78 + halo * 0.28) * envelope;
                        if amount <= 0.02 {
                            continue;
                        }
                        let symbol = if core > 0.72 {
                            Some('█')
                        } else if core > 0.38 {
                            Some('▓')
                        } else if halo > 0.12 {
                            Some('▒')
                        } else {
                            Some('░')
                        };
                        let tint = if core > 0.45 {
                            Color::White
                        } else {
                            effect_color
                        };
                        blend_cell_to(
                            buffer,
                            x,
                            y,
                            tint,
                            amount.clamp(0.0, 1.0),
                            symbol,
                            TRUE_BLACK,
                        );
                    }

                    if glow {
                        let glow_x = core_axis
                            .round()
                            .clamp(region.x as f32, (region.x + region.width - 1) as f32)
                            as u16;
                        apply_glow(buffer, region, glow_x, y, envelope * 0.85);
                    }

                    let branch_gate = n01(region.x.saturating_add(2), y, frame.wrapping_add(911));
                    if branch_gate > 0.885 {
                        let branch_dir = if branch_gate > 0.95 { 1_i32 } else { -1_i32 };
                        let branch_len = 1 + ((branch_gate * 4.0) as i32);
                        for branch_idx in 1..=branch_len.min((strikes as i32) + 2) {
                            let bx_i = core_axis.round() as i32 + branch_dir * branch_idx;
                            let by_i = y as i32 + branch_idx / 2;
                            if !in_region_i32(bx_i, by_i, region) {
                                break;
                            }
                            let local = (envelope - branch_idx as f32 * 0.11).clamp(0.12, 0.9);
                            blend_cell_to(
                                buffer,
                                bx_i as u16,
                                by_i as u16,
                                Color::White,
                                local,
                                Some(bolt_symbol(local)),
                                TRUE_BLACK,
                            );
                        }
                    }
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_NATURAL
    }
}

/// Ambient horizontal lightning band that emits sparse, lighter background flashes
/// without the explicit bolt/core pass used by the transition beat.
pub struct LightningAmbientEffect;

impl Effect for LightningAmbientEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let p = progress.clamp(0.0, 1.0);
        let pulse_count = 5usize;
        let fbm = LightningFbmEffect;

        for pulse_idx in 0..pulse_count {
            let (center, width) = ambient_pulse_descriptor(region, pulse_idx, pulse_count);
            let envelope = ambient_pulse_envelope(p, center, width);
            if envelope <= 0.02 {
                continue;
            }

            let local_progress = ((p - (center - width)) / (width * 2.0)).clamp(0.0, 1.0);

            let mut fbm_params = params.clone();
            fbm_params.orientation = Some(
                params
                    .orientation
                    .clone()
                    .unwrap_or_else(|| "horizontal".to_string()),
            );
            fbm_params.start_x = None;
            fbm_params.end_x = None;
            fbm_params.strikes = None;
            fbm_params.intensity =
                Some((params.intensity.unwrap_or(1.0) * 0.5 * envelope).clamp(0.05, 0.55));
            fbm_params.octave_count = Some(params.octave_count.unwrap_or(7).clamp(2, 12));
            fbm_params.amp_start = Some(params.amp_start.unwrap_or(0.42).clamp(0.05, 1.0));
            fbm_params.amp_coeff = Some(params.amp_coeff.unwrap_or(0.5).clamp(0.1, 1.0));
            fbm_params.freq_coeff = Some(params.freq_coeff.unwrap_or(2.0).clamp(1.0, 5.0));
            fbm_params.speed = Some(params.speed.unwrap_or(0.55).clamp(0.0, 8.0));

            fbm.apply(local_progress, &fbm_params, region, buffer);
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_LIGHTNING_AMBIENT
    }
}

/// Renders a spherical "tesla coil" electrical burst around the center.
pub struct TeslaOrbEffect;

impl Effect for TeslaOrbEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 12_000.0) as u32;
        let strikes = params.strikes.unwrap_or(12).clamp(4, 28) as usize;
        let thickness = params.thickness.unwrap_or(1.5).clamp(1.0, 3.0);
        let glow = params.glow.unwrap_or(true);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.2, 2.4);

        let cx = region.x as f32 + region.width as f32 * 0.5;
        let cy = region.y as f32 + region.height as f32 * 0.5;
        let min_dim = region.width.min(region.height).max(8) as f32;
        let orb_radius = (min_dim * 0.11 + thickness).clamp(2.0, 8.0);
        let arc_radius = (orb_radius * 2.8).clamp(8.0, min_dim * 0.42);

        let pulse_a = (-((p - 0.14) * 15.0).powi(2)).exp();
        let pulse_b = (-((p - 0.47) * 20.0).powi(2)).exp() * 0.85;
        let pulse_c = (-((p - 0.82) * 24.0).powi(2)).exp() * 0.62;
        let envelope =
            ((0.28 + pulse_a + pulse_b + pulse_c).clamp(0.0, 1.0) * intensity).clamp(0.0, 1.0);

        let orb_i32 = orb_radius.ceil() as i32;
        for oy in -orb_i32..=orb_i32 {
            for ox in -orb_i32..=orb_i32 {
                let x = cx + ox as f32;
                let y = cy + oy as f32;
                let dist = ((ox * ox + oy * oy) as f32).sqrt();
                if dist > orb_radius {
                    continue;
                }
                let xi = x.round() as i32;
                let yi = y.round() as i32;
                if !in_region_i32(xi, yi, region) {
                    continue;
                }
                let xx = xi as u16;
                let yy = yi as u16;
                let core = ((1.0 - dist / orb_radius).clamp(0.0, 1.0) * 0.85 + 0.15) * envelope;
                let sym = bolt_symbol(core);
                let mut fg = Color::White;
                if let Some(cell) = buffer.get(xx, yy) {
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    fg = Color::Rgb {
                        r: r.saturating_add((120.0 * core) as u8),
                        g: g.saturating_add((120.0 * core) as u8),
                        b: b.saturating_add((140.0 * core) as u8),
                    };
                }
                buffer.set(xx, yy, sym, fg, TRUE_BLACK);
                if glow {
                    apply_glow(buffer, region, xx, yy, envelope);
                }
            }
        }

        for s in 0..strikes {
            let seed = frame.wrapping_add((s as u32).wrapping_mul(81_329));
            let angle = n01(region.x.wrapping_add(s as u16), region.y, seed) * TAU;
            let spread = (n01(
                region.x,
                region.y.wrapping_add(s as u16),
                seed.wrapping_add(17),
            ) - 0.5)
                * 1.5;
            let start_r =
                orb_radius * (0.75 + n01(region.x, region.y, seed.wrapping_add(33)) * 0.3);
            let end_r = arc_radius * (0.82 + n01(region.x, region.y, seed.wrapping_add(61)) * 0.38);
            let end_angle = angle + spread + (p * 2.6 * if s % 2 == 0 { 1.0 } else { -1.0 });

            let start_x = cx + angle.cos() * start_r;
            let start_y = cy + angle.sin() * start_r;
            let end_x = cx + end_angle.cos() * end_r;
            let end_y = cy + end_angle.sin() * end_r;

            let dx = end_x - start_x;
            let dy = end_y - start_y;
            let base_steps = dx.abs().max(dy.abs()).ceil() as i32;
            let steps = base_steps.max(8) as usize;

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let jitter_seed = seed.wrapping_add((i as u32).wrapping_mul(977));
                let jitter = (n01(region.x, region.y, jitter_seed) - 0.5) * 2.0;
                let side = (1.0 - (t * 2.0 - 1.0).abs()).max(0.0);
                let jx = jitter * thickness * (0.25 + side);
                let jy = jitter * thickness * (0.18 + side * 0.8);

                let x = start_x + dx * t + jx;
                let y = start_y + dy * t + jy;
                let xi = x.round() as i32;
                let yi = y.round() as i32;
                if !in_region_i32(xi, yi, region) {
                    continue;
                }
                let xx = xi as u16;
                let yy = yi as u16;
                let local = (envelope * (1.0 - t * 0.4) * (0.86 + n01(xx, yy, seed) * 0.14))
                    .clamp(0.0, 1.0);
                let sym = bolt_symbol(local);
                let mut fg = Color::White;
                if let Some(cell) = buffer.get(xx, yy) {
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    fg = Color::Rgb {
                        r: r.saturating_add((100.0 * local) as u8),
                        g: g.saturating_add((110.0 * local) as u8),
                        b: b.saturating_add((130.0 * local) as u8),
                    };
                }
                buffer.set(xx, yy, sym, fg, TRUE_BLACK);
                if glow {
                    apply_glow(buffer, region, xx, yy, local);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_TESLA_ORB
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ambient_pulse_descriptor, Effect, LightningAmbientEffect, LightningBranchEffect,
        LightningFbmEffect, LightningGrowthEffect, LightningNaturalEffect,
    };
    use crate::buffer::Buffer;
    use crate::color::Color;
    use crate::effects::Region;
    use crate::scene::EffectParams;

    fn lit_axes_coverage(buffer: &Buffer) -> (usize, usize) {
        let lit_x = (0..buffer.width)
            .filter(|&x| {
                (0..buffer.height).any(|y| buffer.get(x, y).is_some_and(|cell| cell.symbol != ' '))
            })
            .count();
        let lit_y = (0..buffer.height)
            .filter(|&y| {
                (0..buffer.width).any(|x| buffer.get(x, y).is_some_and(|cell| cell.symbol != ' '))
            })
            .count();
        (lit_x, lit_y)
    }

    fn lit_cell_count(buffer: &Buffer) -> usize {
        (0..buffer.height)
            .flat_map(|y| (0..buffer.width).map(move |x| (x, y)))
            .filter(|&(x, y)| buffer.get(x, y).is_some_and(|cell| cell.symbol != ' '))
            .count()
    }

    #[test]
    fn lightning_fbm_orientation_changes_primary_axis() {
        let effect = LightningFbmEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };
        let vertical_params = EffectParams {
            intensity: Some(0.2),
            ..EffectParams::default()
        };

        let mut vertical = Buffer::new(region.width, region.height);
        vertical.fill(Color::Black);
        effect.apply(0.5, &vertical_params, region, &mut vertical);

        let mut horizontal = Buffer::new(region.width, region.height);
        horizontal.fill(Color::Black);
        effect.apply(
            0.5,
            &EffectParams {
                intensity: Some(0.2),
                orientation: Some("horizontal".to_string()),
                ..EffectParams::default()
            },
            region,
            &mut horizontal,
        );

        let (vertical_x, vertical_y) = lit_axes_coverage(&vertical);
        let (horizontal_x, horizontal_y) = lit_axes_coverage(&horizontal);

        assert!(vertical_y > vertical_x);
        assert!(horizontal_x > horizontal_y);
    }

    #[test]
    fn lightning_branch_orientation_changes_primary_axis() {
        let effect = LightningBranchEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };

        let mut vertical = Buffer::new(region.width, region.height);
        vertical.fill(Color::Black);
        effect.apply(
            0.5,
            &EffectParams {
                strikes: Some(1),
                thickness: Some(1.0),
                glow: Some(false),
                ..EffectParams::default()
            },
            region,
            &mut vertical,
        );

        let mut horizontal = Buffer::new(region.width, region.height);
        horizontal.fill(Color::Black);
        effect.apply(
            0.5,
            &EffectParams {
                strikes: Some(1),
                thickness: Some(1.0),
                glow: Some(false),
                orientation: Some("horizontal".to_string()),
                ..EffectParams::default()
            },
            region,
            &mut horizontal,
        );

        let (vertical_x, vertical_y) = lit_axes_coverage(&vertical);
        let (horizontal_x, horizontal_y) = lit_axes_coverage(&horizontal);

        assert!(vertical_y > vertical_x);
        assert!(horizontal_x > horizontal_y);
    }

    #[test]
    fn lightning_natural_orientation_changes_primary_axis() {
        let effect = LightningNaturalEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };

        let mut vertical = Buffer::new(region.width, region.height);
        vertical.fill(Color::Black);
        effect.apply(
            0.5,
            &EffectParams {
                strikes: Some(2),
                thickness: Some(1.0),
                glow: Some(false),
                intensity: Some(0.7),
                ..EffectParams::default()
            },
            region,
            &mut vertical,
        );

        let mut horizontal = Buffer::new(region.width, region.height);
        horizontal.fill(Color::Black);
        effect.apply(
            0.5,
            &EffectParams {
                strikes: Some(2),
                thickness: Some(1.0),
                glow: Some(false),
                intensity: Some(0.7),
                orientation: Some("horizontal".to_string()),
                ..EffectParams::default()
            },
            region,
            &mut horizontal,
        );

        let (vertical_x, vertical_y) = lit_axes_coverage(&vertical);
        let (horizontal_x, horizontal_y) = lit_axes_coverage(&horizontal);

        assert!(vertical_y > vertical_x);
        assert!(horizontal_x > horizontal_y);
    }

    #[test]
    fn lightning_growth_orientation_changes_primary_axis() {
        let effect = LightningGrowthEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };

        let mut vertical = Buffer::new(region.width, region.height);
        vertical.fill(Color::Black);
        effect.apply(
            0.7,
            &EffectParams {
                strikes: Some(2),
                thickness: Some(1.0),
                glow: Some(false),
                intensity: Some(0.7),
                ..EffectParams::default()
            },
            region,
            &mut vertical,
        );

        let mut horizontal = Buffer::new(region.width, region.height);
        horizontal.fill(Color::Black);
        effect.apply(
            0.7,
            &EffectParams {
                strikes: Some(2),
                thickness: Some(1.0),
                glow: Some(false),
                intensity: Some(0.7),
                orientation: Some("horizontal".to_string()),
                ..EffectParams::default()
            },
            region,
            &mut horizontal,
        );

        let (vertical_x, vertical_y) = lit_axes_coverage(&vertical);
        let (horizontal_x, horizontal_y) = lit_axes_coverage(&horizontal);

        assert!(vertical_y > vertical_x);
        assert!(horizontal_x > horizontal_y);
    }

    #[test]
    fn lightning_growth_reveals_more_cells_later_in_progress() {
        let effect = LightningGrowthEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 48,
            height: 20,
        };
        let params = EffectParams {
            strikes: Some(2),
            thickness: Some(1.0),
            glow: Some(false),
            intensity: Some(0.8),
            orientation: Some("horizontal".to_string()),
            ..EffectParams::default()
        };

        let mut early = Buffer::new(region.width, region.height);
        early.fill(Color::Black);
        effect.apply(0.18, &params, region, &mut early);

        let mut late = Buffer::new(region.width, region.height);
        late.fill(Color::Black);
        effect.apply(0.68, &params, region, &mut late);

        assert!(lit_cell_count(&late) > lit_cell_count(&early));
        let (early_x, _) = lit_axes_coverage(&early);
        let (late_x, _) = lit_axes_coverage(&late);
        assert!(late_x >= early_x);
    }

    #[test]
    fn lightning_ambient_is_sparse_outside_pulses() {
        let effect = LightningAmbientEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 48,
            height: 18,
        };

        let mut buffer = Buffer::new(region.width, region.height);
        buffer.fill(Color::Black);
        effect.apply(
            0.0,
            &EffectParams {
                orientation: Some("horizontal".to_string()),
                intensity: Some(0.48),
                ..EffectParams::default()
            },
            region,
            &mut buffer,
        );

        let (lit_x, lit_y) = lit_axes_coverage(&buffer);
        assert_eq!(lit_x, 0);
        assert_eq!(lit_y, 0);
    }

    #[test]
    fn lightning_ambient_flashes_horizontally_at_pulse_center() {
        let effect = LightningAmbientEffect;
        let region = Region {
            x: 0,
            y: 0,
            width: 48,
            height: 18,
        };
        let (center, _) = ambient_pulse_descriptor(region, 0, 5);

        let mut buffer = Buffer::new(region.width, region.height);
        buffer.fill(Color::Black);
        effect.apply(
            center,
            &EffectParams {
                orientation: Some("horizontal".to_string()),
                intensity: Some(0.48),
                thickness: Some(0.9),
                ..EffectParams::default()
            },
            region,
            &mut buffer,
        );

        let (lit_x, lit_y) = lit_axes_coverage(&buffer);
        assert!(lit_x > 0);
        assert!(lit_y > 0);
        assert!(lit_x > lit_y);
    }
}
