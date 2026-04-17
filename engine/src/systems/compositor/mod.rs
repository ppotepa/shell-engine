//! Compositor system — walks the scene layer/sprite tree and renders each frame into the active `Buffer`.

use crate::buffer::TRUE_BLACK;
#[cfg(feature = "render-3d")]
use crate::obj_prerender::{ObjPrerenderStatus, ObjPrerenderedFrames};
use crate::scene3d_atlas::Scene3DAtlas;
#[cfg(feature = "render-3d")]
use crate::scene3d_runtime_store::Scene3DRuntimeStore;
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_animation::SceneStage;
use engine_core::color::Color;

/// Composites the current scene into the active buffer, applying effects and mode-specific rendering.
pub fn compositor_system(world: &mut World) {
    let asset_root = world.asset_root().cloned();
    let (is_pixel_backend, default_font) = world
        .runtime_settings()
        .map(|s| (s.is_pixel_backend, s.default_font.clone()))
        .unwrap_or((true, None));

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
    let layer_strategy: &dyn crate::strategy::LayerCompositor = if strats_ptr.is_null() {
        &FALLBACK_LAYER
    } else {
        unsafe { (*strats_ptr).layer.as_ref() }
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
        scene_space,
        scene_camera_3d,
        effects_ptr,
        scene_effect_progress,
        camera_x,
        camera_y,
        camera_zoom,
        spatial_context,
        ambient_floor,
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
        let stage = animator.map(|a| a.stage).unwrap_or_default();
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
        let scene_effect_progress = if scene_step_dur == 0 {
            0.0_f32
        } else {
            (elapsed as f32 / scene_step_dur as f32).clamp(0.0, 1.0)
        };

        let (camera_x, camera_y) = world
            .scene_runtime()
            .map(|rt| rt.camera())
            .unwrap_or((0, 0));
        let camera_zoom = world
            .scene_runtime()
            .map(|rt| rt.camera_zoom())
            .unwrap_or(1.0);
        let spatial_context = world
            .scene_runtime()
            .map(|rt| rt.spatial_context())
            .unwrap_or_default();
        let scene_camera_3d = world
            .scene_runtime()
            .map(|rt| rt.scene_camera_3d())
            .unwrap_or_default();
        let ambient_floor = world
            .scene_runtime()
            .map(|rt| {
                rt.scene()
                    .lighting
                    .as_ref()
                    .and_then(|lighting| lighting.ambient_floor)
                    .unwrap_or(0.06)
            })
            .unwrap_or(0.06);

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
            scene.space,
            scene_camera_3d,
            effects_ptr,
            scene_effect_progress,
            camera_x,
            camera_y,
            camera_zoom,
            spatial_context,
            ambient_floor,
        )
    };

    // SAFETY: see comment above layers_ptr declaration.
    let layers: &[crate::scene::Layer] = unsafe { (*layers_ptr).as_slice() };
    // SAFETY: see comment above effects_ptr declaration (#6).
    let scene_effects: &[crate::scene::Effect] = unsafe { &*effects_ptr };
    let layer_timed_visibility = engine_compositor::prepare_layer_timed_visibility(layers);
    // Pre-classify sprites into 2D and 3D buckets before the compositor dispatch.
    // This separates authoring-time sprite detail from compositor frame assembly.
    #[cfg(feature = "render-3d")]
    let prepared_layer_inputs = engine_compositor::prepare_frame_layer_inputs(
        layers,
        &layer_timed_visibility,
        ui_enabled,
        scene_space,
        &current_stage,
    );

    #[cfg(feature = "render-3d")]
    let prerender_frames_ptr: *const ObjPrerenderedFrames = world
        .get::<ObjPrerenderStatus>()
        .filter(|status| matches!(status, ObjPrerenderStatus::Ready))
        .and_then(|_| {
            world
                .get::<ObjPrerenderedFrames>()
                .map(|frames| frames as *const _)
        })
        .unwrap_or(std::ptr::null());
    // SAFETY: ObjPrerenderedFrames is a singleton world resource that can be read for this
    // frame without aliasing the mutable Buffer borrow. See comments on other resource pointers.
    #[cfg(feature = "render-3d")]
    let prerender_frames: Option<&ObjPrerenderedFrames> = if prerender_frames_ptr.is_null() {
        None
    } else {
        Some(unsafe { &*prerender_frames_ptr })
    };
    #[cfg(not(feature = "render-3d"))]
    let prerender_frames: Option<&engine_compositor::ObjPrerenderedFrames> = None;

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

    // Extract Scene3DRuntimeStore pointer for real-time clip rendering.
    // SAFETY: same pattern — stored separately in World, not mutated during rendering.
    #[cfg(feature = "render-3d")]
    let runtime_store_ptr: *const Scene3DRuntimeStore = world
        .get::<Scene3DRuntimeStore>()
        .map(|s| s as *const _)
        .unwrap_or(std::ptr::null());
    #[cfg(feature = "render-3d")]
    let runtime_store: Option<&Scene3DRuntimeStore> = if runtime_store_ptr.is_null() {
        None
    } else {
        Some(unsafe { &*runtime_store_ptr })
    };

    let celestial_catalogs_ptr: *const engine_celestial::CelestialCatalogs = world
        .get::<engine_behavior::catalog::ModCatalogs>()
        .map(|catalogs| &catalogs.celestial as *const _)
        .unwrap_or(std::ptr::null());
    let celestial_catalogs: Option<&engine_celestial::CelestialCatalogs> =
        if celestial_catalogs_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*celestial_catalogs_ptr })
        };

    let buffer = match world.buffer_mut() {
        Some(b) => b,
        None => return,
    };

    // Enable pixel canvas for direct SDL2 pixel output (bypass Cell encoding).
    if is_pixel_backend {
        buffer.enable_pixel_canvas(buffer.width, buffer.height);
    }

    let buf_w = buffer.width;
    let buf_h = buffer.height;

    let params = crate::strategy::CompositeParams {
        bg,
        frame: engine_compositor::FrameAssemblyInputs {
            layers,
            layer_timed_visibility: &layer_timed_visibility,
            ui_enabled,
            scene_space,
            scene_effects,
            #[cfg(feature = "render-3d")]
            prepared_layer_inputs: Some(prepared_layer_inputs),
        },
        prepared: engine_compositor::PreparedCompositeInputs {
            camera: engine_compositor::PreparedCameraInputs {
                scene_camera_3d: &scene_camera_3d,
                camera_x,
                camera_y,
                camera_zoom,
                spatial_context,
            },
            target_resolver: target_resolver.as_ref(),
            object_states: &object_states,
            obj_camera_states: &obj_camera_states,
            current_stage: &current_stage,
            step_idx,
            elapsed_ms,
            scene_elapsed_ms,
            scene_effect_progress,
            asset_root: asset_root.as_ref(),
            celestial_catalogs,
            is_pixel_backend,
            default_font: default_font.as_deref(),
            prerender_frames,
            ambient_floor,
        },
    };
    engine_render_2d::clear_vector_primitives();
    #[cfg(feature = "render-3d")]
    let object_regions = crate::scene3d_runtime_store::with_runtime_store(runtime_store, || {
        crate::scene3d_atlas::with_atlas(atlas, || {
            engine_compositor::dispatch_composite(&params, layer_strategy, buffer)
        })
    });
    #[cfg(not(feature = "render-3d"))]
    let object_regions = {
        let _ = atlas;
        engine_compositor::dispatch_composite(&params, layer_strategy, buffer)
    };

    // Apply script-triggered runtime effects on top of the composited frame.
    // Each entry tracks its own start time so progress is independent of authored steps.
    {
        let runtime_effects_ptr: *const crate::runtime_effects::RuntimeEffectsResource = world
            .get::<crate::runtime_effects::RuntimeEffectsResource>()
            .filter(|r| !r.is_empty())
            .map(|r| r as *const _)
            .unwrap_or(std::ptr::null());

        if !runtime_effects_ptr.is_null() {
            let buffer = match world.buffer_mut() {
                Some(b) => b,
                None => return,
            };
            // SAFETY: RuntimeEffectsResource is a scoped world resource stored at a separate
            // TypeId from Buffer. We only hold an immutable pointer to it here; the buffer
            // borrow is exclusive but targets a different HashMap entry.
            let runtime_effects = unsafe { &*runtime_effects_ptr };
            let full_region = engine_core::effects::Region::full(buffer);
            for effect_entry in runtime_effects.effects() {
                let progress = effect_entry.progress(scene_elapsed_ms);
                let scene_effect = effect_entry.as_scene_effect();
                engine_effects::apply_effect(&scene_effect, progress, full_region, buffer);
            }
        }
    }

    // Collect vector primitives produced during compositing for SDL2 native rendering.
    // `buffer` borrow is dropped here; use saved dimensions.
    let vector_prims = engine_render_2d::take_vector_primitives();
    if !vector_prims.is_empty() {
        world.register(engine_render::VectorOverlay {
            buffer_width: buf_w,
            buffer_height: buf_h,
            primitives: vector_prims,
        });
    } else {
        world.remove::<engine_render::VectorOverlay>();
    }
    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.set_object_regions(object_regions);
    }
}

