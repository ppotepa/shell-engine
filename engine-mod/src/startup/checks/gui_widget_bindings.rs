//! Verifies that interactive-looking sprite ids are bound to scene GUI widgets.
//!
//! This catches HUD controls that look clickable (`btn-*`, `tab-btn-*`, `preset-*`, `toggle-*`)
//! but are authored only as sprites and therefore cannot emit `gui.*` events.

use std::collections::BTreeSet;

use engine_core::scene::model::SceneGuiWidgetDef;
use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that fails when interactive-looking sprite ids are not bound to a GUI widget id/sprite.
pub struct GuiWidgetBindingsCheck;

impl StartupCheck for GuiWidgetBindingsCheck {
    fn name(&self) -> &'static str {
        "gui-widget-bindings"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let mut scanned = 0usize;
        let mut failures = Vec::new();

        for sf in scenes {
            let mut widget_ids: BTreeSet<&str> = BTreeSet::new();
            let mut widget_sprites: BTreeSet<&str> = BTreeSet::new();
            for widget in &sf.scene.gui.widgets {
                match widget {
                    SceneGuiWidgetDef::Slider { id, sprite, .. }
                    | SceneGuiWidgetDef::Button { id, sprite, .. }
                    | SceneGuiWidgetDef::Toggle { id, sprite, .. }
                    | SceneGuiWidgetDef::Panel { id, sprite, .. } => {
                        if !id.is_empty() {
                            widget_ids.insert(id.as_str());
                        }
                        if !sprite.is_empty() {
                            widget_sprites.insert(sprite.as_str());
                        }
                    }
                }
            }

            let mut missing_in_scene = Vec::new();
            for layer in &sf.scene.layers {
                for sprite in &layer.sprites {
                    sprite.walk_recursive(&mut |node| {
                        let Some(id) = node.id() else {
                            return;
                        };
                        if !looks_interactive(id) {
                            return;
                        }
                        scanned += 1;
                        if widget_ids.contains(id) || widget_sprites.contains(id) {
                            return;
                        }
                        missing_in_scene.push(id.to_string());
                    });
                }
            }

            missing_in_scene.sort();
            missing_in_scene.dedup();
            if !missing_in_scene.is_empty() {
                failures.push(format!(
                    "{} ({}) missing gui widget bindings for: {}",
                    sf.scene.id,
                    sf.path,
                    missing_in_scene.join(", ")
                ));
            }
        }

        if !failures.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!(
                    "interactive sprites must be bound via scene.gui.widgets (by widget id or sprite id):\n{}",
                    failures.join("\n")
                ),
            });
        }

        report.add_info(
            self.name(),
            format!("gui widget bindings verified ({scanned} interactive sprites)"),
        );
        Ok(())
    }
}

fn looks_interactive(id: &str) -> bool {
    id.starts_with("btn-")
        || id.starts_with("tab-btn-")
        || id.starts_with("preset-")
        || id.starts_with("toggle-")
}

#[cfg(test)]
mod tests {
    use super::GuiWidgetBindingsCheck;
    use crate::startup::{StartupCheck, StartupContext, StartupIssueLevel, StartupReport};
    use engine_error::EngineError;
    use serde_yaml::Value;

    fn scene_loader(
        _mod_source: &std::path::Path,
    ) -> Result<Vec<crate::startup::StartupSceneFile>, EngineError> {
        let scene_ok = serde_yaml::from_str(
            r##"
id: ok
title: OK
layers:
  - name: hud
    sprites:
      - type: text
        id: btn-start-label
        content: "[ START ]"
gui:
  widgets:
    - type: button
      id: btn-start
      sprite: btn-start-label
"##,
        )
        .expect("ok scene parse");

        let scene_bad = serde_yaml::from_str(
            r##"
id: bad
title: BAD
layers:
  - name: hud
    sprites:
      - type: text
        id: btn-reset
        content: "[ RESET ]"
gui:
  widgets: []
"##,
        )
        .expect("bad scene parse");

        Ok(vec![
            crate::startup::StartupSceneFile {
                path: "/scenes/ok.yml".to_string(),
                scene: scene_ok,
            },
            crate::startup::StartupSceneFile {
                path: "/scenes/bad.yml".to_string(),
                scene: scene_bad,
            },
        ])
    }

    #[test]
    fn fails_when_interactive_sprite_has_no_widget_binding() {
        let mod_dir = tempfile::tempdir().expect("temp dir");
        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/ok.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(mod_dir.path(), &manifest, "/scenes/ok.yml", &scene_loader);
        let mut report = StartupReport::default();
        let err = GuiWidgetBindingsCheck
            .run(&ctx, &mut report)
            .expect_err("must fail");
        match err {
            EngineError::StartupCheckFailed { check, details } => {
                assert_eq!(check, "gui-widget-bindings");
                assert!(details.contains("btn-reset"), "details: {details}");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn passes_when_all_interactive_sprites_are_widget_bound() {
        let scene_loader_ok = |_mod_source: &std::path::Path| -> Result<
            Vec<crate::startup::StartupSceneFile>,
            EngineError,
        > {
            let scene = serde_yaml::from_str(
                r##"
id: ok
title: OK
layers:
  - name: hud
    sprites:
      - type: text
        id: btn-start-label
        content: "[ START ]"
      - type: text
        id: preset-earth
        content: "EARTH"
gui:
  widgets:
    - type: button
      id: btn-start
      sprite: btn-start-label
    - type: button
      id: preset-btn-earth
      sprite: preset-earth
"##,
            )
            .expect("scene parse");
            Ok(vec![crate::startup::StartupSceneFile {
                path: "/scenes/ok.yml".to_string(),
                scene,
            }])
        };

        let mod_dir = tempfile::tempdir().expect("temp dir");
        let manifest: Value =
            serde_yaml::from_str("name: Test\nversion: 0.1.0\nentrypoint: /scenes/ok.yml\n")
                .expect("manifest");
        let ctx = StartupContext::new(
            mod_dir.path(),
            &manifest,
            "/scenes/ok.yml",
            &scene_loader_ok,
        );
        let mut report = StartupReport::default();

        GuiWidgetBindingsCheck
            .run(&ctx, &mut report)
            .expect("must pass");

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.check == "gui-widget-bindings"
                && issue.message.contains("verified")
        }));
    }
}
