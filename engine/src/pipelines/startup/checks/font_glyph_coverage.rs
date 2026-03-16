use std::collections::{BTreeMap, BTreeSet};

use crate::markup::strip_markup;
use crate::rasterizer;
use crate::scene::Sprite;
use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct FontGlyphCoverageCheck;

impl StartupCheck for FontGlyphCoverageCheck {
    fn name(&self) -> &'static str {
        "font-glyph-coverage"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let mut required_chars: BTreeMap<String, BTreeSet<char>> = BTreeMap::new();
        let mut used_in_scenes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for sf in scenes {
            for layer in &sf.scene.layers {
                for sprite in &layer.sprites {
                    let Sprite::Text { content, font, .. } = sprite;
                    let Some(font_name) = font.as_ref() else {
                        continue;
                    };
                    if font_name.starts_with("generic") {
                        continue;
                    }
                    let visible = strip_markup(&content);
                    let chars = required_chars.entry(font_name.clone()).or_default();
                    for ch in visible.chars() {
                        if ch.is_whitespace() {
                            continue;
                        }
                        chars.insert(ch);
                    }
                    used_in_scenes
                        .entry(font_name.clone())
                        .or_default()
                        .insert(sf.scene.id.clone());
                }
            }
        }

        let mut issues = Vec::new();
        for (font_name, chars) in &required_chars {
            let text: String = chars.iter().collect();
            let Some(missing) = rasterizer::missing_glyphs(font_name, &text) else {
                issues.push(format!("{font_name}: font assets are missing"));
                continue;
            };

            if !missing.is_empty() {
                let missing_list = missing
                    .iter()
                    .map(|ch| format!("'{}'", ch))
                    .collect::<Vec<_>>()
                    .join(", ");
                let scenes = used_in_scenes
                    .get(font_name)
                    .map(|set| set.iter().cloned().collect::<Vec<_>>().join(", "))
                    .unwrap_or_else(|| "unknown".to_string());
                issues.push(format!(
                    "{font_name}: missing glyphs [{missing_list}] (used in: {scenes})"
                ));
            }
        }

        if !issues.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: issues.join("\n"),
            });
        }

        report.add_info(
            self.name(),
            format!(
                "glyph coverage verified ({} external fonts)",
                required_chars.len()
            ),
        );
        Ok(())
    }
}
