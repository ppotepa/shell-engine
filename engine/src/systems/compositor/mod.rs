//! Compositor system — walks the scene layer/sprite tree and renders each frame into the terminal `Buffer`.

mod effect_applicator;
mod image_render;
mod layer_compositor;
mod layout;
pub(crate) mod obj_loader;
pub(crate) mod obj_render;
mod render;
mod sprite_renderer;
mod text_render;

use crate::assets::AssetRoot;
use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::{apply_effect, Region};
use crate::obj_prerender::{ObjPrerenderedFrames, ObjPrerenderStatus};
use crate::scene::SceneRenderedMode;
use crate::scene3d_atlas::Scene3DAtlas;
use crate::scene_runtime::{ObjectRuntimeState, SceneRuntime, TargetResolver};
use crate::services::EngineWorldAccess;
use engine_animation::SceneStage;
use crate::world::World;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static HALFBLOCK_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
}

/// Composites the current scene into the active buffer, applying effects and mode-specific rendering.
pub fn compositor_system(world: &mut World) {
    let asset_root = world.asset_root().cloned();
    let runtime_mode_override = world
        .runtime_settings()
        .and_then(|s| s.renderer_mode_override);

    // Extract raw pointer to PipelineStrategies to avoid a long-lived borrow that would
    // conflict with the buffer/virtual_buffer borrows taken later in this function.
    // SAFETY: PipelineStrategies is registered at startup and never mutated or dropped
    // during frame processing. The pointer remains valid for the duration of compositor_system.
    let strats_ptr: *const crate::strategy::PipelineStrategies = world
        .get::<crate::strategy::PipelineStrategies>()
        .map(|s| s as *const _)
        .unwrap_or(std::ptr::null());
    static FALLBACK_LAYER: crate::strategy::ScratchLayerCompositor = crate::strategy::ScratchLayerCompositor;
    static FALLBACK_HALFBLOCK: crate::strategy::FullScanPacker = crate::strategy::FullScanPacker;
    let layer_strategy: &dyn crate::strategy::LayerCompositor =
        if strats_ptr.is_null() { &FALLBACK_LAYER }
        else { unsafe { (*strats_ptr).layer.as_ref() } };
    let halfblock_strategy: &dyn crate::strategy::HalfblockPacker =
        if strats_ptr.is_null() { &FALLBACK_HALFBLOCK }
        else { unsafe { (*strats_ptr).halfblock.as_ref() } };

    // Extract a raw pointer to the scene layer slice to avoid deep-cloning the entire
    // layer tree (all Sprite::Obj fields, Strings, etc.) every frame.
    // SAFETY: SceneRuntime is stored under TypeId::of::<SceneRuntime>() in World::scoped.
    // The mutable borrows taken later (buffer_mut / virtual_buffer_mut) target
    // TypeId::of::<Buffer>() / TypeId::of::<VirtualBuffer>() — distinct HashMap entries.
    // No aliasing occurs. The pointer remains valid for the duration of this function
    // because SceneRuntime is not dropped or mutated until scene_runtime_mut() is called
    // at the very end (after all rendering is complete).
    let layers_ptr: *const Vec<crate::scene::Layer> = world
        .scene_runtime()
        .map(|rt| &rt.scene().layers as *const _)
        .unwrap_or(std::ptr::null());
    if layers_ptr.is_null() {
        return;
    }

    let (
        bg,
        ui_enabled,
        target_resolver,
        object_states,
        obj_camera_states,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        effects_ptr,
        scene_step_dur,
        rendered_mode,
    ) = {
        // Get both Arc snapshots first (requires &mut)
        let (object_states, obj_camera_states) = world
            .scene_runtime_mut()
            .map(|rt| (rt.object_states_snapshot(), rt.obj_camera_states_snapshot()))
            .unwrap_or_default();
        
        // Now get immutable references
        let scene = world.scene_runtime().unwrap().scene();
        let target_resolver = world
            .scene_runtime()
            .map(SceneRuntime::target_resolver)
            .unwrap_or_default();
        let animator = world.animator();
        let stage = animator.map(|a| a.stage.clone()).unwrap_or_default();
        let step = animator.map(|a| a.step_idx).unwrap_or(0);
        let elapsed = animator.map(|a| a.elapsed_ms).unwrap_or(0);
        let scene_elapsed = animator.map(|a| a.scene_elapsed_ms).unwrap_or(0);

        let bg = scene
            .bg_colour
            .as_ref()
            .map(Color::from)
            .unwrap_or(TRUE_BLACK);
        let ui_enabled = scene.ui.enabled;

        let current_step = match &stage {
            SceneStage::OnEnter => scene.stages.on_enter.steps.get(step),
            SceneStage::OnIdle => scene.stages.on_idle.steps.get(step),
            SceneStage::OnLeave => scene.stages.on_leave.steps.get(step),
            SceneStage::Done => None,
        };
        // #6 opt-comp-effectsref: raw pointer avoids Vec<Effect> clone every frame.
        // SAFETY: same reasoning as layers_ptr — scene is not mutated until rendering is complete.
        let effects_ptr: *const [crate::scene::Effect] = current_step
            .map(|s| s.effects.as_slice() as *const _)
            .unwrap_or(&[] as *const _);
        let scene_step_dur = current_step.map(|s| s.duration_ms()).unwrap_or(0);

        (
            bg,
            ui_enabled,
            target_resolver,
            object_states,
            obj_camera_states,
            stage,
            step,
            elapsed,
            scene_elapsed,
            effects_ptr,
            scene_step_dur,
            runtime_mode_override.unwrap_or(scene.rendered_mode),
        )
    };

    // SAFETY: see comment above layers_ptr declaration.
    let layers: &[crate::scene::Layer] = unsafe { (*layers_ptr).as_slice() };
    // SAFETY: see comment above effects_ptr declaration (#6).
    let scene_effects: &[crate::scene::Effect] = unsafe { &*effects_ptr };

    let use_virtual = world
        .runtime_settings()
        .map(|s| s.use_virtual_buffer)
        .unwrap_or(false);

    // Determine if prerendering is complete and we can use the prerendered frame store.
    // We extract a raw pointer to avoid holding a borrow while also needing mut access to world.
    let prerender_ready = matches!(world.get::<ObjPrerenderStatus>(), Some(ObjPrerenderStatus::Ready));
    let prerender_frames_ptr: *const ObjPrerenderedFrames = if prerender_ready {
        world
            .get::<ObjPrerenderedFrames>()
            .map(|c| c as *const _)
            .unwrap_or(std::ptr::null())
    } else {
        std::ptr::null()
    };
    // SAFETY: ObjPrerenderedFrames is a singleton world resource (Send+Sync) that lives for the
    // duration of this function. The mutable borrows below (buffer_mut / virtual_buffer_mut)
    // do not alias the ObjPrerenderedFrames resource since World stores each type separately.
    let prerender_frames: Option<&ObjPrerenderedFrames> = if prerender_frames_ptr.is_null() {
        None
    } else {
        Some(unsafe { &*prerender_frames_ptr })
    };

    // Extract Scene3DAtlas pointer for zero-overhead access during sprite rendering.
    // SAFETY: same reasoning as prerender_frames — Scene3DAtlas is stored separately in World
    // and not mutated during rendering.
    let atlas_ptr: *const Scene3DAtlas = world
        .get::<Scene3DAtlas>()
        .map(|a| a as *const _)
        .unwrap_or(std::ptr::null());
    let atlas: Option<&Scene3DAtlas> = if atlas_ptr.is_null() {
        None
    } else {
        Some(unsafe { &*atlas_ptr })
    };

    if use_virtual {
        let buffer = match world.virtual_buffer_mut() {
            Some(v) => &mut v.0,
            None => return,
        };
        
        let params = crate::strategy::CompositeParams {
            bg,
            layers,
            ui_enabled,
            scene_rendered_mode: rendered_mode,
            asset_root: asset_root.as_ref(),
            target_resolver: &target_resolver,
            object_states: &object_states,
            obj_camera_states: &obj_camera_states,
            current_stage: &current_stage,
            step_idx,
            elapsed_ms,
            scene_elapsed_ms,
            scene_effects: &scene_effects,
            scene_step_dur,
        };
        let object_regions = crate::scene3d_atlas::with_atlas(atlas, || {
            obj_render::with_prerender_frames(prerender_frames, || {
                dispatch_composite(rendered_mode, &params, layer_strategy, halfblock_strategy, buffer)
            })
        });
        if let Some(runtime) = world.scene_runtime_mut() {
            runtime.set_object_regions(object_regions);
        }
        return;
    }

    let buffer = match world.buffer_mut() {
        Some(b) => b,
        None => return,
    };
    
    let params = crate::strategy::CompositeParams {
        bg,
        layers,
        ui_enabled,
        scene_rendered_mode: rendered_mode,
        asset_root: asset_root.as_ref(),
        target_resolver: &target_resolver,
        object_states: &object_states,
        obj_camera_states: &obj_camera_states,
        current_stage: &current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        scene_effects: &scene_effects,
        scene_step_dur,
    };
    let object_regions = crate::scene3d_atlas::with_atlas(atlas, || {
        obj_render::with_prerender_frames(prerender_frames, || {
            dispatch_composite(rendered_mode, &params, layer_strategy, halfblock_strategy, buffer)
        })
    });
    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.set_object_regions(object_regions);
    }
}

