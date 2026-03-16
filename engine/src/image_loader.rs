use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use image::ImageReader;

use crate::assets::resolve_asset_path;

#[derive(Debug, Clone)]
pub struct LoadedRgbaImage {
    pub width: u32,
    pub height: u32,
    pixels: Vec<[u8; 4]>,
}

impl LoadedRgbaImage {
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let idx = (y as usize)
            .saturating_mul(self.width as usize)
            .saturating_add(x as usize);
        self.pixels.get(idx).copied()
    }
}

static IMAGE_CACHE: OnceLock<Mutex<HashMap<String, Option<LoadedRgbaImage>>>> = OnceLock::new();

pub fn load_rgba_image(mod_source: &Path, asset_path: &str) -> Option<LoadedRgbaImage> {
    let full_path = resolve_asset_path(mod_source, asset_path);
    let key = full_path.to_string_lossy().to_string();
    let cache = IMAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&key) {
            return cached.clone();
        }
    }

    let loaded = load_rgba_image_uncached(&full_path);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, loaded.clone());
    }
    loaded
}

pub fn has_image_asset(mod_source: &Path, asset_path: &str) -> bool {
    load_rgba_image(mod_source, asset_path).is_some()
}

fn load_rgba_image_uncached(full_path: &Path) -> Option<LoadedRgbaImage> {
    let reader = ImageReader::open(full_path).ok()?;
    let decoded = reader.decode().ok()?;
    let rgba = decoded.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.pixels().map(|p| p.0).collect();
    Some(LoadedRgbaImage {
        width,
        height,
        pixels,
    })
}
