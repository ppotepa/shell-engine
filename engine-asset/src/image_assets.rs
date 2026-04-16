//! Decoded image assets with process-wide caching keyed by mod source + asset path.

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use crate::build_keys::{resolve_image_asset_key, ImageAssetKey};
use crate::ModAssetSourceLoader;
use engine_core::asset_cache::AssetCache;
use engine_core::asset_source::{
    has_source, load_decoded_source, SourceAdapter, SourceLoader, SourceRef,
};
use engine_error::EngineError;
use image::codecs::gif::GifDecoder;
use image::{load_from_memory, AnimationDecoder, Delay, RgbaImage};

/// A decoded RGBA image whose pixels are addressable by `(x, y)` coordinates.
#[derive(Debug, Clone)]
pub struct RgbaImageAsset {
    pub width: u32,
    pub height: u32,
    pixels: Vec<[u8; 4]>,
}

impl RgbaImageAsset {
    /// Returns the `[r, g, b, a]` pixel at `(x, y)`, or `None` if out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let idx = (y as usize)
            .saturating_mul(self.width as usize)
            .saturating_add(x as usize);
        self.pixels.get(idx).copied()
    }

    #[allow(dead_code)]
    pub(crate) fn from_pixels(width: u32, height: u32, pixels: Vec<[u8; 4]>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedImageAssetFrame {
    pub duration_ms: u64,
    pub image: RgbaImageAsset,
}

#[derive(Debug, Clone)]
pub struct AnimatedImageAsset {
    pub width: u32,
    pub height: u32,
    frames: Vec<AnimatedImageAssetFrame>,
    total_duration_ms: u64,
}

impl AnimatedImageAsset {
    pub fn first_frame(&self) -> &RgbaImageAsset {
        &self.frames[0].image
    }

    pub fn frame_at(&self, elapsed_ms: u64) -> &RgbaImageAsset {
        if self.frames.len() == 1 {
            return self.first_frame();
        }
        let loop_ms = self.total_duration_ms.max(1);
        let mut cursor_ms = elapsed_ms % loop_ms;
        for frame in &self.frames {
            if cursor_ms < frame.duration_ms {
                return &frame.image;
            }
            cursor_ms = cursor_ms.saturating_sub(frame.duration_ms);
        }
        self.first_frame()
    }
}

#[derive(Debug, Clone)]
pub enum ImageAsset {
    Static(RgbaImageAsset),
    Animated(AnimatedImageAsset),
}

impl ImageAsset {
    pub fn first_frame(&self) -> &RgbaImageAsset {
        match self {
            Self::Static(image) => image,
            Self::Animated(animation) => animation.first_frame(),
        }
    }

    pub fn frame_at(&self, elapsed_ms: u64) -> &RgbaImageAsset {
        match self {
            Self::Static(image) => image,
            Self::Animated(animation) => animation.frame_at(elapsed_ms),
        }
    }

    pub fn is_animated(&self) -> bool {
        matches!(self, Self::Animated(_))
    }
}

static IMAGE_CACHE: AssetCache<ImageAsset> = AssetCache::new();

struct ImageAssetAdapter;

impl SourceAdapter<ImageAsset> for ImageAssetAdapter {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        _loader: &dyn SourceLoader,
    ) -> Result<ImageAsset, Box<dyn std::error::Error + Send + Sync>> {
        if source
            .normalized_value()
            .rsplit('.')
            .next()
            .map(|ext| ext.eq_ignore_ascii_case("gif"))
            .unwrap_or(false)
        {
            return Ok(decode_gif(bytes)?);
        }
        Ok(decode_static_image(bytes)?)
    }
}

fn decode_static_image(bytes: &[u8]) -> Result<ImageAsset, EngineError> {
    let decoded = load_from_memory(bytes).map_err(|_| EngineError::StartupCheckFailed {
        check: "image-decode".to_string(),
        details: "failed to decode image bytes".to_string(),
    })?;
    Ok(ImageAsset::Static(rgba_from_buffer(decoded.to_rgba8())))
}

