use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use super::types::{GlyphManifest, LoadedFont, LoadedGlyph};

pub fn load_font_assets(font_name: &str) -> Option<LoadedFont> {
    let (slug, preferred_mode) = parse_font_spec(font_name);
    let font_dir = find_font_dir(&slug, preferred_mode.as_deref())?;
    let manifest_path = font_dir.join("manifest.yaml");
    let manifest_raw = fs::read_to_string(manifest_path).ok()?;
    let manifest: GlyphManifest = serde_yaml::from_str(&manifest_raw).ok()?;

    let mut glyphs = HashMap::new();
    let mut total_width: u32 = 0;
    let mut count_width: u32 = 0;

    for g in manifest.glyphs {
        let ch = g.character.chars().next()?;
        let glyph_path = font_dir.join(g.file);
        let glyph_raw = fs::read_to_string(glyph_path).unwrap_or_default();
        let lines: Vec<String> = if glyph_raw.is_empty() {
            Vec::new()
        } else {
            glyph_raw.lines().map(ToOwned::to_owned).collect()
        };

        if ch != ' ' && g.width > 0 {
            total_width += g.width as u32;
            count_width += 1;
        }

        let inferred_w = lines.iter().map(|l| l.chars().count() as u16).max().unwrap_or(0);
        let advance = g.width.max(inferred_w);
        glyphs.insert(ch, LoadedGlyph { lines, advance, height: g.height.max(1) });
    }

    let avg_width = if count_width > 0 { (total_width / count_width) as u16 } else { 3 };
    let fallback_space_advance = (avg_width / 3).max(2);

    if let Some(space) = glyphs.get_mut(&' ') {
        if space.advance == 0 {
            space.advance = fallback_space_advance;
        }
    }

    Some(LoadedFont { glyphs, fallback_space_advance })
}

fn find_font_dir(slug: &str, preferred_mode: Option<&str>) -> Option<PathBuf> {
    let mut roots = vec![
        PathBuf::from("mod/shell-quest/assets/fonts"),
        PathBuf::from("../mod/shell-quest/assets/fonts"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            roots.push(bin_dir.join("../mod/shell-quest/assets/fonts"));
            roots.push(bin_dir.join("../../mod/shell-quest/assets/fonts"));
        }
    }

    for root in roots {
        let by_name = root.join(slug);
        if let Ok(size_dirs) = fs::read_dir(&by_name) {
            for size_dir in size_dirs.flatten() {
                let size_path = size_dir.path();
                let mode_order = mode_order(preferred_mode);
                for mode in mode_order {
                    let candidate = size_path.join(mode);
                    if candidate.join("manifest.yaml").exists() {
                        return Some(candidate);
                    }
                }
            }
        }

        let entries = match fs::read_dir(&root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for size_dir in entries.flatten() {
            let candidate = size_dir.path().join(slug);
            if candidate.join("manifest.yaml").exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn parse_font_spec(input: &str) -> (String, Option<String>) {
    let mut parts = input.split(':');
    let name = parts.next().unwrap_or_default().trim();
    let mode = parts.next().map(|m| m.trim().to_ascii_lowercase());
    (slugify_font_name(name), mode)
}

fn mode_order(preferred_mode: Option<&str>) -> [&'static str; 2] {
    match preferred_mode {
        Some("ascii") => ["ascii", "raster"],
        Some("raster") => ["raster", "ascii"],
        _ => ["raster", "ascii"],
    }
}

fn slugify_font_name(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
