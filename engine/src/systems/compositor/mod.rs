mod effect_applicator;
mod grid_tracks;
mod image_render;
mod layer_compositor;
mod sprite_renderer;
mod text_render;

use crate::assets::AssetRoot;
use crate::buffer::{Buffer, Cell, VirtualBuffer, TRUE_BLACK};
use crate::effects::{apply_effect, Region};
use crate::runtime_settings::RuntimeSettings;
use crate::scene::{Scene, SceneRenderedMode};
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;
use crossterm::style::Color;

pub fn compositor_system(world: &mut World) {
    let asset_root = world.get::<AssetRoot>().cloned();
    let runtime_mode_override = world
        .get::<RuntimeSettings>()
        .and_then(|s| s.renderer_mode_override);

    let (
        bg,
        mut layers,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        scene_effects,
        scene_step_dur,
        rendered_mode,
    ) = {
        let scene = match world.get::<Scene>() {
            Some(s) => s,
            None => return,
        };
        let animator = world.get::<Animator>();
        let stage = animator.map(|a| a.stage.clone()).unwrap_or_default();
        let step = animator.map(|a| a.step_idx).unwrap_or(0);
        let elapsed = animator.map(|a| a.elapsed_ms).unwrap_or(0);
        let scene_elapsed = animator.map(|a| a.scene_elapsed_ms).unwrap_or(0);

        let bg = scene
            .bg_colour
            .as_ref()
            .map(Color::from)
            .unwrap_or(TRUE_BLACK);
        let layers = scene.layers.clone();

        let (scene_effects, scene_step_dur) = match &stage {
            SceneStage::OnEnter => {
                let st = scene.stages.on_enter.steps.get(step);
                (
                    st.map(|s| s.effects.clone()).unwrap_or_default(),
                    st.map(|s| s.duration_ms()).unwrap_or(0),
                )
            }
            SceneStage::OnIdle => {
                let st = scene.stages.on_idle.steps.get(step);
                (
                    st.map(|s| s.effects.clone()).unwrap_or_default(),
                    st.map(|s| s.duration_ms()).unwrap_or(0),
                )
            }
            SceneStage::OnLeave => {
                let st = scene.stages.on_leave.steps.get(step);
                (
                    st.map(|s| s.effects.clone()).unwrap_or_default(),
                    st.map(|s| s.duration_ms()).unwrap_or(0),
                )
            }
            SceneStage::Done => (Vec::new(), 0),
        };

        (
            bg,
            layers,
            stage,
            step,
            elapsed,
            scene_elapsed,
            scene_effects,
            scene_step_dur,
            runtime_mode_override.unwrap_or(scene.rendered_mode),
        )
    };

    let use_virtual = world
        .get::<RuntimeSettings>()
        .map(|s| s.use_virtual_buffer)
        .unwrap_or(false);

    if use_virtual {
        let buffer = match world.get_mut::<VirtualBuffer>() {
            Some(v) => &mut v.0,
            None => return,
        };
        match rendered_mode {
            SceneRenderedMode::Cell | SceneRenderedMode::QuadBlock | SceneRenderedMode::Braille => {
                composite_scene(
                    bg,
                    &mut layers,
                    rendered_mode,
                    asset_root.as_ref(),
                    &current_stage,
                    step_idx,
                    elapsed_ms,
                    scene_elapsed_ms,
                    &scene_effects,
                    scene_step_dur,
                    buffer,
                );
            }
            SceneRenderedMode::HalfBlock => {
                composite_scene_halfblock(
                    bg,
                    &mut layers,
                    rendered_mode,
                    asset_root.as_ref(),
                    &current_stage,
                    step_idx,
                    elapsed_ms,
                    scene_elapsed_ms,
                    &scene_effects,
                    scene_step_dur,
                    buffer,
                );
            }
        }
        return;
    }

    let buffer = match world.get_mut::<Buffer>() {
        Some(b) => b,
        None => return,
    };
    match rendered_mode {
        SceneRenderedMode::Cell | SceneRenderedMode::QuadBlock | SceneRenderedMode::Braille => {
            composite_scene(
                bg,
                &mut layers,
                rendered_mode,
                asset_root.as_ref(),
                &current_stage,
                step_idx,
                elapsed_ms,
                scene_elapsed_ms,
                &scene_effects,
                scene_step_dur,
                buffer,
            );
        }
        SceneRenderedMode::HalfBlock => {
            composite_scene_halfblock(
                bg,
                &mut layers,
                rendered_mode,
                asset_root.as_ref(),
                &current_stage,
                step_idx,
                elapsed_ms,
                scene_elapsed_ms,
                &scene_effects,
                scene_step_dur,
                buffer,
            );
        }
    }
}

