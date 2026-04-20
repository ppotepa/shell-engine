//! Validates that all `rhai-script` behavior payloads compile before runtime starts.

use engine_behavior_registry::{load_mod_behaviors_with_errors, ModBehaviorRegistry};
use engine_core::scene::{BehaviorSpec, Scene, Sprite};
use engine_error::EngineError;
use std::path::{Path, PathBuf};

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that pre-compiles every Rhai behavior script found across all scenes.
pub struct RhaiScriptsCheck;

const LARGE_SCENE_SCRIPT_WARNING_LINES: usize = 200;

impl StartupCheck for RhaiScriptsCheck {
    fn name(&self) -> &'static str {
        "rhai-scripts"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let (mod_registry, mod_behavior_errors) = load_mod_behaviors_with_errors(ctx.mod_source());
        let mut checked = 0usize;
        let mut failures: Vec<String> = mod_behavior_errors
            .into_iter()
            .map(|err| format!("mod behaviors: {err}"))
            .collect();
        let mut warnings = Vec::new();

        for scene_file in scenes {
            collect_scene_failures(
                ctx,
                &scene_file.scene,
                &scene_file.path,
                &mod_registry,
                &mut checked,
                &mut failures,
                &mut warnings,
            );
        }

        collect_imported_module_policy_findings(ctx.mod_source(), &mut failures);

        failures.sort();
        warnings.sort();
        for warning in warnings {
            report.add_warning(self.name(), warning);
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

fn collect_imported_module_policy_findings(mod_source: &Path, failures: &mut Vec<String>) {
    let scripts_dir = mod_source.join("scripts");
    if !scripts_dir.is_dir() {
        return;
    }

    let mut module_files = Vec::new();
    collect_rhai_module_files(&scripts_dir, &mut module_files);
    module_files.sort();

    for module_path in module_files {
        let Ok(script) = std::fs::read_to_string(&module_path) else {
            continue;
        };
        collect_module_policy_findings(mod_source, &module_path, &script, failures);
    }
}

fn collect_rhai_module_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rhai_module_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rhai") {
            files.push(path);
        }
    }
}

