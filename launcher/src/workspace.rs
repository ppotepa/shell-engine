use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ModManifest {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub entrypoint: String,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct DisplayConfig {
    #[serde(default)]
    pub min_colours: u16,
    #[serde(default)]
    pub min_width: u16,
    #[serde(default)]
    pub min_height: u16,
    #[serde(default)]
    pub default_font: String,
    #[serde(default)]
    #[serde(alias = "render_size")]
    pub world_render_size: String,
    #[serde(default)]
    pub presentation_policy: String,
}

pub struct ModEntry {
    pub dir: PathBuf,
    pub manifest: ModManifest,
    pub scenes: Vec<SceneEntry>,
}

pub struct SceneEntry {
    pub path: PathBuf,
    pub dir_name: String,
    pub id: Option<String>,
    pub title: Option<String>,
}

pub fn find_workspace_root() -> Result<PathBuf> {
    let current = std::env::current_dir().context("failed to get current directory")?;

    for ancestor in current.ancestors() {
        let cargo_toml = ancestor.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml).context("failed to read Cargo.toml")?;
            if content.contains("[workspace]") {
                return Ok(ancestor.to_path_buf());
            }
        }
    }

    anyhow::bail!("not in a workspace — no Cargo.toml with [workspace] found")
}

pub fn scan_mods(workspace_root: &Path) -> Result<Vec<ModEntry>> {
    let mods_dir = workspace_root.join("mods");
    if !mods_dir.exists() {
        anyhow::bail!("mods directory not found at {}", mods_dir.display());
    }

    let mut mods = Vec::new();

    for entry in fs::read_dir(&mods_dir).context("failed to read mods directory")? {
        let entry = entry.context("failed to read mods dir entry")?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("mod.yaml");
        if !manifest_path.exists() {
            continue;
        }

        let manifest_str = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;

        let manifest: ModManifest = serde_yaml::from_str(&manifest_str)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

        let scenes = scan_scenes(&path)?;

        mods.push(ModEntry {
            dir: path,
            manifest,
            scenes,
        });
    }

    mods.sort_by(|a, b| {
        if a.manifest.name == "Shell Engine" {
            return std::cmp::Ordering::Less;
        }
        if b.manifest.name == "Shell Engine" {
            return std::cmp::Ordering::Greater;
        }
        a.manifest.name.cmp(&b.manifest.name)
    });

    Ok(mods)
}

fn scan_scenes(mod_dir: &Path) -> Result<Vec<SceneEntry>> {
    let scenes_dir = mod_dir.join("scenes");
    if !scenes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut scenes = Vec::new();

    for entry in fs::read_dir(&scenes_dir).context("failed to read scenes directory")? {
        let entry = entry.context("failed to read scenes dir entry")?;
        let path = entry.path();

        let scene_yml = if path.is_dir() {
            let pkg = path.join("scene.yml");
            if pkg.exists() {
                pkg
            } else {
                continue;
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            path.clone()
        } else {
            continue;
        };

        let dir_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let (id, title) = extract_scene_metadata(&scene_yml);

        scenes.push(SceneEntry {
            path: scene_yml,
            dir_name,
            id,
            title,
        });
    }

    scenes.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));

    Ok(scenes)
}

fn extract_scene_metadata(scene_path: &Path) -> (Option<String>, Option<String>) {
    let Ok(content) = fs::read_to_string(scene_path) else {
        return (None, None);
    };

    let mut id = None;
    let mut title = None;

    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("id:") {
            id = Some(
                rest.trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string(),
            );
        }
        if let Some(rest) = line.trim().strip_prefix("title:") {
            title = Some(
                rest.trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string(),
            );
        }
    }

    (id, title)
}
