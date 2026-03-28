//! Compositor system — walks the scene layer/sprite tree and renders each frame into the terminal `Buffer`.

use crate::buffer::TRUE_BLACK;
use crate::obj_prerender::{ObjPrerenderStatus, ObjPrerenderedFrames};
use crate::scene3d_atlas::Scene3DAtlas;
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_animation::SceneStage;
use engine_core::color::Color;

/// Composites the current scene into the active buffer, applying effects and mode-specific rendering.
pub fn compositor_system(world: &mut World) {
    let asset_root = world.asset_root().cloned();
    let (runtime_mode_override, is_pixel_backend, default_font) = world
        .runtime_settings()
        .map(|s| {
            (
                s.renderer_mode_override,
                s.is_pixel_backend,
                s.default_font.clone(),
            )
        })
        .unwrap_or((None, false, None));

    // Extract raw pointer to PipelineStrategies to avoid a long-lived borrow that would
    // conflict with the buffer borrow taken later in this function.
    // SAFETY: PipelineStrategies is registered at startup and never mutated or dropped
    // during frame processing. The pointer remains valid for the duration of compositor_system.
    let strats_ptr: *const crate::strategy::PipelineStrategies = world
        .get::<crate::strategy::PipelineStrategies>()
        .map(|s| s as *const _)
        .unwrap_or(std::ptr::null());
    static FALLBACK_LAYER: crate::strategy::ScratchLayerCompositor =
        crate::strategy::ScratchLayerCompositor;
    static FALLBACK_HALFBLOCK: crate::strategy::FullScanPacker = crate::strategy::FullScanPacker;
    let layer_strategy: &dyn crate::strategy::LayerCompositor = if strats_ptr.is_null() {
        &FALLBACK_LAYER
    } else {
        unsafe { (*strats_ptr).layer.as_ref() }
    };
    let halfblock_strategy: &dyn crate::strategy::HalfblockPacker = if strats_ptr.is_null() {
        &FALLBACK_HALFBLOCK
    } else {
        unsafe { (*strats_ptr).halfblock.as_ref() }
    };

    // Extract a raw pointer to the scene layer slice to avoid deep-cloning the entire
    // layer tree (all Sprite::Obj fields, Strings, etc.) every frame.
    // SAFETY: SceneRuntime is stored under TypeId::of::<SceneRuntime>() in World::scoped.
    // The mutable borrow taken later targets Buffer, so no aliasing occurs.
    // The pointer remains valid for the duration of this function
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
            .map(SceneRuntime::target_resolver_arc)
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

    // Determine if prerendering is complete and we can use the prerendered frame store.
    // We extract a raw pointer to avoid holding a borrow while also needing mut access to world.
    let prerender_ready = matches!(
        world.get::<ObjPrerenderStatus>(),
        Some(ObjPrerenderStatus::Ready)
    );
    let prerender_frames_ptr: *const ObjPrerenderedFrames = if prerender_ready {
        world
            .get::<ObjPrerenderedFrames>()
            .map(|c| c as *const _)
            .unwrap_or(std::ptr::null())
    } else {
        std::ptr::null()
    };
    // SAFETY: ObjPrerenderedFrames is a singleton world resource (Send+Sync) that lives for the
    // duration of this function. The mutable buffer borrow below does not alias it because
    // World stores each type separately.
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
        target_resolver: target_resolver.as_ref(),
        object_states: &object_states,
        obj_camera_states: &obj_camera_states,
        current_stage: &current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        scene_effects: &scene_effects,
        scene_step_dur,
        is_pixel_backend,
        default_font: default_font.as_deref(),
    };
    let object_regions = crate::scene3d_atlas::with_atlas(atlas, || {
        engine_compositor::with_prerender_frames(prerender_frames, || {
            engine_compositor::dispatch_composite(
                rendered_mode,
                &params,
                layer_strategy,
                halfblock_strategy,
                buffer,
            )
        })
    });
    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.set_object_regions(object_regions);
    }
}

#[cfg(test)]
mod tests {
    use engine_core::color::Color;
    use std::path::PathBuf;

    use crate::assets::AssetRoot;
    use crate::buffer::{Buffer, TRUE_BLACK};
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene::Scene;
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::world::World;
    use engine_animation::{Animator, SceneStage};

    use super::compositor_system;
    use crate::strategy::FullScanPacker;

    #[test]
    fn packs_two_virtual_rows_into_one_terminal_cell() {
        let mut source = Buffer::new(1, 2);
        source.fill(TRUE_BLACK);
        source.set(0, 0, '#', Color::Red, TRUE_BLACK);
        source.set(0, 1, '#', Color::Blue, TRUE_BLACK);

        let mut target = Buffer::new(1, 1);
        engine_compositor::pack_halfblock_buffer(&source, &mut target, TRUE_BLACK, &FullScanPacker);

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
        engine_compositor::pack_halfblock_buffer(&source, &mut target, TRUE_BLACK, &FullScanPacker);

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
