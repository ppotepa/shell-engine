use std::collections::{BTreeMap, BTreeSet};

use crate::rasterizer;
use crate::render_policy;
use crate::scene::Sprite;
use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct FontManifestCheck;

impl StartupCheck for FontManifestCheck {
    fn name(&self) -> &'static str {
        "font-manifest"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
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
                        let Some(font_name) = render_policy::resolve_text_font_spec(
                            font.as_deref(),
                            force_font_mode.as_deref(),
                            *size,
                            sf.scene.rendered_mode,
                            *force_renderer_mode,
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
            if !rasterizer::has_font_assets(Some(ctx.mod_source()), font_name) {
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
