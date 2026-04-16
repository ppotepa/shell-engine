//! Image sprite rendering for the composed frame buffer.

use engine_asset::{load_image_asset, RgbaImageAsset};
use engine_core::assets::AssetRoot;
use engine_core::buffer::{Buffer, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::scene::SpriteSizePreset;

const ALPHA_THRESHOLD: u8 = 16;

/// #9 opt-img-sheetview: zero-copy view into a spritesheet frame.
/// Avoids cloning/allocating pixels for sub-frame selection.
struct ImageView<'a> {
    source: &'a RgbaImageAsset,
    offset_x: u32,
    offset_y: u32,
    width: u32,
    height: u32,
}

impl<'a> ImageView<'a> {
    fn full(image: &'a RgbaImageAsset) -> Self {
        Self {
            source: image,
            offset_x: 0,
            offset_y: 0,
            width: image.width,
            height: image.height,
        }
    }

    fn sub(image: &'a RgbaImageAsset, x: u32, y: u32, w: u32, h: u32) -> Self {
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

#[allow(clippy::too_many_arguments)]
pub fn render_image_content(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
    sheet_columns: Option<u16>,
    sheet_rows: Option<u16>,
    frame_index: Option<u16>,
    elapsed_ms: u64,
    asset_root: Option<&AssetRoot>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let Some(root) = asset_root else {
        return;
    };
    let Some(image_asset) = load_image_asset(root.mod_source(), source) else {
        return;
    };
    let base_image = image_asset.frame_at(elapsed_ms);
    let image = select_spritesheet_frame(base_image, sheet_columns, sheet_rows, frame_index);
    let (target_w, target_h) = resolve_image_dimensions(&image, req_width, req_height, size);
    if target_w == 0 || target_h == 0 {
        return;
    }

    buf.mark_dirty_region(x, y, target_w, target_h);

    if let Some(pc) = &mut buf.pixel_canvas {
        let (virt_w, virt_h) = (target_w as u32, target_h as u32);
        let pc_w = pc.width as usize;
        let base_vx = x as usize;
        let base_vy = y as usize;
        for vy in 0..virt_h {
            for vx in 0..virt_w {
                let px = sample_scaled(&image, vx, vy, virt_w, virt_h);
                if px[3] < ALPHA_THRESHOLD {
                    continue;
                }
                let px_x = base_vx + vx as usize;
                let px_y = base_vy + vy as usize;
                if px_x < pc.width as usize && px_y < pc.height as usize {
                    let idx = (px_y * pc_w + px_x) * 4;
                    pc.data[idx] = px[0];
                    pc.data[idx + 1] = px[1];
                    pc.data[idx + 2] = px[2];
                    pc.data[idx + 3] = px[3];
                    pc.dirty = true;
                }
            }
        }
        return;
    }

    rasterize_image_cell(&image, target_w, target_h, x, y, buf);
}

#[allow(clippy::too_many_arguments)]
pub fn image_sprite_dimensions(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
    sheet_columns: Option<u16>,
    sheet_rows: Option<u16>,
    frame_index: Option<u16>,
    asset_root: Option<&AssetRoot>,
) -> (u16, u16) {
    let Some(root) = asset_root else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    let Some(image_asset) = load_image_asset(root.mod_source(), source) else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    let image = select_spritesheet_frame(
        image_asset.first_frame(),
        sheet_columns,
        sheet_rows,
        frame_index,
    );
    resolve_image_dimensions(&image, req_width, req_height, size)
}

fn select_spritesheet_frame<'a>(
    image: &'a RgbaImageAsset,
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
    image: &ImageView<'_>,
    req_width: Option<u16>,
    req_height: Option<u16>,
    size: Option<SpriteSizePreset>,
) -> (u16, u16) {
    let (natural_w, natural_h) = natural_image_dimensions(image);
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

fn natural_image_dimensions(image: &ImageView<'_>) -> (u16, u16) {
    let w = image.width.max(1);
    let h = image.height.max(1);
    (w.min(u16::MAX as u32) as u16, h.min(u16::MAX as u32) as u16)
}

fn rasterize_image_cell(
    image: &ImageView<'_>,
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

#[inline]
fn sample_scaled(image: &ImageView<'_>, x: u32, y: u32, virtual_w: u32, virtual_h: u32) -> [u8; 4] {
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
