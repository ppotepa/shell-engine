//! Image sprite rendering — maps RGBA pixels to terminal cells using the active render mode.

use engine_core::color::Color;

use engine_core::assets::AssetRoot;
use engine_core::buffer::{Buffer, TRUE_BLACK};
use engine_core::scene::{SceneRenderedMode, SpriteSizePreset};
use engine_render::image_loader::{self, LoadedRgbaImage};

const ALPHA_THRESHOLD: u8 = 16;

/// #9 opt-img-sheetview: zero-copy view into a spritesheet frame.
/// Avoids cloning/allocating pixels for sub-frame selection.
struct ImageView<'a> {
    source: &'a LoadedRgbaImage,
    offset_x: u32,
    offset_y: u32,
    width: u32,
    height: u32,
}

impl<'a> ImageView<'a> {
    fn full(image: &'a LoadedRgbaImage) -> Self {
        Self {
            source: image,
            offset_x: 0,
            offset_y: 0,
            width: image.width,
            height: image.height,
        }
    }

    fn sub(image: &'a LoadedRgbaImage, x: u32, y: u32, w: u32, h: u32) -> Self {
        Self {
            source: image,
            offset_x: x,
            offset_y: y,
            width: w,
            height: h,
        }
    }

    fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.source.pixel(self.offset_x + x, self.offset_y + y)
    }
}

pub fn render_image_content(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
    sheet_columns: Option<u16>,
    sheet_rows: Option<u16>,
    frame_index: Option<u16>,
    mode: SceneRenderedMode,
    elapsed_ms: u64,
    asset_root: Option<&AssetRoot>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let Some(root) = asset_root else {
        return;
    };
    let Some(image_asset) = image_loader::load_image_asset(root.mod_source(), source) else {
        return;
    };
    let base_image = image_asset.frame_at(elapsed_ms);
    let image = select_spritesheet_frame(base_image, sheet_columns, sheet_rows, frame_index);
    let (target_w, target_h) = resolve_image_dimensions(&image, mode, req_width, req_height, size);
    if target_w == 0 || target_h == 0 {
        return;
    }

    // Mark the full sprite bounding box dirty for all image sprites so that
    // DirtyRegionDiff always covers the area even when frame changes or pixels shrink.
    // This is essential for animated (GIF) images, but also helps non-animated images
    // that may have transparency changes due to scene effects.
    buf.mark_dirty_region(x, y, target_w, target_h);

    match mode {
        SceneRenderedMode::Cell => rasterize_image_cell(&image, target_w, target_h, x, y, buf),
        SceneRenderedMode::HalfBlock => {
            rasterize_image_halfblock(&image, target_w, target_h, x, y, buf)
        }
        SceneRenderedMode::QuadBlock => {
            rasterize_image_quadblock(&image, target_w, target_h, x, y, buf)
        }
        SceneRenderedMode::Braille => {
            rasterize_image_braille(&image, target_w, target_h, x, y, buf)
        }
    }
}

pub fn image_sprite_dimensions(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
    sheet_columns: Option<u16>,
    sheet_rows: Option<u16>,
    frame_index: Option<u16>,
    mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> (u16, u16) {
    let Some(root) = asset_root else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    let Some(image_asset) = image_loader::load_image_asset(root.mod_source(), source) else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    let image = select_spritesheet_frame(
        image_asset.first_frame(),
        sheet_columns,
        sheet_rows,
        frame_index,
    );
    resolve_image_dimensions(&image, mode, req_width, req_height, size)
}

fn select_spritesheet_frame<'a>(
    image: &'a LoadedRgbaImage,
    sheet_columns: Option<u16>,
    sheet_rows: Option<u16>,
    frame_index: Option<u16>,
) -> ImageView<'a> {
    let cols = sheet_columns.unwrap_or(1).max(1) as u32;
    let rows = sheet_rows.unwrap_or(1).max(1) as u32;
    if cols == 1 && rows == 1 {
        return ImageView::full(image);
    }
    let cell_w = (image.width / cols).max(1);
    let cell_h = (image.height / rows).max(1);
    let total = cols.saturating_mul(rows).max(1);
    let index = (frame_index.unwrap_or(0) as u32).min(total.saturating_sub(1));
    let col = index % cols;
    let row = index / cols;
    let start_x = col.saturating_mul(cell_w);
    let start_y = row.saturating_mul(cell_h);
    let end_x = if col + 1 == cols {
        image.width
    } else {
        (start_x + cell_w).min(image.width)
    };
    let end_y = if row + 1 == rows {
        image.height
    } else {
        (start_y + cell_h).min(image.height)
    };

    let out_w = end_x.saturating_sub(start_x).max(1);
    let out_h = end_y.saturating_sub(start_y).max(1);
    ImageView::sub(image, start_x, start_y, out_w, out_h)
}