fn composite_scene(
    bg: Color,
    layers: &[crate::scene::Layer],
    ui_enabled: bool,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &HashMap<String, crate::scene_runtime::ObjCameraState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[crate::scene::Effect],
    scene_step_dur: u64,
    layer: &dyn engine_pipeline::LayerCompositor,
    halfblock: &dyn engine_pipeline::HalfblockPacker,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    buffer.fill(bg);
    // #5 opt-comp-halfblock: HalfblockPacker strategy calls prepare_source() after fill so
    // only subsequent sprite/effect writes contribute to dirty_bounds. FullScanPacker no-ops.
    halfblock.prepare_source(buffer);
    let scene_state = object_states
        .get(target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return HashMap::new();
    }
    // Pre-allocate with capacity to avoid re-hashing as objects are inserted.
    let mut object_regions = HashMap::with_capacity(layers.len() + 4);
    object_regions.insert(
        target_resolver.scene_object_id().to_string(),
        offset_region(
            buffer.width,
            buffer.height,
            scene_state.offset_x,
            scene_state.offset_y,
        ),
    );

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    layer_compositor::composite_layers(
        layers,
        ui_enabled,
        scene_w,
        scene_h,
        scene_rendered_mode,
        asset_root,
        Some(target_resolver),
        &mut object_regions,
        scene_state.offset_x,
        scene_state.offset_y,
        object_states,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        obj_camera_states,
        layer,
        buffer,
    );

    let scene_progress = if scene_step_dur == 0 {
        0.0_f32
    } else {
        (elapsed_ms as f32 / scene_step_dur as f32).clamp(0.0, 1.0)
    };
    let full_region = Region::full(buffer);
    for effect in scene_effects {
        let region = target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(effect, scene_progress, region, buffer);
    }
    object_regions
}

