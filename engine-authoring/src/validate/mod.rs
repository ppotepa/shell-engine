//! Authoring validation helpers.
//!
//! This module will contain reusable authoring checks shared by tests, editor
//! tooling, and future compile-time diagnostics.

mod render3d;

use engine_core::scene::{Scene, Sprite, TextOverflowMode, TextWrapMode};
pub use render3d::{validate_render_scene3d_document, Render3dDiagnostic};

/// Validation diagnostic for sprite timeline issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineDiagnostic {
    /// Sprite appear_at_ms is after on_enter stage duration (will never be visible during cutscene)
    SpriteAppearsAfterSceneEnd {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        scene_duration_ms: u64,
    },
    /// Sprite disappear_at_ms is before appear_at_ms (always hidden)
    SpriteDisappearsBeforeAppear {
        layer_name: String,
        sprite_index: usize,
        appear_at_ms: u64,
        disappear_at_ms: u64,
    },
}

/// Validation diagnostic for text layout semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextLayoutDiagnostic {
    /// Ellipsis was requested without any authored width/line bound.
    EllipsisWithoutBounds {
        layer_name: String,
        sprite_index: usize,
    },
    /// A line clamp was set without a wrap contract to give it multi-line meaning.
    LineClampWithoutWrap {
        layer_name: String,
        sprite_index: usize,
        line_clamp: u16,
    },
    /// Reserved width smaller than the visible max width defeats the reserved layout footprint.
    ReserveWidthTooSmall {
        layer_name: String,
        sprite_index: usize,
        reserve_width_ch: u16,
        max_width: u16,
    },
}

