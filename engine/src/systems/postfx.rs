//! Software post-process pipeline ("shader-like" passes) applied after compositing.

mod glow;
mod pass_burn_in;
mod pass_crt;
mod pass_crt_distort;
mod pass_ruby_crt;
mod pass_scan_glitch;
mod pass_underlay;
mod registry;

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::apply_effect;
use crate::effects::effect::Region;
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::noise::crt_hash;
use crate::scene::Effect;
use crate::services::EngineWorldAccess;
use crate::world::World;
use crossterm::style::Color;
use registry::{compile_passes, CompiledPostFx, PostFxBuiltin};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

thread_local! {
    static POSTFX_RUNTIME: RefCell<PostFxRuntime> = RefCell::new(PostFxRuntime::default());
}

#[derive(Default)]
struct PostFxRuntime {
    frame_count: u64,
    last_scene_id: Option<String>,
    previous_output: Option<Buffer>,
    compiled_passes: Vec<CompiledPostFx>,
    last_pass_fingerprint: u64,
    scratch_a: Option<Buffer>,
    scratch_b: Option<Buffer>,
    /// Frame-skip: run full pipeline every N+1 frames, blit cached result in between.
    skip_interval: u8,
    skip_counter: u8,
}

pub(super) struct PostFxContext<'a> {
    pub frame_count: u64,
    pub scene_elapsed_ms: u64,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

pub fn postfx_system(world: &mut World) {
    // #16 opt-postfx-earlyret: skip all work when scene has no postfx passes.
    if world.scene_runtime().map_or(true, |rt| rt.scene().postfx.is_empty()) {
        return;
    }

    let (scene_id, fingerprint, passes, scene_elapsed_ms) = {
        let Some(runtime) = world.scene_runtime() else {
            return;
        };
        let scene = runtime.scene();
        let scene_id = scene.id.clone();
        let fingerprint = passes_fingerprint(&scene.postfx);
        let scene_elapsed_ms = world.animator().map(|a| a.scene_elapsed_ms).unwrap_or(0);
        // Only clone the pass list when the scene or pass config has changed.
        let needs_recompile = POSTFX_RUNTIME.with(|rt| {
            let rt = rt.borrow();
            rt.last_scene_id.as_deref() != Some(&scene_id) || rt.last_pass_fingerprint != fingerprint
        });
        let passes = if needs_recompile {
            scene.postfx.clone()
        } else {
            Vec::new()
        };
        (scene_id, fingerprint, passes, scene_elapsed_ms)
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
            .apply(&scene_id, fingerprint, &passes, scene_elapsed_ms, buffer);
    });
}

impl PostFxRuntime {
    fn apply(
        &mut self,
        scene_id: &str,
        fingerprint: u64,
        passes: &[Effect],
        scene_elapsed_ms: u64,
        buffer: &mut Buffer,
    ) {
        if self.last_scene_id.as_deref() != Some(scene_id)
            || self.last_pass_fingerprint != fingerprint
        {
            self.previous_output = None;
            self.frame_count = 0;
            self.skip_counter = 0;
            self.skip_interval = 1; // run every other frame
            self.last_scene_id = Some(scene_id.to_string());
            self.compiled_passes = compile_passes(passes);
            self.last_pass_fingerprint = fingerprint;
        }

        if self.compiled_passes.is_empty() {
            self.frame_count = self.frame_count.saturating_add(1);
            return;
        }

        // Frame-skip: blit cached result on skipped frames.
        if self.skip_counter > 0 {
            if let Some(cached) = self.previous_output.as_ref() {
                if cached.width == buffer.width && cached.height == buffer.height {
                    // #7 opt-postfx-swap: only copy back buffer (front not needed for postfx cache).
                    buffer.copy_back_from(cached);
                    self.skip_counter -= 1;
                    self.frame_count = self.frame_count.saturating_add(1);
                    return;
                }
            }
        }

        self.ensure_scratch(buffer.width, buffer.height);
        let Some(a) = self.scratch_a.as_mut() else {
            return;
        };
        let Some(b) = self.scratch_b.as_mut() else {
            return;
        };

        // Swap buffer content into scratch_a (O(1) pointer swap instead of clone).
        std::mem::swap(a, buffer);
        let mut src_is_a = true;
        let mut last_written_is_a = true;
        let frame_count = self.frame_count;

        for compiled in &self.compiled_passes {
            if src_is_a {
                apply_compiled_pass(
                    compiled,
                    &PostFxContext {
                        frame_count,
                        scene_elapsed_ms,
                        _phantom: std::marker::PhantomData,
                    },
                    &*a,
                    b,
                    scene_elapsed_ms,
                    frame_count,
                );
                last_written_is_a = false;
            } else {
                apply_compiled_pass(
                    compiled,
                    &PostFxContext {
                        frame_count,
                        scene_elapsed_ms,
                        _phantom: std::marker::PhantomData,
                    },
                    &*b,
                    a,
                    scene_elapsed_ms,
                    frame_count,
                );
                last_written_is_a = true;
            }
            src_is_a = !src_is_a;
        }

        // Swap result back into buffer (O(1) pointer swap instead of clone).
        if last_written_is_a {
            std::mem::swap(buffer, a);
        } else {
            std::mem::swap(buffer, b);
        }

        // #7 opt-postfx-swap: reuse cache allocation, copy only back buffer.
        match &mut self.previous_output {
            Some(cached) if cached.width == buffer.width && cached.height == buffer.height => {
                cached.copy_back_from(buffer);
            }
            slot => *slot = Some(buffer.clone()),
        }
        self.skip_counter = self.skip_interval;
        self.frame_count = self.frame_count.saturating_add(1);
    }

