use super::{colour_luma, lerp_colour_local, normalize_bg, rand01, PostFxContext};
use crate::buffer::{Buffer, Cell};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::Effect;
use crossterm::style::Color;

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    if src.width == 0 || src.height == 0 {
        dst.clone_from(src);
        return;
    }
    // Photoshop-style model:
    // 1) duplicate visible scene content,
    // 2) offset the duplicate,
    // 3) blur it,
    // 4) blend only into empty cells under the original content.
    let intensity = pass.params.intensity.unwrap_or(1.05).clamp(0.0, 2.0);
    let alpha = pass.params.alpha.unwrap_or(0.30).clamp(0.0, 1.0);
    let spread = pass.params.transparency.unwrap_or(0.32).clamp(0.0, 1.0);
    let brightness = pass.params.brightness.unwrap_or(1.08).clamp(0.6, 2.0);
    let speed = pass.params.speed.unwrap_or(0.35).clamp(0.0, 1.2);
    // Keep underlay spatially aligned with source by default (offset 0x0).
    let _legacy_offset = pass.params.sphericality.unwrap_or(0.18).clamp(0.0, 1.0);
    let frame = ctx.frame_count as u32;
    let glow = build_underlay_glow_map(src, intensity, spread, frame);

    for y in 0..src.height {
        for x in 0..src.width {
            let Some(current) = src.get(x, y).cloned() else {
                continue;
            };
            if current.symbol != ' ' {
                // Preserve content layer untouched.
                dst.set(x, y, current.symbol, current.fg, current.bg);
                continue;
            }
            let pix = glow[(y as usize) * src.width as usize + x as usize];
            if pix.a < 0.004 {
                dst.set(x, y, current.symbol, current.fg, current.bg);
                continue;
            }

            let t = ctx.scene_elapsed_ms as f32 / 1000.0;
            let pulse =
                0.90 + 0.10 * ((t * (0.95 + speed * 1.9) + y as f32 * 0.07).sin() * 0.5 + 0.5);
            let shimmer = 0.92 + 0.16 * rand01(x, y, frame.wrapping_add(4901));
            let aura = (pix.a * brightness * pulse * shimmer * alpha).clamp(0.0, 1.0);
            let glow_colour = Color::Rgb {
                r: (pix.r * 255.0).round().clamp(0.0, 255.0) as u8,
                g: (pix.g * 255.0).round().clamp(0.0, 255.0) as u8,
                b: (pix.b * 255.0).round().clamp(0.0, 255.0) as u8,
            };
            let bg = lerp_colour_local(
                normalize_bg(current.bg),
                glow_colour,
                (aura * (0.60 + 0.65 * intensity)).clamp(0.0, 0.35),
            );
            dst.set(x, y, current.symbol, current.fg, bg);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct GlowPixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl GlowPixel {
    fn add_scaled(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.r += r * a;
        self.g += g * a;
        self.b += b * a;
        self.a += a;
    }

    fn normalized(self) -> Self {
        if self.a <= 0.0001 {
            return Self::default();
        }
        Self {
            r: (self.r / self.a).clamp(0.0, 1.0),
            g: (self.g / self.a).clamp(0.0, 1.0),
            b: (self.b / self.a).clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }
}

fn build_underlay_glow_map(
    src: &Buffer,
    intensity: f32,
    spread: f32,
    frame: u32,
) -> Vec<GlowPixel> {
    let width = src.width as usize;
    let height = src.height as usize;
    let mut seed = vec![GlowPixel::default(); width * height];
    if width == 0 || height == 0 {
        return seed;
    }

    // Force centered glow (0x0) for stable text alignment.
    let off_x = 0;
    let off_y = 0;

    for y in 0..src.height {
        for x in 0..src.width {
            let Some(cell) = src.get(x, y) else {
                continue;
            };

            let Some(source_colour) = glow_source_colour(cell) else {
                continue;
            };
            let (sr, sg, sb) = colour_to_rgb(source_colour);
            let r = sr as f32 / 255.0;
            let g = sg as f32 / 255.0;
            let b = sb as f32 / 255.0;
            let luma = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 1.0);
            let mut base =
                (0.22 + 0.78 * luma) * (0.26 + 0.72 * intensity) * (0.42 + 0.55 * spread);
            if rand01(x, y, frame.wrapping_add(911)) > 0.86 {
                let sparkle = rand01(x, y, frame.wrapping_add(1337));
                base *= 1.0 + 0.45 * sparkle;
            }
            if base <= 0.0 {
                continue;
            }

            let tx = x as i32 + off_x;
            let ty = y as i32 + off_y;
            add_glow_seed(
                &mut seed,
                width,
                height,
                tx,
                ty,
                r,
                g,
                b,
                base.clamp(0.0, 1.0),
            );
        }
    }

    // Blur pass count scales with "spread" (transparency param in metadata).
    let blur_passes = 2 + (spread * 4.0).round() as usize;
    let mut blurred = seed;
    for _ in 0..blur_passes {
        blurred = blur_glow3x3(&blurred, width, height);
    }

    // A broader halo pass blended back in gives a visible soft edge.
    let broad = blur_glow3x3(&blurred, width, height);
    let mut out = vec![GlowPixel::default(); width * height];

    for y in 0..src.height {
        for x in 0..src.width {
            let idx = y as usize * width + x as usize;
            let Some(cell) = src.get(x, y) else {
                continue;
            };
            if cell.symbol != ' ' {
                // We only render underlay on empty cells.
                out[idx] = GlowPixel::default();
                continue;
            }
            let core = blurred[idx];
            let halo = broad[idx];
            let mut mix = GlowPixel {
                r: core.r * 0.60 + halo.r * 0.40,
                g: core.g * 0.60 + halo.g * 0.40,
                b: core.b * 0.60 + halo.b * 0.40,
                a: core.a * 0.62 + halo.a * 0.38,
            }
            .normalized();
            let shimmer = 0.92 + 0.16 * rand01(x, y, frame.wrapping_add(1703));
            mix.a = (mix.a * shimmer).clamp(0.0, 1.0);
            out[idx] = mix;
        }
    }

    out
}

fn glow_source_colour(cell: &Cell) -> Option<Color> {
    if cell.symbol != ' ' {
        return Some(cell.fg);
    }
    let bg = normalize_bg(cell.bg);
    if colour_luma(bg) > 0.02 {
        Some(bg)
    } else {
        None
    }
}

fn add_glow_seed(
    seed: &mut [GlowPixel],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = y as usize * width + x as usize;
    seed[idx].add_scaled(r, g, b, a);
}

fn blur_glow3x3(src: &[GlowPixel], width: usize, height: usize) -> Vec<GlowPixel> {
    let mut out = vec![GlowPixel::default(); width * height];
    for y in 0..height {
        for x in 0..width {
            let mut acc = GlowPixel::default();
            let mut wsum = 0.0_f32;
            for oy in -1_i32..=1 {
                for ox in -1_i32..=1 {
                    let nx = x as i32 + ox;
                    let ny = y as i32 + oy;
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    let weight = match (ox.abs(), oy.abs()) {
                        (0, 0) => 0.22,
                        (0, 1) | (1, 0) => 0.14,
                        _ => 0.09,
                    };
                    let p = src[ny as usize * width + nx as usize];
                    acc.r += p.r * weight;
                    acc.g += p.g * weight;
                    acc.b += p.b * weight;
                    acc.a += p.a * weight;
                    wsum += weight;
                }
            }
            out[y * width + x] = if wsum > 0.0 {
                GlowPixel {
                    r: acc.r / wsum,
                    g: acc.g / wsum,
                    b: acc.b / wsum,
                    a: acc.a / wsum,
                }
            } else {
                GlowPixel::default()
            };
        }
    }
    out
}
