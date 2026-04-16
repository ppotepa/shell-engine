use engine_3d::scene3d_format::FrameDef;
use engine_core::scene_runtime_types::SceneCamera3D;

use super::{
    build_object_specs, clip_progress_at, extract_light_params, Scene3DRuntimeEntry, Scene3DWorkItem,
};

pub fn build_scene3d_frame_item_at(
    entry: &Scene3DRuntimeEntry,
    frame_name: &str,
    elapsed_ms: u64,
    camera_override: Option<&SceneCamera3D>,
) -> Option<Scene3DWorkItem> {
    let frame_def = entry.def.frames.get(frame_name)?;
    let light_params = extract_light_params(&entry.def.lights);
    let objects = match frame_def {
        FrameDef::Static(static_def) => build_object_specs(
            &static_def.show,
            &entry.def.objects,
            &entry.def.materials,
            &entry.def.camera,
            camera_override,
            &light_params,
            None,
            &[],
            0.0,
        ),
        FrameDef::Clip(clip) => {
            let t = clip_progress_at(elapsed_ms, clip.clip.duration_ms as u64);
            build_object_specs(
                &clip.show,
                &entry.def.objects,
                &entry.def.materials,
                &entry.def.camera,
                camera_override,
                &light_params,
                clip.clip.orbit_origin,
                &clip.clip.tweens,
                t,
            )
        }
    };

    Some(Scene3DWorkItem {
        src: String::new(),
        frame_id: frame_name.to_string(),
        viewport_w: entry.def.viewport.width,
        viewport_h: entry.def.viewport.height,
        objects,
    })
}
