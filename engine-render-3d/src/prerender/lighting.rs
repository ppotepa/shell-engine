use engine_3d::scene3d_format::{LightDef, LightKind};
use engine_core::color::Color;

#[derive(Clone)]
pub struct LightParams {
    pub dir1: [f32; 3],
    pub dir2: [f32; 3],
    pub dir2_intensity: f32,
    pub point1: [f32; 3],
    pub point1_intensity: f32,
    pub point1_colour: Option<Color>,
    pub point1_snap_hz: f32,
    pub point1_falloff: f32,
    pub point2: [f32; 3],
    pub point2_intensity: f32,
    pub point2_colour: Option<Color>,
    pub point2_snap_hz: f32,
    pub point2_falloff: f32,
    pub ambient: f32,
}

pub fn parse_hex_color(raw: &str) -> Option<Color> {
    let value = raw.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some(Color::Rgb { r, g, b })
}

pub fn extract_light_params(lights: &[LightDef]) -> LightParams {
    let mut out = LightParams {
        dir1: [-0.45, 0.70, -0.85],
        dir2: [0.0, 0.0, -1.0],
        dir2_intensity: 0.0,
        point1: [0.0, 2.0, 0.0],
        point1_intensity: 0.0,
        point1_colour: None,
        point1_snap_hz: 0.0,
        point1_falloff: 0.7,
        point2: [0.0, 0.0, 0.0],
        point2_intensity: 0.0,
        point2_colour: None,
        point2_snap_hz: 0.0,
        point2_falloff: 0.7,
        ambient: 0.0,
    };

    let mut dir_count = 0u8;
    let mut point_count = 0u8;
    for light in lights {
        match light.kind {
            LightKind::Directional => {
                let dir = light.direction.unwrap_or([-0.45, 0.70, -0.85]);
                if dir_count == 0 {
                    out.dir1 = dir;
                    dir_count += 1;
                } else if dir_count == 1 {
                    out.dir2 = dir;
                    out.dir2_intensity = light.intensity;
                    dir_count += 1;
                }
            }
            LightKind::Point => {
                let pos = light.position.unwrap_or([0.0, 2.0, 0.0]);
                let colour = light.colour.as_deref().and_then(parse_hex_color);
                if point_count == 0 {
                    out.point1 = pos;
                    out.point1_intensity = light.intensity;
                    out.point1_colour = colour;
                    out.point1_snap_hz = light.snap_hz;
                    out.point1_falloff = light.falloff_constant;
                    point_count += 1;
                } else if point_count == 1 {
                    out.point2 = pos;
                    out.point2_intensity = light.intensity;
                    out.point2_colour = colour;
                    out.point2_snap_hz = light.snap_hz;
                    out.point2_falloff = light.falloff_constant;
                    point_count += 1;
                }
            }
            LightKind::Ambient => {
                // Sum multiple ambient sources; max avoids unintended brightness stacking.
                out.ambient = (out.ambient + light.intensity).min(1.0);
            }
        }
    }

    out
}
