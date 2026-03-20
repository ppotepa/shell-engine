//! Software post-process pipeline ("shader-like" passes) applied after compositing.

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::apply_effect;
use crate::effects::effect::Region;
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::math::smoothstep;
use crate::effects::utils::noise::crt_hash;
use crate::scene::Effect;
use crate::services::EngineWorldAccess;
use crate::world::World;
use crossterm::style::Color;
use std::cell::RefCell;
use std::f32::consts::TAU;

thread_local! {
    static POSTFX_RUNTIME: RefCell<PostFxRuntime> = RefCell::new(PostFxRuntime::default());
}

#[derive(Default)]
struct PostFxRuntime {
    frame_count: u64,
    last_scene_id: Option<String>,
    previous_output: Option<Buffer>,
}

struct PostFxContext<'a> {
    frame_count: u64,
    scene_elapsed_ms: u64,
    previous_output: Option<&'a Buffer>,
}

trait PostFxPass {
    fn apply(&mut self, ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect);
}

#[derive(Default)]
struct TerminalCrtPass;

impl PostFxPass for TerminalCrtPass {
    fn apply(&mut self, ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
        if src.width == 0 || src.height == 0 {
            return;
        }
        let intensity = pass.params.intensity.unwrap_or(0.55).clamp(0.0, 1.5);
        let sphericality = pass.params.sphericality.unwrap_or(0.10).clamp(0.0, 0.35);
        let noise = pass.params.transparency.unwrap_or(0.08).clamp(0.0, 0.5);
        let brightness = pass.params.brightness.unwrap_or(0.95).clamp(0.65, 1.2);
        let speed = pass.params.speed.unwrap_or(0.35).clamp(0.0, 1.2);
        let ghosting = (pass.params.transparency.unwrap_or(0.08) * 0.35).clamp(0.0, 0.35);
        let frame = ctx.frame_count as u32;

        let flicker_phase = (ctx.scene_elapsed_ms as f32 / 1000.0) * TAU * (0.7 + speed * 0.9);
        let flicker = 1.0 + flicker_phase.sin() * (0.003 + 0.008 * intensity);

        for y in 0..src.height {
            for x in 0..src.width {
                let (nx, ny) = normalized_coords(x, y, src.width, src.height);
                let radius2 = (nx * nx + ny * ny).min(1.0);

                let warp = 1.0 - sphericality * (0.06 + 0.22 * radius2);
                let sx = remap_axis_with_margin((nx * warp).clamp(-0.985, 0.985), src.width, 1);
                let sy = remap_axis_with_margin(
                    (ny * (1.0 - sphericality * (0.05 + 0.18 * radius2))).clamp(-0.985, 0.985),
                    src.height,
                    1,
                );

                let Some(current) = src.get(x, y).cloned() else {
                    continue;
                };
                let sample = src.get(sx, sy).cloned().unwrap_or_else(|| current.clone());
                let prev = ctx
                    .previous_output
                    .and_then(|b| b.get(sx, sy))
                    .cloned()
                    .unwrap_or_else(|| sample.clone());

                let scanline = if (y.wrapping_add((frame & 1) as u16) & 1) == 0 {
                    1.0 - (0.03 + 0.10 * intensity)
                } else {
                    1.0
                };
                let vignette = 1.0
                    - smoothstep(((radius2 - 0.44) / 0.56).clamp(0.0, 1.0))
                        * (0.07 + 0.18 * intensity);
                let grain = (rand01(x, y, frame.wrapping_add((speed * 101.0) as u32)) - 0.5)
                    * (0.025 * noise);
                let mul = (brightness * flicker * scanline * vignette + grain).clamp(0.6, 1.2);

                let sampled_fg = scale_colour(sample.fg, mul);
                let ghost_fg = scale_colour(prev.fg, 0.90);
                let merged_fg = lerp_colour(sampled_fg, ghost_fg, ghosting);
                let fg = lerp_colour(
                    current.fg,
                    merged_fg,
                    (0.20 + 0.35 * intensity).clamp(0.0, 0.60),
                );

                let sample_bg = normalize_bg(sample.bg);
                let ghost_bg = normalize_bg(prev.bg);
                let merged_bg = lerp_colour(
                    scale_colour(sample_bg, (mul * 0.70).clamp(0.5, 1.0)),
                    scale_colour(ghost_bg, 0.75),
                    ghosting * 0.6,
                );
                let bg = lerp_colour(
                    normalize_bg(current.bg),
                    merged_bg,
                    (0.06 + 0.12 * intensity).clamp(0.0, 0.25),
                );

                // Keep glyphs stable to avoid "double text" artifacts in low-resolution terminal grids.
                dst.set(x, y, current.symbol, fg, bg);
            }
        }
    }
}

