use engine_3d::scene3d_format::{FrameDef, Scene3DDefinition};

use super::{build_object_specs, expand_frame_samples, extract_light_params, ObjectRenderSpec};

#[derive(Debug, Clone)]
pub struct Scene3DWorkItem {
    pub src: String,
    pub frame_id: String,
    pub viewport_w: u16,
    pub viewport_h: u16,
    pub objects: Vec<ObjectRenderSpec>,
}

pub fn build_work_items(src: &str, def: &Scene3DDefinition) -> Vec<Scene3DWorkItem> {
    let vw = def.viewport.width;
    let vh = def.viewport.height;
    let light_params = extract_light_params(&def.lights);

    let mut items = Vec::new();
    for sample in expand_frame_samples(def) {
        let Some(frame_def) = def.frames.get(sample.base_frame_id.as_str()) else {
            continue;
        };
        match frame_def {
            FrameDef::Static(static_def) => {
                let objects = build_object_specs(
                    &static_def.show,
                    &def.objects,
                    &def.materials,
                    &def.camera,
                    None,
                    &light_params,
                    None,
                    &[],
                    0.0,
                );
                items.push(Scene3DWorkItem {
                    src: src.to_string(),
                    frame_id: sample.output_frame_id,
                    viewport_w: vw,
                    viewport_h: vh,
                    objects,
                });
            }
            FrameDef::Clip(clip_def) => {
                let objects = build_object_specs(
                    &clip_def.show,
                    &def.objects,
                    &def.materials,
                    &def.camera,
                    None,
                    &light_params,
                    clip_def.clip.orbit_origin,
                    &clip_def.clip.tweens,
                    sample.t,
                );
                items.push(Scene3DWorkItem {
                    src: src.to_string(),
                    frame_id: sample.output_frame_id,
                    viewport_w: vw,
                    viewport_h: vh,
                    objects,
                });
            }
        }
    }

    items
}