fn collect_scene_failures(
    ctx: &StartupContext,
    scene: &Scene,
    path: &str,
    mod_registry: &ModBehaviorRegistry,
    checked: &mut usize,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for (idx, behavior) in scene.behaviors.iter().enumerate() {
        collect_behavior_failure(
            ctx,
            scene,
            behavior,
            path,
            &scene.id,
            &format!("scene.behaviors[{idx}]"),
            mod_registry,
            checked,
            failures,
            warnings,
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
                mod_registry,
                checked,
                failures,
                warnings,
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
                mod_registry,
                checked,
                failures,
                warnings,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_sprite_failures(
    ctx: &StartupContext,
    scene: &Scene,
    sprite: &Sprite,
    path: &str,
    scene_id: &str,
    scope: &str,
    mod_registry: &ModBehaviorRegistry,
    checked: &mut usize,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for (behavior_idx, behavior) in sprite.behaviors().iter().enumerate() {
        collect_behavior_failure(
            ctx,
            scene,
            behavior,
            path,
            scene_id,
            &format!("{scope}.behaviors[{behavior_idx}]"),
            mod_registry,
            checked,
            failures,
            warnings,
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
                    mod_registry,
                    checked,
                    failures,
                    warnings,
                );
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_behavior_failure(
    ctx: &StartupContext,
    scene: &Scene,
    behavior: &BehaviorSpec,
    path: &str,
    scene_id: &str,
    scope: &str,
    mod_registry: &ModBehaviorRegistry,
    checked: &mut usize,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if behavior.name.eq_ignore_ascii_case("rhai-script") {
        *checked += 1;

        let src = behavior.params.src.as_deref().unwrap_or("<inline>");
        let Some(script) = behavior.params.script.as_deref() else {
            failures.push(format!(
                "{path} (scene `{scene_id}`, {scope}): src `{src}` missing params.script payload"
            ));
            return;
        };

        collect_script_policy_findings(
            path, scene_id, scope, src, script, true, failures, warnings,
        );

        if let Err(error) = ctx.validate_rhai_script(script, behavior.params.src.as_deref(), scene)
        {
            failures.push(format!(
                "{path} (scene `{scene_id}`, {scope}): src `{src}` preflight failed: {error}"
            ));
        }
        return;
    }

    let Some(mod_behavior) = mod_registry.get(behavior.name.trim()) else {
        return;
    };

    *checked += 1;

    let src = mod_behavior
        .src
        .as_deref()
        .unwrap_or("<inline mod behavior>");
    let script = mod_behavior.script.as_str();

    collect_script_policy_findings(
        path, scene_id, scope, src, script, false, failures, warnings,
    );

    if let Err(error) = ctx.validate_rhai_script(script, mod_behavior.src.as_deref(), scene) {
        failures.push(format!(
            "{path} (scene `{scene_id}`, {scope}): src `{src}` preflight failed: {error}"
        ));
    }
}

fn collect_script_policy_findings(
    path: &str,
    scene_id: &str,
    scope: &str,
    src: &str,
    script: &str,
    is_scene_entrypoint: bool,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let compact = compact_rhai_code(script);
    let script_ref = format!("{path} (scene `{scene_id}`, {scope}): src `{src}`");

    if compact.contains("scene.get(") {
        failures.push(format!(
            "{script_ref} policy violation: `scene.get(...)` is banned; use `scene.object(...)`, `scene.objects.find(...)`, or `scene.inspect(...)`."
        ));
    }

    if compact.contains("scene.set(") {
        failures.push(format!(
            "{script_ref} policy violation: `scene.set(...)` is banned; use live handles such as `scene.object(...).set(...)` or `runtime.scene.objects.find(...).set(...)`."
        ));
    }

    if compact.contains("world.body_info(") {
        failures.push(format!(
            "{script_ref} policy violation: `world.body_info(...)` is banned; use typed body snapshots via `world.body(...)` / `world.body_snapshot(...)`, and call `.inspect()` only when a map snapshot is truly needed."
        ));
    }

    for legacy_helper in [
        "scene.batch(",
        "scene.set_visible(",
        "scene.set_text_style(",
        "scene.set_vector(",
        "scene.set_multi(",
        "scene.spawn_object(",
        "scene.despawn_object(",
    ] {
        if compact.contains(legacy_helper) {
            failures.push(format!(
                "{script_ref} policy violation: legacy compatibility helper `{legacy_helper}` is banned; use `scene.object(...)`, `scene.objects.find(...)`, or typed runtime/domain APIs instead."
            ));
        }
    }

    if is_scene_entrypoint && compact.contains("type_of(local)") {
        failures.push(format!(
            "{script_ref} policy violation: legacy `type_of(local)` bootstrap is banned in scene entrypoints; initialize one owned state object such as `local.state` from a bootstrap/state module."
        ));
    }

    if is_scene_entrypoint && compact.contains("local[\"") {
        failures.push(format!(
            "{script_ref} policy violation: raw `local[\"...\"]` state access is banned in scene entrypoints; keep entrypoints thin and route mutable state through one owned object such as `local.state` plus imported state modules."
        ));
    }

    let line_count = script.lines().count();
    let has_import = compact.contains("import\"");
    if is_scene_entrypoint && line_count > LARGE_SCENE_SCRIPT_WARNING_LINES && !has_import {
        warnings.push(format!(
            "{script_ref} policy warning: scene entrypoint is {line_count} lines and has no `import`; consider moving helpers into `mods/<mod>/scripts/...` modules and keeping the entrypoint thin."
        ));
    }
}

fn collect_module_policy_findings(
    mod_source: &Path,
    module_path: &Path,
    script: &str,
    failures: &mut Vec<String>,
) {
    let compact = compact_rhai_code(script);
    let src = module_path
        .strip_prefix(mod_source)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| module_path.to_string_lossy().replace('\\', "/"));
    let script_ref = format!("{} (imported module): src `/{src}`", mod_source.display());

    if compact.contains("scene.get(") {
        failures.push(format!(
            "{script_ref} policy violation: `scene.get(...)` is banned in imported modules too; use `scene.object(...)`, `scene.objects.find(...)`, or `scene.inspect(...)`."
        ));
    }

    if compact.contains("scene.set(") {
        failures.push(format!(
            "{script_ref} policy violation: `scene.set(...)` is banned in imported modules too; use live handles such as `scene.object(...).set(...)` or `runtime.scene.objects.find(...).set(...)`."
        ));
    }

    if compact.contains("world.body_info(") {
        failures.push(format!(
            "{script_ref} policy violation: `world.body_info(...)` is banned in imported modules too; use typed body snapshots via `world.body(...)` / `world.body_snapshot(...)`, and call `.inspect()` only when a map snapshot is truly needed."
        ));
    }

    for legacy_helper in [
        "scene.batch(",
        "scene.set_visible(",
        "scene.set_text_style(",
        "scene.set_vector(",
        "scene.set_multi(",
        "scene.spawn_object(",
        "scene.despawn_object(",
    ] {
        if compact.contains(legacy_helper) {
            failures.push(format!(
                "{script_ref} policy violation: legacy compatibility helper `{legacy_helper}` is banned in imported modules; use `scene.object(...)`, `scene.objects.find(...)`, or typed runtime/domain APIs instead."
            ));
        }
    }
}

fn compact_rhai_code(script: &str) -> String {
    let mut compact = String::with_capacity(script.len());
    for line in script.lines() {
        let code = line.split_once("//").map_or(line, |(code, _)| code);
        for ch in code.chars() {
            if !ch.is_whitespace() {
                compact.push(ch);
            }
        }
    }
    compact
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
                    let content =
                        fs::read_to_string(&path).map_err(|e| EngineError::ManifestRead {
                            path: path.clone(),
                            source: e,
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
        let validator = |_script: &str,
                         _src: Option<&str>,
                         _scene: &engine_core::scene::Scene|
         -> Result<(), String> {
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

    #[test]
    fn validates_external_mod_behavior_sources() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::create_dir_all(mod_dir.join("behaviors")).expect("create behaviors");

        fs::write(
            mod_dir.join("scenes/main.yml"),
            r#"
id: main
title: Main
behaviors:
  - name: my-mod-behavior
layers: []
"#,
        )
        .expect("write scene");

        fs::write(
            mod_dir.join("behaviors/my-mod-behavior.yml"),
            r#"
kind: behavior
name: my-mod-behavior
src: ./my-mod-behavior.rhai
"#,
        )
        .expect("write behavior yml");

        fs::write(
            mod_dir.join("behaviors/my-mod-behavior.rhai"),
            "let _value = 1;\n",
        )
        .expect("write behavior script");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");

        let validator = |script: &str,
                         src: Option<&str>,
                         _scene: &engine_core::scene::Scene|
         -> Result<(), String> {
            assert_eq!(src, Some("/behaviors/my-mod-behavior.rhai"));
            assert!(script.contains("let _value = 1;"));
            Ok(())
        };

        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader)
            .with_rhai_script_validator(&validator);
        let mut report = StartupReport::default();
        RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect("external mod behavior script should pass");

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.check == "rhai-scripts"
                && issue.message.contains("preflight ok")
        }));
    }

    #[test]
    fn rejects_scene_get_policy_usage() {
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
        let hud = scene.get("hud");
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("scene.get policy usage should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("scene.get(...)` is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn rejects_type_of_local_in_scene_entrypoint() {
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
        if type_of(local) == "()" { local = #{}; }
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("legacy local bootstrap should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("type_of(local)` bootstrap is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn rejects_scene_set_policy_usage() {
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
        scene.set("hud", "text.content", "ok");
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("scene.set policy usage should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("scene.set(...)` is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn rejects_world_body_info_policy_usage() {
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
        let body = world.body_info("main-planet");
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("world.body_info policy usage should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("world.body_info(...)` is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn rejects_legacy_scene_compat_helper_usage() {
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
        scene.batch("hud", #{ visible: true });
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("legacy scene compat helper should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("legacy compatibility helper `scene.batch(` is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn rejects_raw_local_index_state_in_scene_entrypoint() {
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
        local["score"] = 10;
layers: []
"#,
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("raw local indexing in scene entrypoint should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("raw `local[\"...\"]` state access is banned"));
                assert!(details.contains("./scene.rhai"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn warns_on_large_scene_script_without_import() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        let mut script = String::new();
        for idx in 0..210 {
            script.push_str(&format!("let value_{idx} = {idx};\n"));
        }
        fs::write(
            mod_dir.join("scenes/main.yml"),
            format!(
                r#"
id: main
title: Main
behaviors:
  - name: rhai-script
    params:
      src: ./scene.rhai
      script: |
{}
layers: []
"#,
                script
                    .lines()
                    .map(|line| format!("        {line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .expect("write scene");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect("large script without import should only warn");

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.check == "rhai-scripts"
                && issue.message.contains("has no `import`")
        }));
    }

    #[test]
    fn allows_type_of_local_in_external_mod_behavior() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::create_dir_all(mod_dir.join("behaviors")).expect("create behaviors");

        fs::write(
            mod_dir.join("scenes/main.yml"),
            r#"
id: main
title: Main
behaviors:
  - name: my-mod-behavior
layers: []
"#,
        )
        .expect("write scene");

        fs::write(
            mod_dir.join("behaviors/my-mod-behavior.yml"),
            r#"
kind: behavior
name: my-mod-behavior
src: ./my-mod-behavior.rhai
"#,
        )
        .expect("write behavior yml");

        fs::write(
            mod_dir.join("behaviors/my-mod-behavior.rhai"),
            "if type_of(local) == \"()\" { local = #{}; }\n",
        )
        .expect("write behavior script");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect("type_of(local) is only banned in scene entrypoints");

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.check == "rhai-scripts"
                && issue.message.contains("type_of(local)")
        }));
    }

    #[test]
    fn rejects_legacy_scene_usage_inside_imported_module() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::create_dir_all(mod_dir.join("scripts")).expect("create scripts");

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
        import "shared" as shared;
        #{}
layers: []
"#,
        )
        .expect("write scene");

        fs::write(
            mod_dir.join("scripts/shared.rhai"),
            r#"
fn touch_scene() {
    scene.get("hud");
}
"#,
        )
        .expect("write shared module");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        let error = RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect_err("legacy scene usage in imported module should fail startup");

        match error {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "rhai-scripts");
                assert!(details.contains("imported module"));
                assert!(details.contains("/scripts/shared.rhai"));
                assert!(details.contains("scene.get(...)` is banned in imported modules"));
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn allows_bootstrap_style_type_of_local_inside_imported_module() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::create_dir_all(mod_dir.join("scripts/std")).expect("create scripts");

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
        import "std/bootstrap" as bootstrap;
        local = bootstrap::ensure(local);
        #{}
layers: []
"#,
        )
        .expect("write scene");

        fs::write(
            mod_dir.join("scripts/std/bootstrap.rhai"),
            r#"
fn ensure(local) {
    if type_of(local) == "()" { return #{}; }
    local
}
"#,
        )
        .expect("write bootstrap module");

        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &scene_loader);
        let mut report = StartupReport::default();
        RhaiScriptsCheck
            .run(&ctx, &mut report)
            .expect("bootstrap-style type_of(local) in imported module should remain allowed");
    }
}