pub fn postfx_system(world: &mut World) {
    let (scene_id, passes, scene_elapsed_ms) = {
        let Some(runtime) = world.scene_runtime() else {
            return;
        };
        let scene_id = runtime.scene().id.clone();
        let passes = runtime.scene().postfx.clone();
        let scene_elapsed_ms = world.animator().map(|a| a.scene_elapsed_ms).unwrap_or(0);
        (scene_id, passes, scene_elapsed_ms)
    };

    let use_virtual = world
        .runtime_settings()
        .map(|settings| settings.use_virtual_buffer)
        .unwrap_or(false);
    let buffer = if use_virtual {
        match world.virtual_buffer_mut() {
            Some(v) => &mut v.0,
            None => return,
        }
    } else {
        match world.buffer_mut() {
            Some(b) => b,
            None => return,
        }
    };

    POSTFX_RUNTIME.with(|runtime| {
        runtime
            .borrow_mut()
            .apply(&scene_id, &passes, scene_elapsed_ms, buffer);
    });
}

impl PostFxRuntime {
    fn apply(
        &mut self,
        scene_id: &str,
        passes: &[Effect],
        scene_elapsed_ms: u64,
        buffer: &mut Buffer,
    ) {
        if self.last_scene_id.as_deref() != Some(scene_id) {
            self.previous_output = None;
            self.frame_count = 0;
            self.last_scene_id = Some(scene_id.to_string());
        }

        if passes.is_empty() {
            self.previous_output = Some(buffer.clone());
            self.frame_count = self.frame_count.saturating_add(1);
            return;
        }

        let mut current = buffer.clone();
        for pass in passes {
            if pass.name == "terminal-crt" {
                let mut next = current.clone();
                let mut shader = TerminalCrtPass;
                shader.apply(
                    &PostFxContext {
                        frame_count: self.frame_count,
                        scene_elapsed_ms,
                        previous_output: self.previous_output.as_ref(),
                    },
                    &current,
                    &mut next,
                    pass,
                );
                current = next;
                continue;
            }

            // Fallback: reuse regular effect dispatch as a post-process pass.
            let mut next = current.clone();
            let progress = effect_progress(pass, scene_elapsed_ms, self.frame_count);
            apply_effect(pass, progress, Region::full(&next), &mut next);
            current = next;
        }

        *buffer = current.clone();
        self.previous_output = Some(current);
        self.frame_count = self.frame_count.saturating_add(1);
    }
}

fn effect_progress(pass: &Effect, scene_elapsed_ms: u64, frame_count: u64) -> f32 {
    if pass.duration == 0 {
        return ((frame_count % 1000) as f32 / 1000.0).clamp(0.0, 1.0);
    }
    if pass.looping {
        (scene_elapsed_ms % pass.duration) as f32 / pass.duration as f32
    } else {
        (scene_elapsed_ms as f32 / pass.duration as f32).clamp(0.0, 1.0)
    }
}

fn normalized_coords(x: u16, y: u16, width: u16, height: u16) -> (f32, f32) {
    let nx = if width <= 1 {
        0.0
    } else {
        (x as f32 / (width - 1) as f32) * 2.0 - 1.0
    };
    let ny = if height <= 1 {
        0.0
    } else {
        (y as f32 / (height - 1) as f32) * 2.0 - 1.0
    };
    (nx, ny)
}

fn remap_axis_with_margin(value: f32, extent: u16, margin: u16) -> u16 {
    if extent <= 1 {
        return 0;
    }
    let max_idx = extent - 1;
    let scaled = ((value + 1.0) * 0.5 * max_idx as f32).round() as i32;
    let lo = margin as i32;
    let hi = max_idx.saturating_sub(margin) as i32;
    scaled.clamp(lo, hi.max(lo)) as u16
}

fn rand01(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

fn normalize_bg(c: Color) -> Color {
    if matches!(c, Color::Reset) {
        TRUE_BLACK
    } else {
        c
    }
}

fn scale_colour(base: Color, mul: f32) -> Color {
    let (r, g, b) = colour_to_rgb(base);
    let m = mul.clamp(0.0, 2.0);
    Color::Rgb {
        r: ((r as f32 * m).round()).clamp(0.0, 255.0) as u8,
        g: ((g as f32 * m).round()).clamp(0.0, 255.0) as u8,
        b: ((b as f32 * m).round()).clamp(0.0, 255.0) as u8,
    }
}