fn composite_scene(
    bg: Color,
    layers: &mut Vec<crate::scene::Layer>,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[crate::scene::Effect],
    scene_step_dur: u64,
    buffer: &mut Buffer,
) {
    buffer.fill(bg);

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    layer_compositor::composite_layers(
        layers,
        scene_w,
        scene_h,
        scene_rendered_mode,
        asset_root,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        buffer,
    );

    let scene_progress = if scene_step_dur == 0 {
        0.0_f32
    } else {
        (elapsed_ms as f32 / scene_step_dur as f32).clamp(0.0, 1.0)
    };
    let full_region = Region::full(buffer);
    for effect in scene_effects {
        apply_effect(effect, scene_progress, full_region, buffer);
    }
}

fn composite_scene_halfblock(
    bg: Color,
    layers: &mut Vec<crate::scene::Layer>,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[crate::scene::Effect],
    scene_step_dur: u64,
    target: &mut Buffer,
) {
    let mut virtual_buf = Buffer::new(target.width, target.height.saturating_mul(2));
    composite_scene(
        bg,
        layers,
        scene_rendered_mode,
        asset_root,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        scene_effects,
        scene_step_dur,
        &mut virtual_buf,
    );
    pack_halfblock_buffer(&virtual_buf, target, bg);
}

fn pack_halfblock_buffer(source: &Buffer, target: &mut Buffer, fallback_bg: Color) {
    target.fill(fallback_bg);
    for y in 0..target.height {
        let top_y = y.saturating_mul(2);
        let bottom_y = top_y.saturating_add(1);
        for x in 0..target.width {
            let top = cell_or_blank(source, x, top_y, fallback_bg);
            let bottom = if bottom_y < source.height {
                cell_or_blank(source, x, bottom_y, fallback_bg)
            } else {
                Cell::blank(resolve_bg(top.bg, fallback_bg))
            };

            let top_bg = resolve_bg(top.bg, fallback_bg);
            let bottom_bg = resolve_bg(bottom.bg, fallback_bg);
            let out_bg = select_background(top_bg, bottom_bg, fallback_bg);
            let top_on = top.symbol != ' ';
            let bottom_on = bottom.symbol != ' ';

            let (symbol, fg, bg) = match (top_on, bottom_on) {
                (false, false) => (' ', TRUE_BLACK, out_bg),
                (true, false) => ('▀', top.fg, bottom_bg),
                (false, true) => ('▄', bottom.fg, top_bg),
                (true, true) => {
                    if top.fg == bottom.fg {
                        ('█', top.fg, out_bg)
                    } else {
                        ('▀', top.fg, bottom.fg)
                    }
                }
            };
            target.set(x, y, symbol, fg, bg);
        }
    }
}

fn cell_or_blank(buffer: &Buffer, x: u16, y: u16, fallback_bg: Color) -> Cell {
    buffer
        .get(x, y)
        .cloned()
        .unwrap_or_else(|| Cell::blank(fallback_bg))
}

fn resolve_bg(colour: Color, fallback_bg: Color) -> Color {
    if matches!(colour, Color::Reset) {
        fallback_bg
    } else {
        colour
    }
}

fn select_background(top: Color, bottom: Color, fallback_bg: Color) -> Color {
    if top != fallback_bg {
        top
    } else {
        bottom
    }
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;

    use crate::buffer::{Buffer, TRUE_BLACK};

    use super::pack_halfblock_buffer;

    #[test]
    fn packs_two_virtual_rows_into_one_terminal_cell() {
        let mut source = Buffer::new(1, 2);
        source.fill(TRUE_BLACK);
        source.set(0, 0, '#', Color::Red, TRUE_BLACK);
        source.set(0, 1, '#', Color::Blue, TRUE_BLACK);

        let mut target = Buffer::new(1, 1);
        pack_halfblock_buffer(&source, &mut target, TRUE_BLACK);

        let cell = target.get(0, 0).expect("cell exists");
        assert_eq!(cell.symbol, '▀');
        assert_eq!(cell.fg, Color::Red);
        assert_eq!(cell.bg, Color::Blue);
    }

    #[test]
    fn keeps_background_for_empty_virtual_pixels() {
        let mut source = Buffer::new(1, 2);
        source.fill(Color::DarkGrey);

        let mut target = Buffer::new(1, 1);
        pack_halfblock_buffer(&source, &mut target, TRUE_BLACK);

        let cell = target.get(0, 0).expect("cell exists");
        assert_eq!(cell.symbol, ' ');
        assert_eq!(cell.bg, Color::DarkGrey);
    }
}