fn resolve_image_dimensions(
    image: &ImageView,
    mode: SceneRenderedMode,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
) -> (u16, u16) {
    let (natural_w, natural_h) = natural_image_dimensions(image, mode);
    match (req_width, req_height) {
        (Some(w), Some(h)) => (w.max(1), h.max(1)),
        (Some(w), None) => {
            let h = ((natural_h as u32 * w.max(1) as u32) / natural_w.max(1) as u32).max(1);
            (w.max(1), h.min(u16::MAX as u32) as u16)
        }
        (None, Some(h)) => {
            let w = ((natural_w as u32 * h.max(1) as u32) / natural_h.max(1) as u32).max(1);
            (w.min(u16::MAX as u32) as u16, h.max(1))
        }
        (None, None) => match size {
            Some(size) => scale_dimensions(natural_w, natural_h, size.image_scale_ratio()),
            None => (natural_w.max(1), natural_h.max(1)),
        },
    }
}

fn scale_dimensions(width: u16, height: u16, ratio: (u16, u16)) -> (u16, u16) {
    let (num, den) = ratio;
    let scaled_w = ((width.max(1) as u32 * num.max(1) as u32) / den.max(1) as u32).max(1);
    let scaled_h = ((height.max(1) as u32 * num.max(1) as u32) / den.max(1) as u32).max(1);
    (
        scaled_w.min(u16::MAX as u32) as u16,
        scaled_h.min(u16::MAX as u32) as u16,
    )
}

fn natural_image_dimensions(image: &ImageView, mode: SceneRenderedMode) -> (u16, u16) {
    let w = image.width.max(1);
    let h = image.height.max(1);
    let (cell_w, cell_h) = match mode {
        SceneRenderedMode::Cell => (w, h),
        SceneRenderedMode::HalfBlock => (w, h.div_ceil(2)),
        SceneRenderedMode::QuadBlock => (w.div_ceil(2), h.div_ceil(2)),
        SceneRenderedMode::Braille => (w.div_ceil(2), h.div_ceil(4)),
    };
    (
        cell_w.min(u16::MAX as u32) as u16,
        cell_h.min(u16::MAX as u32) as u16,
    )
}

fn rasterize_image_cell(
    image: &ImageView,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    for oy in 0..target_h {
        for ox in 0..target_w {
            let px = sample_scaled(
                image,
                ox as u32,
                oy as u32,
                target_w as u32,
                target_h as u32,
            );
            if px[3] < ALPHA_THRESHOLD {
                continue;
            }
            buf.set(x + ox, y + oy, '█', rgb_color(px), TRUE_BLACK);
        }
    }
}

fn rasterize_image_halfblock(
    image: &ImageView,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_h = target_h as u32 * 2;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let top = sample_scaled(image, ox as u32, oy as u32 * 2, target_w as u32, virtual_h);
            let bottom = sample_scaled(
                image,
                ox as u32,
                oy as u32 * 2 + 1,
                target_w as u32,
                virtual_h,
            );
            let top_on = top[3] >= ALPHA_THRESHOLD;
            let bottom_on = bottom[3] >= ALPHA_THRESHOLD;
            let (symbol, fg, bg) = match (top_on, bottom_on) {
                (false, false) => continue,
                (true, false) => ('▀', rgb_color(top), TRUE_BLACK),
                (false, true) => ('▄', rgb_color(bottom), TRUE_BLACK),
                (true, true) => ('▀', rgb_color(top), rgb_color(bottom)),
            };
            buf.set(x + ox, y + oy, symbol, fg, bg);
        }
    }
}