fn decode_gif(bytes: &[u8]) -> Result<ImageAsset, EngineError> {
    let decoder =
        GifDecoder::new(Cursor::new(bytes)).map_err(|_| EngineError::StartupCheckFailed {
            check: "image-decode".to_string(),
            details: "failed to decode GIF bytes".to_string(),
        })?;
    let frames =
        decoder
            .into_frames()
            .collect_frames()
            .map_err(|_| EngineError::StartupCheckFailed {
                check: "image-decode".to_string(),
                details: "failed to decode GIF frames".to_string(),
            })?;
    if frames.is_empty() {
        return Err(EngineError::StartupCheckFailed {
            check: "image-decode".to_string(),
            details: "GIF contained no frames".to_string(),
        });
    }

    let mut decoded_frames = Vec::with_capacity(frames.len());
    let mut total_duration_ms = 0_u64;
    for frame in frames {
        let duration_ms = delay_to_ms(frame.delay());
        total_duration_ms = total_duration_ms.saturating_add(duration_ms);
        decoded_frames.push(AnimatedImageAssetFrame {
            duration_ms,
            image: rgba_from_buffer(frame.into_buffer()),
        });
    }
    let first = decoded_frames
        .first()
        .map(|frame| &frame.image)
        .expect("gif frames should not be empty");
    Ok(ImageAsset::Animated(AnimatedImageAsset {
        width: first.width,
        height: first.height,
        frames: decoded_frames,
        total_duration_ms,
    }))
}

fn rgba_from_buffer(rgba: RgbaImage) -> RgbaImageAsset {
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.pixels().map(|p| p.0).collect();
    RgbaImageAsset {
        width,
        height,
        pixels,
    }
}

fn delay_to_ms(delay: Delay) -> u64 {
    let (numer, denom) = delay.numer_denom_ms();
    let numer = numer as u64;
    let denom = denom.max(1) as u64;
    numer.div_ceil(denom).max(1)
}

/// Loads the decoded image asset at `asset_path` from `mod_source`.
pub fn load_image_asset(mod_source: &Path, asset_path: &str) -> Option<Arc<ImageAsset>> {
    let key = resolve_image_asset_key(asset_path);
    load_image_asset_with_key(mod_source, &key)
}

/// Loads the decoded image asset at the canonical image key from `mod_source`.
pub fn load_image_asset_with_key(
    mod_source: &Path,
    image_key: &ImageAssetKey,
) -> Option<Arc<ImageAsset>> {
    let loader = ModAssetSourceLoader::new(mod_source).ok()?;
    let source = SourceRef::mod_asset(image_key.as_str());
    load_decoded_source(&IMAGE_CACHE, &loader, &source, &ImageAssetAdapter)
}

/// Loads the first RGBA frame at `asset_path` from `mod_source`.
pub fn load_rgba_image(mod_source: &Path, asset_path: &str) -> Option<RgbaImageAsset> {
    let key = resolve_image_asset_key(asset_path);
    load_rgba_image_with_key(mod_source, &key)
}

/// Loads the first RGBA frame at the canonical image key from `mod_source`.
pub fn load_rgba_image_with_key(
    mod_source: &Path,
    image_key: &ImageAssetKey,
) -> Option<RgbaImageAsset> {
    load_image_asset_with_key(mod_source, image_key).map(|asset| asset.first_frame().clone())
}

/// Returns `true` if `asset_path` resolves to a loadable image within `mod_source`.
pub fn has_image_asset(mod_source: &Path, asset_path: &str) -> bool {
    let key = resolve_image_asset_key(asset_path);
    has_image_asset_with_key(mod_source, &key)
}

/// Returns `true` if `image_key` resolves to a loadable image within `mod_source`.
pub fn has_image_asset_with_key(mod_source: &Path, image_key: &ImageAssetKey) -> bool {
    let loader = match ModAssetSourceLoader::new(mod_source) {
        Ok(loader) => loader,
        Err(_) => return false,
    };
    let source = SourceRef::mod_asset(image_key.as_str());
    has_source(&loader, &source) && load_image_asset_with_key(mod_source, image_key).is_some()
}

