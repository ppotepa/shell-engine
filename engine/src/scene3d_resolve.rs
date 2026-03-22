//! Cross-file reference resolution for `.scene3d.yml` definitions.
//!
//! Supports three optional ref fields on [`Scene3DDefinition`]:
//!
//! - `materials-ref` — path to a YAML file containing a `HashMap<name, MaterialDef>`.
//!   Ref materials fill in any keys absent from the local `materials` block (local wins).
//!
//! - `camera-ref` — path to a YAML file containing a `CameraDef`.
//!   Replaces `def.camera` entirely. Do not combine with a local `camera:` block.
//!
//! - `lights-ref` — path to a YAML file containing a `Vec<LightDef>`.
//!   Applied only when the local `lights` list is empty (local lights win).
//!
//! ## Path resolution
//!
//! Paths follow the same rules as asset paths elsewhere in the engine:
//! - `/absolute` — resolved against the mod root.
//! - `./relative` or `../relative` — resolved relative to the `.scene3d.yml` file.
//! - plain name (no `/` or `.`) — auto-prefixed as `/assets/3d/{name}.yml`.

use std::collections::HashMap;

use crate::assets::AssetRoot;
use crate::scene3d_format::{CameraDef, LightDef, MaterialDef, Scene3DDefinition};

/// Resolve all `*-ref` fields in `def`, loading and merging external YAML files.
///
/// `src_path` is the mod-root-relative path of the `.scene3d.yml` file being loaded
/// (used to resolve `./relative` references).
pub fn resolve_scene3d_refs(def: &mut Scene3DDefinition, src_path: &str, asset_root: &AssetRoot) {
    if let Some(ref path) = def.materials_ref.clone() {
        let resolved = resolve_ref_path(path, src_path, "assets/3d");
        let full = asset_root.resolve(&resolved);
        let full_str = full.to_string_lossy();
        match load_materials_ref(&full_str) {
            Ok(ref_materials) => {
                for (key, mat) in ref_materials {
                    def.materials.entry(key).or_insert(mat);
                }
            }
            Err(e) => {
                engine_core::logging::warn(
                    "engine.scene3d",
                    format!("scene={}: failed to load materials-ref '{}': {e}", def.id, path),
                );
            }
        }
    }

    if let Some(ref path) = def.camera_ref.clone() {
        let resolved = resolve_ref_path(path, src_path, "assets/3d");
        let full = asset_root.resolve(&resolved);
        let full_str = full.to_string_lossy();
        match load_camera_ref(&full_str) {
            Ok(camera) => {
                def.camera = camera;
            }
            Err(e) => {
                engine_core::logging::warn(
                    "engine.scene3d",
                    format!("scene={}: failed to load camera-ref '{}': {e}", def.id, path),
                );
            }
        }
    }

    if let Some(ref path) = def.lights_ref.clone() {
        if def.lights.is_empty() {
            let resolved = resolve_ref_path(path, src_path, "assets/3d");
            let full = asset_root.resolve(&resolved);
            let full_str = full.to_string_lossy();
            match load_lights_ref(&full_str) {
                Ok(lights) => {
                    def.lights = lights;
                }
                Err(e) => {
                    engine_core::logging::warn(
                        "engine.scene3d",
                        format!("scene={}: failed to load lights-ref '{}': {e}", def.id, path),
                    );
                }
            }
        }
    }
}

// ── Path resolution ──────────────────────────────────────────────────────────

/// Resolve a ref path string to a mod-root-relative path.
fn resolve_ref_path(reference: &str, src_path: &str, default_prefix: &str) -> String {
    if reference.starts_with('/') {
        return normalize_path(reference);
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        let src_dir = parent_dir(src_path);
        return normalize_path(&format!("{src_dir}/{reference}"));
    }
    // Plain name — auto-prefix with the default directory.
    let has_ext = reference.ends_with(".yml") || reference.ends_with(".yaml");
    if has_ext {
        normalize_path(&format!("/{default_prefix}/{reference}"))
    } else {
        normalize_path(&format!("/{default_prefix}/{reference}.yml"))
    }
}

fn parent_dir(path: &str) -> String {
    let norm = normalize_path(path);
    match norm.rsplit_once('/') {
        Some(("", _)) | None => "/".to_string(),
        Some((dir, _)) => dir.to_string(),
    }
}

/// Collapse `.` and `..` components and ensure a leading `/`.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    format!("/{}", parts.join("/"))
}

// ── Ref loaders ──────────────────────────────────────────────────────────────

fn load_materials_ref(
    path: &str,
) -> Result<HashMap<String, MaterialDef>, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::fs::read_to_string(path)?;
    let map: HashMap<String, MaterialDef> = serde_yaml::from_str(&text)?;
    Ok(map)
}

fn load_camera_ref(
    path: &str,
) -> Result<CameraDef, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::fs::read_to_string(path)?;
    let camera: CameraDef = serde_yaml::from_str(&text)?;
    Ok(camera)
}

fn load_lights_ref(
    path: &str,
) -> Result<Vec<LightDef>, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::fs::read_to_string(path)?;
    let lights: Vec<LightDef> = serde_yaml::from_str(&text)?;
    Ok(lights)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_absolute_path() {
        assert_eq!(
            resolve_ref_path("/assets/3d/materials/cyber.yml", "/scenes/foo.yml", "assets/3d"),
            "/assets/3d/materials/cyber.yml"
        );
    }

    #[test]
    fn resolve_relative_path() {
        assert_eq!(
            resolve_ref_path("./materials/cyber.yml", "/assets/3d/portraits.scene3d.yml", "assets/3d"),
            "/assets/3d/materials/cyber.yml"
        );
    }

    #[test]
    fn resolve_plain_name_no_ext() {
        assert_eq!(
            resolve_ref_path("cyber", "/scenes/foo.yml", "assets/3d"),
            "/assets/3d/cyber.yml"
        );
    }

    #[test]
    fn resolve_plain_name_with_ext() {
        assert_eq!(
            resolve_ref_path("cyber.yml", "/scenes/foo.yml", "assets/3d"),
            "/assets/3d/cyber.yml"
        );
    }

    #[test]
    fn resolve_parent_relative() {
        assert_eq!(
            resolve_ref_path("../shared/camera.yml", "/assets/3d/portraits.scene3d.yml", "assets/3d"),
            "/assets/shared/camera.yml"
        );
    }
}
