use crate::effects::EffectDispatcher;
use crate::scene::{LayerStages, Scene, Stage};
use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct EffectRegistryCheck;

impl StartupCheck for EffectRegistryCheck {
    fn name(&self) -> &'static str {
        "effect-registry"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let dispatcher = EffectDispatcher::new();
        let mut unknown = Vec::new();

        for sf in scenes {
            collect_scene_unknown_effects(&sf.scene, &sf.path, &dispatcher, &mut unknown);
        }

        if !unknown.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("unknown effects:\n{}", unknown.join("\n")),
            });
        }

        report.add_info(
            self.name(),
            format!("effect names verified in {} scenes", scenes.len()),
        );
        Ok(())
    }
}

fn collect_scene_unknown_effects(
    scene: &Scene,
    path: &str,
    dispatcher: &EffectDispatcher,
    out: &mut Vec<String>,
) {
    collect_stage_unknown_effects(
        &scene.stages.on_enter,
        path,
        &scene.id,
        "scene.on_enter",
        dispatcher,
        out,
    );
    collect_stage_unknown_effects(
        &scene.stages.on_idle,
        path,
        &scene.id,
        "scene.on_idle",
        dispatcher,
        out,
    );
    collect_stage_unknown_effects(
        &scene.stages.on_leave,
        path,
        &scene.id,
        "scene.on_leave",
        dispatcher,
        out,
    );

    for (layer_idx, layer) in scene.layers.iter().enumerate() {
        collect_layer_unknown_effects(
            &layer.stages,
            path,
            &scene.id,
            &format!("layer[{layer_idx}] `{}`", layer.name),
            dispatcher,
            out,
        );
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            let mut node_idx = 0usize;
            sprite.walk_recursive(&mut |node| {
                collect_layer_unknown_effects(
                    node.stages(),
                    path,
                    &scene.id,
                    &format!("layer[{layer_idx}].sprite[{sprite_idx}].node[{node_idx}]"),
                    dispatcher,
                    out,
                );
                node_idx += 1;
            });
        }
    }
}

fn collect_layer_unknown_effects(
    stages: &LayerStages,
    path: &str,
    scene_id: &str,
    scope: &str,
    dispatcher: &EffectDispatcher,
    out: &mut Vec<String>,
) {
    collect_stage_unknown_effects(
        &stages.on_enter,
        path,
        scene_id,
        &format!("{scope}.on_enter"),
        dispatcher,
        out,
    );
    collect_stage_unknown_effects(
        &stages.on_idle,
        path,
        scene_id,
        &format!("{scope}.on_idle"),
        dispatcher,
        out,
    );
    collect_stage_unknown_effects(
        &stages.on_leave,
        path,
        scene_id,
        &format!("{scope}.on_leave"),
        dispatcher,
        out,
    );
}

fn collect_stage_unknown_effects(
    stage: &Stage,
    path: &str,
    scene_id: &str,
    scope: &str,
    dispatcher: &EffectDispatcher,
    out: &mut Vec<String>,
) {
    for (step_idx, step) in stage.steps.iter().enumerate() {
        for effect in &step.effects {
            if !dispatcher.supports(&effect.name) {
                out.push(format!(
                    "{path} (scene `{scene_id}`, {scope}, step {step_idx}): `{}`",
                    effect.name
                ));
            }
        }
    }
}
