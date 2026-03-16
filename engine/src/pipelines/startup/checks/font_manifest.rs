use std::collections::{BTreeMap, BTreeSet};

use crate::rasterizer;
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
                    let Sprite::Text { font, .. } = sprite;
                    let Some(font_name) = font.as_ref() else {
                        continue;
                    };
                    if font_name.starts_with("generic") {
                        continue;
                    }
                    fonts
                        .entry(font_name.clone())
                        .or_default()
                        .insert(sf.scene.id.clone());
                }
            }
        }

        let mut missing = Vec::new();
        for (font_name, scenes_using) in &fonts {
            if !rasterizer::has_font_assets(font_name) {
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
