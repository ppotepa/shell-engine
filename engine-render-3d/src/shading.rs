use crate::geom::math::{cross3, dot3, normalize3, sub3};
use engine_core::color::Color;

#[inline(always)]
pub fn color_to_rgb(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb { r, g, b } => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::DarkGrey => [80, 80, 80],
        Color::Grey => [160, 160, 160],
        Color::White => [255, 255, 255],
        Color::Red | Color::DarkRed => [220, 64, 64],
        Color::Green | Color::DarkGreen => [64, 220, 64],
        Color::Blue | Color::DarkBlue => [64, 64, 220],
        Color::Yellow | Color::DarkYellow => [220, 220, 64],
        Color::Magenta | Color::DarkMagenta => [220, 64, 220],
        Color::Cyan | Color::DarkCyan => [64, 220, 220],
        _ => [255, 255, 255],
    }
}

pub fn quantize_shade(value: f32, levels: u8) -> f32 {
    if levels <= 1 {
        return value.clamp(0.0, 1.0);
    }
    let levels = levels.clamp(2, 8) as f32;
    let steps = levels - 1.0;
    let v = value.clamp(0.0, 1.0);
    (v * steps).round() / steps
}

#[allow(clippy::too_many_arguments)]
pub fn face_shading_with_specular(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    ka: [f32; 3],
    ks: f32,
    ns: f32,
    light_dir: [f32; 3],
    light_2_dir: [f32; 3],
    half_dir_1: [f32; 3],
    half_dir_2: [f32; 3],
    light_2_intensity: f32,
    light_point: [f32; 3],
    light_point_intensity: f32,
    light_point_2: [f32; 3],
    light_point_2_intensity: f32,
    cel_levels: u8,
    tone_mix: f32,
    ambient: f32,
    view_dir: [f32; 3],
    point_falloff: f32,
    point_2_falloff: f32,
) -> (f32, f32, f32, f32) {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let normal = normalize3(cross3(e1, e2));
    let light_2_strength = light_2_intensity.clamp(0.0, 2.0);
    let point_strength = light_point_intensity.clamp(0.0, 4.0);
    let point_2_strength = light_point_2_intensity.clamp(0.0, 4.0);
    let centroid = [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ];
    let to_point = sub3(light_point, centroid);
    let point_dir = normalize3(to_point);
    let point_dist =
        (to_point[0] * to_point[0] + to_point[1] * to_point[1] + to_point[2] * to_point[2])
            .sqrt()
            .max(0.0001);
    let point_atten = 1.0 / (1.0 + point_falloff * point_dist * point_dist);
    let to_point_2 = sub3(light_point_2, centroid);
    let point_2_dir = normalize3(to_point_2);
    let point_2_dist = (to_point_2[0] * to_point_2[0]
        + to_point_2[1] * to_point_2[1]
        + to_point_2[2] * to_point_2[2])
        .sqrt()
        .max(0.0001);
    let point_2_atten = 1.0 / (1.0 + point_2_falloff * point_2_dist * point_2_dist);
    let lambert_1 = dot3(normal, light_dir).max(0.0);
    let lambert_2 = dot3(normal, light_2_dir).max(0.0) * light_2_strength;
    let lambert_point = dot3(normal, point_dir).max(0.0) * point_strength * point_atten;
    let lambert_point_2 = dot3(normal, point_2_dir).max(0.0) * point_2_strength * point_2_atten;
    let lambert = (lambert_1 + lambert_2 + lambert_point + lambert_point_2).clamp(0.0, 1.0);
    let material_influence = (1.0 - tone_mix.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let ka_lum_material = (ka[0] * 0.299 + ka[1] * 0.587 + ka[2] * 0.114).clamp(0.03, 0.25);
    let ka_lum = (0.06 + (ka_lum_material - 0.06) * material_influence).max(ambient);
    let half_dir_point = normalize3([
        point_dir[0] + view_dir[0],
        point_dir[1] + view_dir[1],
        point_dir[2] + view_dir[2],
    ]);
    let half_dir_point_2 = normalize3([
        point_2_dir[0] + view_dir[0],
        point_2_dir[1] + view_dir[1],
        point_2_dir[2] + view_dir[2],
    ]);
    let shininess = 24.0 + (ns.clamp(2.0, 200.0) - 24.0) * material_influence;
    let spec_1 = dot3(normal, half_dir_1).abs().powf(shininess);
    let spec_2 = dot3(normal, half_dir_2).abs().powf(shininess) * light_2_strength * 0.7;
    let spec_point =
        dot3(normal, half_dir_point).abs().powf(shininess) * point_strength * point_atten * 0.9;
    let spec_point_2 = dot3(normal, half_dir_point_2).abs().powf(shininess)
        * point_2_strength
        * point_2_atten
        * 0.9;
    let ks_strength = 0.08 + (ks.clamp(0.0, 0.6) - 0.08) * material_influence;
    let spec = (spec_1 + spec_2 + spec_point + spec_point_2) * ks_strength;
    let diffuse = ka_lum + (1.0 - ka_lum) * lambert * 0.9;
    let cel_diffuse = quantize_shade(diffuse, cel_levels);
    (
        (cel_diffuse + spec).clamp(0.0, 1.0),
        cel_diffuse,
        lambert_point.clamp(0.0, 1.0),
        lambert_point_2.clamp(0.0, 1.0),
    )
}

#[inline(always)]
pub fn apply_shading(rgb: [u8; 3], shade: f32) -> [u8; 3] {
    let lin = [
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    ];
    let sat_lin = saturate(lin, 1.25);
    [
        linear_to_srgb((sat_lin[0] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[1] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[2] * shade).clamp(0.0, 1.0)),
    ]
}

#[inline(always)]
pub fn apply_tone_palette(
    base_rgb: [u8; 3],
    tone: f32,
    shadow: Option<Color>,
    midtone: Option<Color>,
    highlight: Option<Color>,
    tone_mix: f32,
) -> [u8; 3] {
    let mix = tone_mix.clamp(0.0, 1.0);
    if mix <= 0.0 {
        return base_rgb;
    }
    let shadow_rgb = shadow.map(color_to_rgb).unwrap_or([0, 0, 0]);
    let midtone_rgb = midtone
        .map(color_to_rgb)
        .unwrap_or(mix_rgb(shadow_rgb, base_rgb, 0.45));
    let highlight_rgb = highlight.map(color_to_rgb).unwrap_or(base_rgb);
    let t = tone.clamp(0.0, 1.0);
    let toon_rgb = if t <= 0.5 {
        mix_rgb(shadow_rgb, midtone_rgb, t * 2.0)
    } else {
        mix_rgb(midtone_rgb, highlight_rgb, (t - 0.5) * 2.0)
    };
    mix_rgb(base_rgb, toon_rgb, mix)
}

#[inline(always)]
pub fn apply_point_light_tint(
    base_rgb: [u8; 3],
    light_1_colour: Option<Color>,
    light_1_strength: f32,
    light_2_colour: Option<Color>,
    light_2_strength: f32,
) -> [u8; 3] {
    let mut out = base_rgb;
    if let Some(colour) = light_1_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_1_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    if let Some(colour) = light_2_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_2_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    out
}

#[inline(always)]
pub fn flicker_multiplier(elapsed_s: f32, hz: f32, depth: f32, phase: f32) -> f32 {
    let d = depth.clamp(0.0, 1.0);
    if d <= f32::EPSILON {
        return 1.0;
    }
    let rate = hz.clamp(0.1, 40.0);
    let base = ((elapsed_s * std::f32::consts::TAU * rate + phase).sin() * 0.5 + 0.5).powf(1.5);
    let chatter = ((elapsed_s * std::f32::consts::TAU * (rate * 2.31) + phase * 1.7)
        .sin()
        .abs())
    .powf(2.3);
    let pulse = (base * 0.65 + chatter * 0.35).clamp(0.0, 1.0);
    ((1.0 - d) + d * pulse).clamp(0.0, 1.0)
}

#[inline(always)]
pub fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

#[inline(always)]
pub fn srgb_to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
pub fn linear_to_srgb(v: f32) -> u8 {
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[inline(always)]
pub fn saturate(lin: [f32; 3], factor: f32) -> [f32; 3] {
    let lum = lin[0] * 0.299 + lin[1] * 0.587 + lin[2] * 0.114;
    [
        (lum + (lin[0] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[1] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[2] - lum) * factor).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_bounds_and_steps() {
        assert_eq!(quantize_shade(-1.0, 4), 0.0);
        assert_eq!(quantize_shade(2.0, 4), 1.0);
        assert_eq!(quantize_shade(0.49, 2), 0.0);
        assert_eq!(quantize_shade(0.51, 2), 1.0);
    }

    #[test]
    fn mix_rgb_endpoints() {
        let a = [10, 20, 30];
        let b = [200, 210, 220];
        assert_eq!(mix_rgb(a, b, 0.0), a);
        assert_eq!(mix_rgb(a, b, 1.0), b);
    }
}