#[cfg(test)]
mod tests {
    use super::{
        has_image_asset, has_image_asset_with_key, load_image_asset, load_image_asset_with_key,
        load_rgba_image, load_rgba_image_with_key, ImageAsset,
    };
    use crate::build_keys::resolve_image_asset_key;
    use image::codecs::gif::GifEncoder;
    use image::{Delay, DynamicImage, Frame, ImageFormat, Rgba, RgbaImage};
    use std::fs;
    use std::io::Cursor;
    use std::sync::Arc;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn tiny_png_bytes() -> Vec<u8> {
        tiny_png_bytes_with_rgba([255, 0, 0, 255])
    }

    fn tiny_png_bytes_with_rgba(rgba: [u8; 4]) -> Vec<u8> {
        let img: RgbaImage = RgbaImage::from_pixel(1, 1, Rgba(rgba));
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encode png");
        out
    }

    fn tiny_blue_png_bytes() -> Vec<u8> {
        tiny_png_bytes_with_rgba([0, 0, 255, 255])
    }

    fn write_png_to_dir(mod_dir: &std::path::Path, rel_path: &str, bytes: &[u8]) {
        let full_path = mod_dir.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("create image parent dirs");
        }
        fs::write(full_path, bytes).expect("write image bytes");
    }

    fn write_png_to_zip(zip_path: &std::path::Path, rel_path: &str, bytes: &[u8]) {
        let file = fs::File::create(zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer.start_file(rel_path, opts).expect("start zip entry");
        std::io::Write::write_all(&mut writer, bytes).expect("write zip image");
        writer.finish().expect("finish zip");
    }

    #[test]
    fn shares_cached_image_asset_for_same_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &tiny_png_bytes());

        let with_leading =
            load_image_asset(&mod_dir, "/assets/images/tiny.png").expect("load with leading slash");
        let without_leading =
            load_image_asset(&mod_dir, "assets/images/tiny.png").expect("load without leading");
        assert!(
            Arc::ptr_eq(&with_leading, &without_leading),
            "same normalized source should reuse decoded cache entry"
        );
    }

    #[test]
    fn shares_cached_image_asset_for_same_zip_source() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        write_png_to_zip(&zip_path, "assets/images/tiny.png", &tiny_png_bytes());

        let with_leading = load_image_asset(&zip_path, "/assets/images/tiny.png")
            .expect("load with leading slash");
        let without_leading =
            load_image_asset(&zip_path, "assets/images/tiny.png").expect("load without leading");
        assert!(
            Arc::ptr_eq(&with_leading, &without_leading),
            "zip sources should also share decoded cache entries"
        );
    }

    #[test]
    fn does_not_share_cached_images_across_mod_sources() {
        let temp = tempdir().expect("temp dir");
        let mod_a = temp.path().join("mod-a");
        let mod_b = temp.path().join("mod-b");
        write_png_to_dir(&mod_a, "assets/images/tiny.png", &tiny_png_bytes());
        write_png_to_dir(&mod_b, "assets/images/tiny.png", &tiny_blue_png_bytes());

        let from_a = load_image_asset(&mod_a, "/assets/images/tiny.png").expect("load a");
        let from_b = load_image_asset(&mod_b, "/assets/images/tiny.png").expect("load b");
        assert!(
            !Arc::ptr_eq(&from_a, &from_b),
            "cache entries must be isolated by mod source"
        );
        assert_eq!(from_a.first_frame().pixel(0, 0), Some([255, 0, 0, 255]));
        assert_eq!(from_b.first_frame().pixel(0, 0), Some([0, 0, 255, 255]));
    }

    #[test]
    fn has_image_asset_accepts_normalized_and_absolute_asset_refs() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &tiny_png_bytes());

        assert!(has_image_asset(&mod_dir, "/assets/images/tiny.png"));
        assert!(has_image_asset(&mod_dir, "assets/images/tiny.png"));
        assert!(!has_image_asset(&mod_dir, "/assets/images/missing.png"));
    }

    #[test]
    fn directory_and_zip_sources_decode_to_same_pixels() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        let zip_path = temp.path().join("mod.zip");
        let png = tiny_png_bytes();
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &png);
        write_png_to_zip(&zip_path, "assets/images/tiny.png", &png);

        let dir_image = load_rgba_image(&mod_dir, "/assets/images/tiny.png").expect("load dir");
        let zip_image = load_rgba_image(&zip_path, "/assets/images/tiny.png").expect("load zip");
        assert_eq!(dir_image.width, zip_image.width);
        assert_eq!(dir_image.height, zip_image.height);
        assert_eq!(dir_image.pixel(0, 0), zip_image.pixel(0, 0));
    }

    #[test]
    fn directory_and_zip_have_matching_has_image_semantics() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        let zip_path = temp.path().join("mod.zip");
        let png = tiny_png_bytes();
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &png);
        write_png_to_zip(&zip_path, "assets/images/tiny.png", &png);

        assert!(has_image_asset(&mod_dir, "/assets/images/tiny.png"));
        assert!(has_image_asset(&zip_path, "/assets/images/tiny.png"));
        assert!(!has_image_asset(&mod_dir, "/assets/images/missing.png"));
        assert!(!has_image_asset(&zip_path, "/assets/images/missing.png"));
    }

    #[test]
    fn missing_image_returns_none_for_directory_and_zip_sources() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        let zip_path = temp.path().join("mod.zip");
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &tiny_png_bytes());
        write_png_to_zip(&zip_path, "assets/images/tiny.png", &tiny_png_bytes());

        assert!(load_image_asset(&mod_dir, "/assets/images/missing.png").is_none());
        assert!(load_image_asset(&zip_path, "/assets/images/missing.png").is_none());
        assert!(load_rgba_image(&mod_dir, "/assets/images/missing.png").is_none());
        assert!(load_rgba_image(&zip_path, "/assets/images/missing.png").is_none());
    }

    #[test]
    fn first_frame_surface_matches_decoded_asset_surface_for_directory_and_zip() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        let zip_path = temp.path().join("mod.zip");
        let png = tiny_png_bytes();
        write_png_to_dir(&mod_dir, "assets/images/tiny.png", &png);
        write_png_to_zip(&zip_path, "assets/images/tiny.png", &png);

        let dir_decoded =
            load_image_asset(&mod_dir, "/assets/images/tiny.png").expect("dir decoded");
        let dir_first = load_rgba_image(&mod_dir, "/assets/images/tiny.png").expect("dir first");
        assert_eq!(dir_first.width, dir_decoded.first_frame().width);
        assert_eq!(dir_first.height, dir_decoded.first_frame().height);
        assert_eq!(dir_first.pixel(0, 0), dir_decoded.first_frame().pixel(0, 0));

        let zip_decoded =
            load_image_asset(&zip_path, "/assets/images/tiny.png").expect("zip decoded");
        let zip_first = load_rgba_image(&zip_path, "/assets/images/tiny.png").expect("zip first");
        assert_eq!(zip_first.width, zip_decoded.first_frame().width);
        assert_eq!(zip_first.height, zip_decoded.first_frame().height);
        assert_eq!(zip_first.pixel(0, 0), zip_decoded.first_frame().pixel(0, 0));
    }

    fn tiny_red_png_bytes() -> Vec<u8> {
        let img: RgbaImage = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encode png");
        out
    }

    fn tiny_gif_bytes() -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = GifEncoder::new(&mut out);
        let red = Frame::from_parts(
            RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255])),
            0,
            0,
            Delay::from_numer_denom_ms(100, 1),
        );
        let blue = Frame::from_parts(
            RgbaImage::from_pixel(1, 1, Rgba([0, 0, 255, 255])),
            0,
            0,
            Delay::from_numer_denom_ms(200, 1),
        );
        encoder
            .encode_frames(vec![red, blue])
            .expect("encode gif frames");
        drop(encoder);
        out
    }

    #[test]
    fn loads_image_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/images")).expect("create images dir");
        fs::write(mod_dir.join("assets/images/tiny.png"), tiny_red_png_bytes()).expect("write png");

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
        std::io::Write::write_all(&mut writer, &tiny_red_png_bytes()).expect("write png entry");
        writer.finish().expect("finish zip");

        let loaded = load_rgba_image(&zip_path, "/assets/images/tiny.png").expect("load image");
        assert_eq!(loaded.width, 1);
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn loads_gif_frames_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/images")).expect("create images dir");
        fs::write(mod_dir.join("assets/images/tiny.gif"), tiny_gif_bytes()).expect("write gif");

        let loaded = load_image_asset(&mod_dir, "/assets/images/tiny.gif").expect("load gif");
        match loaded.as_ref() {
            ImageAsset::Animated(animation) => {
                assert_eq!(animation.width, 1);
                assert_eq!(animation.height, 1);
                assert_eq!(animation.frame_at(0).pixel(0, 0), Some([255, 0, 0, 255]));
                assert_eq!(animation.frame_at(99).pixel(0, 0), Some([255, 0, 0, 255]));
                assert_eq!(animation.frame_at(100).pixel(0, 0), Some([0, 0, 255, 255]));
                assert_eq!(animation.frame_at(299).pixel(0, 0), Some([0, 0, 255, 255]));
                assert_eq!(animation.frame_at(300).pixel(0, 0), Some([255, 0, 0, 255]));
            }
            ImageAsset::Static(_) => panic!("expected animated gif asset, got static"),
        }
    }

    #[test]
    fn loads_gif_first_frame_from_zip_source() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("assets/images/tiny.gif", opts)
            .expect("start gif entry");
        std::io::Write::write_all(&mut writer, &tiny_gif_bytes()).expect("write gif entry");
        writer.finish().expect("finish zip");

        let loaded = load_rgba_image(&zip_path, "/assets/images/tiny.gif").expect("load image");
        assert_eq!(loaded.width, 1);
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn key_seam_unifies_2d_and_3d_image_asset_consumption() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        write_png_to_dir(&mod_dir, "assets/images/shared.png", &tiny_png_bytes());

        let from_2d = load_image_asset(&mod_dir, "/assets/images/shared.png")
            .expect("2d consumer should load image");
        let key = resolve_image_asset_key("assets/images/shared.png");
        let from_3d =
            load_image_asset_with_key(&mod_dir, &key).expect("3d consumer should load image");

        assert!(
            Arc::ptr_eq(&from_2d, &from_3d),
            "2d and 3d consumers should share canonical image cache entry"
        );
    }

    #[test]
    fn key_based_and_path_based_image_queries_match_semantics() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        write_png_to_dir(&mod_dir, "assets/images/shared.png", &tiny_png_bytes());
        let key = resolve_image_asset_key("/assets/images/shared.png");
        let missing_key = resolve_image_asset_key("/assets/images/missing.png");

        assert!(has_image_asset_with_key(&mod_dir, &key));
        assert!(has_image_asset(&mod_dir, "assets/images/shared.png"));
        assert!(!has_image_asset_with_key(&mod_dir, &missing_key));

        let key_loaded = load_rgba_image_with_key(&mod_dir, &key).expect("load key image");
        let path_loaded =
            load_rgba_image(&mod_dir, "/assets/images/shared.png").expect("load path image");
        assert_eq!(key_loaded.pixel(0, 0), path_loaded.pixel(0, 0));
    }

    #[test]
    fn path_and_key_seam_normalize_windows_style_asset_paths() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        write_png_to_dir(&mod_dir, "assets/images/shared.png", &tiny_png_bytes());

        let from_path = load_image_asset(&mod_dir, r".\assets\images\shared.png")
            .expect("windows-style path should load");
        let key = resolve_image_asset_key(r"assets\images\shared.png");
        let from_key =
            load_image_asset_with_key(&mod_dir, &key).expect("normalized key should load");

        assert!(Arc::ptr_eq(&from_path, &from_key));
        assert!(has_image_asset_with_key(&mod_dir, &key));
        assert!(has_image_asset(&mod_dir, r".\assets\images\shared.png"));
    }
}