fn composite_scene_halfblock(
    bg: Color,
    layers: &[crate::scene::Layer],
    ui_enabled: bool,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &HashMap<String, crate::scene_runtime::ObjCameraState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[crate::scene::Effect],
    scene_step_dur: u64,
    layer: &dyn engine_pipeline::LayerCompositor,
    halfblock: &dyn engine_pipeline::HalfblockPacker,
    target: &mut Buffer,
) -> HashMap<String, Region> {
    let needed_w = target.width;
    let needed_h = target.height.saturating_mul(2);
    HALFBLOCK_SCRATCH.with(|scratch| {
        let mut virtual_buf = scratch.borrow_mut();
        if virtual_buf.width != needed_w || virtual_buf.height != needed_h {
            virtual_buf.resize(needed_w, needed_h);
        }
        let object_regions = composite_scene(
            bg,
            layers,
            ui_enabled,
            scene_rendered_mode,
            asset_root,
            target_resolver,
            object_states,
            obj_camera_states,
            current_stage,
            step_idx,
            elapsed_ms,
            scene_elapsed_ms,
            scene_effects,
            scene_step_dur,
            layer,
            halfblock,
            &mut *virtual_buf,
        );
        pack_halfblock_buffer(&*virtual_buf, target, bg, halfblock);
        object_regions
    })
}

