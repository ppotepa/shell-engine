mod effect_applicator;
mod grid_tracks;
mod image_render;
mod layer_compositor;
mod sprite_renderer;
mod text_render;

use crate::assets::AssetRoot;
use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::{apply_effect, Region};
use crate::scene::SceneRenderedMode;
use crate::scene_runtime::{ObjectRuntimeState, SceneRuntime, TargetResolver};
use crate::services::EngineWorldAccess;
use crate::systems::animator::SceneStage;
use crate::world::World;
use crossterm::style::Color;
use std::collections::BTreeMap;

pub fn compositor_system(world: &mut World) {
    let asset_root = world.asset_root().cloned();
    let runtime_mode_override = world
        .runtime_settings()
        .and_then(|s| s.renderer_mode_override);

    let (
        bg,
        mut layers,
        target_resolver,
        object_states,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        scene_effects,
        scene_step_dur,
        rendered_mode,
    ) = {
        let scene = match world.scene_runtime() {
            Some(runtime) => runtime.scene(),
            None => return,
        };
        let target_resolver = world
            .scene_runtime()
            .map(SceneRuntime::target_resolver)
            .unwrap_or_default();
        let object_states = world
            .scene_runtime()
            .map(SceneRuntime::object_states_snapshot)
            .unwrap_or_default();
        let animator = world.animator();
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
            target_resolver,
            object_states,
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
        .runtime_settings()
        .map(|s| s.use_virtual_buffer)
        .unwrap_or(false);

    if use_virtual {
        let buffer = match world.virtual_buffer_mut() {
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
                    &target_resolver,
                    &object_states,
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
                    &target_resolver,
                    &object_states,
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

    let buffer = match world.buffer_mut() {
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
                &target_resolver,
                &object_states,
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
                &target_resolver,
                &object_states,
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
    target_resolver: &TargetResolver,
    object_states: &BTreeMap<String, ObjectRuntimeState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[crate::scene::Effect],
    scene_step_dur: u64,
    buffer: &mut Buffer,
) {
    buffer.fill(bg);
    let scene_state = object_states
        .get(target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return;
    }
    let mut object_regions = BTreeMap::new();
    object_regions.insert(
        target_resolver.scene_object_id().to_string(),
        offset_region(
            buffer.width,
            buffer.height,
            scene_state.offset_x,
            scene_state.offset_y,
        ),
    );

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    layer_compositor::composite_layers(
        layers,
        scene_w,
        scene_h,
        scene_rendered_mode,
        asset_root,
        Some(target_resolver),
        &mut object_regions,
        scene_state.offset_x,
        scene_state.offset_y,
        object_states,
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
        let region = target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(effect, scene_progress, region, buffer);
    }
}

fn composite_scene_halfblock(
    bg: Color,
    layers: &mut Vec<crate::scene::Layer>,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &BTreeMap<String, ObjectRuntimeState>,
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
        target_resolver,
        object_states,
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

fn offset_region(width: u16, height: u16, offset_x: i32, offset_y: i32) -> Region {
    let origin_x = offset_x.max(0) as u16;
    let origin_y = offset_y.max(0) as u16;
    let clipped_w = width.saturating_sub(offset_x.unsigned_abs().min(width as u32) as u16);
    let clipped_h = height.saturating_sub(offset_y.unsigned_abs().min(height as u32) as u16);
    Region {
        x: origin_x,
        y: origin_y,
        width: clipped_w,
        height: clipped_h,
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
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene::Scene;
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::animator::{Animator, SceneStage};
    use crate::world::World;

    use super::{compositor_system, pack_halfblock_buffer};

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

    #[test]
    fn higher_z_layer_renders_above_background_layer_effects() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: bg
    z_index: -1
    stages:
      on_idle:
        steps:
          - effects:
              - name: clear-to-colour
                duration: 1
                params:
                  colour: blue
    sprites: []
  - name: fg
    z_index: 1
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(Buffer::new(3, 1));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 1,
            stage_elapsed_ms: 1,
            scene_elapsed_ms: 1,
        });

        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("foreground text").symbol, 'A');
        assert_eq!(
            buffer.get(1, 0).expect("background neighbour").bg,
            Color::Blue
        );
    }
}
