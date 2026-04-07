use engine_core::buffer::{Buffer, Cell, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use crate::metadata::{select, slider, EffectMetadata, P_EASING};
use crate::utils::color::{colour_to_rgb, lerp_colour};
use engine_core::scene::EffectParams;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "cutout",
    display_name: "Cutout",
    summary: "Post-raster cutout look with smoothing, posterization, edge accents, and saturation shaping.",
    category: "colour",
    compatible_targets: EffectTargetMask::ANY,
    params: &[
        slider(
            "levels",
            "Levels",
            "Number of colour quantization bands per RGB channel.",
            1.0,
            100.0,
            1.0,
            "",
        ),
        slider(
            "simplify",
            "Simplify",
            "Box-blur smoothing passes applied before quantization.",
            0.0,
            8.0,
            1.0,
            "",
        ),
        slider(
            "edge_fidelity",
            "Edge Fidelity",
            "Sensitivity threshold for detecting colour boundaries.",
            0.0,
            1.0,
            0.0,
            "",
        ),
        slider(
            "edge_strength",
            "Edge Strength",
            "How strongly detected edges are darkened.",
            0.0,
            1.0,
            0.0,
            "",
        ),
        slider(
            "edge_width",
            "Edge Width",
            "How many cells around an edge receive the accent.",
            1.0,
            3.0,
            1.0,
            "",
        ),
        slider(
            "saturation",
            "Saturation",
            "Per-cell saturation multiplier after edge shaping.",
            0.5,
            1.5,
            0.0,
            "",
        ),
        select(
            "blend_mode",
            "Blend Mode",
            "Whether to replace the source cell or overlay the processed result.",
            &["replace", "overlay"],
            "replace",
        ),
        P_EASING,
    ],
    sample: "- name: cutout\n  duration: 1200\n  params:\n    levels: 8\n    simplify: 2\n    edge_fidelity: 0.35\n    edge_strength: 0.7\n    edge_width: 2\n    saturation: 1.1\n    blend_mode: overlay",
};

pub struct CutoutEffect;

