//! Engine runtime configuration and settings.
//!
//! This module re-exports types from engine-runtime for backward compatibility.

pub use engine_runtime::*;

use crate::scene::Scene;

/// Resolves a scene-level render-size override against the current output dimensions.
///
/// Returns world-space render dimensions and does not apply any UI presentation multiplier.
/// Returns `None` when the scene does not declare an override.
pub fn scene_render_size_override(
    _settings: &RuntimeSettings,
    scene: &Scene,
    output_width: u16,
    output_height: u16,
) -> Option<(u16, u16)> {
    let render_size = parse_render_size(scene.virtual_size_override.as_deref()?)?;
    Some(render_size.resolve(output_width, output_height))
}

/// Computes startup buffer layout while honoring a scene-level render-size override.
pub fn buffer_layout_for_scene(
    settings: &RuntimeSettings,
    scene: &Scene,
    output_width: u16,
    output_height: u16,
) -> BufferLayout {
    let mut layout = settings.buffer_layout(output_width, output_height);
    if let Some((world_width, world_height)) =
        scene_render_size_override(settings, scene, output_width, output_height)
    {
        layout.world_width = world_width;
        layout.world_height = world_height;
        let (ui_width, ui_height) = settings
            .ui_render_size
            .map(|size| size.resolve(output_width, output_height))
            .unwrap_or((world_width, world_height));
        let (ui_layout_width, ui_layout_height) = settings
            .ui_layout_size
            .map(|size| size.resolve(output_width, output_height))
            .unwrap_or((ui_width, ui_height));
        layout.ui_width = ui_width;
        layout.ui_height = ui_height;
        layout.ui_layout_width = ui_layout_width;
        layout.ui_layout_height = ui_layout_height;
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
            world_render_size: RenderSize::Fixed {
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
            world_render_size: RenderSize::Fixed {
                width: 120,
                height: 30,
            },
            ..RuntimeSettings::default()
        };

        let layout = buffer_layout_for_scene(&settings, &scene, 120, 30);
        assert_eq!(layout.world_width, 180);
        assert_eq!(layout.world_height, 30);
        assert_eq!(layout.ui_width, 180);
        assert_eq!(layout.ui_height, 30);
        assert_eq!(layout.output_width, 120);
        assert_eq!(layout.output_height, 30);
    }

    #[test]
    fn buffer_layout_for_scene_keeps_explicit_ui_target_after_world_override() {
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
            world_render_size: RenderSize::Fixed {
                width: 120,
                height: 30,
            },
            ui_render_size: Some(RenderSize::Fixed {
                width: 360,
                height: 60,
            }),
            ui_layout_size: Some(RenderSize::Fixed {
                width: 360,
                height: 60,
            }),
            ..RuntimeSettings::default()
        };

        let layout = buffer_layout_for_scene(&settings, &scene, 120, 30);
        assert_eq!(layout.world_width, 180);
        assert_eq!(layout.world_height, 30);
        assert_eq!(layout.ui_width, 360);
        assert_eq!(layout.ui_height, 60);
        assert_eq!(layout.ui_layout_width, 360);
        assert_eq!(layout.ui_layout_height, 60);
    }
}