    fn ensure_scratch(&mut self, width: u16, height: u16) {
        let needs_new = self
            .scratch_a
            .as_ref()
            .is_none_or(|buf| buf.width != width || buf.height != height);
        if needs_new {
            self.scratch_a = Some(Buffer::new(width, height));
            self.scratch_b = Some(Buffer::new(width, height));
        }
    }
}

fn passes_fingerprint(passes: &[Effect]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for pass in passes {
        pass.name.hash(&mut hasher);
        pass.duration.hash(&mut hasher);
        pass.looping.hash(&mut hasher);
        // Hash discriminant as u8 — avoids a String allocation per pass per frame.
        std::mem::discriminant(&pass.target_kind).hash(&mut hasher);
        // Hash each EffectParams field as bits — no heap allocation.
        let p = &pass.params;
        p.intensity.map(f32::to_bits).hash(&mut hasher);
        p.speed.map(f32::to_bits).hash(&mut hasher);
        p.alpha.map(f32::to_bits).hash(&mut hasher);
        p.sphericality.map(f32::to_bits).hash(&mut hasher);
        p.transparency.map(f32::to_bits).hash(&mut hasher);
        p.brightness.map(f32::to_bits).hash(&mut hasher);
        p.distortion.map(f32::to_bits).hash(&mut hasher);
        p.angle.map(f32::to_bits).hash(&mut hasher);
        p.width.map(f32::to_bits).hash(&mut hasher);
        p.amplitude_x.map(f32::to_bits).hash(&mut hasher);
        p.amplitude_y.map(f32::to_bits).hash(&mut hasher);
        p.pump.map(f32::to_bits).hash(&mut hasher);
        p.decay_tint.map(f32::to_bits).hash(&mut hasher);
        // String/bool params that don't change mid-scene are still worth hashing.
        p.coverage.hash(&mut hasher);
        p.orientation.hash(&mut hasher);
    }
    hasher.finish()
}

fn apply_compiled_pass(
    compiled: &CompiledPostFx,
    ctx: &PostFxContext<'_>,
    src: &Buffer,
    dst: &mut Buffer,
    scene_elapsed_ms: u64,
    frame_count: u64,
) {
    match compiled {
        CompiledPostFx::Builtin {
            kind: PostFxBuiltin::Underlay,
            effect,
        } => pass_underlay::apply(ctx, src, dst, effect),
        CompiledPostFx::Builtin {
            kind: PostFxBuiltin::Distort,
            effect,
        } => pass_crt_distort::apply(ctx, src, dst, effect),
        CompiledPostFx::Builtin {
            kind: PostFxBuiltin::ScanGlitch,
            effect,
        } => pass_scan_glitch::apply(ctx, src, dst, effect),
        CompiledPostFx::Builtin {
            kind: PostFxBuiltin::Ruby,
            effect,
        } => pass_ruby_crt::apply(ctx, src, dst, effect),
        CompiledPostFx::Builtin {
            kind: PostFxBuiltin::BurnIn,
            effect,
        } => pass_burn_in::apply(ctx, src, dst, effect),
        CompiledPostFx::CrtComposite { sub_passes } => {
            pass_crt::apply(ctx, src, dst, sub_passes)
        }
        CompiledPostFx::Generic(effect) => {
            dst.clone_from(src);
            let progress = effect_progress(effect, scene_elapsed_ms, frame_count);
            apply_effect(effect, progress, Region::full(dst), dst);
        }
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

pub(super) fn rand01(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

pub(super) fn normalize_bg(c: Color) -> Color {
    if matches!(c, Color::Reset) {
        TRUE_BLACK
    } else {
        c
    }
}

pub(super) fn scale_colour(base: Color, mul: f32) -> Color {
    let (r, g, b) = colour_to_rgb(base);
    let m = mul.clamp(0.0, 2.0);
    Color::Rgb {
        r: ((r as f32 * m).round()).clamp(0.0, 255.0) as u8,
        g: ((g as f32 * m).round()).clamp(0.0, 255.0) as u8,
        b: ((b as f32 * m).round()).clamp(0.0, 255.0) as u8,
    }
}

pub(super) fn normalized_coords(x: u16, y: u16, width: u16, height: u16) -> (f32, f32) {
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

pub(super) fn colour_luma(c: Color) -> f32 {
    let (r, g, b) = colour_to_rgb(c);
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

pub(super) fn lerp_colour_local(a: Color, b: Color, t: f32) -> Color {
    lerp_colour(a, b, t)
}