impl Effect for CutoutEffect {
    fn apply(&self, _progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let levels = params.levels.unwrap_or(8).clamp(2, 20);
        let simplify = params.simplify.unwrap_or(0.0).clamp(0.0, 8.0).round() as usize;
        let edge_fidelity = params.edge_fidelity.unwrap_or(0.35).clamp(0.0, 1.0);
        let edge_strength = params.edge_strength.unwrap_or(0.65).clamp(0.0, 1.0);
        let edge_width = params.edge_width.unwrap_or(1).clamp(1, 3) as usize;
        let saturation = params.saturation.unwrap_or(1.0).clamp(0.5, 1.5);
        let blend_mode = params.blend_mode.as_deref().unwrap_or("replace");
        let overlay_mode = blend_mode == "overlay";

        let width = region.width as usize;
        let height = region.height as usize;
        let (mut snapshot, source_hash) = snapshot_region_and_hash(buffer, region);
        let cache_key = CutoutCacheKey {
            width,
            height,
            source_hash,
            levels,
            simplify,
            edge_width,
            edge_fidelity_bits: edge_fidelity.to_bits(),
            edge_strength_bits: edge_strength.to_bits(),
            saturation_bits: saturation.to_bits(),
            overlay_mode,
        };

        let cache = cutout_cache();
        if let Ok(guard) = cache.lock() {
            if let Some(entry) = guard.as_ref() {
                if entry.key == cache_key {
                    write_region_cells(region, &entry.cells, buffer);
                    return;
                }
            }
        }

        for _ in 0..simplify {
            snapshot = simplify_pass(&snapshot, width, height);
        }

        let mut quantized = snapshot.clone();
        for cell in &mut quantized {
            if is_transparent(cell) {
                continue;
            }
            cell.fg = quantize_color(cell.fg, levels);
            cell.bg = quantize_color(cell.bg, levels);
        }

        let edge_map = if edge_strength > 0.0 {
            Some(build_edge_map(&quantized, width, height, edge_fidelity))
        } else {
            None
        };
        let apply_saturation = (saturation - 1.0).abs() > f32::EPSILON;
        let overlay_weight = overlay_weight(edge_strength);
        let mut output = snapshot.clone();

        for dy in 0..height {
            for dx in 0..width {
                let idx = dy * width + dx;
                let center = &snapshot[idx];
                if is_transparent(center) {
                    continue;
                }

                let mut fg = quantized[idx].fg;
                let mut bg = quantized[idx].bg;

                if let Some(edges) = edge_map.as_ref() {
                    let edge_influence = edge_influence(edges, width, height, dx, dy, edge_width);
                    if edge_influence > 0.0 {
                        let accent = (edge_strength * edge_influence).clamp(0.0, 1.0);
                        fg = darken_colour(fg, accent);
                        bg = darken_colour(bg, accent);
                    }
                }

                if apply_saturation {
                    fg = adjust_saturation(fg, saturation);
                    bg = adjust_saturation(bg, saturation);
                }

                let final_fg = if overlay_mode {
                    blend_colour(center.fg, fg, overlay_weight)
                } else {
                    fg
                };
                let final_bg = if overlay_mode {
                    blend_colour(center.bg, bg, overlay_weight)
                } else {
                    bg
                };

                output[idx].symbol = center.symbol;
                output[idx].fg = final_fg;
                output[idx].bg = final_bg;
            }
        }

        write_region_cells(region, &output, buffer);
        if let Ok(mut guard) = cache.lock() {
            *guard = Some(CutoutCacheEntry {
                key: cache_key,
                cells: output,
            });
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

fn snapshot_region_and_hash(buffer: &Buffer, region: Region) -> (Vec<Cell>, u64) {
    let mut snapshot = Vec::with_capacity(region.width as usize * region.height as usize);
    let mut hasher = DefaultHasher::new();
    for dy in 0..region.height {
        for dx in 0..region.width {
            let cell = buffer
                .get(region.x + dx, region.y + dy)
                .cloned()
                .unwrap_or_default();
            hash_cell(&cell, &mut hasher);
            snapshot.push(cell);
        }
    }
    (snapshot, hasher.finish())
}

fn hash_cell(cell: &Cell, hasher: &mut DefaultHasher) {
    cell.symbol.hash(hasher);
    hash_colour(cell.fg, hasher);
    hash_colour(cell.bg, hasher);
}

fn hash_colour(color: Color, hasher: &mut DefaultHasher) {
    if matches!(color, Color::Reset) {
        0u8.hash(hasher);
    } else {
        1u8.hash(hasher);
    }
    let (r, g, b) = colour_to_rgb(normalize_reset(color));
    r.hash(hasher);
    g.hash(hasher);
    b.hash(hasher);
}

fn write_region_cells(region: Region, cells: &[Cell], buffer: &mut Buffer) {
    let width = region.width as usize;
    for dy in 0..region.height as usize {
        for dx in 0..width {
            let cell = &cells[dy * width + dx];
            buffer.set(
                region.x.saturating_add(dx as u16),
                region.y.saturating_add(dy as u16),
                cell.symbol,
                cell.fg,
                cell.bg,
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CutoutCacheKey {
    width: usize,
    height: usize,
    source_hash: u64,
    levels: u8,
    simplify: usize,
    edge_width: usize,
    edge_fidelity_bits: u32,
    edge_strength_bits: u32,
    saturation_bits: u32,
    overlay_mode: bool,
}

#[derive(Debug, Clone)]
struct CutoutCacheEntry {
    key: CutoutCacheKey,
    cells: Vec<Cell>,
}

fn cutout_cache() -> &'static Mutex<Option<CutoutCacheEntry>> {
    static CUTOUT_CACHE: OnceLock<Mutex<Option<CutoutCacheEntry>>> = OnceLock::new();
    CUTOUT_CACHE.get_or_init(|| Mutex::new(None))
}

fn simplify_pass(snapshot: &[Cell], width: usize, height: usize) -> Vec<Cell> {
    let mut output = snapshot.to_vec();
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let center = &snapshot[idx];
            if is_transparent(center) {
                continue;
            }

            let mut fg_r = 0u32;
            let mut fg_g = 0u32;
            let mut fg_b = 0u32;
            let mut bg_r = 0u32;
            let mut bg_g = 0u32;
            let mut bg_b = 0u32;
            let mut count = 0u32;

            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let cell = &snapshot[ny * width + nx];
                    if is_transparent(cell) {
                        continue;
                    }
                    let (fr, fg, fb) = colour_to_rgb(cell.fg);
                    let (br, bg, bb) = colour_to_rgb(normalize_reset(cell.bg));
                    fg_r += fr as u32;
                    fg_g += fg as u32;
                    fg_b += fb as u32;
                    bg_r += br as u32;
                    bg_g += bg as u32;
                    bg_b += bb as u32;
                    count += 1;
                }
            }

            if count > 0 {
                output[idx].fg = Color::Rgb {
                    r: (fg_r / count) as u8,
                    g: (fg_g / count) as u8,
                    b: (fg_b / count) as u8,
                };
                output[idx].bg = Color::Rgb {
                    r: (bg_r / count) as u8,
                    g: (bg_g / count) as u8,
                    b: (bg_b / count) as u8,
                };
            }
        }
    }
    output
}

fn build_edge_map(snapshot: &[Cell], width: usize, height: usize, threshold: f32) -> Vec<f32> {
    let mut edges = vec![0.0; snapshot.len()];
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let center = &snapshot[idx];
            if is_transparent(center) {
                continue;
            }

            let mut strongest: f32 = 0.0;
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    if nx == x && ny == y {
                        continue;
                    }
                    let neighbour = &snapshot[ny * width + nx];
                    if is_transparent(neighbour) {
                        continue;
                    }
                    strongest = strongest.max(colour_delta(center, neighbour));
                }
            }