fn rasterize_image_quadblock(
    image: &ImageView,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_w = target_w as u32 * 2;
    let virtual_h = target_h as u32 * 2;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let tl = sample_scaled(image, ox as u32 * 2, oy as u32 * 2, virtual_w, virtual_h);
            let tr = sample_scaled(
                image,
                ox as u32 * 2 + 1,
                oy as u32 * 2,
                virtual_w,
                virtual_h,
            );
            let bl = sample_scaled(
                image,
                ox as u32 * 2,
                oy as u32 * 2 + 1,
                virtual_w,
                virtual_h,
            );
            let br = sample_scaled(
                image,
                ox as u32 * 2 + 1,
                oy as u32 * 2 + 1,
                virtual_w,
                virtual_h,
            );

            // #10 opt-img-quadstack: stack array instead of Vec to avoid heap allocs.
            let mut mask = 0u8;
            let mut colours = [[0u8; 4]; 4];
            let mut count = 0;
            if tl[3] >= ALPHA_THRESHOLD {
                mask |= 0b0001;
                colours[count] = tl;
                count += 1;
            }
            if tr[3] >= ALPHA_THRESHOLD {
                mask |= 0b0010;
                colours[count] = tr;
                count += 1;
            }
            if bl[3] >= ALPHA_THRESHOLD {
                mask |= 0b0100;
                colours[count] = bl;
                count += 1;
            }
            if br[3] >= ALPHA_THRESHOLD {
                mask |= 0b1000;
                colours[count] = br;
                count += 1;
            }
            let Some(symbol) = quadrant_char(mask) else {
                continue;
            };
            let fg = average_rgb(&colours[..count]);
            buf.set(x + ox, y + oy, symbol, fg, TRUE_BLACK);
        }
    }
}

fn rasterize_image_braille(
    image: &ImageView,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_w = target_w as u32 * 2;
    let virtual_h = target_h as u32 * 4;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let sx = ox as u32 * 2;
            let sy = oy as u32 * 4;
            let samples = [
                sample_scaled(image, sx, sy, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 1, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 2, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 1, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 2, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 3, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 3, virtual_w, virtual_h),
            ];
            // #10 opt-img-quadstack: stack array instead of Vec to avoid heap allocs.
            let mut mask = 0u8;
            let mut colours = [[0u8; 4]; 8];
            let mut count = 0;
            for (i, px) in samples.iter().enumerate() {
                if px[3] < ALPHA_THRESHOLD {
                    continue;
                }
                mask |= 1 << i;
                colours[count] = *px;
                count += 1;
            }
            let Some(symbol) = braille_char(mask) else {
                continue;
            };
            let fg = average_rgb(&colours[..count]);
            buf.set(x + ox, y + oy, symbol, fg, TRUE_BLACK);
        }
    }
}


#[inline]
fn sample_scaled(image: &ImageView, x: u32, y: u32, virtual_w: u32, virtual_h: u32) -> [u8; 4] {
    let vw = virtual_w.max(1);
    let vh = virtual_h.max(1);
    let sx = ((x as u64).saturating_mul(image.width as u64) / vw as u64)
        .min(image.width.saturating_sub(1) as u64) as u32;
    let sy = ((y as u64).saturating_mul(image.height as u64) / vh as u64)
        .min(image.height.saturating_sub(1) as u64) as u32;
    image.pixel(sx, sy).unwrap_or([0, 0, 0, 0])
}

#[inline]
fn rgb_color(px: [u8; 4]) -> Color {
    Color::Rgb {
        r: px[0],
        g: px[1],
        b: px[2],
    }
}

#[inline]
fn average_rgb(colours: &[[u8; 4]]) -> Color {
    if colours.is_empty() {
        return TRUE_BLACK;
    }
    let mut rs = 0u32;
    let mut gs = 0u32;
    let mut bs = 0u32;
    for c in colours {
        rs += c[0] as u32;
        gs += c[1] as u32;
        bs += c[2] as u32;
    }
    let len = colours.len() as u32;
    Color::Rgb {
        r: (rs / len) as u8,
        g: (gs / len) as u8,
        b: (bs / len) as u8,
    }
}

fn quadrant_char(mask: u8) -> Option<char> {
    match mask {
        0 => None,
        1 => Some('▘'),
        2 => Some('▝'),
        3 => Some('▀'),
        4 => Some('▖'),
        5 => Some('▌'),
        6 => Some('▞'),
        7 => Some('▛'),
        8 => Some('▗'),
        9 => Some('▚'),
        10 => Some('▐'),
        11 => Some('▜'),
        12 => Some('▄'),
        13 => Some('▙'),
        14 => Some('▟'),
        15 => Some('█'),
        _ => None,
    }
}

fn braille_char(mask: u8) -> Option<char> {
    if mask == 0 {
        None
    } else {
        char::from_u32(0x2800 + mask as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::scale_dimensions;
    use engine_core::scene::SpriteSizePreset;

    #[test]
    fn scales_image_dimensions_from_size_preset() {
        assert_eq!(
            scale_dimensions(300, 150, SpriteSizePreset::Small.image_scale_ratio()),
            (100, 50)
        );
        assert_eq!(
            scale_dimensions(300, 150, SpriteSizePreset::Medium.image_scale_ratio()),
            (150, 75)
        );
        assert_eq!(
            scale_dimensions(300, 150, SpriteSizePreset::Large.image_scale_ratio()),
            (200, 100)
        );
    }
}
