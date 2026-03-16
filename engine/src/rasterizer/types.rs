use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GlyphManifest {
    pub glyphs: Vec<ManifestGlyph>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestGlyph {
    pub character: String,
    pub file: String,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct LoadedGlyph {
    pub lines: Vec<String>,
    pub advance: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct LoadedFont {
    pub glyphs: HashMap<char, LoadedGlyph>,
    pub fallback_space_advance: u16,
}

impl LoadedFont {
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
