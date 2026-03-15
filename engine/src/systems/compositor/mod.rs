mod effect_applicator;
mod sprite_renderer;
mod layer_compositor;

use crossterm::style::Color;
use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::{apply_effect, Region};
use crate::scene::Scene;
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;

pub fn compositor_system(world: &mut World) {
    let (bg, mut layers, current_stage, step_idx, elapsed_ms, scene_elapsed_ms, scene_effects, scene_step_dur) = {
        let scene = match world.get::<Scene>() { Some(s) => s, None => return };
        let animator = world.get::<Animator>();
        let stage   = animator.map(|a| a.stage.clone()).unwrap_or_default();
        let step    = animator.map(|a| a.step_idx).unwrap_or(0);
        let elapsed = animator.map(|a| a.elapsed_ms).unwrap_or(0);
        let scene_elapsed = animator.map(|a| a.scene_elapsed_ms).unwrap_or(0);

        let bg     = scene.bg_colour.as_ref().map(Color::from).unwrap_or(TRUE_BLACK);
        let layers = scene.layers.clone();

        let (scene_effects, scene_step_dur) = match &stage {
            SceneStage::OnEnter => {
                let st = scene.stages.on_enter.steps.get(step);
                (st.map(|s| s.effects.clone()).unwrap_or_default(), st.map(|s| s.duration_ms()).unwrap_or(0))
            }
            SceneStage::OnIdle => {
                let st = scene.stages.on_idle.steps.get(step);
                (st.map(|s| s.effects.clone()).unwrap_or_default(), st.map(|s| s.duration_ms()).unwrap_or(0))
            }
            SceneStage::OnLeave => {
                let st = scene.stages.on_leave.steps.get(step);
                (st.map(|s| s.effects.clone()).unwrap_or_default(), st.map(|s| s.duration_ms()).unwrap_or(0))
            }
            SceneStage::Done => (Vec::new(), 0),
        };

        (bg, layers, stage, step, elapsed, scene_elapsed, scene_effects, scene_step_dur)
    };

    let buffer = match world.get_mut::<Buffer>() { Some(b) => b, None => return };

    buffer.fill(bg);

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    layer_compositor::composite_layers(
        &mut layers,
        scene_w,
        scene_h,
        &current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        buffer,
    );

    let scene_progress = if scene_step_dur == 0 { 0.0_f32 }
                         else { (elapsed_ms as f32 / scene_step_dur as f32).clamp(0.0, 1.0) };
    let full_region = Region::full(buffer);
    for effect in &scene_effects {
        apply_effect(effect, scene_progress, full_region, buffer);
    }
}
