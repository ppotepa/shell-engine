use engine_core::scene::{CameraSource, HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign};

#[derive(Debug, Clone)]
pub struct GeneratedWorldSpriteSpec {
    pub size: Option<SpriteSizePreset>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub spin_deg: Option<f32>,
    pub cloud_spin_deg: Option<f32>,
    pub cloud2_spin_deg: Option<f32>,
    pub observer_altitude_km: Option<f32>,
    pub camera_distance: Option<f32>,
    pub camera_source: CameraSource,
    pub fov_degrees: Option<f32>,
    pub near_clip: Option<f32>,
    pub sun_dir_x: Option<f32>,
    pub sun_dir_y: Option<f32>,
    pub sun_dir_z: Option<f32>,
    pub align_x: Option<HorizontalAlign>,
    pub align_y: Option<VerticalAlign>,
}

pub fn extract_generated_world_sprite_spec(sprite: &Sprite) -> Option<GeneratedWorldSpriteSpec> {
    let Sprite::Planet {
        size,
        width,
        height,
        spin_deg,
        cloud_spin_deg,
        cloud2_spin_deg,
        observer_altitude_km,
        camera_distance,
        camera_source,
        fov_degrees,
        near_clip,
        sun_dir_x,
        sun_dir_y,
        sun_dir_z,
        align_x,
        align_y,
        ..
    } = sprite
    else {
        return None;
    };

    Some(GeneratedWorldSpriteSpec {
        size: *size,
        width: *width,
        height: *height,
        spin_deg: *spin_deg,
        cloud_spin_deg: *cloud_spin_deg,
        cloud2_spin_deg: *cloud2_spin_deg,
        observer_altitude_km: *observer_altitude_km,
        camera_distance: *camera_distance,
        camera_source: *camera_source,
        fov_degrees: *fov_degrees,
        near_clip: *near_clip,
        sun_dir_x: *sun_dir_x,
        sun_dir_y: *sun_dir_y,
        sun_dir_z: *sun_dir_z,
        align_x: align_x.clone(),
        align_y: align_y.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::extract_generated_world_sprite_spec;
    use engine_core::scene::{
        CameraSource, HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign,
    };

    #[test]
    fn extracts_generated_world_planet_fields() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: planet
body-id: earth
size: 3
width: 64
height: 48
spin-deg: 12.5
cloud-spin-deg: 1.5
cloud2-spin-deg: -2.25
observer-altitude-km: 120.0
camera-distance: 4.5
camera-source: scene
fov-degrees: 72.0
near-clip: 0.01
sun-dir-x: 1.0
sun-dir-y: -0.5
sun-dir-z: 0.25
align-x: center
align-y: bottom
"#,
        )
        .expect("planet sprite should parse");

        let spec = extract_generated_world_sprite_spec(&sprite).expect("spec");
        assert_eq!(spec.size, Some(SpriteSizePreset::Large));
        assert_eq!(spec.width, Some(64));
        assert_eq!(spec.height, Some(48));
        assert_eq!(spec.spin_deg, Some(12.5));
        assert_eq!(spec.cloud_spin_deg, Some(1.5));
        assert_eq!(spec.cloud2_spin_deg, Some(-2.25));
        assert_eq!(spec.observer_altitude_km, Some(120.0));
        assert_eq!(spec.camera_distance, Some(4.5));
        assert_eq!(spec.camera_source, CameraSource::Scene);
        assert_eq!(spec.fov_degrees, Some(72.0));
        assert_eq!(spec.near_clip, Some(0.01));
        assert_eq!(spec.sun_dir_x, Some(1.0));
        assert_eq!(spec.sun_dir_y, Some(-0.5));
        assert_eq!(spec.sun_dir_z, Some(0.25));
        assert!(matches!(spec.align_x, Some(HorizontalAlign::Center)));
        assert!(matches!(spec.align_y, Some(VerticalAlign::Bottom)));
    }

    #[test]
    fn rejects_non_planet_sprites() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: /assets/3d/sphere.obj
"#,
        )
        .expect("obj sprite should parse");

        assert!(extract_generated_world_sprite_spec(&sprite).is_none());
    }
}
