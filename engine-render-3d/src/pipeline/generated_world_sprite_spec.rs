use crate::scene::{GeneratedWorldInstance, Node3DInstance, Renderable3D};
use engine_core::render_types::Transform3D;
use engine_core::scene::{CameraSource, HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign};

#[derive(Debug, Clone)]
pub struct GeneratedWorldSpriteSpec {
    pub node: Node3DInstance,
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
        id,
        body_id,
        preset,
        mesh_source,
        x,
        y,
        scale,
        pitch_deg,
        yaw_deg,
        roll_deg,
        visible,
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
        node: Node3DInstance {
            id: id
                .clone()
                .unwrap_or_else(|| format!("planet-{body_id}")),
            transform: Transform3D {
                translation: [*x as f32, *y as f32, 0.0],
                rotation_deg: [
                    pitch_deg.unwrap_or(0.0),
                    yaw_deg.unwrap_or(0.0),
                    roll_deg.unwrap_or(0.0),
                ],
                scale: [
                    scale.unwrap_or(1.0),
                    scale.unwrap_or(1.0),
                    scale.unwrap_or(1.0),
                ],
            },
            visible: *visible,
            renderable: Renderable3D::GeneratedWorld(GeneratedWorldInstance {
                body_id: body_id.clone(),
                preset_id: preset.clone(),
                mesh_source: mesh_source.clone(),
                params_uri: None,
                material: None,
            }),
        },
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
    use crate::scene::Renderable3D;
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
        assert_eq!(spec.node.id, "planet-earth");
        assert_eq!(spec.node.transform.translation, [0.0, 0.0, 0.0]);
        assert_eq!(spec.node.transform.rotation_deg, [0.0, 0.0, 0.0]);
        assert_eq!(spec.node.transform.scale, [1.0, 1.0, 1.0]);
        assert!(spec.node.visible);
        match &spec.node.renderable {
            Renderable3D::GeneratedWorld(world) => {
                assert_eq!(world.body_id, "earth");
            }
            _ => panic!("expected generated world renderable"),
        }
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
