//! Verifies that every font referenced by text sprites has a loadable manifest in the mod's asset tree.

use std::collections::{BTreeMap, BTreeSet};

use engine_core::scene::Sprite;
use engine_error::EngineError;
use engine_render_policy;
use engine_runtime::RuntimeSettings;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that fails if a font used by a scene cannot be resolved to a manifest file.
pub struct FontManifestCheck;

impl StartupCheck for FontManifestCheck {
    fn name(&self) -> &'static str {
        "font-manifest"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let runtime_settings = RuntimeSettings::from_manifest(ctx.manifest());
        let default_font = runtime_settings.default_font.as_deref();
        let mut fonts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for sf in scenes {
            for layer in &sf.scene.layers {
                for sprite in &layer.sprites {
                    sprite.walk_recursive(&mut |node| {
                        let Sprite::Text {
                            font,
                            size,
                            force_renderer_mode,
                            force_font_mode,
                            ..
                        } = node
                        else {
                            return;
                        };
                        let Some(font_name) = engine_render_policy::resolve_text_font_spec(
                            font.as_deref(),
                            force_font_mode.as_deref(),
                            *size,
                            sf.scene.rendered_mode,
                            *force_renderer_mode,
                            false,
                            default_font,
                        ) else {
                            return;
                        };
                        if font_name.starts_with("generic") {
                            return;
                        }
                        fonts
                            .entry(font_name)
                            .or_default()
                            .insert(sf.scene.id.clone());
                    });
                }
            }
        }

        let mut missing = Vec::new();
        for (font_name, scenes_using) in &fonts {
            if !ctx.has_font_assets(font_name) {
                missing.push(format!(
                    "{font_name} (used in: {})",
                    scenes_using.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        if !missing.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("missing font assets:\n{}", missing.join("\n")),
            });
        }

        report.add_info(
            self.name(),
            format!("font manifests verified ({} external fonts)", fonts.len()),
        );
        Ok(())
    }
}
