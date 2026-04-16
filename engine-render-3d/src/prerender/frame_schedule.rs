use engine_3d::scene3d_format::{FrameDef, Scene3DDefinition};

pub struct FrameSample {
    pub base_frame_id: String,
    pub output_frame_id: String,
    pub t: f32,
}

/// Convert elapsed time to normalized clip progress in `[0, 1]`.
pub fn clip_progress_at(elapsed_ms: u64, duration_ms: u64) -> f32 {
    if duration_ms == 0 {
        0.0
    } else {
        (elapsed_ms % duration_ms) as f32 / duration_ms as f32
    }
}

/// Expand Scene3D frame definitions into concrete sample outputs.
///
/// - Static frames produce one sample with `t = 0.0`.
/// - Clip frames produce `keyframes` samples named `{frame}-{idx}` with normalized `t`.
pub fn expand_frame_samples(def: &Scene3DDefinition) -> Vec<FrameSample> {
    let mut samples = Vec::new();
    for (frame_id, frame_def) in &def.frames {
        match frame_def {
            FrameDef::Static(_) => {
                samples.push(FrameSample {
                    base_frame_id: frame_id.clone(),
                    output_frame_id: frame_id.clone(),
                    t: 0.0,
                });
            }
            FrameDef::Clip(clip_def) => {
                let n = clip_def.clip.keyframes.max(1);
                for kf in 0..n {
                    let t = if n <= 1 {
                        0.0
                    } else {
                        kf as f32 / (n - 1) as f32
                    };
                    samples.push(FrameSample {
                        base_frame_id: frame_id.clone(),
                        output_frame_id: format!("{frame_id}-{kf}"),
                        t,
                    });
                }
            }
        }
    }
    samples
}
