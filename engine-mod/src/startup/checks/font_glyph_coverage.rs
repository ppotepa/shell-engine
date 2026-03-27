//! Verifies that every glyph required by text sprites is present in the corresponding font asset.

use std::collections::{BTreeMap, BTreeSet};

use engine_core::markup::strip_markup;
use engine_core::scene::Sprite;
use engine_error::EngineError;
use engine_render_policy;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that warns when a font is missing glyphs required by scene text sprites.
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
                    sprite.walk_recursive(&mut |node| {
                        let Sprite::Text {
                            content,
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
                        ) else {
                            return;
                        };
                        if font_name.starts_with("generic") {
                            return;
                        }
                        let visible = strip_markup(content);
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
                    });
                }
            }
        }

        let mut issues = Vec::new();
        for (font_name, chars) in &required_chars {
            let text: String = chars.iter().collect();
            let Some(missing) = ctx.font_missing_glyphs(font_name, &text) else {
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
