//! Image loading and RGBA pixel access, backed by a process-wide cache keyed on mod source + path.

use std::io::Cursor;
use std::path::Path;

use image::codecs::gif::GifDecoder;
use image::{load_from_memory, AnimationDecoder, Delay, RgbaImage};

use crate::asset_cache::AssetCache;
use crate::asset_source::{
    has_source, load_decoded_source, ModAssetSourceLoader, SourceAdapter, SourceLoader, SourceRef,
};
use crate::EngineError;

/// A decoded RGBA image whose pixels are addressable by (x, y) coordinates.
#[derive(Debug, Clone)]
pub struct LoadedRgbaImage {
    pub width: u32,
    pub height: u32,
    pixels: Vec<[u8; 4]>,
}

impl LoadedRgbaImage {
    /// Returns the `[r, g, b, a]` pixel at `(x, y)`, or `None` if out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let idx = (y as usize)
            .saturating_mul(self.width as usize)
            .saturating_add(x as usize);
        self.pixels.get(idx).copied()
    }
}

#[derive(Debug, Clone)]
pub struct LoadedAnimatedImageFrame {
    pub duration_ms: u64,
    pub image: LoadedRgbaImage,
}

#[derive(Debug, Clone)]
pub struct LoadedAnimatedImage {
    pub width: u32,
    pub height: u32,
    frames: Vec<LoadedAnimatedImageFrame>,
    total_duration_ms: u64,
}

impl LoadedAnimatedImage {
    pub fn first_frame(&self) -> &LoadedRgbaImage {
        &self.frames[0].image
    }

    pub fn frame_at(&self, elapsed_ms: u64) -> &LoadedRgbaImage {
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
pub enum LoadedImageAsset {
    Static(LoadedRgbaImage),
    Animated(LoadedAnimatedImage),
}

impl LoadedImageAsset {
    pub fn first_frame(&self) -> &LoadedRgbaImage {
        match self {
            Self::Static(image) => image,
            Self::Animated(animation) => animation.first_frame(),
        }
    }

    pub fn frame_at(&self, elapsed_ms: u64) -> &LoadedRgbaImage {
        match self {
            Self::Static(image) => image,
            Self::Animated(animation) => animation.frame_at(elapsed_ms),
        }
    }
}

static IMAGE_CACHE: AssetCache<LoadedImageAsset> = AssetCache::new();

struct ImageAssetAdapter;

impl SourceAdapter<LoadedImageAsset> for ImageAssetAdapter {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        _loader: &dyn SourceLoader,
    ) -> Result<LoadedImageAsset, EngineError> {
        if source
            .normalized_value()
            .rsplit('.')
            .next()
            .map(|ext| ext.eq_ignore_ascii_case("gif"))
            .unwrap_or(false)
        {
            return decode_gif(bytes);
        }
        decode_static_image(bytes)
    }
}

fn decode_static_image(bytes: &[u8]) -> Result<LoadedImageAsset, EngineError> {
    let decoded = load_from_memory(bytes).map_err(|_| EngineError::StartupCheckFailed {
        check: "image-decode".to_string(),
        details: "failed to decode image bytes".to_string(),
    })?;
    Ok(LoadedImageAsset::Static(rgba_from_buffer(
        decoded.to_rgba8(),
    )))
}

fn decode_gif(bytes: &[u8]) -> Result<LoadedImageAsset, EngineError> {
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

    let mut loaded_frames = Vec::with_capacity(frames.len());
    let mut total_duration_ms = 0_u64;
    for frame in frames {
        let duration_ms = delay_to_ms(frame.delay());
        total_duration_ms = total_duration_ms.saturating_add(duration_ms);
        loaded_frames.push(LoadedAnimatedImageFrame {
            duration_ms,
            image: rgba_from_buffer(frame.into_buffer()),
        });
    }
    let first = loaded_frames
        .first()
        .map(|frame| &frame.image)
        .expect("gif frames should not be empty");
    Ok(LoadedImageAsset::Animated(LoadedAnimatedImage {
        width: first.width,
        height: first.height,
        frames: loaded_frames,
        total_duration_ms,
    }))
}

fn rgba_from_buffer(rgba: RgbaImage) -> LoadedRgbaImage {
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.pixels().map(|p| p.0).collect();
    LoadedRgbaImage {
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

/// Loads the decoded image asset at `asset_path` from `mod_source`, returning a cached result on repeated calls.
pub fn load_image_asset(mod_source: &Path, asset_path: &str) -> Option<LoadedImageAsset> {
    let loader = ModAssetSourceLoader::new(mod_source).ok()?;
    let source = SourceRef::mod_asset(asset_path);
    load_decoded_source(&IMAGE_CACHE, &loader, &source, &ImageAssetAdapter)
}

/// Loads the first RGBA frame at `asset_path` from `mod_source`, returning a cached result on repeated calls.
pub fn load_rgba_image(mod_source: &Path, asset_path: &str) -> Option<LoadedRgbaImage> {
    load_image_asset(mod_source, asset_path).map(|asset| asset.first_frame().clone())
}

/// Returns `true` if `asset_path` resolves to a loadable image within `mod_source`.
pub fn has_image_asset(mod_source: &Path, asset_path: &str) -> bool {
    let loader = match ModAssetSourceLoader::new(mod_source) {
        Ok(loader) => loader,
        Err(_) => return false,
    };
    let source = SourceRef::mod_asset(asset_path);
    has_source(&loader, &source) && load_image_asset(mod_source, asset_path).is_some()
}

#[cfg(test)]
mod tests {
    use super::{load_image_asset, load_rgba_image, LoadedImageAsset};
    use image::codecs::gif::GifEncoder;
    use image::{Delay, DynamicImage, Frame, ImageFormat, Rgba, RgbaImage};
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

    #[test]
    fn loads_gif_frames_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/images")).expect("create images dir");
        fs::write(mod_dir.join("assets/images/tiny.gif"), tiny_gif_bytes()).expect("write gif");

        let loaded = load_image_asset(&mod_dir, "/assets/images/tiny.gif").expect("load gif");
        match loaded {
            LoadedImageAsset::Animated(animation) => {
                assert_eq!(animation.width, 1);
                assert_eq!(animation.height, 1);
                assert_eq!(animation.frame_at(0).pixel(0, 0), Some([255, 0, 0, 255]));
                assert_eq!(animation.frame_at(99).pixel(0, 0), Some([255, 0, 0, 255]));
                assert_eq!(animation.frame_at(100).pixel(0, 0), Some([0, 0, 255, 255]));
                assert_eq!(animation.frame_at(299).pixel(0, 0), Some([0, 0, 255, 255]));
                assert_eq!(animation.frame_at(300).pixel(0, 0), Some([255, 0, 0, 255]));
            }
            LoadedImageAsset::Static(_) => panic!("expected animated gif asset"),
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
}
