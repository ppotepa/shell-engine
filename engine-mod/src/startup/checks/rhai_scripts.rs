//! Validates that all `rhai-script` behavior payloads compile before runtime starts.

use engine_core::scene::{BehaviorSpec, Scene, Sprite};
use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that pre-compiles every Rhai behavior script found across all scenes.
pub struct RhaiScriptsCheck;

impl StartupCheck for RhaiScriptsCheck {
    fn name(&self) -> &'static str {
        "rhai-scripts"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let mut checked = 0usize;
        let mut failures = Vec::new();

        for scene_file in scenes {
            collect_scene_failures(
                ctx,
                &scene_file.scene,
                &scene_file.path,
                &mut checked,
                &mut failures,
            );
        }

        if !failures.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("rhai script preflight failed:\n{}", failures.join("\n")),
            });
        }

        report.add_info(
            self.name(),
            format!("rhai scripts preflight ok ({checked} scripts)"),
        );
        Ok(())
    }
}

fn collect_scene_failures(
    ctx: &StartupContext,
    scene: &Scene,
    path: &str,
    checked: &mut usize,
    failures: &mut Vec<String>,
) {
    for (idx, behavior) in scene.behaviors.iter().enumerate() {
        collect_behavior_failure(
            ctx,
            scene,
            behavior,
            path,
            &scene.id,
            &format!("scene.behaviors[{idx}]"),
            checked,
            failures,
        );
    }

    for (layer_idx, layer) in scene.layers.iter().enumerate() {
        for (behavior_idx, behavior) in layer.behaviors.iter().enumerate() {
            collect_behavior_failure(
                ctx,
                scene,
                behavior,
                path,
                &scene.id,
                &format!("layer[{layer_idx}].behaviors[{behavior_idx}]"),
                checked,
                failures,
            );
        }

        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            collect_sprite_failures(
                ctx,
                scene,
                sprite,
                path,
                &scene.id,
                &format!("layer[{layer_idx}].sprite[{sprite_idx}]"),
                checked,
                failures,
            );
        }
    }
}

fn collect_sprite_failures(
    ctx: &StartupContext,
    scene: &Scene,
    sprite: &Sprite,
    path: &str,
    scene_id: &str,
    scope: &str,
    checked: &mut usize,
    failures: &mut Vec<String>,
) {
    for (behavior_idx, behavior) in sprite.behaviors().iter().enumerate() {
        collect_behavior_failure(
            ctx,
            scene,
            behavior,
            path,
            scene_id,
            &format!("{scope}.behaviors[{behavior_idx}]"),
            checked,
            failures,
        );
    }

    match sprite {
        Sprite::Panel { children, .. }
        | Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. } => {
            for (child_idx, child) in children.iter().enumerate() {
                collect_sprite_failures(
                    ctx,
                    scene,
                    child,
                    path,
                    scene_id,
                    &format!("{scope}.child[{child_idx}]"),
                    checked,
                    failures,
                );
            }
        }
        _ => {}
    }
}

fn collect_behavior_failure(
    ctx: &StartupContext,
    scene: &Scene,
    behavior: &BehaviorSpec,
    path: &str,
    scene_id: &str,
    scope: &str,
    checked: &mut usize,
    failures: &mut Vec<String>,
) {
    if !behavior.name.eq_ignore_ascii_case("rhai-script") {
        return;
    }

    *checked += 1;

    let src = behavior.params.src.as_deref().unwrap_or("<inline>");
    let Some(script) = behavior.params.script.as_deref() else {
        failures.push(format!(
            "{path} (scene `{scene_id}`, {scope}): src `{src}` missing params.script payload"
        ));
        return;
    };

    if let Err(error) = ctx.validate_rhai_script(script, behavior.params.src.as_deref(), scene) {
        failures.push(format!(
            "{path} (scene `{scene_id}`, {scope}): src `{src}` preflight failed: {error}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::RhaiScriptsCheck;
    use crate::startup::{StartupCheck, StartupContext, StartupIssueLevel, StartupReport};
    use engine_error::EngineError;
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    fn scene_loader(
        mod_source: &std::path::Path,
    ) -> Result<Vec<crate::startup::StartupSceneFile>, EngineError> {
        let scenes_dir = mod_source.join("scenes");
        let mut scenes = Vec::new();
        if scenes_dir.is_dir() {
            for entry in fs::read_dir(&scenes_dir).map_err(|e| EngineError::ManifestRead {
                path: scenes_dir.clone(),
                source: e,
            })? {
                let entry = entry.map_err(|e| EngineError::ManifestRead {
                    path: scenes_dir.clone(),
                    source: e,
                })?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "yml") {
                    let content = fs::read_to_string(&path).map_err(|e| {
                        EngineError::ManifestRead {
                            path: path.clone(),
                            source: e,
                        }
                    })?;
                    let scene = serde_yaml::from_str(&content).map_err(|e| {
                        EngineError::InvalidModYaml {
                            path: path.clone(),
                            source: e,
                        }
                    })?;
                    scenes.push(crate::startup::StartupSceneFile {
                        path: path.display().to_string(),
                        scene,
                    });
                }
            }
        }
        Ok(scenes)
    }

    #[test]
    fn accepts_valid_rhai_scripts_without_validator() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::write(
            mod_dir.join("scenes/main.yml"),
            r#"
id: main
title: Main
behaviors:
  - name: rhai-script
    params:
      src: ./scene.rhai
      script: |
        #{}
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        // No rhai validator registered — should pass (validation skipped)
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect("valid rhai script should pass");

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.check == "rhai-scripts"
                && issue.message.contains("preflight ok")
        }));
    }

    #[test]
    fn rejects_invalid_rhai_scripts_with_validator() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::write(
            mod_dir.join("scenes/main.yml"),
            r#"
id: main
title: Main
behaviors:
  - name: rhai-script
    params:
      src: ./scene.rhai
      script: "let = ;"
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");

        // Inject a validator that always fails for this invalid script
        let validator = |_script: &str, _src: Option<&str>, _scene: &engine_core::scene::Scene| -> Result<(), String> {
            Err("compile error: expected variable name".to_string())
        };

        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader)
            .with_rhai_script_validator(&validator);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("invalid rhai script should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("compile error"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }
}