#[cfg(test)]
mod tests {
    use crate::scene_pipeline::ScenePipeline;
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
    fn playground_rhai_image_lab_renders_non_black_cells() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf();
        let mod_root = repo_root.join("mods/playground");
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_ref("playground-rhai-image-lab")
            .expect("load playground scene");

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

    #[test]
    fn scene_pipeline_2d_only_does_not_schedule_3d_preparation_steps() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: flat-2d
title: Flat2D
prerender: true
bg_colour: black
layers:
  - name: ui
    z_index: 0
    sprites:
      - type: text
        id: t
        content: TEST
        at: cl
"#,
        )
        .expect("scene should parse");

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf();
        let mod_root = repo_root.join("mods/playground");

        let mut world = World::new();
        world.register(AssetRoot::new(mod_root));
        world.register(RuntimeSettings::default());

        let pipeline = ScenePipeline::default();
        pipeline.prepare(&scene, &mut world);

        assert!(
            !world.get::<crate::obj_prerender::ObjPrerenderStatus>().is_some(),
            "Obj prerender status must not be registered for 2D-only scene"
        );

        #[cfg(feature = "render-3d")]
        {
            use crate::scene3d_runtime_store::Scene3DRuntimeStore;

            assert!(
                !world.get::<crate::scene3d_atlas::Scene3DAtlas>().is_some(),
                "Scene3D atlas must not be registered for 2D-only scene"
            );
            assert!(
                !world.get::<Scene3DRuntimeStore>().is_some(),
                "Scene3D runtime store must not be registered for 2D-only scene"
            );
        }
    }

    #[test]
    fn composite_2d_only_scene_runs_without_3d_world_resources() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: flat-2d
title: Flat2D
bg_colour: black
layers:
  - name: main
    z_index: 0
    sprites:
      - type: text
        id: msg
        content: HI
        at: cl
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(Buffer::new(6, 1));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene.clone()));
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
        assert_eq!(buffer.get(0, 0).expect("glyph").symbol, 'H');
    }
}