/// Validates sprite timeline against scene duration.
///
/// Returns warnings for sprites that will never be visible during on_enter stage
/// (the primary cutscene/intro timing for most scenes).
///
/// # Checks
/// - sprite `appear_at_ms` >= on_enter duration → sprite never visible
/// - sprite `disappear_at_ms` <= `appear_at_ms` → sprite always hidden
///
/// # Notes
/// This validation focuses on on_enter because that's where most authored
/// sprite timing lives. Sprites visible only during on_idle or on_leave
/// are uncommon and require runtime state control (layer.visible or Rhai).
pub fn validate_sprite_timeline(scene: &Scene) -> Vec<TimelineDiagnostic> {
    let mut diagnostics = Vec::new();
    let scene_duration = scene.on_enter_duration_ms();

    for layer in &scene.layers {
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            let (appear_at, disappear_at) = match sprite {
                Sprite::Text {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Image {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Obj {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Planet {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                Sprite::Vector {
                    appear_at_ms,
                    disappear_at_ms,
                    ..
                } => (*appear_at_ms, *disappear_at_ms),
                // Panel, Grid, Flex, Scene3D don't have disappear_at_ms timeline validation
                Sprite::Panel { .. }
                | Sprite::Grid { .. }
                | Sprite::Flex { .. }
                | Sprite::Scene3D { .. } => continue,
            };

            let appear = appear_at.unwrap_or(0);

            // Check if sprite appears after scene ends
            if scene_duration > 0 && appear >= scene_duration {
                diagnostics.push(TimelineDiagnostic::SpriteAppearsAfterSceneEnd {
                    layer_name: layer.name.clone(),
                    sprite_index: sprite_idx,
                    appear_at_ms: appear,
                    scene_duration_ms: scene_duration,
                });
            }

            // Check if sprite disappears before appearing
            if let Some(disappear) = disappear_at {
                if disappear <= appear {
                    diagnostics.push(TimelineDiagnostic::SpriteDisappearsBeforeAppear {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        appear_at_ms: appear,
                        disappear_at_ms: disappear,
                    });
                }
            }
        }
    }

    diagnostics
}

/// Validates authored text layout semantics for likely HUD mistakes.
///
/// These checks are warning-only and focus on contracts that authors are likely
/// to assume exist:
/// - `overflow-mode: ellipsis` needs `max-width` or `line-clamp`
/// - `line-clamp` expects `wrap-mode: word|char`
/// - `reserve-width-ch` should not be smaller than `max-width`
pub fn validate_text_layout_semantics(scene: &Scene) -> Vec<TextLayoutDiagnostic> {
    let mut diagnostics = Vec::new();

    for layer in &scene.layers {
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            let Sprite::Text {
                max_width,
                overflow_mode,
                wrap_mode,
                line_clamp,
                reserve_width_ch,
                ..
            } = sprite
            else {
                continue;
            };

            if matches!(overflow_mode, TextOverflowMode::Ellipsis)
                && max_width.is_none()
                && line_clamp.is_none()
            {
                diagnostics.push(TextLayoutDiagnostic::EllipsisWithoutBounds {
                    layer_name: layer.name.clone(),
                    sprite_index: sprite_idx,
                });
            }

            if let Some(clamp) = line_clamp {
                if matches!(wrap_mode, TextWrapMode::None) {
                    diagnostics.push(TextLayoutDiagnostic::LineClampWithoutWrap {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        line_clamp: *clamp,
                    });
                }
            }

            if let (Some(reserved), Some(max_width)) = (reserve_width_ch, max_width) {
                if reserved < max_width {
                    diagnostics.push(TextLayoutDiagnostic::ReserveWidthTooSmall {
                        layer_name: layer.name.clone(),
                        sprite_index: sprite_idx,
                        reserve_width_ch: *reserved,
                        max_width: *max_width,
                    });
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::scene::{Layer, Scene, SceneStages, Sprite, Stage, Step};

    fn make_test_scene(on_enter_duration: u64) -> Scene {
        Scene {
            id: "test".into(),
            title: "Test".into(),
            cutscene: true,
            target_fps: None,
            space: Default::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: None,
            view: None,
            virtual_size_override: None,
            bg_colour: None,
            stages: SceneStages {
                on_enter: Stage {
                    trigger: Default::default(),
                    steps: vec![Step {
                        duration: Some(on_enter_duration),
                        effects: vec![],
                    }],
                    looping: false,
                },
                on_idle: Default::default(),
                on_leave: Default::default(),
            },
            behaviors: vec![],
            audio: Default::default(),
            gui: Default::default(),
            ui: Default::default(),
            layers: vec![],
            menu_options: vec![],
            input: Default::default(),
            postfx: vec![],
            next: None,
            prerender: false,
            palette_bindings: vec![],
            game_state_bindings: vec![],
        }
    }

    fn make_text_sprite(appear_at_ms: Option<u64>, disappear_at_ms: Option<u64>) -> Sprite {
        Sprite::Text {
            id: Some("test".into()),
            content: "test".into(),
            x: 0,
            y: 0,
            z_index: 0,
            grid_row: 0,
            grid_col: 0,
            row_span: 1,
            col_span: 1,
            size: None,
            font: None,
            force_font_mode: None,
            align_x: None,
            align_y: None,
            fg_colour: None,
            bg_colour: None,
            appear_at_ms,
            disappear_at_ms,
            reveal_ms: None,
            hide_on_leave: false,
            visible: true,
            stages: Default::default(),
            animations: vec![],
            behaviors: vec![],
            glow: None,
            text_transform: Default::default(),
            max_width: None,
            overflow_mode: Default::default(),
            wrap_mode: Default::default(),
            line_clamp: None,
            reserve_width_ch: None,
            line_height: 1,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    #[test]
    fn valid_sprite_timeline_passes() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(100), Some(5000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert!(diags.is_empty(), "Valid timeline should pass");
    }

    #[test]
    fn sprite_appears_after_scene_end_warns() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(8200), Some(10000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TimelineDiagnostic::SpriteAppearsAfterSceneEnd { .. }
        ));
    }

    #[test]
    fn sprite_disappears_before_appear_warns() {
        let mut scene = make_test_scene(6000);
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![make_text_sprite(Some(3000), Some(1000))],
            ..Default::default()
        });

        let diags = validate_sprite_timeline(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TimelineDiagnostic::SpriteDisappearsBeforeAppear { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_ellipsis_without_bounds() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text { overflow_mode, .. } = &mut sprite {
            *overflow_mode = TextOverflowMode::Ellipsis;
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::EllipsisWithoutBounds { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_line_clamp_without_wrap() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text { line_clamp, .. } = &mut sprite {
            *line_clamp = Some(2);
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::LineClampWithoutWrap { .. }
        ));
    }

    #[test]
    fn text_layout_semantics_warn_for_reserved_width_smaller_than_max_width() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text {
            max_width,
            reserve_width_ch,
            wrap_mode,
            ..
        } = &mut sprite
        {
            *max_width = Some(12);
            *reserve_width_ch = Some(8);
            *wrap_mode = TextWrapMode::Word;
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert_eq!(diags.len(), 1);
        assert!(matches!(
            diags[0],
            TextLayoutDiagnostic::ReserveWidthTooSmall {
                reserve_width_ch: 8,
                max_width: 12,
                ..
            }
        ));
    }

    #[test]
    fn valid_text_layout_semantics_pass() {
        let mut scene = make_test_scene(6000);
        let mut sprite = make_text_sprite(None, None);
        if let Sprite::Text {
            max_width,
            overflow_mode,
            wrap_mode,
            line_clamp,
            reserve_width_ch,
            ..
        } = &mut sprite
        {
            *max_width = Some(24);
            *overflow_mode = TextOverflowMode::Ellipsis;
            *wrap_mode = TextWrapMode::Word;
            *line_clamp = Some(2);
            *reserve_width_ch = Some(24);
        }
        scene.layers.push(Layer {
            name: "main".into(),
            sprites: vec![sprite],
            ..Default::default()
        });

        let diags = validate_text_layout_semantics(&scene);
        assert!(diags.is_empty(), "Valid text layout semantics should pass");
    }
}
