//! Rasterizer data types: glyph manifest, individual glyph, and the assembled font loaded at runtime.

use serde::Deserialize;
use std::collections::HashMap;

/// Deserialised root of a font's `manifest.yaml` file.
#[derive(Debug, Deserialize)]
pub struct GlyphManifest {
    pub glyphs: Vec<ManifestGlyph>,
}

/// A single glyph entry from the font manifest, referencing an ASCII-art text file.
#[derive(Debug, Deserialize)]
pub struct ManifestGlyph {
    pub character: String,
    pub file: String,
    pub width: u16,
    pub height: u16,
}

/// A loaded, decoded glyph — ASCII-art lines with resolved advance width and height.
#[derive(Debug, Clone)]
pub struct LoadedGlyph {
    pub lines: Vec<String>,
    pub advance: u16,
    pub height: u16,
}

/// A fully loaded font: a map of characters to their [`LoadedGlyph`]s and a fallback space advance.
#[derive(Debug, Clone)]
pub struct LoadedFont {
    pub glyphs: HashMap<char, LoadedGlyph>,
    pub fallback_space_advance: u16,
}

impl LoadedFont {
    /// Returns the advance width and cell height for `ch`, using fallbacks for missing glyphs.
    pub fn advance_and_height(&self, ch: char) -> (u16, u16) {
        if let Some(g) = self.glyphs.get(&ch) {
            let adv = if ch == ' ' && g.advance == 0 {
                self.fallback_space_advance
            } else {
                g.advance.max(1)
            };
            (adv, g.height)
        } else if ch == ' ' {
            (self.fallback_space_advance, 1)
        } else {
            (1, 1)
        }
    }
}
