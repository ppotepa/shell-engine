use super::effect_applicator::apply_layer_effects;
use super::sprite_renderer::render_sprites;
use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::scene::{Layer, SceneRenderedMode};
use crate::systems::animator::SceneStage;
use crossterm::style::Color;

/// Composite all visible layers onto the scene framebuffer.
pub fn composite_layers(
    layers: &mut Vec<Layer>,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    buffer: &mut Buffer,
) {
    layers.sort_by_key(|l| l.z_index);

    for layer in layers.iter_mut() {
        if !layer.visible {
            continue;
        }

        let mut layer_buf = Buffer::new(scene_w, scene_h);
        layer_buf.fill(Color::Reset);

        render_sprites(
            layer,
            scene_w,
            scene_h,
            scene_rendered_mode,
            asset_root,
            scene_elapsed_ms,
            current_stage,
            step_idx,
            elapsed_ms,
            &mut layer_buf,
        );

        apply_layer_effects(
            layer,
            current_stage,
            step_idx,
            elapsed_ms,
            scene_elapsed_ms,
            &mut layer_buf,
        );

        // Blit layer onto scene framebuffer — skip transparent pixels
        for ly in 0..scene_h {
            for lx in 0..scene_w {
                if let Some(cell) = layer_buf.get(lx, ly) {
                    if !(cell.symbol == ' ' && cell.bg == Color::Reset) {
                        let sym = cell.symbol;
                        let fg = cell.fg;
                        let cbg = cell.bg;
                        buffer.set(lx, ly, sym, fg, cbg);
                    }
                }
            }
        }
    }
}
