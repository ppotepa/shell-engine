use std::path::Path;

use image::load_from_memory;

use crate::asset_cache::AssetCache;
use crate::repositories::{create_asset_repository, AssetRepository};

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

static IMAGE_CACHE: AssetCache<LoadedRgbaImage> = AssetCache::new();

pub fn load_rgba_image(mod_source: &Path, asset_path: &str) -> Option<LoadedRgbaImage> {
    let normalized = asset_path.trim_start_matches('/');
    let key = format!("{}::{normalized}", mod_source.display());
    IMAGE_CACHE.get_or_load(key, || load_rgba_image_uncached(mod_source, asset_path))
}

pub fn has_image_asset(mod_source: &Path, asset_path: &str) -> bool {
    load_rgba_image(mod_source, asset_path).is_some()
}

fn load_rgba_image_uncached(mod_source: &Path, asset_path: &str) -> Option<LoadedRgbaImage> {
    let repo = create_asset_repository(mod_source).ok()?;
    let bytes = repo.read_asset_bytes(asset_path).ok()?;
    let decoded = load_from_memory(&bytes).ok()?;
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

#[cfg(test)]
mod tests {
    use super::load_rgba_image;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::fs;
    use std::io::Cursor;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn tiny_png_bytes() -> Vec<u8> {
        let img: RgbaImage = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encode png");
        out
    }

    #[test]
    fn loads_image_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/images")).expect("create images dir");
        fs::write(mod_dir.join("assets/images/tiny.png"), tiny_png_bytes()).expect("write png");

        let loaded = load_rgba_image(&mod_dir, "/assets/images/tiny.png").expect("load image");
        assert_eq!(loaded.width, 1);
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn loads_image_from_zip_source() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("assets/images/tiny.png", opts)
            .expect("start png entry");
        std::io::Write::write_all(&mut writer, &tiny_png_bytes()).expect("write png entry");
        writer.finish().expect("finish zip");

        let loaded = load_rgba_image(&zip_path, "/assets/images/tiny.png").expect("load image");
        assert_eq!(loaded.width, 1);
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.pixel(0, 0), Some([255, 0, 0, 255]));
    }
}
