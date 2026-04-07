//! Verifies that every effect name referenced in the mod's scenes is registered in the effect dispatcher.

use engine_effects::{shared_dispatcher, EffectDispatcher};
use engine_core::scene::{Effect, EffectTargetKind, LayerStages, Scene, Sprite, Stage};
use engine_error::EngineError;
use std::collections::BTreeMap;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that fails if any scene references an unknown or incompatible effect.
pub struct EffectRegistryCheck;

impl StartupCheck for EffectRegistryCheck {
    fn name(&self) -> &'static str {
        "effect-registry"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let mut unknown = Vec::new();
        let mut incompatible = Vec::new();

        for sf in scenes {
            collect_scene_effect_issues(
                &sf.scene,
                &sf.path,
                shared_dispatcher(),
                &mut unknown,
                &mut incompatible,
            );
        }

        if !unknown.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("unknown effects:\n{}", unknown.join("\n")),
            });
        }

        if !incompatible.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("incompatible effects:\n{}", incompatible.join("\n")),
            });
        }

        report.add_info(
            self.name(),
            format!("effect names verified in {} scenes", scenes.len()),
        );
        Ok(())
    }
}

fn collect_scene_effect_issues(
    scene: &Scene,
    path: &str,
    dispatcher: &EffectDispatcher,
    unknown: &mut Vec<String>,
    incompatible: &mut Vec<String>,
) {
    let target_kinds = collect_target_kinds(scene);

    collect_stage_effect_issues(
        &scene.stages.on_enter,
        path,
        &scene.id,
        "scene.on_enter",
        EffectTargetKind::Scene,
        dispatcher,
        &target_kinds,
        unknown,
        incompatible,
    );
    collect_stage_effect_issues(
        &scene.stages.on_idle,
        path,
        &scene.id,
        "scene.on_idle",
        EffectTargetKind::Scene,
        dispatcher,
        &target_kinds,
        unknown,
        incompatible,
    );
    collect_stage_effect_issues(
        &scene.stages.on_leave,
        path,
        &scene.id,
        "scene.on_leave",
        EffectTargetKind::Scene,
        dispatcher,
        &target_kinds,
        unknown,
        incompatible,
    );

    for (layer_idx, layer) in scene.layers.iter().enumerate() {
        collect_layer_effect_issues(
            &layer.stages,
            path,
            &scene.id,
            &format!("layer[{layer_idx}] `{}`", layer.name),
            EffectTargetKind::Layer,
            dispatcher,
            &target_kinds,
            unknown,
            incompatible,
        );
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            collect_sprite_effect_issues(
                sprite,
                path,
                &scene.id,
                &format!("layer[{layer_idx}].sprite[{sprite_idx}]"),
                dispatcher,
                &target_kinds,
                unknown,
                incompatible,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_layer_effect_issues(
    stages: &LayerStages,
    path: &str,
    scene_id: &str,
    scope: &str,
    placement_kind: EffectTargetKind,
    dispatcher: &EffectDispatcher,
    target_kinds: &BTreeMap<String, EffectTargetKind>,
    unknown: &mut Vec<String>,
    incompatible: &mut Vec<String>,
) {
    collect_stage_effect_issues(
        &stages.on_enter,
        path,
        scene_id,
        &format!("{scope}.on_enter"),
        placement_kind,
        dispatcher,
        target_kinds,
        unknown,
        incompatible,
    );
    collect_stage_effect_issues(
        &stages.on_idle,
        path,
        scene_id,
        &format!("{scope}.on_idle"),
        placement_kind,
        dispatcher,
        target_kinds,
        unknown,
        incompatible,
    );
    collect_stage_effect_issues(
        &stages.on_leave,
        path,
        scene_id,
        &format!("{scope}.on_leave"),
        placement_kind,
        dispatcher,
        target_kinds,
        unknown,
        incompatible,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_stage_effect_issues(
    stage: &Stage,
    path: &str,
    scene_id: &str,
    scope: &str,
    placement_kind: EffectTargetKind,
    dispatcher: &EffectDispatcher,
    target_kinds: &BTreeMap<String, EffectTargetKind>,
    unknown: &mut Vec<String>,
    incompatible: &mut Vec<String>,
) {
    for (step_idx, step) in stage.steps.iter().enumerate() {
        for effect in &step.effects {
            if !dispatcher.supports(&effect.name) {
                unknown.push(format!(
                    "{path} (scene `{scene_id}`, {scope}, step {step_idx}): `{}`",
                    effect.name
                ));
                continue;
            }

            if let Some(issue) = validate_effect_target(
                effect,
                placement_kind,
                dispatcher,
                target_kinds,
                path,
                scene_id,
                scope,
                step_idx,
            ) {
                incompatible.push(issue);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_sprite_effect_issues(
    sprite: &Sprite,
    path: &str,
    scene_id: &str,
    scope: &str,
    dispatcher: &EffectDispatcher,
    target_kinds: &BTreeMap<String, EffectTargetKind>,
    unknown: &mut Vec<String>,
    incompatible: &mut Vec<String>,
) {
    let placement_kind = sprite_target_kind(sprite);
    collect_layer_effect_issues(
        sprite.stages(),
        path,
        scene_id,
        scope,
        placement_kind,
        dispatcher,
        target_kinds,
        unknown,
        incompatible,
    );

    if let Sprite::Grid { children, .. } = sprite {
        for (child_idx, child) in children.iter().enumerate() {
            collect_sprite_effect_issues(
                child,
                path,
                scene_id,
                &format!("{scope}.child[{child_idx}]"),
                dispatcher,
                target_kinds,
                unknown,
                incompatible,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_effect_target(
    effect: &Effect,
    placement_kind: EffectTargetKind,
    dispatcher: &EffectDispatcher,
    target_kinds: &BTreeMap<String, EffectTargetKind>,
    path: &str,
    scene_id: &str,
    scope: &str,
    step_idx: usize,
) -> Option<String> {
    let resolved_kind = effect
        .params
        .target
        .as_deref()
        .and_then(|target| target_kinds.get(target).copied())
        .unwrap_or(placement_kind);

    if effect.target_kind != EffectTargetKind::Any
        && !effect.target_kind.matches_effective(resolved_kind)
    {
        return Some(format!(
            "{path} (scene `{scene_id}`, {scope}, step {step_idx}): `{}` declares target_kind `{:?}` but resolves to `{:?}`",
            effect.name, effect.target_kind, resolved_kind
        ));
    }

    if !dispatcher.supports_target_kind(&effect.name, resolved_kind) {
        return Some(format!(
            "{path} (scene `{scene_id}`, {scope}, step {step_idx}): `{}` does not support target `{:?}`",
            effect.name, resolved_kind
        ));
    }

    None
}

fn collect_target_kinds(scene: &Scene) -> BTreeMap<String, EffectTargetKind> {
    let mut target_kinds = BTreeMap::new();
    target_kinds.insert(scene.id.clone(), EffectTargetKind::Scene);

    for layer in &scene.layers {
        if !layer.name.trim().is_empty() {
            target_kinds.insert(layer.name.clone(), EffectTargetKind::Layer);
        }

        for sprite in &layer.sprites {
            collect_sprite_target_kinds(sprite, &mut target_kinds);
        }
    }

    target_kinds
}

fn collect_sprite_target_kinds(
    sprite: &Sprite,
    target_kinds: &mut BTreeMap<String, EffectTargetKind>,
) {
    if let Some(id) = sprite.id() {
        target_kinds.insert(id.to_string(), sprite_target_kind(sprite));
    }

    if let Sprite::Grid { children, .. }
    | Sprite::Flex { children, .. }
    | Sprite::Panel { children, .. } = sprite
    {
        for child in children {
            collect_sprite_target_kinds(child, target_kinds);
        }
    }
}

fn sprite_target_kind(sprite: &Sprite) -> EffectTargetKind {
    match sprite {
        Sprite::Text { .. } => EffectTargetKind::SpriteText,
        Sprite::Image { .. } | Sprite::Obj { .. } | Sprite::Vector { .. } => {
            EffectTargetKind::SpriteBitmap
        }
        Sprite::Grid { .. }
        | Sprite::Flex { .. }
        | Sprite::Panel { .. }
        | Sprite::Scene3D { .. } => EffectTargetKind::Sprite,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_scene_effect_issues;
    use engine_effects::shared_dispatcher;

    #[test]
    fn accepts_backward_compatible_effects_without_target_kind() {
        let scene: engine_core::scene::Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
stages:
  on_enter:
    steps:
      - effects:
          - name: fade-in
            duration: 100
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: HELLO
"#,
        )
        .expect("scene should parse");

        let mut unknown = Vec::new();
        let mut incompatible = Vec::new();
        collect_scene_effect_issues(
            &scene,
            "intro.yml",
            shared_dispatcher(),
            &mut unknown,
            &mut incompatible,
        );

        assert!(unknown.is_empty());
        assert!(incompatible.is_empty());
    }

    #[test]
    fn rejects_mismatched_declared_target_kind() {
        let scene: engine_core::scene::Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: HELLO
        stages:
          on_idle:
            steps:
              - effects:
                  - name: fade-in
                    duration: 100
                    target_kind: sprite_bitmap
"#,
        )
        .expect("scene should parse");

        let mut unknown = Vec::new();
        let mut incompatible = Vec::new();
        collect_scene_effect_issues(
            &scene,
            "intro.yml",
            shared_dispatcher(),
            &mut unknown,
            &mut incompatible,
        );

        assert!(unknown.is_empty());
        assert_eq!(incompatible.len(), 1);
        assert!(incompatible[0].contains("SpriteBitmap"));
    }
}
