//! Engine runtime configuration and settings.
//!
//! This module re-exports types from engine-runtime for backward compatibility.

pub use engine_runtime::*;

use crate::scene::Scene;

/// Resolves a scene-level render-size override against the current output dimensions.
///
/// Returns `None` when the scene does not declare an override.
pub fn scene_render_size_override(
    _settings: &RuntimeSettings,
    scene: &Scene,
    output_width: u16,
    output_height: u16,
) -> Option<(u16, u16)> {
    let size_override = scene.virtual_size_override.as_deref()?;
    let (width, height, matches_output) = parse_virtual_size_str(size_override)?;
    Some(if matches_output {
        (output_width.max(1), output_height.max(1))
    } else {
        (width.max(1), height.max(1))
    })
}

/// Computes startup buffer layout while honoring a scene-level render-size override.
pub fn buffer_layout_for_scene(
    settings: &RuntimeSettings,
    scene: &Scene,
    output_width: u16,
    output_height: u16,
) -> BufferLayout {
    let mut layout = settings.buffer_layout(output_width, output_height);
    if let Some((render_width, render_height)) =
        scene_render_size_override(settings, scene, output_width, output_height)
    {
        layout.render_width = render_width;
        layout.render_height = render_height;
    }
    layout
}

#[cfg(test)]
mod tests {
    use super::{buffer_layout_for_scene, scene_render_size_override, RenderSize, RuntimeSettings};
    use crate::scene::Scene;

    #[test]
    fn scene_override_resolves_fixed_size() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: intro
title: Intro
virtual-size-override: 180x30
layers: []
"#,
        )
        .expect("scene");

        let settings = RuntimeSettings {
            render_size: RenderSize::Fixed {
                width: 120,
                height: 30,
            },
            ..RuntimeSettings::default()
        };

        assert_eq!(
            scene_render_size_override(&settings, &scene, 120, 30),
            Some((180, 30))
        );
    }

    #[test]
    fn buffer_layout_for_scene_uses_entry_scene_override() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: intro
title: Intro
virtual-size-override: 180x30
layers: []
"#,
        )
        .expect("scene");

        let settings = RuntimeSettings {
            render_size: RenderSize::Fixed {
                width: 120,
                height: 30,
            },
            ..RuntimeSettings::default()
        };

        let layout = buffer_layout_for_scene(&settings, &scene, 120, 30);
        assert_eq!(layout.render_width, 180);
        assert_eq!(layout.render_height, 30);
        assert_eq!(layout.output_width, 120);
        assert_eq!(layout.output_height, 30);
    }
}
