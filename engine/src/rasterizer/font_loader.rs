//! Font asset loader — discovers, parses, and caches glyph manifest files from a mod's asset tree.

use super::types::{GlyphManifest, LoadedFont, LoadedGlyph};
use crate::asset_cache::AssetCache;
use crate::repositories::{create_asset_repository, AssetRepository};
use std::collections::HashMap;
use std::path::Path;

static FONT_CACHE: AssetCache<LoadedFont> = AssetCache::new();

/// Loads and caches the [`LoadedFont`] for `font_name` from `mod_source`, returning `None` if not found.
pub fn load_font_assets(mod_source: &Path, font_name: &str) -> Option<std::sync::Arc<LoadedFont>> {
    let key = format!("{}::{}", mod_source.display(), font_name.trim());
    FONT_CACHE.get_or_load(key, || load_font_assets_uncached(mod_source, font_name))
}

fn load_font_assets_uncached(mod_source: &Path, font_name: &str) -> Option<LoadedFont> {
    let repo = create_asset_repository(mod_source).ok()?;
    let (slug, preferred_mode) = parse_font_spec(font_name);
    let manifest_path = find_font_manifest_path(&repo, &slug, preferred_mode.as_deref())?;
    let manifest_raw = String::from_utf8(repo.read_asset_bytes(&manifest_path).ok()?).ok()?;
    let manifest: GlyphManifest = serde_yaml::from_str(&manifest_raw).ok()?;
    let manifest_dir = asset_parent(&manifest_path);

    let mut glyphs = HashMap::new();
    let mut total_width: u32 = 0;
    let mut count_width: u32 = 0;

    for g in manifest.glyphs {
        let ch = g.character.chars().next()?;
        let glyph_path = resolve_asset_ref(&manifest_dir, &g.file);
        let glyph_raw = repo
            .read_asset_bytes(&glyph_path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();
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
            .map(|line| line.chars().count() as u16)
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

fn find_font_manifest_path(
    repo: &impl AssetRepository,
    slug: &str,
    preferred_mode: Option<&str>,
) -> Option<String> {
    let manifests = repo
        .list_assets_under("/assets/fonts")
        .ok()?
        .into_iter()
        .filter(|path| path.ends_with("/manifest.yaml"))
        .collect::<Vec<_>>();
    let mode_order = mode_order(preferred_mode);

    manifests
        .into_iter()
        .filter_map(|path| font_manifest_candidate(&path, slug, &mode_order))
        .min_by(|left, right| left.sort_key().cmp(&right.sort_key()))
        .map(|candidate| candidate.path)
}

#[derive(Debug)]
struct FontManifestCandidate {
    path: String,
    layout_rank: u8,
    mode_rank: usize,
}

impl FontManifestCandidate {
    fn sort_key(&self) -> (u8, usize, &str) {
        (self.layout_rank, self.mode_rank, self.path.as_str())
    }
}

fn font_manifest_candidate(
    path: &str,
    slug: &str,
    mode_order: &[String],
) -> Option<FontManifestCandidate> {
    let rel = path.trim_start_matches('/').strip_prefix("assets/fonts/")?;
    let segments = rel.split('/').collect::<Vec<_>>();

    match segments.as_slice() {
        [font_slug, _, mode, "manifest.yaml"] if *font_slug == slug => {
            Some(FontManifestCandidate {
                path: path.to_string(),
                layout_rank: 0,
                mode_rank: mode_rank(mode_order, mode),
            })
        }
        [font_slug, _, "manifest.yaml"] if *font_slug == slug => Some(FontManifestCandidate {
            path: path.to_string(),
            layout_rank: 1,
            mode_rank: mode_order.len(),
        }),
        [_, font_slug, mode, "manifest.yaml"] if *font_slug == slug => {
            Some(FontManifestCandidate {
                path: path.to_string(),
                layout_rank: 2,
                mode_rank: mode_rank(mode_order, mode),
            })
        }
        [_, font_slug, "manifest.yaml"] if *font_slug == slug => Some(FontManifestCandidate {
            path: path.to_string(),
            layout_rank: 3,
            mode_rank: mode_order.len(),
        }),
        _ => None,
    }
}

fn mode_rank(mode_order: &[String], mode: &str) -> usize {
    let normalized = mode.trim().to_ascii_lowercase();
    mode_order
        .iter()
        .position(|candidate| candidate == &normalized)
        .unwrap_or(mode_order.len() + 1)
}

fn asset_parent(asset_path: &str) -> String {
    let normalized = asset_path.replace('\\', "/");
    let mut parts = normalized
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    parts.pop();
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn resolve_asset_ref(base_dir: &str, asset_ref: &str) -> String {
    if asset_ref.starts_with('/') {
        return normalize_asset_path(asset_ref);
    }
    normalize_asset_path(&format!("{}/{}", base_dir.trim_end_matches('/'), asset_ref))
}

fn normalize_asset_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.replace('\\', "/").split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            value => parts.push(value.to_string()),
        }
    }
    format!("/{}", parts.join("/"))
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
        if !dedup.iter().any(|candidate: &String| candidate == &mode) {
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
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::{find_font_manifest_path, load_font_assets, mode_order, resolve_asset_ref};
    use crate::repositories::create_asset_repository;
    use std::fs;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn resolves_relative_glyph_paths_inside_manifest_directory() {
        assert_eq!(
            resolve_asset_ref("/assets/fonts/test/8px/ascii", "../common/a.txt"),
            "/assets/fonts/test/8px/common/a.txt"
        );
    }

    #[test]
    fn prefers_canonical_layout_and_requested_mode() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/fonts/test-font/8px/ascii")).expect("create ascii");
        fs::create_dir_all(mod_dir.join("assets/fonts/test-font/8px/terminal-pixels"))
            .expect("create terminal-pixels");
        fs::create_dir_all(mod_dir.join("assets/fonts/8px/test-font/ascii"))
            .expect("create legacy");
        fs::write(
            mod_dir.join("assets/fonts/test-font/8px/ascii/manifest.yaml"),
            "glyphs: []\n",
        )
        .expect("write ascii manifest");
        fs::write(
            mod_dir.join("assets/fonts/test-font/8px/terminal-pixels/manifest.yaml"),
            "glyphs: []\n",
        )
        .expect("write terminal manifest");
        fs::write(
            mod_dir.join("assets/fonts/8px/test-font/ascii/manifest.yaml"),
            "glyphs: []\n",
        )
        .expect("write legacy manifest");

        let repo = create_asset_repository(&mod_dir).expect("asset repo");
        let path =
            find_font_manifest_path(&repo, "test-font", Some("ascii")).expect("manifest path");
        assert_eq!(path, "/assets/fonts/test-font/8px/ascii/manifest.yaml");

        let fallback =
            find_font_manifest_path(&repo, "test-font", Some("raster")).expect("fallback path");
        assert_eq!(
            fallback,
            "/assets/fonts/test-font/8px/terminal-pixels/manifest.yaml"
        );
        assert_eq!(
            mode_order(Some("ascii")),
            vec![
                "ascii".to_string(),
                "terminal-pixels".to_string(),
                "raster".to_string()
            ]
        );
    }

    #[test]
    fn loads_font_assets_from_directory_and_zip_mods() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/fonts/test-font/8px/ascii"))
            .expect("create font dir");
        fs::write(
            mod_dir.join("assets/fonts/test-font/8px/ascii/manifest.yaml"),
            "glyphs:\n  - character: \"A\"\n    file: \"A.txt\"\n    width: 3\n    height: 2\n",
        )
        .expect("write manifest");
        fs::write(
            mod_dir.join("assets/fonts/test-font/8px/ascii/A.txt"),
            "##\n##\n",
        )
        .expect("write glyph");

        let dir_font = load_font_assets(&mod_dir, "test-font:ascii").expect("directory font");
        assert_eq!(dir_font.glyphs.get(&'A').expect("glyph").advance, 3);

        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("assets/fonts/test-font/8px/ascii/manifest.yaml", opts)
            .expect("start manifest");
        std::io::Write::write_all(
            &mut writer,
            b"glyphs:\n  - character: \"A\"\n    file: \"A.txt\"\n    width: 3\n    height: 2\n",
        )
        .expect("write manifest");
        writer
            .start_file("assets/fonts/test-font/8px/ascii/A.txt", opts)
            .expect("start glyph");
        std::io::Write::write_all(&mut writer, b"##\n##\n").expect("write glyph");
        writer.finish().expect("finish zip");

        let zip_font = load_font_assets(&zip_path, "test-font:ascii").expect("zip font");
        assert_eq!(zip_font.glyphs.get(&'A').expect("glyph").height, 2);
    }
}
