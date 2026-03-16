use super::types::{GlyphManifest, LoadedFont, LoadedGlyph};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static FONT_CACHE: OnceLock<Mutex<HashMap<String, Option<LoadedFont>>>> = OnceLock::new();

pub fn load_font_assets(font_name: &str) -> Option<LoadedFont> {
    let key = font_name.trim().to_string();
    let cache = FONT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&key) {
            return cached.clone();
        }
    }

    let loaded = load_font_assets_uncached(font_name);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, loaded.clone());
    }
    loaded
}

fn load_font_assets_uncached(font_name: &str) -> Option<LoadedFont> {
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

        let inferred_w = lines
            .iter()
            .map(|l| l.chars().count() as u16)
            .max()
            .unwrap_or(0);
        let advance = g.width.max(inferred_w);
        glyphs.insert(
            ch,
            LoadedGlyph {
                lines,
                advance,
                height: g.height.max(1),
            },
        );
    }

    let avg_width = if count_width > 0 {
        (total_width / count_width) as u16
    } else {
        3
    };
    let fallback_space_advance = (avg_width / 3).max(2);

    if let Some(space) = glyphs.get_mut(&' ') {
        if space.advance == 0 {
            space.advance = fallback_space_advance;
        }
    }

    Some(LoadedFont {
        glyphs,
        fallback_space_advance,
    })
}

fn find_font_dir(slug: &str, preferred_mode: Option<&str>) -> Option<PathBuf> {
    for root in font_roots() {
        // Canonical layout: assets/fonts/{font-slug}/{size}px/{mode}/manifest.yaml
        let by_name = root.join(slug);
        if let Ok(size_dirs) = fs::read_dir(&by_name) {
            for size_dir in size_dirs.flatten() {
                let size_path = size_dir.path();
                let mode_order = mode_order(preferred_mode);
                for mode in &mode_order {
                    let candidate = size_path.join(mode);
                    if candidate.join("manifest.yaml").exists() {
                        return Some(candidate);
                    }
                }
                if size_path.join("manifest.yaml").exists() {
                    return Some(size_path);
                }
            }
        }

        // Legacy fallback: assets/fonts/{size}px/{font-slug}/{mode}/manifest.yaml
        let entries = match fs::read_dir(&root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for size_dir in entries.flatten() {
            let font_root = size_dir.path().join(slug);
            let mode_order = mode_order(preferred_mode);
            for mode in &mode_order {
                let candidate = font_root.join(mode);
                if candidate.join("manifest.yaml").exists() {
                    return Some(candidate);
                }
            }
            if font_root.join("manifest.yaml").exists() {
                return Some(font_root);
            }
        }
    }
    None
}

fn font_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Ok(mod_source) = std::env::var("SHELL_QUEST_MOD_SOURCE") {
        push_unique(&mut roots, PathBuf::from(mod_source).join("assets/fonts"));
    }

    for base in ["mods", "../mods", "mod", "../mod"] {
        collect_mod_roots(Path::new(base), &mut roots);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            for rel in ["../mods", "../../mods", "../mod", "../../mod"] {
                collect_mod_roots(&bin_dir.join(rel), &mut roots);
            }
        }
    }

    roots
}

fn collect_mod_roots(base: &Path, roots: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let candidate = entry.path().join("assets/fonts");
        if candidate.is_dir() {
            push_unique(roots, candidate);
        }
    }
}

fn push_unique(roots: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !roots.iter().any(|p| p == &candidate) {
        roots.push(candidate);
    }
}

fn parse_font_spec(input: &str) -> (String, Option<String>) {
    let mut parts = input.split(':');
    let name = parts.next().unwrap_or_default().trim();
    let mode = parts.next().map(|m| m.trim().to_ascii_lowercase());
    (slugify_font_name(name), mode)
}

fn mode_order(preferred_mode: Option<&str>) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    if let Some(mode) = preferred_mode.map(|m| m.trim().to_ascii_lowercase()) {
        let canonical = match mode.as_str() {
            "cell" | "raster" => "terminal-pixels".to_string(),
            _ => mode,
        };
        order.push(canonical.clone());
        if canonical == "terminal-pixels" {
            order.push("raster".to_string());
        } else if canonical == "raster" {
            order.push("terminal-pixels".to_string());
        }
    }
    order.push("terminal-pixels".to_string());
    order.push("raster".to_string());
    order.push("ascii".to_string());

    let mut dedup = Vec::new();
    for mode in order {
        if !dedup.iter().any(|m: &String| m == &mode) {
            dedup.push(mode);
        }
    }
    dedup
}

fn slugify_font_name(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