            if strongest >= threshold {
                edges[idx] = strongest;
            }
        }
    }
    edges
}

fn edge_influence(
    edges: &[f32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    edge_width: usize,
) -> f32 {
    let radius = edge_width.saturating_sub(1);
    let x0 = x.saturating_sub(radius);
    let y0 = y.saturating_sub(radius);
    let x1 = (x + radius).min(width - 1);
    let y1 = (y + radius).min(height - 1);
    let mut strongest: f32 = 0.0;

    for ny in y0..=y1 {
        for nx in x0..=x1 {
            let idx = ny * width + nx;
            let dist = x.abs_diff(nx).max(y.abs_diff(ny)) as f32;
            let falloff = if edge_width <= 1 {
                1.0
            } else {
                1.0 - (dist / edge_width as f32).clamp(0.0, 1.0)
            };
            strongest = strongest.max(edges[idx] * falloff.max(0.0));
        }
    }

    strongest.clamp(0.0, 1.0)
}

fn colour_delta(a: &Cell, b: &Cell) -> f32 {
    let (ar, ag, ab) = colour_to_rgb(normalize_reset(a.fg));
    let (br, bg, bb) = colour_to_rgb(normalize_reset(b.fg));
    let fg_delta = channel_delta(ar, br)
        .max(channel_delta(ag, bg))
        .max(channel_delta(ab, bb));

    let (ar, ag, ab) = colour_to_rgb(normalize_reset(a.bg));
    let (br, bg, bb) = colour_to_rgb(normalize_reset(b.bg));
    let bg_delta = channel_delta(ar, br)
        .max(channel_delta(ag, bg))
        .max(channel_delta(ab, bb));

    fg_delta.max(bg_delta)
}

fn channel_delta(a: u8, b: u8) -> f32 {
    (a.abs_diff(b) as f32 / 255.0).clamp(0.0, 1.0)
}

fn quantize_color(color: Color, levels: u8) -> Color {
    let (r, g, b) = colour_to_rgb(normalize_reset(color));
    Color::Rgb {
        r: quantize_component(r, levels),
        g: quantize_component(g, levels),
        b: quantize_component(b, levels),
    }
}

fn quantize_component(value: u8, levels: u8) -> u8 {
    if levels <= 1 {
        return 0;
    }
    let step = 255.0 / (levels as f32 - 1.0);
    ((value as f32 / step).round() * step)
        .clamp(0.0, 255.0)
        .round() as u8
}

fn darken_colour(color: Color, amount: f32) -> Color {
    lerp_colour(color, TRUE_BLACK, amount.clamp(0.0, 1.0))
}

fn adjust_saturation(color: Color, saturation: f32) -> Color {
    let (r, g, b) = colour_to_rgb(color);
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let luminance = 0.299 * rf + 0.587 * gf + 0.114 * bf;
    let mix = |channel: f32| (luminance + (channel - luminance) * saturation).clamp(0.0, 1.0);
    Color::Rgb {
        r: (mix(rf) * 255.0).round() as u8,
        g: (mix(gf) * 255.0).round() as u8,
        b: (mix(bf) * 255.0).round() as u8,
    }
}

fn blend_colour(original: Color, processed: Color, weight: f32) -> Color {
    lerp_colour(original, processed, weight.clamp(0.0, 1.0))
}

fn overlay_weight(edge_strength: f32) -> f32 {
    if edge_strength <= 0.0 {
        0.5
    } else {
        edge_strength.clamp(0.0, 1.0)
    }
}

fn is_transparent(cell: &Cell) -> bool {
    cell.symbol == ' ' && matches!(cell.bg, Color::Reset)
}

fn normalize_reset(color: Color) -> Color {
    if matches!(color, Color::Reset) {
        TRUE_BLACK
    } else {
        color
    }
}

#[cfg(test)]
mod tests {
    use super::{CutoutEffect, METADATA};
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;
    use engine_core::effects::{Effect, EffectTargetMask, Region};
    use engine_core::scene::EffectParams;
    use std::collections::BTreeSet;

    #[test]
    fn metadata_is_any_target() {
        assert_eq!(METADATA.compatible_targets, EffectTargetMask::ANY);
    }

    #[test]
    fn levels_reduce_unique_channel_values() {
        let mut buf = Buffer::new(3, 1);
        buf.set(
            0,
            0,
            '█',
            Color::Rgb {
                r: 12,
                g: 40,
                b: 80,
            },
            Color::Black,
        );
        buf.set(
            1,
            0,
            '█',
            Color::Rgb {
                r: 128,
                g: 40,
                b: 80,
            },
            Color::Black,
        );
        buf.set(
            2,
            0,
            '█',
            Color::Rgb {
                r: 240,
                g: 40,
                b: 80,
            },
            Color::Black,
        );

        let before: BTreeSet<_> = (0..3)
            .map(|x| match buf.get(x, 0).expect("cell").fg {
                Color::Rgb { r, .. } => r,
                _ => 0,
            })
            .collect();

        CutoutEffect.apply(
            1.0,
            &EffectParams {
                levels: Some(2),
                simplify: Some(0.0),
                edge_fidelity: Some(1.0),
                edge_strength: Some(0.0),
                edge_width: Some(1),
                saturation: Some(1.0),
                blend_mode: Some("replace".to_string()),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 3,
                height: 1,
            },
            &mut buf,
        );

        let after: BTreeSet<_> = (0..3)
            .map(|x| match buf.get(x, 0).expect("cell").fg {
                Color::Rgb { r, .. } => r,
                _ => 0,
            })
            .collect();

        assert!(after.len() <= before.len());
        assert!(after.len() < before.len());
    }

    #[test]
    fn edge_strength_modifies_boundary_cells() {
        let mut buf = Buffer::new(2, 1);
        buf.set(
            0,
            0,
            '█',
            Color::Rgb {
                r: 255,
                g: 20,
                b: 20,
            },
            Color::Black,
        );
        buf.set(
            1,
            0,
            '█',
            Color::Rgb {
                r: 20,
                g: 20,
                b: 255,
            },
            Color::Black,
        );

        let before_left = buf.get(0, 0).cloned().expect("left before");
        let before_right = buf.get(1, 0).cloned().expect("right before");

        CutoutEffect.apply(
            1.0,
            &EffectParams {
                levels: Some(20),
                simplify: Some(0.0),
                edge_fidelity: Some(0.0),
                edge_strength: Some(1.0),
                edge_width: Some(1),
                saturation: Some(1.0),
                blend_mode: Some("replace".to_string()),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 2,
                height: 1,
            },
            &mut buf,
        );

        assert_ne!(buf.get(0, 0).cloned().expect("left after"), before_left);
        assert_ne!(buf.get(1, 0).cloned().expect("right after"), before_right);
    }
}