fn pack_halfblock_buffer(source: &Buffer, target: &mut Buffer, fallback_bg: Color, halfblock: &dyn engine_pipeline::HalfblockPacker) {
    // Always fill target with fallback background at the start of each frame.
    // This ensures stale data from the previous frame is cleared, even when
    // DirtyRegionPacker finds no dirty region to update.
    target.fill(fallback_bg);

    // #5 opt-comp-halfblock: HalfblockPacker strategy owns the iteration bounds.
    // DirtyRegionPacker returns None when there is no dirty region (skip packing).
    // FullScanPacker always returns the full extent.
    let Some((x_start, x_end, y_start, y_end)) = halfblock.iteration_bounds(source, target.height) else {
        return;
    };

    for y in y_start..=y_end {
        let top_y = y.saturating_mul(2);
        let bottom_y = top_y.saturating_add(1);
        for x in x_start..=x_end {
            let top = cell_or_blank(source, x, top_y, fallback_bg);
            let bottom = if bottom_y < source.height {
                cell_or_blank(source, x, bottom_y, fallback_bg)
            } else {
                Cell::blank(resolve_bg(top.bg, fallback_bg))
            };

            let top_bg = resolve_bg(top.bg, fallback_bg);
            let bottom_bg = resolve_bg(bottom.bg, fallback_bg);
            let out_bg = select_background(top_bg, bottom_bg, fallback_bg);
            let top_on = top.symbol != ' ';
            let bottom_on = bottom.symbol != ' ';

            let (symbol, fg, bg) = match (top_on, bottom_on) {
                (false, false) => (' ', TRUE_BLACK, out_bg),
                (true, false) => ('▀', top.fg, bottom_bg),
                (false, true) => ('▄', bottom.fg, top_bg),
                (true, true) => {
                    if top.fg == bottom.fg {
                        ('█', top.fg, out_bg)
                    } else {
                        ('▀', top.fg, bottom.fg)
                    }
                }
            };
            target.set(x, y, symbol, fg, bg);
        }
    }
}

fn offset_region(width: u16, height: u16, offset_x: i32, offset_y: i32) -> Region {
    let origin_x = offset_x.max(0) as u16;
    let origin_y = offset_y.max(0) as u16;
    let clipped_w = width.saturating_sub(offset_x.unsigned_abs().min(width as u32) as u16);
    let clipped_h = height.saturating_sub(offset_y.unsigned_abs().min(height as u32) as u16);
    Region {
        x: origin_x,
        y: origin_y,
        width: clipped_w,
        height: clipped_h,
    }
}

fn cell_or_blank(buffer: &Buffer, x: u16, y: u16, fallback_bg: Color) -> Cell {
    buffer
        .get(x, y)
        .cloned()
        .unwrap_or_else(|| Cell::blank(fallback_bg))
}

fn resolve_bg(colour: Color, fallback_bg: Color) -> Color {
    if matches!(colour, Color::Reset) {
        fallback_bg
    } else {
        colour
    }
}

fn select_background(top: Color, bottom: Color, fallback_bg: Color) -> Color {
    if top != fallback_bg {
        top
    } else {
        bottom
    }
}

