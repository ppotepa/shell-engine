use crate::scene::{Node3DInstance, Renderable3D, SceneClip3DInstance};
use engine_core::render_types::Transform3D;
use engine_core::scene::{CameraSource, Sprite};

pub struct SceneClipSpriteSpec<'a> {
    pub sprite: &'a Sprite,
    pub node: Node3DInstance,
    pub id: Option<&'a str>,
    pub source: &'a str,
    pub frame: &'a str,
    pub camera_source: CameraSource,
    pub use_scene_camera: bool,
    pub x: i32,
    pub y: i32,
    pub z_index: i32,
    pub grid_row: u16,
    pub grid_col: u16,
    pub row_span: u16,
    pub col_span: u16,
    pub stretch_to_area: bool,
    pub appear_at_ms: Option<u64>,
    pub disappear_at_ms: Option<u64>,
    pub hide_on_leave: bool,
    pub visible: bool,
}

pub fn extract_scene_clip_sprite_spec(sprite: &Sprite) -> Option<SceneClipSpriteSpec<'_>> {
    let Sprite::Scene3D {
        id,
        src,
        frame,
        camera_source,
        x,
        y,
        z_index,
        grid_row,
        grid_col,
        row_span,
        col_span,
        stretch_to_area,
        appear_at_ms,
        disappear_at_ms,
        hide_on_leave,
        visible,
        ..
    } = sprite
    else {
        return None;
    };

    let use_scene_camera = *camera_source == CameraSource::Scene;

    Some(SceneClipSpriteSpec {
        sprite,
        node: Node3DInstance {
            id: id.clone().unwrap_or_else(|| "scene3d-node".to_string()),
            transform: Transform3D {
                translation: [*x as f32, *y as f32, 0.0],
                rotation_deg: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            visible: *visible,
            lod_hint: None,
            renderable: Renderable3D::SceneClip(SceneClip3DInstance {
                source: src.clone(),
                frame: frame.clone(),
                use_scene_camera,
            }),
        },
        id: id.as_deref(),
        source: src,
        frame,
        camera_source: *camera_source,
        use_scene_camera,
        x: *x,
        y: *y,
        z_index: *z_index,
        grid_row: *grid_row,
        grid_col: *grid_col,
        row_span: *row_span,
        col_span: *col_span,
        stretch_to_area: *stretch_to_area,
        appear_at_ms: *appear_at_ms,
        disappear_at_ms: *disappear_at_ms,
        hide_on_leave: *hide_on_leave,
        visible: *visible,
    })
}

#[cfg(test)]
mod tests {
    use super::extract_scene_clip_sprite_spec;
    use crate::scene::Renderable3D;
    use engine_core::scene::{CameraSource, Sprite};

    #[test]
    fn extracts_scene3d_sprite_spec() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: scene3_d
id: clip-view
src: /assets/3d/sample.scene3d.yml
frame: main
camera-source: scene
x: 12
y: 7
z_index: 4
grid-row: 2
grid-col: 3
row-span: 2
col-span: 1
appear_at_ms: 100
disappear_at_ms: 900
hide_on_leave: true
visible: true
"#,
        )
        .expect("scene3d sprite should parse");

        let spec = extract_scene_clip_sprite_spec(&sprite).expect("spec");
        assert_eq!(spec.id, Some("clip-view"));
        assert_eq!(spec.source, "/assets/3d/sample.scene3d.yml");
        assert_eq!(spec.frame, "main");
        assert_eq!(spec.camera_source, CameraSource::Scene);
        assert!(spec.use_scene_camera);
        assert_eq!(spec.x, 12);
        assert_eq!(spec.y, 7);
        assert_eq!(spec.z_index, 4);
        assert_eq!(spec.grid_row, 2);
        assert_eq!(spec.grid_col, 3);
        assert_eq!(spec.row_span, 2);
        assert_eq!(spec.col_span, 1);
        assert_eq!(spec.appear_at_ms, Some(100));
        assert_eq!(spec.disappear_at_ms, Some(900));
        assert!(spec.hide_on_leave);
        assert!(spec.visible);

        match &spec.node.renderable {
            Renderable3D::SceneClip(clip) => {
                assert_eq!(clip.source, "/assets/3d/sample.scene3d.yml");
                assert_eq!(clip.frame, "main");
                assert!(clip.use_scene_camera);
            }
            _ => panic!("expected scene clip renderable"),
        }
    }

    #[test]
    fn rejects_non_scene3d_sprites() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: /assets/3d/sphere.obj
"#,
        )
        .expect("obj sprite should parse");

        assert!(extract_scene_clip_sprite_spec(&sprite).is_none());
    }
}
