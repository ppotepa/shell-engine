use std::collections::{BTreeMap, BTreeSet};

use crate::image_loader;
use crate::scene::Sprite;
use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct ImageAssetsCheck;

impl StartupCheck for ImageAssetsCheck {
    fn name(&self) -> &'static str {
        "image-assets"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let mut images: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for sf in scenes {
            for layer in &sf.scene.layers {
                for sprite in &layer.sprites {
                    sprite.walk_recursive(&mut |node| {
                        let Sprite::Image { source, .. } = node else {
                            return;
                        };
                        images
                            .entry(source.clone())
                            .or_default()
                            .insert(sf.scene.id.clone());
                    });
                }
            }
        }

        let mut missing = Vec::new();
        for (source, used_in_scenes) in &images {
            if !image_loader::has_image_asset(ctx.mod_source(), source) {
                missing.push(format!(
                    "{source} (used in: {})",
                    used_in_scenes
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if !missing.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("missing/invalid image assets:\n{}", missing.join("\n")),
            });
        }

        report.add_info(
            self.name(),
            format!("image assets verified ({} images)", images.len()),
        );
        Ok(())
    }
}