/// Dispatches compositing to the correct function based on rendered mode.
fn dispatch_composite(
    mode: SceneRenderedMode,
    params: &engine_compositor::CompositeParams<'_>,
    layer: &dyn engine_pipeline::LayerCompositor,
    halfblock: &dyn engine_pipeline::HalfblockPacker,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    match mode {
        SceneRenderedMode::HalfBlock => composite_scene_halfblock(
            params.bg,
            params.layers,
            params.ui_enabled,
            params.scene_rendered_mode,
            params.asset_root,
            params.target_resolver,
            params.object_states,
            params.obj_camera_states,
            params.current_stage,
            params.step_idx,
            params.elapsed_ms,
            params.scene_elapsed_ms,
            params.scene_effects,
            params.scene_step_dur,
            layer,
            halfblock,
            buffer,
        ),
        _ => composite_scene(
            params.bg,
            params.layers,
            params.ui_enabled,
            params.scene_rendered_mode,
            params.asset_root,
            params.target_resolver,
            params.object_states,
            params.obj_camera_states,
            params.current_stage,
            params.step_idx,
            params.elapsed_ms,
            params.scene_elapsed_ms,
            params.scene_effects,
            params.scene_step_dur,
            layer,
            halfblock,
            buffer,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;
    use std::path::PathBuf;

    use crate::assets::AssetRoot;
    use crate::buffer::{Buffer, TRUE_BLACK};
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene_loader::SceneLoader;
    use crate::scene::Scene;
    use crate::scene_runtime::SceneRuntime;
    use engine_animation::{Animator, SceneStage};
    use crate::world::World;

    use super::{compositor_system, pack_halfblock_buffer};
    use crate::strategy::FullScanPacker;

    #[test]
    fn packs_two_virtual_rows_into_one_terminal_cell() {
        let mut source = Buffer::new(1, 2);
        source.fill(TRUE_BLACK);
        source.set(0, 0, '#', Color::Red, TRUE_BLACK);
        source.set(0, 1, '#', Color::Blue, TRUE_BLACK);

        let mut target = Buffer::new(1, 1);
        pack_halfblock_buffer(&source, &mut target, TRUE_BLACK, &FullScanPacker);

        let cell = target.get(0, 0).expect("cell exists");
        assert_eq!(cell.symbol, '▀');
        assert_eq!(cell.fg, Color::Red);
        assert_eq!(cell.bg, Color::Blue);
    }

    #[test]
    fn keeps_background_for_empty_virtual_pixels() {
        let mut source = Buffer::new(1, 2);
        source.fill(Color::DarkGrey);

        let mut target = Buffer::new(1, 1);
        pack_halfblock_buffer(&source, &mut target, TRUE_BLACK, &FullScanPacker);

        let cell = target.get(0, 0).expect("cell exists");
        assert_eq!(cell.symbol, ' ');
        assert_eq!(cell.bg, Color::DarkGrey);
    }

    #[test]
    fn higher_z_layer_renders_above_background_layer_effects() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: bg
    z_index: -1
    stages:
      on_idle:
        steps:
          - effects:
              - name: clear-to-colour
                duration: 1
                params:
                  colour: blue
    sprites: []
  - name: fg
    z_index: 1
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(Buffer::new(3, 1));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 1,
            stage_elapsed_ms: 1,
            scene_elapsed_ms: 1,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("foreground text").symbol, 'A');
        assert_eq!(
            buffer.get(1, 0).expect("background neighbour").bg,
            Color::Blue
        );
    }

    #[test]
    fn scene_ui_disabled_hides_ui_layers() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: ui-toggle
title: UI Toggle
bg_colour: black
ui:
  enabled: false
layers:
  - name: world
    z_index: 0
    sprites:
      - type: text
        id: world-text
        content: W
  - name: hud
    z_index: 1
    ui: true
    sprites:
      - type: text
        id: hud-text
        content: H
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(Buffer::new(2, 1));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 1,
            stage_elapsed_ms: 1,
            scene_elapsed_ms: 1,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("world text").symbol, 'W');
    }

    #[test]
    fn shell_quest_intro_logo_renders_non_black_cells() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf();
        let mod_root = repo_root.join("mods/shell-quest");
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_ref("00.intro.logo")
            .expect("load shell-quest intro logo");

        let mut world = World::new();
        world.register(Buffer::new(120, 40));
        world.register(RuntimeSettings::default());
        world.register(AssetRoot::new(mod_root));
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 300,
            stage_elapsed_ms: 300,
            scene_elapsed_ms: 300,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        let has_visible_glyph = (0..buffer.height).any(|y| {
            (0..buffer.width).any(|x| {
                let cell = buffer.get(x, y).expect("cell in bounds");
                cell.symbol != ' ' && (cell.fg != TRUE_BLACK || cell.bg != TRUE_BLACK)
            })
        });
        assert!(has_visible_glyph, "intro logo should draw visible glyphs");
    }
}
