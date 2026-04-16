use std::collections::HashSet;

use engine_3d::scene3d_format::{load_scene3d, Scene3DDefinition};
use engine_3d::scene3d_resolve::{resolve_scene3d_refs, Scene3DAssetResolver};
use engine_core::assets::AssetRoot;
use engine_core::scene::{Layer, Sprite};

pub struct AssetRootScene3dResolver<'a> {
    asset_root: &'a AssetRoot,
}

impl Scene3DAssetResolver for AssetRootScene3dResolver<'_> {
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = self.asset_root.resolve(asset_path);
        Ok(std::fs::read_to_string(full)?)
    }
}

impl<'a> AssetRootScene3dResolver<'a> {
    pub fn new(asset_root: &'a AssetRoot) -> Self {
        Self { asset_root }
    }
}

pub fn load_and_resolve_scene3d(
    asset_root: &AssetRoot,
    source: &str,
) -> Result<Scene3DDefinition, Box<dyn std::error::Error + Send + Sync>> {
    let path = asset_root.resolve(source);
    let path_str = path.to_string_lossy();
    let mut def = load_scene3d(&path_str)?;
    let resolver = AssetRootScene3dResolver::new(asset_root);
    resolve_scene3d_refs(&mut def, source, &resolver);
    Ok(def)
}

pub fn collect_scene3d_sources(layers: &[Layer]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for layer in layers {
        collect_sources_from_sprites(&layer.sprites, &mut seen, &mut out);
    }
    out
}

fn collect_sources_from_sprites(
    sprites: &[Sprite],
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    for sprite in sprites {
        match sprite {
            Sprite::Scene3D { src, .. } => {
                if seen.insert(src.clone()) {
                    out.push(src.clone());
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                collect_sources_from_sprites(children, seen, out);
            }
            _ => {}
        }
    }
}
