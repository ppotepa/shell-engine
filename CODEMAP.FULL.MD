# CODEMAP — Full Symbol Reference

Generated: 2026-03-30 10:48 UTC | 386 files | 3272 symbols | 34 crates

> Symbol columns: `line` · `kind` · `name` · visibility · usages

---

## app

### `src/main.rs`

- **17** `struct` **Cli**
- **481** `mod` **tests**

---

## editor

### `src/app.rs`

- **27** `fn` **run** `pub`

### `src/cli.rs`

- **8** `struct` **Cli** `pub`

### `src/domain/asset_index.rs`

- **8** `struct` **AssetIndex** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`

### `src/domain/diagnostics.rs`

- **5** `struct` **Diagnostics** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/indexer.rs`

### `src/domain/effect_params.rs`

- **8** `enum` **EffectParamControl** `pub`
- **17** `impl` **EffectParamControl** — *impl EffectParamControl*
- **18** `const` **fn** `pub`
- **27** `fn` **bounds** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/buffer.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **33** `fn` **step** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/animator.rs`, `engine-animation/src/systems.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_render.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/components.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-pipeline/src/strategies/present.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/warmup.rs`, `rust-os/src/app_host.rs`, `tools/schema-gen/src/main.rs`
- **39** `fn` **unit** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`, `engine-game/src/components.rs`, `engine/src/bench.rs`, `engine/src/scene3d_format.rs`, `tools/schema-gen/src/main.rs`
- **45** `fn` **clamp** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/scenes_browser.rs`, `engine-animation/src/animator.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/effects/utils/math.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/strategy.rs`, `engine-mod/src/terminal_caps.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine-terminal/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/splash.rs`, `tools/sound-server/src/main.rs`
- **53** `struct` **EffectParamSpec** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`
- **59** `impl` **EffectParamSpec** — *impl EffectParamSpec*
- **60** `const` **fn** `pub`
- **75** `fn` **default_value** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/cli.rs`, `editor/src/state/effects_browser.rs`, `tools/devtool/src/cli.rs`, `tools/sound-server/src/main.rs`
- **79** `fn` **adjust** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`
- **85** `fn` **render_value** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`
- **106** `struct` **EffectParamValue** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`
- **108** `impl` **EffectParamValue** — *impl EffectParamValue*
- **109** `fn` **as_float** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`
- **119** `struct` **EffectParamDescriptor** `pub`
- **126** `macro_rules!` **spec**
- **316** `fn` **effect_param_specs** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **333** `fn` **default_effect_params** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`
- **367** `fn` **effect_param_value** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **373** `fn` **param_label** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **393** `fn` **param_text_value** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **435** `fn` **apply_overrides** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`

### `src/domain/effects_catalog.rs`

- **8** `struct` **EffectDoc** `pub`
- **16** `fn` **builtin_effect_names** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **24** `fn` **effect_doc** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **51** `mod` **tests**

### `src/domain/effects_preview_scene.rs`

- **8** `const` **PREVIEW_DURATION_MS** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **13** `enum` **PreviewPlacement** `pub`
- **20** `impl` **PreviewPlacement** — *impl PreviewPlacement*
- **22** `fn` **is_ambient** `pub`
- **28** `struct` **PreviewSceneDoc**
- **41** `struct` **PreviewStages**
- **46** `struct` **PreviewStage**
- **52** `struct` **PreviewStep**
- **58** `struct` **PreviewEffect**
- **68** `enum` **PreviewEffectTargetKind**
- **78** `struct` **PreviewEffectParams**
- **132** `struct` **PreviewLayer**
- **142** `enum` **PreviewSprite**
- **164** `fn` **build_preview_scene_yaml** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **174** `fn` **build_preview_scene_yaml_default** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`
- **178** `fn` **preview_asset_root** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **189** `fn` **choose_preview_placement** `pub`
- **205** `impl` **PreviewPlacement** — *impl PreviewPlacement*
- **241** `struct` **PreviewLayout**
- **248** `impl` **PreviewLayout** — *impl PreviewLayout*
- **409** `impl` **PreviewEffectParams** — *impl PreviewEffectParams*
- **465** `mod` **tests**

### `src/domain/mod.rs`

- **3** `mod` **asset_index** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`
- **4** `mod` **diagnostics** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/asset_index.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/logging.rs`, `engine-mod/src/startup/report.rs`, `engine-scene-runtime/src/object_graph.rs`, `tools/sound-server/src/main.rs`
- **5** `mod` **effect_params** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`
- **6** `mod` **effects_catalog** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`
- **7** `mod` **effects_preview_scene** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`
- **8** `mod` **mod_manifest** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/fs_scan.rs`
- **9** `mod` **preview_renderer** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`
- **10** `mod` **scene_index** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/indexer.rs`, `editor/src/state/scene_run.rs`

### `src/domain/mod_manifest.rs`

- **7** `struct` **ModManifestSummary** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/fs_scan.rs`

### `src/domain/preview_renderer.rs`

- **17** `const` **DEFAULT_SCENE_PREVIEW_DURATION_MS** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`
- **19** `struct` **PreviewRenderRequest** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`
- **28** `fn` **render_scene_buffer** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`
- **71** `fn` **buffer_to_lines** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`

### `src/domain/scene_index.rs`

- **5** `struct` **SceneIndex** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/indexer.rs`

### `src/input/commands.rs`

- **5** `enum` **Command** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-io/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `tools/devtool/src/main.rs`

### `src/input/keys.rs`

- **9** `fn` **map_key_event** `pub`<br>↳ used in: `editor/src/app.rs`

### `src/input/mod.rs`

- **3** `mod` **commands** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/commands.rs`, `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-audio/src/audio.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/gpu/render.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`
- **4** `mod` **keys** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/state/cutscene.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/game_loop.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `rust-os/src/vfs/mod.rs`

### `src/io/fs_scan.rs`

- **10** `struct` **ProjectValidation** `pub`
- **17** `fn` **collect_files** `pub`<br>↳ used in: `editor/src/io/indexer.rs`
- **26** `fn` **collect_project_files** `pub`<br>↳ used in: `editor/src/state/watch.rs`
- **34** `fn` **validate_project_dir** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`
- **41** `fn` **validate_project_dir_with_manifest** `pub`<br>↳ used in: `editor/src/io/indexer.rs`
- **171** `fn` **collect_schema_project_yml_files** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`
- **270** `fn` **collect_game_yaml_files** `pub`<br>↳ used in: `editor/src/io/indexer.rs`
- **278** `fn` **collect_scene_entry_files** `pub`<br>↳ used in: `editor/src/io/indexer.rs`
- **351** `fn` **infer_mod_root_from_project_yml** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`
- **366** `mod` **tests**

### `src/io/indexer.rs`

- **15** `fn` **build_project_index** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **51** `mod` **tests**

### `src/io/mod.rs`

- **3** `mod` **fs_scan** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **4** `mod` **indexer** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **5** `mod` **recent** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/io/recent.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/start_screen.rs`, `engine-core/src/buffer.rs`, `engine-core/src/logging.rs`, `engine-render-terminal/src/renderer.rs`
- **6** `mod` **yaml** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/mod_manifest.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/registry/mod.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/terminal_caps.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/types.rs`, `engine-runtime/src/lib.rs`, `engine-terminal/src/lib.rs`, `engine/src/audio_sequencer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/types.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `engine/src/systems/hot_reload.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`

### `src/io/recent.rs`

- **25** `fn` **load_recent** `pub`<br>↳ used in: `editor/src/app.rs`
- **59** `fn` **save_recent** `pub`<br>↳ used in: `editor/src/app.rs`
- **74** `fn` **push_recent** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`

### `src/io/yaml.rs`

- **9** `fn` **load_yaml** `pub`<br>↳ used in: `editor/src/ui/components/start_screen.rs`

### `src/main.rs`

- **1** `mod` **app**
- **2** `mod` **cli**
- **3** `mod` **domain**
- **4** `mod` **input**
- **5** `mod` **io**
- **6** `mod` **state**
- **7** `mod` **ui**

### `src/state/cutscene.rs`

- **11** `impl` **AppState** — *impl AppState*
- **12** `fn` **activate_cutscene_maker** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **21** `fn` **refresh_cutscene_source_folder** `pub`<br>↳ used in: `editor/src/state/editor_pane.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **166** `fn` **cutscene_status_message** `pub`<br>↳ used in: `editor/src/state/watch.rs`
- **181** `fn` **handle_cutscene_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/editor_pane.rs`

- **10** `impl` **AppState** — *impl AppState*
- **11** `fn` **enter_edit_mode** `pub`<br>↳ used in: `editor/src/state/project_explorer.rs`
- **38** `fn` **exit_edit_mode** `pub`
- **47** `fn` **handle_editor_pane_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/effects_browser.rs`

- **9** `impl` **AppState** — *impl AppState*
- **10** `fn` **restart_effect_preview_clock** `pub`
- **15** `fn` **effect_param_specs** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **22** `fn` **selected_effect_param_spec** `pub`
- **28** `fn` **effect_param_value** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **39** `fn` **sync_effect_param_cursor** `pub`
- **57** `fn` **move_effect_selection** `pub`
- **66** `fn` **move_effect_param_cursor** `pub`
- **77** `fn` **adjust_selected_effect_param** `pub`
- **92** `fn` **activate_effects_browser** `pub`<br>↳ used in: `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`
- **104** `fn` **toggle_effects_preview** `pub`<br>↳ used in: `editor/src/state/editor_pane.rs`
- **121** `fn` **sync_effect_preview_scene_yaml** `pub`
- **137** `fn` **handle_effects_browser_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/filters.rs`

- **6** `struct` **FilterState** `pub`

### `src/state/focus.rs`

- **5** `enum` **FocusPane** `pub`<br>↳ used in: `editor/src/state/cutscene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`
- **11** `impl` **FocusPane** — *impl FocusPane*
- **13** `fn` **next** `pub`
- **22** `fn` **prev** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/level_state.rs`, `engine-game/src/gameplay.rs`

### `src/state/mod.rs`

- **3** `mod` **cutscene** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine/src/systems/scene_lifecycle.rs`
- **4** `mod` **editor_pane** `pub`
- **5** `mod` **effects_browser** `pub`
- **6** `mod` **filters** `pub`<br>↳ used in: `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`
- **7** `mod` **focus** `pub`<br>↳ used in: `editor/src/state/cutscene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/focus.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `engine-core/src/scene/model.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/scene_lifecycle.rs`
- **8** `mod` **project_explorer** `pub`
- **9** `mod` **scene_run** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/mod.rs`
- **10** `mod` **scenes_browser** `pub`
- **11** `mod` **selection** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scenes_browser.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-compositor/src/image_render.rs`, `engine-core/src/level_state.rs`, `engine-io/src/lib.rs`, `engine-mod/src/output_backend.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine/src/level_state.rs`
- **12** `mod` **start_screen** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **13** `mod` **watch** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **32** `enum` **EffectsCodeTab** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`
- **39** `impl` **EffectsCodeTab** — *impl EffectsCodeTab*
- **40** `const` **ALL** `pub`
- **43** `fn` **label** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/icons.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/gameplay.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/mesh.rs`, `engine/src/gpu/render.rs`, `engine/src/systems/scene_lifecycle.rs`
- **53** `fn` **next** `pub`
- **63** `fn` **prev** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/level_state.rs`, `engine-game/src/gameplay.rs`
- **75** `enum` **AppMode** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`
- **84** `enum` **StartDialog** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/ui/components/start_screen.rs`
- **92** `enum` **StartFocus** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/ui/components/start_screen.rs`
- **99** `enum` **StartAction** `pub`
- **108** `enum` **StartItem** `pub`
- **115** `enum` **DirBrowserItem** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/ui/components/start_screen.rs`
- **127** `enum` **TreeItem** `pub`<br>↳ used in: `editor/src/state/editor_pane.rs`, `editor/src/state/project_explorer.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`
- **139** `enum` **SidebarItem** `pub`<br>↳ used in: `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`
- **148** `struct` **EffectsBrowserState** `pub`
- **164** `struct` **StartScreenState** `pub`
- **173** `struct` **StartPickerState** `pub`
- **191** `struct` **SceneBrowserState** `pub`
- **205** `struct` **ProjectExplorerState** `pub`
- **212** `struct` **SidebarState** `pub`
- **219** `enum` **SceneRunKind** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/ui/components/scene_run.rs`
- **225** `struct` **SceneRunState** `pub`
- **233** `impl` **std** — *impl std::fmt::Debug for SceneRunState*
- **247** `struct` **CutsceneMakerState** `pub`
- **258** `struct` **ProjectWatchState** `pub`
- **266** `struct` **EditorPaneState** `pub`
- **273** `struct` **AppState** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/mod.rs`
- **296** `impl` **AppState** — *impl AppState*
- **298** `fn` **new** `pub`
- **399** `fn` **selected_builtin_effect** `pub`<br>↳ used in: `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`
- **407** `fn` **selected_scene_path** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/watch.rs`
- **416** `fn` **selected_scene_ref_path** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`
- **422** `fn` **scene_display_name** `pub`<br>↳ used in: `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`
- **432** `fn` **selected_scene_display_name** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/ui/components/scenes_preview.rs`
- **441** `fn` **scene_layer_enabled** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`
- **450** `fn` **effect_preview_progress** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **463** `fn` **scene_preview_progress** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`
- **473** `fn` **scene_preview_fullscreen_active** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **478** `fn` **current_mode_label** `pub`<br>↳ used in: `editor/src/ui/components/header.rs`, `editor/src/ui/components/status_bar.rs`
- **491** `fn` **current_screen_name** `pub`<br>↳ used in: `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/status_bar.rs`
- **541** `fn` **current_shortcuts** `pub`<br>↳ used in: `editor/src/ui/components/status_bar.rs`
- **599** `fn` **current_help** `pub`<br>↳ used in: `editor/src/ui/components/help.rs`
- **706** `fn` **start_items** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`
- **720** `fn` **recent_project_status** `pub`<br>↳ used in: `editor/src/ui/components/start_screen.rs`
- **732** `fn` **apply_command** `pub`<br>↳ used in: `editor/src/app.rs`
- **924** `fn` **update_transition** `pub`<br>↳ used in: `editor/src/app.rs`

### `src/state/project_explorer.rs`

- **7** `impl` **AppState** — *impl AppState*
- **9** `fn` **build_tree_items** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **39** `fn` **selected_tree_item** `pub`<br>↳ used in: `editor/src/state/editor_pane.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/preview.rs`
- **43** `fn` **handle_project_explorer_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/scene_run.rs`

- **28** `impl` **AppState** — *impl AppState*
- **29** `fn` **start_scene_soft_run** `pub`<br>↳ used in: `editor/src/state/scenes_browser.rs`
- **33** `fn` **start_scene_hard_run** `pub`<br>↳ used in: `editor/src/state/scenes_browser.rs`
- **116** `fn` **stop_scene_run** `pub`
- **135** `fn` **enqueue_scene_run_key** `pub`<br>↳ used in: `editor/src/app.rs`
- **147** `fn` **enqueue_scene_run_resize** `pub`<br>↳ used in: `editor/src/app.rs`
- **159** `fn` **ensure_scene_run_buffer_size** `pub`<br>↳ used in: `editor/src/app.rs`
- **188** `fn` **scene_run_buffer** `pub`<br>↳ used in: `editor/src/ui/components/scene_run.rs`
- **192** `fn` **tick_scene_run** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **318** `fn` **handle_scene_run_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/scenes_browser.rs`

- **11** `impl` **AppState** — *impl AppState*
- **12** `fn` **build_scene_display_names** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **39** `fn` **activate_scenes_browser** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **58** `fn` **scene_browser_status_message** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`
- **75** `fn` **reset_scene_fullscreen_state** `pub`<br>↳ used in: `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`
- **80** `fn` **set_scene_fullscreen_hold** `pub`
- **94** `fn` **toggle_scene_fullscreen** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **107** `fn` **move_scene_selection** `pub`
- **133** `fn` **move_scene_layer_cursor** `pub`
- **143** `fn` **toggle_selected_scene_layer** `pub`
- **177** `fn` **isolate_selected_scene_layer** `pub`
- **202** `fn` **normalize_scene_ref_path** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`
- **206** `fn` **normalize_scene_ref_path_static** `pub`
- **222** `fn` **sync_scene_preview_selection** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`
- **273** `fn` **handle_scenes_browser_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`

### `src/state/selection.rs`

- **6** `struct` **SelectionState** `pub`

### `src/state/start_screen.rs`

- **18** `impl` **AppState** — *impl AppState*
- **19** `fn` **open_project** `pub`
- **70** `fn` **prune_stale_recents** `pub`
- **85** `fn` **close_project** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **263** `fn` **handle_start_screen_command** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **271** `fn` **apply_start_recent_menu** `pub`
- **360** `fn` **apply_start_schema_picker** `pub`
- **420** `fn` **apply_start_directory_browser** `pub`

### `src/state/watch.rs`

- **14** `impl` **AppState** — *impl AppState*
- **15** `fn` **refresh_project_index_now** `pub`<br>↳ used in: `editor/src/state/scenes_browser.rs`
- **24** `fn` **poll_project_refresh** `pub`<br>↳ used in: `editor/src/state/mod.rs`
- **134** `fn` **compute_project_watch_stamp** `pub`<br>↳ used in: `editor/src/state/start_screen.rs`

### `src/ui/components/cutscene_preview.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/editor.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/effects_preview.rs`

- **15** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/header.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/help.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/mod.rs`

- **3** `mod` **cutscene_preview** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **4** `mod` **editor** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/input/commands.rs`, `editor/src/input/keys.rs`, `editor/src/input/mod.rs`, `editor/src/io/indexer.rs`, `editor/src/io/recent.rs`, `editor/src/main.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/layout.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/authoring/mod.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/mod.rs`
- **5** `mod` **effects_preview** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **6** `mod` **header** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/layout.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-authoring/src/document/scene.rs`, `engine-capture/src/capture.rs`, `engine-capture/src/compare.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`
- **7** `mod` **help** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`
- **8** `mod` **preview** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`
- **9** `mod` **scene_run** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/mod.rs`
- **10** `mod` **scenes_preview** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **11** `mod` **sidebar** `pub`<br>↳ used in: `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/layout.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`
- **12** `mod` **start_screen** `pub`<br>↳ used in: `editor/src/ui/mod.rs`
- **13** `mod` **status_bar** `pub`<br>↳ used in: `editor/src/ui/mod.rs`

### `src/ui/components/preview.rs`

- **14** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/scene_run.rs`

- **14** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/scenes_preview.rs`

- **16** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`
- **44** `fn` **render_fullscreen** `pub`<br>↳ used in: `editor/src/ui/mod.rs`

### `src/ui/components/sidebar/effects.rs`

- **14** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/sidebar/explorer.rs`

- **13** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/sidebar/icons.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/sidebar/mod.rs`

- **3** `mod` **effects** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/mod.rs`, `editor/src/io/indexer.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/effects/utils/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-game/src/components.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **4** `mod` **explorer** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/mod.rs`
- **5** `mod` **icons** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/layout.rs`
- **6** `mod` **placeholder** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/mod.rs`, `engine-compositor/src/layout/measure.rs`, `engine-core/src/authoring/catalog.rs`, `rust-os/src/commands/net.rs`

### `src/ui/components/sidebar/placeholder.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/components/start_screen.rs`

- **16** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`
- **489** `struct` **SceneEffect**
- **495** `struct` **SceneStep**
- **501** `struct` **SceneStage**
- **506** `struct` **SceneStages**
- **511** `struct` **SceneDoc**

### `src/ui/components/status_bar.rs`

- **12** `fn` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/ui/icons.rs`

- **16** `enum` **IconTheme** `pub`
- **22** `fn` **theme**
- **41** `fn` **current_theme** `pub`
- **54** `fn` **sidebar_glyph** `pub`
- **64** `fn` **theme_hint** `pub`
- **73** `mod` **nerd** `pub`
- **74** `const` **FILE** `pub`
- **75** `const` **FOLDER** `pub`
- **76** `const` **FOLDER_OPEN** `pub`
- **77** `const` **RUST** `pub`
- **78** `const` **YAML** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/io/mod.rs`, `editor/src/io/yaml.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/template.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `engine/src/systems/hot_reload.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **79** `const` **PNG** `pub`
- **80** `const` **JSON** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-persistence/src/lib.rs`
- **81** `const` **CHECK** `pub`
- **82** `const` **CROSS** `pub`
- **83** `const` **WARN** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **84** `const` **INFO** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **85** `const` **ARROW_RIGHT** `pub`
- **86** `const` **ARROW_DOWN** `pub`
- **87** `const` **ARROW_UP** `pub`
- **88** `const` **PLAY** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`
- **89** `const` **PAUSE** `pub`
- **90** `const` **STOP** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **93** `mod` **emoji** `pub`
- **94** `const` **FILE** `pub`
- **95** `const` **FOLDER** `pub`
- **96** `const` **FOLDER_OPEN** `pub`
- **97** `const` **RUST** `pub`
- **98** `const` **YAML** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/io/mod.rs`, `editor/src/io/yaml.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/template.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `engine/src/systems/hot_reload.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **99** `const` **PNG** `pub`
- **100** `const` **JSON** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-persistence/src/lib.rs`
- **101** `const` **CHECK** `pub`
- **102** `const` **CROSS** `pub`
- **103** `const` **WARN** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **104** `const` **INFO** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **105** `const` **ARROW_RIGHT** `pub`
- **106** `const` **ARROW_DOWN** `pub`
- **107** `const` **ARROW_UP** `pub`
- **108** `const` **PLAY** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`
- **109** `const` **PAUSE** `pub`
- **110** `const` **STOP** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **113** `mod` **ascii** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-policy/src/lib.rs`, `engine-render/src/font_loader.rs`, `engine/src/rasterizer/font_loader.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/state.rs`, `rust-os/src/vfs/seed.rs`
- **114** `const` **FILE** `pub`
- **115** `const` **FOLDER** `pub`
- **116** `const` **FOLDER_OPEN** `pub`
- **117** `const` **RUST** `pub`
- **118** `const` **YAML** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/io/mod.rs`, `editor/src/io/yaml.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/template.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `engine/src/systems/hot_reload.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **119** `const` **PNG** `pub`
- **120** `const` **JSON** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-persistence/src/lib.rs`
- **121** `const` **CHECK** `pub`
- **122** `const` **CROSS** `pub`
- **123** `const` **WARN** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **124** `const` **INFO** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **125** `const` **ARROW_RIGHT** `pub`
- **126** `const` **ARROW_DOWN** `pub`
- **127** `const` **ARROW_UP** `pub`
- **128** `const` **PLAY** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`
- **129** `const` **PAUSE** `pub`
- **130** `const` **STOP** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`

### `src/ui/layout.rs`

- **7** `struct` **MainChunks** `pub`
- **17** `fn` **main_chunks** `pub`<br>↳ used in: `editor/src/ui/mod.rs`

### `src/ui/mod.rs`

- **3** `mod` **components** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/ui/components/sidebar/mod.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/obj_loader.rs`, `engine-core/src/effects/utils/color.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/systems/gameplay.rs`
- **4** `mod` **icons** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/layout.rs`
- **5** `mod` **layout** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/layout.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`
- **6** `mod` **theme** `pub`<br>↳ used in: `editor/src/main.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **13** `fn` **draw** `pub`<br>↳ used in: `editor/src/app.rs`, `engine-compositor/src/render/common.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-render/src/lib.rs`, `engine/src/systems/compositor/mod.rs`

### `src/ui/theme.rs`

- **12** `struct` **Theme** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-core/src/scene/model.rs`, `engine-scene-runtime/src/lib.rs`
- **52** `impl` **Default** — *impl Default for Theme*
- **66** `impl` **Theme** — *impl Theme*
- **68** `fn` **load** `pub`
- **85** `fn` **init_theme** `pub`<br>↳ used in: `editor/src/main.rs`
- **89** `fn` **theme**
- **94** `fn` **fg_normal** `pub`<br>↳ used in: `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`
- **100** `fn` **fg_active** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`
- **108** `fn` **fg_disabled** `pub`<br>↳ used in: `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`
- **114** `fn` **accent** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `engine-core/src/effects/builtin/cutout.rs`
- **120** `fn` **sidebar_active_entry** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/icons.rs`
- **129** `fn` **pane_title** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`
- **138** `fn` **pane_border** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`
- **152** `fn` **mode_badge** `pub`<br>↳ used in: `editor/src/ui/components/header.rs`, `editor/src/ui/components/status_bar.rs`
- **157** `fn` **pane_background** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`
- **171** `fn` **preview_background** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`
- **176** `fn` **badge_transition** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/effects.rs`
- **184** `fn` **badge_effect** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/effects.rs`

---

## engine

### `src/audio_sequencer.rs`

- **16** `struct` **AudioSequencerState** `pub`<br>↳ used in: `engine/src/lib.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **22** `impl` **AudioSequencerState** — *impl AudioSequencerState*
- **23** `fn` **synthesize_note_sheets_if_any** `pub`<br>↳ used in: `engine/src/lib.rs`
- **47** `fn` **from_mod_source** `pub`<br>↳ used in: `engine/src/lib.rs`
- **84** `fn` **resolve_sfx_event** `pub`
- **96** `fn` **trigger_event** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **105** `fn` **play_song** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **113** `fn` **stop_song** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **117** `fn` **active_song_id** `pub`
- **122** `fn` **tick_song** `pub`<br>↳ used in: `engine/src/systems/audio_sequencer.rs`

### `src/bench.rs`

- **10** `struct` **FrameSample** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **37** `struct` **BenchmarkState** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/lib.rs`
- **49** `impl` **BenchmarkState** — *impl BenchmarkState*
- **50** `fn` **new** `pub`
- **64** `fn` **push** `pub`
- **68** `fn` **time_up** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **72** `fn` **should_quit** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/state/mod.rs`, `engine/src/game_loop.rs`
- **78** `fn` **mark_results_shown** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **85** `fn` **results** `pub`<br>↳ used in: `engine-compositor/src/prerender.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-frame/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`
- **99** `struct` **MetricStats** `pub`
- **109** `impl` **MetricStats** — *impl MetricStats*
- **135** `struct` **SceneStats** `pub`
- **146** `struct` **BenchResults** `pub`
- **177** `impl` **BenchResults** — *impl BenchResults*
- **284** `fn` **report_text** `pub`
- **393** `fn` **render_bench_results** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **472** `fn` **write_report** `pub`<br>↳ used in: `engine/src/lib.rs`

### `src/game_loop.rs`

- **14** `fn` **game_loop** `pub`
- **269** `mod` **tests**

### `src/gpu/context.rs`

- **8** `struct` **GpuContext** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`, `engine/src/gpu/render.rs`
- **15** `impl` **GpuContext** — *impl GpuContext*
- **17** `fn` **new** `pub`
- **58** `fn` **create_render_target** `pub`<br>↳ used in: `engine/src/gpu/render.rs`
- **76** `fn` **create_depth_texture** `pub`<br>↳ used in: `engine/src/gpu/render.rs`
- **94** `fn` **create_staging_buffer** `pub`<br>↳ used in: `engine/src/gpu/render.rs`
- **104** `fn` **set_render_pipeline** `pub`
- **109** `fn` **render_pipeline** `pub`<br>↳ used in: `engine/src/gpu/render.rs`

### `src/gpu/mesh.rs`

- **10** `struct` **Vertex** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`
- **15** `impl` **Vertex** — *impl Vertex*
- **16** `const` **ATTRIBS** `pub`
- **19** `fn` **desc** `pub`<br>↳ used in: `engine-core/src/effects/metadata.rs`, `rust-os/src/commands/fs.rs`
- **29** `struct` **GpuMesh** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`, `engine/src/gpu/render.rs`
- **37** `impl` **GpuMesh** — *impl GpuMesh*
- **39** `fn` **new** `pub`
- **87** `struct` **RenderParams** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`, `engine/src/gpu/render.rs`
- **95** `impl` **RenderParams** — *impl RenderParams*
- **96** `fn` **zero** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/systems.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/mod.rs`, `engine-debug/src/log.rs`, `engine-debug/src/profiling.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_host.rs`

### `src/gpu/mod.rs`

- **12** `mod` **context** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/status_bar.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/render/common.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/runner.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/gpu/context.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/main.rs`
- **13** `mod` **mesh** `pub`<br>↳ used in: `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/mesh.rs`, `engine/src/gpu/render.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/warmup.rs`
- **14** `mod` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`

### `src/gpu/render.rs`

- **10** `fn` **render_obj_gpu** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`
- **108** `fn` **convert_to_terminal_colors** `pub`<br>↳ used in: `engine/src/gpu/mod.rs`

### `src/image_loader.rs`

- **17** `struct` **LoadedRgbaImage** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **23** `impl` **LoadedRgbaImage** — *impl LoadedRgbaImage*
- **25** `fn` **pixel** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/utils/noise.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/bitmap_font.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/vector_overlay.rs`, `engine-runtime/src/lib.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`
- **33** `fn` **from_pixels** `pub`
- **43** `struct` **LoadedAnimatedImageFrame** `pub`
- **49** `struct` **LoadedAnimatedImage** `pub`
- **56** `impl` **LoadedAnimatedImage** — *impl LoadedAnimatedImage*
- **57** `fn` **first_frame** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **61** `fn` **frame_at** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **78** `enum` **LoadedImageAsset** `pub`
- **83** `impl` **LoadedImageAsset** — *impl LoadedImageAsset*
- **84** `fn` **first_frame** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **91** `fn` **frame_at** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **98** `fn` **is_animated** `pub`
- **105** `struct` **ImageAssetAdapter**
- **107** `impl` **SourceAdapter** — *impl SourceAdapter<LoadedImageAsset> for ImageAssetAdapter*
- **199** `fn` **load_image_asset** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **209** `fn` **load_rgba_image** `pub`
- **214** `fn` **has_image_asset** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine/src/lib.rs`
- **224** `mod` **tests**

### `src/level_state.rs`

- **20** `fn` **load_level_state** `pub`<br>↳ used in: `engine/src/lib.rs`
- **131** `mod` **tests**

### `src/lib.rs`

- **3** `mod` **bench** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`
- **4** `mod` **debug_features** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **5** `mod` **debug_log** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`
- **6** `mod` **error**
- **7** `mod` **frame_capture** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **8** `mod` **frame_compare** `pub`<br>↳ used in: `engine/tests/frame_regression.rs`
- **9** `mod` **game_loop**
- **10** `mod` **mod_loader**
- **32** `mod` **asset_cache** `pub`<br>↳ used in: `engine-compositor/src/obj_loader.rs`, `engine-core/src/asset_source.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/font_loader.rs`
- **33** `mod` **asset_source** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`, `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **34** `mod` **assets** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/context.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/generic.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **35** `mod` **audio_sequencer** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **36** `mod` **behavior** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scene_run.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-behavior/src/registry/provider.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-game/src/strategy.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`
- **37** `mod` **events** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/commands.rs`, `editor/src/state/scene_run.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-events/src/input_backend.rs`, `engine-events/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/kernel/events.rs`, `rust-os/src/main.rs`
- **38** `mod` **game_object** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/lib.rs`
- **39** `mod` **game_state** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/level_state.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`
- **40** `mod` **image_loader** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/image_render.rs`
- **41** `mod` **level_state** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/level_state.rs`, `engine/src/systems/behavior.rs`
- **42** `mod` **mod_behaviors** `pub`<br>↳ used in: `engine-behavior/src/registry/mod.rs`, `engine/src/systems/behavior.rs`
- **43** `mod` **obj_prerender** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`
- **44** `mod` **pipeline_flags** `pub`
- **45** `mod` **pipelines** `pub`<br>↳ used in: `engine/src/gpu/context.rs`, `engine/src/gpu/mod.rs`
- **46** `mod` **rasterizer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **47** `mod` **render_policy** `pub`
- **48** `mod` **runtime_settings** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **49** `mod` **scene3d_atlas** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine/src/systems/compositor/mod.rs`
- **50** `mod` **scene3d_format** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **51** `mod` **scene3d_resolve** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine/src/services.rs`
- **52** `mod` **scene_loader**
- **53** `mod` **scene_pipeline** `pub`<br>↳ used in: `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/warmup.rs`
- **54** `mod` **scene_runtime** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-core/src/scene_runtime_types.rs`, `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`
- **55** `mod` **services**
- **56** `mod` **splash**
- **57** `mod` **strategy** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-game/src/strategy.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **58** `mod` **systems** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-events/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine-physics/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **59** `mod` **terminal_caps** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/game_loop.rs`
- **60** `mod` **world** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/access.rs`, `engine-audio/src/access.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/strategy.rs`, `engine-render-terminal/src/renderer.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `engine/src/world.rs`
- **64** `fn` **behavior_catalog** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`
- **80** `struct` **ShellEngine** `pub`<br>↳ used in: `app/src/main.rs`
- **87** `enum` **BackendKind** `pub`<br>↳ used in: `app/src/main.rs`
- **94** `struct` **EngineConfig** `pub`<br>↳ used in: `app/src/main.rs`
- **150** `impl` **Default** — *impl Default for EngineConfig*
- **175** `impl` **ShellEngine** — *impl ShellEngine*
- **177** `fn` **new** `pub`
- **182** `fn` **new_with_config** `pub`<br>↳ used in: `app/src/main.rs`
- **197** `fn` **run** `pub`
- **552** `fn` **mod_source** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/io/indexer.rs`, `editor/src/main.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/audio_sequencer.rs`, `engine/src/image_loader.rs`, `engine/src/level_state.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene_loader.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`
- **557** `fn` **mod_manifest** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/io/fs_scan.rs`
- **573** `mod` **tests**

### `src/obj_frame_cache.rs`

- **6** `const` **YAW_STEP_DEG** `pub`
- **10** `struct` **BakeCacheKey** `pub`
- **18** `type` **BakedCanvas** `pub`
- **21** `struct` **ObjFrameCache** `pub`<br>↳ used in: `engine-core/src/scene/model.rs`
- **25** `impl` **ObjFrameCache** — *impl ObjFrameCache*
- **26** `fn` **new** `pub`
- **32** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **36** `fn` **get** `pub`
- **41** `fn` **snap_yaw** `pub`
- **48** `impl` **Default** — *impl Default for ObjFrameCache*
- **57** `enum` **ObjBakeStatus** `pub`

### `src/pipelines/mod.rs`

- **3** `mod` **startup** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`, `editor/src/ui/icons.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-debug/src/log.rs`, `engine-error/src/lib.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/boot.rs`

### `src/prepared_frame.rs`

- **26** `struct` **PreparedFrame** `pub`

### `src/rasterizer/font_loader.rs`

- **12** `fn` **load_font_assets** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **104** `struct` **FontManifestCandidate**
- **110** `impl` **FontManifestCandidate** — *impl FontManifestCandidate*
- **248** `mod` **tests**

### `src/rasterizer/mod.rs`

- **1** `mod` **font_loader**
- **3** `mod` **types**
- **47** `fn` **rasterize_cached** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/lib.rs`
- **71** `fn` **rasterize** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-render/src/lib.rs`
- **158** `fn` **has_font_assets** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render/src/lib.rs`, `engine/src/lib.rs`
- **167** `fn` **missing_glyphs** `pub`<br>↳ used in: `app/src/main.rs`, `engine-render/src/lib.rs`, `engine/src/lib.rs`
- **219** `fn` **blit** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render/src/lib.rs`
- **230** `mod` **tests**

### `src/rasterizer/types.rs`

- **8** `struct` **GlyphManifest** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/font_loader.rs`
- **14** `struct` **ManifestGlyph** `pub`
- **23** `struct` **LoadedGlyph** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/font_loader.rs`
- **31** `struct` **LoadedFont** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/font_loader.rs`
- **36** `impl` **LoadedFont** — *impl LoadedFont*
- **38** `fn` **advance_and_height** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/mod.rs`

### `src/runtime_settings.rs`

- **12** `fn` **scene_render_size_override** `pub`<br>↳ used in: `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **28** `fn` **buffer_layout_for_scene** `pub`<br>↳ used in: `engine/src/lib.rs`
- **45** `mod` **tests**

### `src/scene3d_format.rs`

- **15** `struct` **Scene3DDefinition** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **45** `struct` **ViewportDef** `pub`
- **56** `struct` **CameraDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **71** `impl` **Default** — *impl Default for CameraDef*
- **96** `struct` **LightDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **117** `enum` **LightKind** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **130** `struct` **MaterialDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **164** `enum` **SurfaceMode** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **181** `struct` **ObjectDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **193** `struct` **RotationDef** `pub`
- **209** `enum` **ScaleDef** `pub`
- **214** `impl` **ScaleDef** — *impl ScaleDef*
- **217** `fn` **uniform** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/scene/model.rs`
- **226** `struct` **TransformDef** `pub`
- **246** `impl` **TransformDef** — *impl TransformDef*
- **249** `fn` **resolved_rotation** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **258** `fn` **resolved_scale** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **267** `enum` **FrameDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **275** `struct` **StaticFrameDef** `pub`
- **284** `struct` **ClipFrameDef** `pub`
- **291** `struct` **ClipDef** `pub`
- **301** `struct` **TweenDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **315** `enum` **Easing** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`
- **323** `impl` **Easing** — *impl Easing*
- **325** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`
- **343** `struct` **ObjectOverride** `pub`
- **352** `fn` **load_scene3d** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`

### `src/scene3d_resolve.rs`

- **29** `trait` **Scene3DAssetResolver** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine/src/services.rs`
- **41** `fn` **resolve_scene3d_refs** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`
- **207** `mod` **tests**

### `src/scene_loader.rs`

- **16** `fn` **load_scene** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `engine/src/lib.rs`
- **22** `struct` **SceneLoader** `pub`<br>↳ used in: `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **28** `impl` **SceneLoader** — *impl SceneLoader*
- **30** `fn` **new** `pub`
- **41** `fn` **load_by_path** `pub`
- **46** `fn` **load_by_id** `pub`<br>↳ used in: `engine/src/systems/hot_reload.rs`
- **57** `fn` **load_by_ref** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **66** `fn` **prev_scene_id** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **80** `fn` **next_scene_id** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **116** `mod` **tests**

### `src/scene_pipeline.rs`

- **49** `trait` **ScenePreparationStep** `pub`
- **61** `struct` **ScenePipeline** `pub`<br>↳ used in: `engine/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **65** `impl` **ScenePipeline** — *impl ScenePipeline*
- **67** `fn` **new** `pub`
- **72** `fn` **prepare** `pub`<br>↳ used in: `engine/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **83** `impl` **Default** — *impl Default for ScenePipeline*

### `src/services.rs`

- **26** `trait` **EngineWorldAccess** `pub`<br>↳ used in: `engine-scene-runtime/src/access.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`
- **28** `fn` **scene_runtime**
- **30** `fn` **animator**
- **32** `fn` **buffer**
- **36** `fn` **runtime_settings**
- **43** `impl` **EngineWorldAccess** — *impl EngineWorldAccess for World*
- **48** `fn` **scene_runtime**
- **56** `fn` **animator**
- **64** `fn` **buffer**
- **86** `fn` **runtime_settings**
- **109** `impl` **AudioProvider** — *impl AudioProvider for World*
- **116** `impl` **AnimatorProvider** — *impl AnimatorProvider for World*
- **117** `fn` **scene**
- **121** `fn` **animator**
- **133** `trait` **Asset3DProvider** `pub`
- **138** `impl` **Asset3DProvider** — *impl Asset3DProvider for World*
- **145** `impl` **crate** — *impl crate::scene3d_resolve::Scene3DAssetResolver for AssetRoot*
- **157** `impl` **RendererProvider** — *impl RendererProvider for World*
- **158** `fn` **buffer**
- **166** `fn` **runtime_settings**
- **170** `fn` **debug_features**
- **174** `fn` **debug_log**
- **178** `fn` **animator**
- **215** `fn` **restore_front_to_back**
- **227** `fn` **vector_overlay**
- **233** `impl` **LifecycleProvider** — *impl LifecycleProvider for World*
- **234** `fn` **animator**
- **243** `fn` **scene_runtime**
- **258** `fn` **runtime_settings**
- **263** `fn` **debug_features**
- **279** `impl` **BehaviorProvider** — *impl BehaviorProvider for World*
- **280** `fn` **scene**
- **284** `fn` **animator**
- **293** `fn` **game_state**
- **298** `fn` **mod_behaviors**
- **331** `impl` **CompositorProvider** — *impl CompositorProvider for World*
- **337** `fn` **scene_runtime**
- **342** `fn` **animator**
- **350** `fn` **runtime_settings**
- **355** `fn` **debug_features**
- **361** `impl` **engine_compositor** — *impl engine_compositor::CompositorAccess for World*
- **362** `fn` **scene_runtime**
- **367** `fn` **animator**
- **377** `fn` **runtime_settings**
- **385** `fn` **scene3d_atlas**
- **395** `fn` **layer_compositor**
- **407** `impl` **engine_scene_runtime** — *impl engine_scene_runtime::SceneRuntimeAccess for World*
- **408** `fn` **scene_runtime**

### `src/splash.rs`

- **30** `struct` **SplashConfig** `pub`
- **35** `impl` **Default** — *impl Default for SplashConfig*
- **50** `fn` **config_from_manifest** `pub`<br>↳ used in: `engine/src/lib.rs`
- **91** `struct` **SplashSceneDefinition**
- **133** `struct` **SplashScene**
- **166** `struct` **SplashVisuals**
- **172** `struct` **SvgRasterLayer**
- **182** `struct` **ImagePlacement**
- **190** `enum` **ImageColourMode**
- **194** `impl` **SplashScene** — *impl SplashScene*
- **267** `fn` **show_splash** `pub`<br>↳ used in: `engine/src/lib.rs`
- **283** `fn` **show_splash_on_output** `pub`<br>↳ used in: `engine/src/lib.rs`
- **1277** `fn` **lerp_colour**
- **1288** `fn` **to_rgb**
- **1506** `fn` **phase_progress**
- **1780** `mod` **tests**

### `src/strategy/mod.rs`

- **21** `mod` **behavior_factory** `pub`
- **22** `mod` **scene_compositor** `pub`<br>↳ used in: `engine-compositor/src/scene_compositor.rs`

### `src/systems/audio_sequencer.rs`

- **9** `fn` **audio_sequencer_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/behavior.rs`

- **14** `fn` **behavior_system** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-behavior/src/lib.rs`, `engine/src/game_loop.rs`
- **187** `mod` **tests**

### `src/systems/collision.rs`

- **4** `fn` **collision_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/compositor/mod.rs`

- **13** `fn` **compositor_system** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`
- **227** `mod` **tests**

### `src/systems/engine_io.rs`

- **15** `struct` **EngineIoRuntime** `pub`
- **28** `fn` **engine_io_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/gameplay.rs`

- **4** `fn` **gameplay_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/gameplay_events.rs`

- **5** `struct` **GameplayEventBuffer** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **9** `fn` **push_collisions** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **26** `fn` **clear** `pub`

### `src/systems/hot_reload.rs`

- **22** `struct` **SceneTreeSignature**
- **29** `struct` **DebugSceneHotReloadState** `pub`
- **35** `impl` **DebugSceneHotReloadState** — *impl DebugSceneHotReloadState*
- **45** `fn` **debug_scene_hot_reload_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **209** `fn` **is_yaml_path**
- **240** `mod` **tests**

### `src/systems/mod.rs`

- **3** `mod` **audio_sequencer** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **4** `mod` **behavior** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scene_run.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-behavior/src/registry/provider.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-game/src/strategy.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`
- **5** `mod` **compositor** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-io/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/construction.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/warmup.rs`
- **6** `mod` **engine_io** `pub`<br>↳ used in: `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/main.rs`, `rust-os/src/protocol.rs`, `rust-os/src/screen_buffer.rs`
- **7** `mod` **gameplay** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/strategy.rs`, `engine-physics/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/gameplay.rs`
- **8** `mod` **collision** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-physics/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/systems/collision.rs`
- **9** `mod` **gameplay_events** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`
- **10** `mod` **ship_controller** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **11** `mod` **visual_binding** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/systems/gameplay.rs`
- **12** `mod` **visual_sync** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **13** `mod` **hot_reload** `pub`<br>↳ used in: `engine/src/bench.rs`, `engine/src/game_loop.rs`
- **14** `mod` **prerender** `pub`<br>↳ used in: `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/warmup.rs`
- **15** `mod` **scene3d_prerender** `pub`<br>↳ used in: `engine/src/scene_pipeline.rs`
- **17** `mod` **renderer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/log.rs`, `engine-pipeline/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`
- **18** `mod` **renderer_tests**
- **19** `mod` **scene_lifecycle** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`
- **20** `mod` **warmup** `pub`<br>↳ used in: `engine-compositor/src/warmup.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/warmup.rs`

### `src/systems/prerender.rs`

- **18** `struct` **ObjPrerenderStep** `pub`<br>↳ used in: `engine/src/scene_pipeline.rs`
- **20** `impl` **ScenePreparationStep** — *impl ScenePreparationStep for ObjPrerenderStep*

### `src/systems/scene3d_prerender.rs`

- **13** `struct` **Scene3DPrerenderStep** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`
- **15** `impl` **ScenePreparationStep** — *impl ScenePreparationStep for Scene3DPrerenderStep*

### `src/systems/scene_lifecycle.rs`

- **16** `struct` **SceneLifecycleManager** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/game_loop.rs`
- **21** `struct` **LifecycleEvents**
- **32** `impl` **SceneLifecycleManager** — *impl SceneLifecycleManager*
- **35** `fn` **process_events** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/game_loop.rs`
- **591** `mod` **tests**

### `src/systems/ship_controller.rs`

- **18** `fn` **ship_controller_system** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`

### `src/systems/visual_binding.rs`

- **4** `struct` **VisualCleanupBuffer** `pub`
- **8** `fn` **queue_visual_despawn** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **21** `fn` **cleanup_visuals** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/visual_sync.rs`

- **13** `fn` **visual_sync_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/systems/warmup.rs`

- **13** `struct` **MeshWarmupStep** `pub`<br>↳ used in: `engine/src/scene_pipeline.rs`
- **15** `impl` **ScenePreparationStep** — *impl ScenePreparationStep for MeshWarmupStep*

### `src/world.rs`

- **12** `struct` **World** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/lib.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/prepared_frame.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `rust-os/src/commands/net.rs`
- **14** `impl` **World** — *impl World*
- **16** `fn` **new** `pub`
- **21** `impl` **Default** — *impl Default for World*
- **27** `impl` **Deref** — *impl Deref for World*
- **34** `impl` **DerefMut** — *impl DerefMut for World*

---

## engine-3d

### `src/lib.rs`

- **9** `mod` **obj_frame_cache** `pub`
- **10** `mod` **obj_prerender** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`
- **11** `mod` **scene3d_atlas** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine/src/systems/compositor/mod.rs`
- **12** `mod` **scene3d_format** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **13** `mod` **scene3d_resolve** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine/src/services.rs`

### `src/obj_frame_cache.rs`

- **6** `const` **YAW_STEP_DEG** `pub`
- **10** `struct` **BakeCacheKey** `pub`
- **18** `type` **BakedCanvas** `pub`
- **21** `struct` **ObjFrameCache** `pub`<br>↳ used in: `engine-core/src/scene/model.rs`
- **25** `impl` **ObjFrameCache** — *impl ObjFrameCache*
- **26** `fn` **new** `pub`
- **32** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **36** `fn` **get** `pub`
- **41** `fn` **snap_yaw** `pub`
- **48** `impl` **Default** — *impl Default for ObjFrameCache*
- **57** `enum` **ObjBakeStatus** `pub`

### `src/obj_prerender.rs`

- **8** `type` **PrerenderedCanvas** `pub`
- **11** `struct` **PrerenderedFrame** `pub`<br>↳ used in: `engine-compositor/src/prerender.rs`
- **27** `struct` **ObjPrerenderedFrames** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`
- **31** `impl` **ObjPrerenderedFrames** — *impl ObjPrerenderedFrames*
- **32** `fn` **new** `pub`
- **36** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **40** `fn` **get** `pub`
- **44** `fn` **is_empty** `pub`
- **48** `fn` **len** `pub`
- **54** `enum` **ObjPrerenderStatus** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`

### `src/scene3d_atlas.rs`

- **22** `struct` **Scene3DAtlas** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene3d_prerender.rs`
- **27** `impl` **Scene3DAtlas** — *impl Scene3DAtlas*
- **28** `fn` **new** `pub`
- **32** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **40** `fn` **get** `pub`
- **44** `fn` **is_empty** `pub`
- **49** `fn` **len** `pub`
- **65** `fn` **with_atlas** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **73** `impl` **Scene3DAtlas** — *impl Scene3DAtlas*
- **76** `fn` **current_get** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`

### `src/scene3d_format.rs`

- **15** `struct` **Scene3DDefinition** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **45** `struct` **ViewportDef** `pub`
- **56** `struct` **CameraDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **71** `impl` **Default** — *impl Default for CameraDef*
- **96** `struct` **LightDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **117** `enum` **LightKind** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **130** `struct` **MaterialDef** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/scene3d_resolve.rs`
- **164** `enum` **SurfaceMode** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **181** `struct` **ObjectDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **193** `struct` **RotationDef** `pub`
- **209** `enum` **ScaleDef** `pub`
- **214** `impl` **ScaleDef** — *impl ScaleDef*
- **217** `fn` **uniform** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/scene/model.rs`
- **226** `struct` **TransformDef** `pub`
- **246** `impl` **TransformDef** — *impl TransformDef*
- **249** `fn` **resolved_rotation** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **258** `fn` **resolved_scale** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **267** `enum` **FrameDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **275** `struct` **StaticFrameDef** `pub`
- **284** `struct` **ClipFrameDef** `pub`
- **291** `struct` **ClipDef** `pub`
- **301** `struct` **TweenDef** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`
- **315** `enum` **Easing** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`
- **323** `impl` **Easing** — *impl Easing*
- **325** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`
- **343** `struct` **ObjectOverride** `pub`
- **352** `fn` **load_scene3d** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`

### `src/scene3d_resolve.rs`

- **29** `trait` **Scene3DAssetResolver** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine/src/services.rs`
- **41** `fn` **resolve_scene3d_refs** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`
- **207** `mod` **tests**

---

## engine-animation

### `src/access.rs`

- **7** `trait` **AnimatorAccess** `pub`
- **8** `fn` **animator**
- **12** `impl` **AnimatorAccess** — *impl AnimatorAccess for World*
- **13** `fn` **animator**

### `src/animator.rs`

- **5** `enum` **SceneStage** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **15** `struct` **Animator** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-animation/src/access.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/systems.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **25** `impl` **Animator** — *impl Animator*
- **26** `fn` **new** `pub`
- **31** `fn` **step_progress** `pub`
- **40** `mod` **tests**

### `src/lib.rs`

- **9** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **10** `mod` **animator** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `engine-animation/src/animator.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **11** `mod` **menu** `pub`<br>↳ used in: `editor/src/ui/components/start_screen.rs`, `engine-animation/src/menu.rs`, `engine-animation/src/provider.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/schema-gen/src/main.rs`
- **12** `mod` **provider** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-audio/src/systems_audio.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/services.rs`, `engine/src/world.rs`
- **13** `mod` **systems** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-events/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine-physics/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`

### `src/menu.rs`

- **8** `enum` **MenuAction** `pub`<br>↳ used in: `engine-animation/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **15** `fn` **evaluate_menu_action** `pub`<br>↳ used in: `engine-animation/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`

### `src/provider.rs`

- **9** `trait` **LifecycleProvider** `pub`<br>↳ used in: `engine-animation/src/lib.rs`
- **10** `fn` **animator**
- **12** `fn` **scene_runtime**
- **15** `fn` **runtime_settings**
- **16** `fn` **debug_features**

### `src/systems.rs`

- **11** `trait` **AnimatorProvider** `pub`<br>↳ used in: `engine-animation/src/lib.rs`, `engine/src/world.rs`
- **12** `fn` **scene**
- **13** `fn` **animator**
- **17** `fn` **animator_system** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/lib.rs`, `engine/src/game_loop.rs`
- **158** `mod` **tests**
- **165** `struct` **TestAnimatorProvider**
- **170** `impl` **AnimatorProvider** — *impl AnimatorProvider for TestAnimatorProvider*
- **171** `fn` **scene**
- **174** `fn` **animator**

---

## engine-asset

### `src/lib.rs`

- **6** `mod` **repositories** `pub`<br>↳ used in: `engine-asset/src/scene_compiler.rs`, `engine-asset/src/source_loader.rs`, `engine-authoring/src/compile/scene.rs`
- **7** `mod` **scene_compiler** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **15** `mod` **source_loader** `pub`

### `src/repositories.rs`

- **22** `trait` **SceneRepository** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `engine-asset/src/lib.rs`, `engine/src/lib.rs`, `engine/src/scene_loader.rs`
- **23** `fn` **load_scene**
- **28** `trait` **AssetRepository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-asset/src/source_loader.rs`, `engine-render/src/font_loader.rs`, `engine/src/level_state.rs`, `engine/src/rasterizer/font_loader.rs`
- **37** `enum` **AnySceneRepository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine/src/scene_loader.rs`
- **42** `impl` **SceneRepository** — *impl SceneRepository for AnySceneRepository*
- **43** `fn` **load_scene**
- **60** `enum` **AnyAssetRepository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-asset/src/source_loader.rs`
- **65** `impl` **AssetRepository** — *impl AssetRepository for AnyAssetRepository*
- **90** `struct` **FsSceneRepository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`
- **94** `impl` **FsSceneRepository** — *impl FsSceneRepository*
- **96** `fn` **new** `pub`
- **129** `impl` **SceneRepository** — *impl SceneRepository for FsSceneRepository*
- **130** `fn` **load_scene**
- **162** `impl` **AssetRepository** — *impl AssetRepository for FsSceneRepository*
- **200** `struct` **ZipSceneRepository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`
- **204** `impl` **ZipSceneRepository** — *impl ZipSceneRepository*
- **206** `fn` **new** `pub`
- **212** `fn` **normalized**
- **280** `impl` **SceneRepository** — *impl SceneRepository for ZipSceneRepository*
- **281** `fn` **load_scene**
- **317** `impl` **AssetRepository** — *impl AssetRepository for ZipSceneRepository*
- **363** `fn` **create_scene_repository** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `engine-asset/src/lib.rs`, `engine/src/lib.rs`, `engine/src/scene_loader.rs`
- **377** `fn` **create_asset_repository** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-asset/src/source_loader.rs`, `engine-render/src/font_loader.rs`, `engine/src/level_state.rs`, `engine/src/rasterizer/font_loader.rs`
- **524** `mod` **tests**

### `src/scene_compiler.rs`

- **17** `fn` **compile_scene_document_with_loader_and_source** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/mod.rs`

### `src/source_loader.rs`

- **12** `struct` **ModAssetSourceLoader** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-compositor/src/obj_loader.rs`, `engine-core/src/asset_source.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **17** `impl` **ModAssetSourceLoader** — *impl ModAssetSourceLoader*
- **18** `fn` **new** `pub`
- **26** `impl` **SourceLoader** — *impl SourceLoader for ModAssetSourceLoader*
- **39** `fn` **has_source**
- **61** `mod` **tests**

---

## engine-audio

### `src/access.rs`

- **7** `trait` **AudioAccess** `pub`
- **12** `impl` **AudioAccess** — *impl AudioAccess for World*

### `src/audio.rs`

- **15** `struct` **AudioCommand** `pub`<br>↳ used in: `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **21** `trait` **AudioBackend** `pub`<br>↳ used in: `engine-audio/src/lib.rs`
- **28** `struct` **NullAudioBackend** `pub`<br>↳ used in: `engine-audio/src/lib.rs`
- **30** `impl` **AudioBackend** — *impl AudioBackend for NullAudioBackend*
- **41** `struct` **RodioAudioBackend** `pub`<br>↳ used in: `engine-audio/src/lib.rs`
- **52** `impl` **Send** — *unsafe impl Send for RodioAudioBackend {}*
- **53** `impl` **Sync** — *unsafe impl Sync for RodioAudioBackend {}*
- **55** `impl` **RodioAudioBackend** — *impl RodioAudioBackend*
- **57** `fn` **new** `pub`
- **87** `fn` **register_memory_cue** `pub`<br>↳ used in: `engine/src/lib.rs`
- **93** `impl` **AudioBackend** — *impl AudioBackend for RodioAudioBackend*
- **249** `struct` **AudioRuntime** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-audio/src/access.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`
- **255** `impl` **AudioRuntime** — *impl AudioRuntime*
- **257** `fn` **null** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/menu.rs`, `engine-asset/src/repositories.rs`, `engine-audio/src/systems_audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/lib.rs`, `engine/src/scene_loader.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/process.rs`, `rust-os/src/vfs/seed.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **262** `fn` **with_backend** `pub`
- **273** `fn` **from_env** `pub`
- **285** `fn` **from_options** `pub`<br>↳ used in: `engine/src/lib.rs`
- **306** `fn` **queue** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-events/src/access.rs`, `engine-events/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/kernel/events.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/kernel/syscall.rs`
- **311** `fn` **register_memory_cue** `pub`<br>↳ used in: `engine/src/lib.rs`
- **322** `fn` **flush** `pub`<br>↳ used in: `app/src/main.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/buffer.rs`, `engine-core/src/logging.rs`, `engine-io/src/lib.rs`, `engine-persistence/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `rust-os/src/main.rs`, `tools/sound-server/src/main.rs`
- **330** `fn` **played** `pub`<br>↳ used in: `engine-audio/src/systems_audio.rs`
- **335** `fn` **pending_len** `pub`<br>↳ used in: `engine-audio/src/systems_audio.rs`, `engine/src/systems/behavior.rs`
- **353** `mod` **tests**
- **358** `struct` **RecordingBackend**
- **362** `impl` **AudioBackend** — *impl AudioBackend for RecordingBackend*

### `src/lib.rs`

- **10** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **11** `mod` **audio** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-animation/src/systems.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/access.rs`, `engine-audio/src/audio.rs`, `engine-audio/src/systems_audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/audio_sequencer.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/sound-server/src/main.rs`
- **12** `mod` **systems_audio** `pub`

### `src/systems_audio.rs`

- **11** `trait` **AudioProvider** `pub`<br>↳ used in: `engine-audio/src/lib.rs`, `engine/src/world.rs`
- **18** `fn` **audio_system** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-audio/src/lib.rs`, `engine/src/game_loop.rs`
- **26** `mod` **tests**
- **30** `struct` **TestProvider**
- **34** `impl` **AudioProvider** — *impl AudioProvider for TestProvider*

---

## engine-audio-sequencer

### `src/lib.rs`

- **10** `enum` **SequencerError** `pub`
- **26** `struct` **SfxBank** `pub`<br>↳ used in: `engine-mod/src/startup/checks/audio_sequencer.rs`
- **34** `struct` **SfxEvent** `pub`
- **46** `struct` **SfxVariant** `pub`
- **57** `struct` **SongFile** `pub`<br>↳ used in: `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/audio_sequencer.rs`
- **72** `struct` **LoopRegion** `pub`
- **78** `struct` **SongTrack** `pub`
- **91** `struct` **TrackClip** `pub`
- **99** `struct` **Pattern** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **105** `struct` **PatternStep** `pub`
- **113** `struct` **NoteSheetFile** `pub`
- **121** `struct` **SynthSound** `pub`
- **137** `struct` **SynthNote** `pub`
- **149** `struct` **ResolvedSfxEvent** `pub`
- **159** `struct` **SfxEventRuntime** `pub`<br>↳ used in: `engine/src/audio_sequencer.rs`
- **165** `impl` **SfxEventRuntime** — *impl SfxEventRuntime*
- **166** `fn` **new** `pub`
- **177** `fn` **resolve_event** `pub`<br>↳ used in: `engine/src/audio_sequencer.rs`
- **210** `fn` **has_event** `pub`
- **216** `struct` **SequencedEvent** `pub`
- **222** `struct` **SongRuntime** `pub`<br>↳ used in: `engine/src/audio_sequencer.rs`
- **228** `impl` **SongRuntime** — *impl SongRuntime*
- **229** `fn` **new** `pub`
- **237** `fn` **id** `pub`
- **241** `fn` **beat** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **246** `fn` **tick** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/utils/math.rs`, `engine-game/src/gameplay.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`
- **367** `fn` **load_sfx_bank** `pub`<br>↳ used in: `engine/src/audio_sequencer.rs`
- **380** `fn` **load_song_file** `pub`
- **396** `fn` **synthesize_note_sheets** `pub`<br>↳ used in: `engine/src/audio_sequencer.rs`
- **429** `fn` **validate_sfx_bank** `pub`<br>↳ used in: `engine-mod/src/startup/checks/audio_sequencer.rs`
- **452** `fn` **validate_song_file** `pub`<br>↳ used in: `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/audio_sequencer.rs`
- **712** `mod` **tests**

---

## engine-authoring

### `src/compile/cutscene.rs`

- **10** `struct` **CutsceneCompileFrame** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`
- **23** `trait` **CutsceneCompileFilter** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`
- **25** `fn` **apply**
- **34** `struct` **CutsceneFilterRegistry** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`
- **38** `impl` **CutsceneFilterRegistry** — *impl CutsceneFilterRegistry*
- **39** `fn` **with_builtin_filters** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`
- **45** `fn` **register** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-authoring/src/compile/scene.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`
- **71** `struct` **DurationScaleFilter** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`
- **73** `impl` **CutsceneCompileFilter** — *impl CutsceneCompileFilter for DurationScaleFilter*
- **78** `fn` **apply**
- **103** `fn` **expand_scene_cutscene_ref_with_filters** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`
- **170** `struct` **CutsceneDocument**
- **186** `struct` **CutsceneSpriteDefaults**
- **202** `struct` **CutsceneFrameDocument**
- **223** `struct` **CutsceneFilterSpec**

### `src/compile/mod.rs`

- **6** `mod` **cutscene**
- **7** `mod` **scene**

### `src/compile/scene.rs`

- **15** `fn` **compile_scene_document_with_loader** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`
- **38** `fn` **compile_scene_document_with_loader_and_source** `pub`<br>↳ used in: `engine-asset/src/lib.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/mod.rs`
- **57** `fn` **compile_scene_document_with_loader_and_source_and_filters** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`
- **89** `struct` **SceneLogicSpec**
- **101** `struct` **SceneScriptController**
- **954** `struct` **LoadedObjectInstance**
- **1258** `mod` **tests**
- **2291** `struct` **AddDelayFilter**
- **2293** `impl` **CutsceneCompileFilter** — *impl CutsceneCompileFilter for AddDelayFilter*
- **2298** `fn` **apply**

### `src/document/mod.rs`

- **6** `mod` **object**
- **7** `mod` **scene**
- **8** `mod` **value**

### `src/document/object.rs`

- **11** `struct` **ObjectDocument** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`
- **26** `struct` **LogicSpec** `pub`<br>↳ used in: `engine-authoring/src/document/mod.rs`
- **38** `enum` **LogicKind** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`

### `src/document/scene.rs`

- **20** `struct` **SceneDocument** `pub`<br>↳ used in: `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`
- **24** `impl` **SceneDocument** — *impl SceneDocument*
- **29** `fn` **compile** `pub`<br>↳ used in: `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/effect.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine/src/services.rs`
- **408** `struct` **MenuUiOption**
- **1752** `mod` **tests**

### `src/document/value.rs`

- **10** `enum` **ScalarValue** `pub`<br>↳ used in: `engine-authoring/src/document/mod.rs`
- **19** `enum` **ColorValue** `pub`<br>↳ used in: `engine-authoring/src/document/mod.rs`

### `src/lib.rs`

- **8** `mod` **compile** `pub`<br>↳ used in: `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/effect.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine/src/services.rs`
- **9** `mod` **document** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`
- **10** `mod` **package** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-core/src/scene/model.rs`, `engine/src/scene_loader.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **11** `mod` **repository** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `engine-asset/src/lib.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine/src/lib.rs`, `engine/src/scene_loader.rs`, `tools/devtool/src/fs_utils.rs`
- **12** `mod` **schema** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/metadata.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/scaffold.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`
- **13** `mod` **validate** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/document/scene.rs`
- **16** `type` **AuthoringResult** `pub`

### `src/package/mod.rs`

- **14** `struct` **PackageYamlFile** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **21** `impl` **PackageYamlFile** — *impl PackageYamlFile*
- **23** `fn` **new** `pub`
- **33** `struct` **ScenePackagePartials** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **44** `struct` **PackageError** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **49** `impl` **PackageError** — *impl PackageError*
- **58** `fn` **into_parts** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **63** `impl` **Display** — *impl Display for PackageError*
- **73** `impl` **Error** — *impl Error for PackageError*
- **87** `fn` **assemble_scene_package** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **189** `mod` **tests**

### `src/repository/mod.rs`

- **9** `fn` **is_discoverable_scene_path** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/schema/mod.rs`
- **14** `fn` **is_scene_package_manifest** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **21** `fn` **is_reserved_scene_partial_path** `pub`
- **43** `fn` **is_yaml_path** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`
- **48** `mod` **tests**

### `src/schema/mod.rs`

- **18** `struct` **GeneratedSchemaFile** `pub`
- **26** `fn` **generate_mod_schema_files** `pub`<br>↳ used in: `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`
- **189** `fn` **render_schema_file** `pub`<br>↳ used in: `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`
- **2726** `mod` **tests**

### `src/validate/mod.rs`

- **10** `enum` **TimelineDiagnostic** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`
- **40** `fn` **validate_sprite_timeline** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`
- **104** `mod` **tests**

---

## engine-behavior

### `src/factory.rs`

- **13** `trait` **BehaviorFactory** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/strategy/mod.rs`
- **20** `struct` **BuiltInBehaviorFactory** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/strategy/mod.rs`
- **22** `impl` **BehaviorFactory** — *impl BehaviorFactory for BuiltInBehaviorFactory*

### `src/lib.rs`

- **3** `mod` **factory** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-core/src/strategy/mod.rs`
- **4** `mod` **registry** `pub`<br>↳ used in: `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/exec/pipeline.rs`
- **36** `struct` **BehaviorContext** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **93** `enum` **BehaviorCommand** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`
- **165** `enum` **DebugLogSeverity** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **172** `trait` **Behavior** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-core/src/authoring/catalog.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/systems/behavior.rs`
- **186** `fn` **built_in_behavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **191** `fn` **builtin_behavior_names** `pub`
- **207** `fn` **behavior_metadata** `pub`
- **217** `fn` **init_behavior_system** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **223** `struct` **SceneAudioBehavior** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **227** `impl` **Behavior** — *impl Behavior for SceneAudioBehavior*
- **272** `struct` **BlinkBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **279** `impl` **BlinkBehavior** — *impl BlinkBehavior*
- **280** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **290** `impl` **Behavior** — *impl Behavior for BlinkBehavior*
- **310** `struct` **BobBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **318** `impl` **BobBehavior** — *impl BobBehavior*
- **319** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **330** `impl` **Behavior** — *impl Behavior for BobBehavior*
- **349** `struct` **FollowBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **355** `impl` **FollowBehavior** — *impl FollowBehavior*
- **356** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **365** `impl` **Behavior** — *impl Behavior for FollowBehavior*
- **390** `struct` **StageVisibilityBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **396** `struct` **MenuSelectedBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **401** `impl` **MenuSelectedBehavior** — *impl MenuSelectedBehavior*
- **402** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **410** `impl` **Behavior** — *impl Behavior for MenuSelectedBehavior*
- **427** `struct` **MenuCarouselBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **437** `impl` **MenuCarouselBehavior** — *impl MenuCarouselBehavior*
- **438** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **456** `impl` **Behavior** — *impl Behavior for MenuCarouselBehavior*
- **512** `struct` **MenuCarouselObjectBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **522** `impl` **MenuCarouselObjectBehavior** — *impl MenuCarouselObjectBehavior*
- **523** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **547** `impl` **Behavior** — *impl Behavior for MenuCarouselObjectBehavior*
- **1602** `struct` **RhaiScriptBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **1616** `struct` **ScriptSceneApi**
- **1627** `struct` **ScriptObjectApi**
- **1634** `struct` **ScriptGameApi**
- **1640** `struct` **ScriptLevelApi**
- **1645** `struct` **ScriptPersistenceApi**
- **1650** `struct` **ScriptGameplayApi**
- **1660** `struct` **ScriptGameplayEntityApi**
- **1667** `struct` **ScriptTerminalApi**
- **1672** `struct` **ScriptInputApi**
- **1680** `struct` **ScriptDebugApi**
- **1687** `struct` **ScriptAudioApi**
- **1691** `impl` **ScriptSceneApi** — *impl ScriptSceneApi*
- **1792** `fn` **spawn**
- **1806** `fn` **despawn**
- **1836** `impl` **ScriptObjectApi** — *impl ScriptObjectApi*
- **1862** `impl` **ScriptGameApi** — *impl ScriptGameApi*
- **1892** `fn` **remove**
- **1939** `impl` **ScriptLevelApi** — *impl ScriptLevelApi*
- **1969** `fn` **remove**
- **1986** `fn` **select**
- **2008** `impl` **ScriptTerminalApi** — *impl ScriptTerminalApi*
- **2030** `impl` **ScriptInputApi** — *impl ScriptInputApi*
- **2099** `impl` **ScriptDebugApi** — *impl ScriptDebugApi*
- **2112** `fn` **info**
- **2116** `fn` **warn**
- **2120** `fn` **error**
- **2141** `impl` **ScriptAudioApi** — *impl ScriptAudioApi*
- **2176** `fn` **play_song**
- **2190** `fn` **stop_song**
- **2199** `impl` **ScriptPersistenceApi** — *impl ScriptPersistenceApi*
- **2229** `fn` **remove**
- **2246** `fn` **reload**
- **2254** `impl` **ScriptGameplayApi** — *impl ScriptGameplayApi*
- **2294** `fn` **count**
- **2331** `fn` **spawn**
- **2344** `fn` **despawn**
- **2374** `fn` **exists**
- **2384** `fn` **kind**
- **2394** `fn` **tags**
- **2473** `fn` **remove**
- **2496** `fn` **set_transform**
- **2513** `fn` **transform**
- **2530** `fn` **set_physics**
- **2559** `fn` **physics**
- **2602** `fn` **set_lifetime**
- **3159** `impl` **ScriptGameplayEntityApi** — *impl ScriptGameplayEntityApi*
- **3160** `fn` **exists**
- **3181** `fn` **despawn**
- **3225** `fn` **kind**
- **3232** `fn` **tags**
- **3402** `fn` **transform**
- **3440** `fn` **physics**
- **3482** `fn` **collider**
- **3540** `fn` **data**
- **3639** `fn` **heading_vector**
- **3656** `impl` **RhaiScriptBehavior** — *impl RhaiScriptBehavior*
- **3657** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **3686** `impl` **Drop** — *impl Drop for RhaiScriptBehavior*
- **3695** `impl` **RhaiScriptBehavior** — *impl RhaiScriptBehavior*
- **3727** `impl` **Behavior** — *impl Behavior for RhaiScriptBehavior*
- **4032** `fn` **smoke_validate_rhai_script** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **4461** `struct` **SelectedArrowsBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **4475** `enum` **ArrowSide**
- **4480** `impl` **SelectedArrowsBehavior** — *impl SelectedArrowsBehavior*
- **4481** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **4509** `impl` **Behavior** — *impl Behavior for SelectedArrowsBehavior*
- **4572** `impl` **StageVisibilityBehavior** — *impl StageVisibilityBehavior*
- **4573** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **4586** `impl` **Behavior** — *impl Behavior for StageVisibilityBehavior*
- **4604** `struct` **TimedVisibilityBehavior** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`
- **4611** `impl` **TimedVisibilityBehavior** — *impl TimedVisibilityBehavior*
- **4612** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **4622** `impl` **Behavior** — *impl Behavior for TimedVisibilityBehavior*
- **4655** `fn` **resolve_target**
- **4934** `enum` **TimeScope**
- **4939** `impl` **TimeScope** — *impl TimeScope*
- **4940** `fn` **from_params** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-scene-runtime/src/behavior_runner.rs`
- **4981** `impl` **BehaviorContext** — *impl BehaviorContext*
- **4982** `fn` **resolve_target** `pub`<br>↳ used in: `engine/src/gpu/render.rs`
- **4986** `fn` **object_state** `pub`<br>↳ used in: `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/lib.rs`
- **4990** `fn` **object_region** `pub`
- **4994** `fn` **resolved_object_state** `pub`
- **4999** `fn` **resolved_object_region** `pub`
- **5006** `mod` **tests**

### `src/registry/mod.rs`

- **3** `mod` **provider** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-audio/src/systems_audio.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/services.rs`, `engine/src/world.rs`
- **14** `struct` **ModBehaviorRegistry** `pub`<br>↳ used in: `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`
- **20** `struct` **ModBehavior** `pub`
- **26** `impl` **ModBehaviorRegistry** — *impl ModBehaviorRegistry*
- **28** `fn` **get** `pub`
- **33** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **38** `fn` **is_empty** `pub`
- **44** `struct` **RawModBehavior**
- **115** `fn` **load_mod_behaviors_with_errors** `pub`<br>↳ used in: `engine-mod/src/startup/checks/rhai_scripts.rs`
- **160** `fn` **load_mod_behaviors** `pub`<br>↳ used in: `engine/src/lib.rs`
- **169** `mod` **tests**

### `src/registry/provider.rs`

- **6** `trait` **BehaviorProvider** `pub`<br>↳ used in: `engine-behavior/src/registry/mod.rs`
- **7** `fn` **scene**
- **8** `fn` **animator**
- **10** `fn` **game_state**
- **11** `fn` **mod_behaviors**

---

## engine-capture

### `src/capture.rs`

- **11** `struct` **FrameCapture** `pub`<br>↳ used in: `engine-capture/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`
- **16** `impl` **FrameCapture** — *impl FrameCapture*
- **18** `fn` **new** `pub`
- **31** `fn` **capture** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/color.rs`, `engine-core/src/world.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/exec/tokenizer.rs`
- **103** `mod` **tests**

### `src/compare.rs`

- **9** `struct` **FrameHeader** `pub`<br>↳ used in: `engine-capture/src/lib.rs`
- **16** `struct` **SerializedCell** `pub`<br>↳ used in: `engine-capture/src/lib.rs`
- **27** `fn` **load_frame** `pub`<br>↳ used in: `engine-capture/src/lib.rs`
- **87** `fn` **compare_frames** `pub`<br>↳ used in: `engine/tests/frame_regression.rs`
- **110** `fn` **list_frame_files** `pub`<br>↳ used in: `engine/tests/frame_regression.rs`
- **125** `mod` **tests**

### `src/lib.rs`

- **1** `mod` **capture** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/color.rs`, `engine-core/src/world.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/exec/tokenizer.rs`
- **2** `mod` **compare** `pub`<br>↳ used in: `engine-frame/src/lib.rs`, `engine/tests/frame_regression.rs`

---

## engine-compositor

### `src/access.rs`

- **13** `trait` **CompositorAccess** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/services.rs`
- **15** `fn` **scene_runtime**
- **18** `fn` **animator**
- **24** `fn` **runtime_settings**
- **30** `fn` **scene3d_atlas**
- **36** `fn` **layer_compositor**

### `src/buffer_pool.rs`

- **15** `struct` **BufferPoolConfig** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **24** `impl` **Default** — *impl Default for BufferPoolConfig*
- **37** `struct` **BufferPool** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **44** `impl` **BufferPool** — *impl BufferPool*
- **46** `fn` **new** `pub`
- **60** `fn` **acquire** `pub`
- **79** `fn` **release**
- **87** `fn` **stats** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `rust-os/src/vfs/mod.rs`
- **99** `struct` **PoolStats** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **106** `struct` **PooledBuffer** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **111** `impl` **PooledBuffer** — *impl PooledBuffer*
- **113** `fn` **as_mut** `pub`
- **118** `fn` **as_ref** `pub`
- **124** `fn` **take** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/io/recent.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-debug/src/profiling.rs`, `engine-events/src/lib.rs`, `engine-io/src/lib.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/services.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/visual_binding.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/kernel/unit_of_work.rs`
- **129** `impl` **std** — *impl std::ops::Deref for PooledBuffer*
- **136** `impl` **std** — *impl std::ops::DerefMut for PooledBuffer*
- **142** `impl` **Drop** — *impl Drop for PooledBuffer*
- **163** `fn` **acquire_buffer** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/lib.rs`
- **168** `fn` **pool_stats** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **173** `mod` **tests**

### `src/compositor.rs`

- **161** `fn` **pack_halfblock_buffer** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/compositor/mod.rs`
- **248** `fn` **dispatch_composite** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/compositor/mod.rs`

### `src/effect_applicator.rs`

- **15** `fn` **apply_sprite_effects** `pub`<br>↳ used in: `engine-compositor/src/render/common.rs`
- **65** `fn` **apply_layer_effects** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`
- **150** `fn` **resolve_step_by_elapsed** `pub`
- **185** `mod` **tests**

### `src/image_render.rs`

- **14** `struct` **ImageView**
- **23** `fn` **full**
- **43** `fn` **pixel**
- **51** `fn` **render_image_content** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **99** `fn` **image_sprite_dimensions** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **452** `mod` **tests**

### `src/layer_compositor.rs`

- **22** `fn` **composite_layers** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`

### `src/layout/area.rs`

- **7** `struct` **RenderArea** `pub`<br>↳ used in: `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/sprite_renderer.rs`
- **16** `struct` **GridCellRect** `pub`<br>↳ used in: `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/render/containers.rs`
- **26** `fn` **resolve_x** `pub`<br>↳ used in: `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **43** `fn` **resolve_y** `pub`<br>↳ used in: `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`

### `src/layout/flex.rs`

- **21** `fn` **invalidate_flex_cache** `pub`
- **50** `fn` **compute_flex_cells** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **169** `mod` **tests**

### `src/layout/grid.rs`

- **24** `fn` **invalidate_grid_cache** `pub`
- **61** `fn` **compute_grid_cells** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **227** `mod` **tests**

### `src/layout/measure.rs`

- **26** `fn` **with_render_context** `pub`<br>↳ used in: `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **44** `fn` **measure_sprite_for_layout** `pub`<br>↳ used in: `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/sprite_renderer.rs`

### `src/layout/mod.rs`

- **3** `mod` **area** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/layout.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `rust-os/src/commands/net.rs`
- **4** `mod` **flex** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`
- **5** `mod` **grid** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-render/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`
- **6** `mod` **measure** `pub`
- **7** `mod` **tracks** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-compositor/src/layout/grid.rs`, `engine-core/src/authoring/catalog.rs`

### `src/layout/tracks.rs`

- **5** `enum` **TrackSpec** `pub`<br>↳ used in: `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`
- **12** `fn` **parse_track_spec** `pub`<br>↳ used in: `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/lib.rs`
- **30** `fn` **resolve_track_sizes** `pub`
- **99** `fn` **track_start** `pub`
- **113** `fn` **span_size** `pub`
- **129** `mod` **tests**

### `src/lib.rs`

- **10** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **11** `mod` **buffer_pool** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`
- **12** `mod` **compositor** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-io/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/construction.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/warmup.rs`
- **13** `mod` **effect_applicator** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`
- **14** `mod` **image_render** `pub`
- **15** `mod` **layer_compositor** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/layer_compositor.rs`
- **16** `mod` **layout** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/layout.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`
- **17** `mod` **obj_loader** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/warmup.rs`
- **18** `mod` **obj_prerender** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`
- **19** `mod` **obj_render** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-compositor/src/scene3d_atlas.rs`
- **20** `mod` **prerender** `pub`<br>↳ used in: `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/warmup.rs`
- **21** `mod` **provider** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-audio/src/systems_audio.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/services.rs`, `engine/src/world.rs`
- **22** `mod` **render** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/prepared_frame.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`
- **23** `mod` **scene3d_atlas** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine/src/systems/compositor/mod.rs`
- **24** `mod` **scene3d_prerender** `pub`<br>↳ used in: `engine/src/scene_pipeline.rs`
- **25** `mod` **scene_compositor** `pub`<br>↳ used in: `engine-compositor/src/scene_compositor.rs`
- **26** `mod` **sprite_renderer** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/obj_render.rs`
- **27** `mod` **systems** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-events/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine-physics/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **28** `mod` **text_render** `pub`
- **29** `mod` **warmup** `pub`<br>↳ used in: `engine-compositor/src/warmup.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/warmup.rs`
- **56** `fn` **clear_vector_primitives** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **61** `fn` **take_vector_primitives** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`

### `src/obj_loader.rs`

- **11** `struct` **ObjMesh** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`
- **20** `struct` **ObjFace** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`
- **34** `struct` **ObjMeshAdapter**
- **36** `impl` **SourceAdapter** — *impl SourceAdapter<ObjMesh> for ObjMeshAdapter*
- **64** `struct` **MaterialProps**
- **75** `impl` **Default** — *impl Default for MaterialProps*
- **86** `fn` **load_obj_mesh** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/warmup.rs`
- **322** `mod` **tests**

### `src/obj_prerender.rs`

- **8** `type` **PrerenderedCanvas** `pub`
- **11** `struct` **PrerenderedFrame** `pub`<br>↳ used in: `engine-compositor/src/prerender.rs`
- **27** `struct` **ObjPrerenderedFrames** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`
- **31** `impl` **ObjPrerenderedFrames** — *impl ObjPrerenderedFrames*
- **32** `fn` **new** `pub`
- **36** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **40** `fn` **get** `pub`
- **44** `fn` **is_empty** `pub`
- **48** `fn` **len** `pub`
- **54** `enum` **ObjPrerenderStatus** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`

### `src/obj_render.rs`

- **22** `fn` **with_prerender_frames** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/compositor/mod.rs`
- **71** `struct` **ProjectedVertex**
- **86** `struct` **ObjRenderParams** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`
- **145** `fn` **obj_sprite_dimensions** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/sprite_renderer.rs`
- **161** `fn` **render_obj_to_canvas** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/prerender.rs`
- **870** `fn` **render_obj_to_shared_buffers** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`
- **911** `fn` **render_obj_content** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **971** `fn` **try_blit_prerendered** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **1030** `struct` **Viewport**
- **1038** `fn` **virtual_dimensions** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`
- **1500** `fn` **blit_color_canvas** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`
- **1842** `mod` **tests**

### `src/prerender.rs`

- **13** `fn` **prerender_scene_sprites** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/prerender.rs`
- **84** `struct` **PrerenderTarget**

### `src/provider.rs`

- **6** `trait` **CompositorProvider** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **8** `fn` **scene_runtime**
- **9** `fn` **animator**
- **11** `fn` **runtime_settings**
- **12** `fn` **debug_features**

### `src/render/common.rs`

- **20** `struct` **RenderCtx** `pub`<br>↳ used in: `engine-compositor/src/render/containers.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **35** `fn` **check_visibility** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **60** `fn` **is_sprite_offscreen** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **68** `fn` **sprite_transform_offset** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **81** `fn` **compute_draw_pos** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`
- **102** `fn` **finalize_sprite** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`

### `src/render/containers.rs`

- **13** `fn` **render_children_in_cells** `pub`<br>↳ used in: `engine-compositor/src/render/mod.rs`, `engine-compositor/src/sprite_renderer.rs`

### `src/render/mod.rs`

- **3** `mod` **common** `pub`<br>↳ used in: `engine-asset/src/repositories.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/render/containers.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/rasterizer/font_loader.rs`, `tools/devtool/src/cli.rs`, `tools/schema-gen/src/main.rs`
- **4** `mod` **containers** `pub`<br>↳ used in: `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/sprite_renderer.rs`

### `src/scene3d_atlas.rs`

- **22** `struct` **Scene3DAtlas** `pub`<br>↳ used in: `engine-3d/src/lib.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene3d_prerender.rs`
- **27** `impl` **Scene3DAtlas** — *impl Scene3DAtlas*
- **28** `fn` **new** `pub`
- **32** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **40** `fn` **get** `pub`
- **44** `fn` **is_empty** `pub`
- **49** `fn` **len** `pub`
- **65** `fn` **with_atlas** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **73** `impl` **Scene3DAtlas** — *impl Scene3DAtlas*
- **76** `fn` **current_get** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`

### `src/scene3d_prerender.rs`

- **21** `fn` **prerender_scene3d_atlas** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/scene3d_prerender.rs`
- **87** `struct` **AssetRootResolver**
- **91** `impl` **Scene3DAssetResolver** — *impl Scene3DAssetResolver for AssetRootResolver<'_>*
- **132** `struct` **WorkItem**
- **141** `struct` **ObjectRenderSpec**
- **215** `struct` **LightParams**

### `src/scene_compositor.rs`

- **15** `struct` **CompositeParams** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/lib.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **39** `trait` **SceneCompositor** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/strategy/mod.rs`
- **50** `struct` **CellSceneCompositor** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/strategy/mod.rs`
- **53** `struct` **HalfblockSceneCompositor** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/strategy/mod.rs`

### `src/sprite_renderer.rs`

- **27** `static` **VECTOR_PRIMITIVES** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`
- **98** `fn` **render_sprites** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`

### `src/systems/mod.rs`

- **1** `mod` **postfx** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-pipeline/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/prepared_frame.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`

### `src/systems/postfx/glow.rs`

- **13** `struct` **GlowPixel** `pub`
- **20** `impl` **GlowPixel** — *impl GlowPixel*
- **21** `fn` **add_scaled** `pub`
- **28** `fn` **normalized** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/recent.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/ui/icons.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_render.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/effects/effect.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render/src/font_loader.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/splash.rs`
- **43** `struct` **GlowScratch** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **50** `static` **GLOW_SCRATCH** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **65** `fn` **build_glow_map_inplace** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`

### `src/systems/postfx/mod.rs`

- **3** `mod` **glow**
- **4** `mod` **pass_burn_in**
- **5** `mod` **pass_crt**
- **6** `mod` **pass_crt_distort**
- **7** `mod` **pass_ruby_crt**
- **8** `mod` **pass_scan_glitch**
- **9** `mod` **pass_underlay**
- **10** `mod` **registry**
- **31** `struct` **PostFxRuntime**
- **41** `struct` **PostFxContext** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **47** `fn` **postfx_system** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/game_loop.rs`
- **123** `impl` **PostFxRuntime** — *impl PostFxRuntime*
- **124** `fn` **apply**
- **327** `fn` **rand01** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **331** `fn` **normalize_bg** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **339** `fn` **scale_colour** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`
- **349** `fn` **normalized_coords** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`
- **363** `fn` **colour_luma** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`
- **368** `fn` **lerp_colour_local** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`

### `src/systems/postfx/pass_burn_in.rs`

- **37** `struct` **BurnInState**
- **50** `impl` **Default** — *impl Default for BurnInState*
- **65** `impl` **BurnInState** — *impl BurnInState*
- **113** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`

### `src/systems/postfx/pass_crt.rs`

- **25** `struct` **GlitchBand**
- **35** `struct` **DistortKey**
- **43** `impl` **DistortKey** — *impl DistortKey*
- **56** `struct` **DistortCache**
- **63** `struct` **CrtCache**
- **71** `impl` **CrtCache** — *impl CrtCache*
- **178** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`
- **309** `struct` **RubyCfg**

### `src/systems/postfx/pass_crt_distort.rs`

- **5** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`
- **78** `mod` **tests**

### `src/systems/postfx/pass_ruby_crt.rs`

- **9** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`

### `src/systems/postfx/pass_scan_glitch.rs`

- **7** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`

### `src/systems/postfx/pass_underlay.rs`

- **7** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`

### `src/systems/postfx/registry.rs`

- **4** `enum` **PostFxBuiltin** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`
- **13** `enum` **CompiledPostFx** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`
- **50** `fn` **compile_passes** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`
- **109** `mod` **tests**
- **113** `fn` **effect**

### `src/text_render.rs`

- **18** `struct` **ClipRect** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/sprite_renderer.rs`
- **25** `impl` **ClipRect** — *impl ClipRect*
- **27** `fn` **contains**
- **36** `fn` **render_text_content** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **158** `fn` **text_sprite_dimensions** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **246** `fn` **dim_colour** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`
- **257** `mod` **tests**

### `src/warmup.rs`

- **9** `fn` **warmup_scene_meshes** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine/src/systems/warmup.rs`

---

## engine-core

### `src/access.rs`

- **12** `trait` **BufferAccess** `pub`<br>↳ used in: `engine-core/src/access_tests.rs`
- **13** `fn` **buffer**
- **17** `impl` **BufferAccess** — *impl BufferAccess for World*
- **18** `fn` **buffer**
- **27** `trait` **GameStateAccess** `pub`<br>↳ used in: `engine-core/src/access_tests.rs`
- **28** `fn` **game_state**
- **32** `impl` **GameStateAccess** — *impl GameStateAccess for World*
- **33** `fn` **game_state**
- **42** `trait` **AssetAccess** `pub`<br>↳ used in: `engine-core/src/access_tests.rs`
- **46** `impl` **AssetAccess** — *impl AssetAccess for World*

### `src/access_tests.rs`

- **2** `mod` **access_trait_integration_tests**

### `src/animations/animation.rs`

- **5** `struct` **Transform** `pub`<br>↳ used in: `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`
- **13** `trait` **SpriteAnimation** `pub`<br>↳ used in: `engine-core/src/animations/mod.rs`

### `src/animations/builtin/float.rs`

- **8** `struct` **FloatAnimation** `pub`<br>↳ used in: `engine-core/src/animations/builtin/mod.rs`, `engine-core/src/animations/mod.rs`
- **10** `impl` **SpriteAnimation** — *impl SpriteAnimation for FloatAnimation*

### `src/animations/builtin/mod.rs`

- **4** `mod` **float** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `tools/schema-gen/src/main.rs`

### `src/animations/mod.rs`

- **1** `mod` **animation** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-animation/src/access.rs`, `engine-animation/src/animator.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/builtin/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **2** `mod` **builtin** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/strategy/effect_factory.rs`, `rust-os/src/exec/builtins.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **3** `mod` **params** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **12** `struct` **AnimationDispatcher** `pub`<br>↳ used in: `engine-compositor/src/render/common.rs`, `engine-core/src/authoring/catalog.rs`
- **16** `impl` **AnimationDispatcher** — *impl AnimationDispatcher*
- **17** `fn` **new** `pub`
- **29** `fn` **compute_transform** `pub`<br>↳ used in: `engine-compositor/src/render/common.rs`
- **47** `fn` **builtin_names** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-core/src/authoring/catalog.rs`, `tools/schema-gen/src/main.rs`
- **52** `fn` **metadata** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/compare.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`
- **122** `mod` **tests**

### `src/animations/params.rs`

- **9** `enum` **AnimationAxis** `pub`<br>↳ used in: `engine-core/src/animations/builtin/float.rs`
- **19** `struct` **AnimationParams** `pub`<br>↳ used in: `engine-core/src/animations/animation.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`

### `src/asset_cache.rs`

- **5** `struct` **AssetCache** `pub`<br>↳ used in: `engine-compositor/src/obj_loader.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/lib.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/font_loader.rs`
- **10** `const` **fn** `pub`
- **18** `fn` **get_or_load** `pub`<br>↳ used in: `engine-core/src/asset_source.rs`, `engine-render/src/font_loader.rs`, `engine/src/rasterizer/font_loader.rs`

### `src/asset_source.rs`

- **11** `enum` **SourceKind** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`
- **17** `struct` **SourceRef** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`, `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **22** `impl` **SourceRef** — *impl SourceRef*
- **24** `fn` **mod_asset** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`, `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **37** `fn` **kind** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/scene_run.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/source_loader.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/game_object.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/game_object.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/scene3d_format.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`
- **41** `fn` **value** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effect_params.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/icons.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/source_loader.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/template.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/context.rs`, `engine-persistence/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/font_loader.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/scene3d_format.rs`, `engine/src/splash.rs`, `engine/src/systems/visual_sync.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`
- **45** `fn` **normalized_value** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **51** `trait` **SourceLoader** `pub`<br>↳ used in: `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **56** `fn` **has_source**
- **64** `trait` **SourceAdapter** `pub`
- **76** `fn` **load_source_bytes** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`
- **82** `fn` **has_source** `pub`<br>↳ used in: `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`
- **87** `fn` **load_decoded_source** `pub`<br>↳ used in: `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`

### `src/assets.rs`

- **7** `struct` **AssetRoot** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/lib.rs`, `engine/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`
- **11** `impl` **AssetRoot** — *impl AssetRoot*
- **13** `fn` **new** `pub`
- **18** `fn` **mod_source** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/io/indexer.rs`, `editor/src/main.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/audio_sequencer.rs`, `engine/src/image_loader.rs`, `engine/src/level_state.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene_loader.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`
- **23** `fn` **resolve** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/compile/scene.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/services.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/scaffold.rs`
- **29** `fn` **resolve_asset_path** `pub`

### `src/authoring/catalog.rs`

- **8** `struct` **InputProfileShape** `pub`
- **129** `struct` **SugarCatalog** `pub`
- **140** `struct` **ShorthandSpec** `pub`
- **152** `fn` **sugar_catalog** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`
- **233** `struct` **StaticAuthoringCatalog** `pub`
- **241** `fn` **static_catalog** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `engine-authoring/src/schema/mod.rs`
- **252** `fn` **effect_fields** `pub`
- **266** `fn` **behavior_catalog** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`
- **1119** `fn` **animation_catalog** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/animations/mod.rs`
- **1127** `fn` **input_profile_catalog** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`
- **1132** `fn` **input_profile_shapes** `pub`
- **1150** `mod` **tests**

### `src/authoring/metadata.rs`

- **5** `enum` **TargetKind** `pub`<br>↳ used in: `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`
- **16** `enum` **ValueKind** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`
- **28** `enum` **ValueSource** `pub`<br>↳ used in: `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`
- **37** `enum` **Requirement** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`
- **48** `struct` **FieldMetadata** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`, `engine/src/lib.rs`

### `src/authoring/mod.rs`

- **4** `mod` **catalog** `pub`<br>↳ used in: `editor/src/domain/mod.rs`, `editor/src/io/indexer.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine/src/lib.rs`, `tools/schema-gen/src/main.rs`
- **6** `mod` **metadata** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/compare.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`

### `src/buffer.rs`

- **6** `struct` **Cell** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/capture.rs`, `engine-capture/src/compare.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene/model.rs`, `engine-pipeline/src/lib.rs`, `engine-render-policy/src/lib.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **13** `const` **TRUE_BLACK** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/systems/compositor/mod.rs`
- **15** `impl` **Cell** — *impl Cell*
- **17** `fn` **blank** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-render-terminal/src/renderer.rs`
- **26** `impl` **Default** — *impl Default for Cell*
- **33** `struct` **CellDiff** `pub`
- **49** `struct` **Buffer** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/hardware.rs`, `rust-os/src/kernel/resources.rs`
- **74** `impl` **Buffer** — *impl Buffer*
- **75** `fn` **new** `pub`
- **108** `fn` **fill** `pub`<br>↳ used in: `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/present.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/vector_overlay.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`
- **121** `fn` **set** `pub`
- **147** `fn` **get** `pub`
- **157** `fn` **back_cells** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`
- **165** `fn` **back_cells_mut** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`
- **171** `fn` **diff** `pub`<br>↳ used in: `app/src/main.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/strategy/diff.rs`, `engine-io/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/present.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/main.rs`
- **192** `fn` **diff_into** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/renderer.rs`
- **208** `fn` **diff_into_dirty** `pub`<br>↳ used in: `engine-core/src/strategy/diff.rs`
- **226** `fn` **diff_into_row_skip** `pub`<br>↳ used in: `engine-core/src/strategy/diff.rs`
- **247** `fn` **swap** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/services.rs`, `engine/src/splash.rs`
- **263** `fn` **restore_front_to_back** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **275** `fn` **invalidate** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine/src/systems/scene_lifecycle.rs`
- **289** `fn` **reset_dirty** `pub`<br>↳ used in: `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/lib.rs`
- **299** `fn` **back_hash** `pub`
- **317** `fn` **dirty_bounds** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-pipeline/src/strategies/halfblock.rs`
- **333** `fn` **mark_dirty_region** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **364** `fn` **total_cells** `pub`<br>↳ used in: `engine/src/bench.rs`, `engine/src/game_loop.rs`
- **369** `fn` **dirty_cell_count** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **378** `fn` **expand_dirty_bounds** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`
- **408** `fn` **narrow_for_effect** `pub`
- **425** `fn` **resize** `pub`<br>↳ used in: `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/text_render.rs`, `engine-frame/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **451** `fn` **blit_from** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-core/src/effects/builtin/fade.rs`
- **514** `fn` **copy_back_from** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`
- **523** `fn` **mark_all_dirty** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/pass_crt.rs`
- **560** `mod` **tests**

### `src/color.rs`

- **10** `enum` **Color** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/theme.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-sdl2/src/color_convert.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render-terminal/src/strategy/flush_trait.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`
- **36** `impl` **Color** — *impl Color*
- **38** `const` **fn** `pub`
- **43** `const` **BLACK** `pub`<br>↳ used in: `engine-core/src/buffer.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/gpu/render.rs`
- **46** `const` **WHITE** `pub`
- **54** `fn` **to_rgb** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine-render-sdl2/src/color_convert.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`
- **78** `fn` **is_reset** `pub`
- **83** `impl` **Default** — *impl Default for Color*
- **91** `mod` **tests**

### `src/effects/builtin/artifact.rs`

- **10** `static` **METADATA** `pub`
- **23** `fn` **noise**
- **31** `struct` **ArtifactOutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **33** `impl` **Effect** — *impl Effect for ArtifactOutEffect*
- **34** `fn` **apply**
- **136** `fn` **metadata**

### `src/effects/builtin/blur.rs`

- **14** `static` **METADATA** `pub`
- **35** `struct` **BlurEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **37** `impl` **Effect** — *impl Effect for BlurEffect*
- **39** `fn` **apply**
- **118** `fn` **metadata**
- **124** `mod` **tests**

### `src/effects/builtin/brighten.rs`

- **8** `static` **METADATA** `pub`
- **21** `struct` **BrightenEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **23** `impl` **Effect** — *impl Effect for BrightenEffect*
- **24** `fn` **apply**
- **56** `fn` **metadata**

### `src/effects/builtin/clear_to_colour.rs`

- **10** `static` **METADATA** `pub`
- **29** `struct` **ClearToColourEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **31** `impl` **Effect** — *impl Effect for ClearToColourEffect*
- **32** `fn` **apply**
- **48** `fn` **metadata**

### `src/effects/builtin/crt_burn_in.rs`

- **8** `static` **METADATA** `pub`
- **26** `struct` **CrtBurnInEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **28** `impl` **Effect** — *impl Effect for CrtBurnInEffect*
- **29** `fn` **apply**
- **33** `fn` **metadata**

### `src/effects/builtin/crt_distort.rs`

- **8** `static` **METADATA** `pub`
- **26** `struct` **CtrDistortEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **28** `impl` **Effect** — *impl Effect for CtrDistortEffect*
- **29** `fn` **apply**
- **32** `fn` **metadata**

### `src/effects/builtin/crt_on.rs`

- **15** `static` **METADATA** `pub`
- **26** `struct` **CrtOnEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **28** `impl` **Effect** — *impl Effect for CrtOnEffect*
- **29** `fn` **apply**
- **116** `fn` **metadata**
- **122** `fn` **crt_blank_row** `pub`<br>↳ used in: `engine-core/src/effects/builtin/power_off.rs`
- **129** `fn` **crt_static_row** `pub`<br>↳ used in: `engine-core/src/effects/builtin/power_off.rs`
- **139** `fn` **crt_dim_row** `pub`<br>↳ used in: `engine-core/src/effects/builtin/power_off.rs`

### `src/effects/builtin/crt_reflection.rs`

- **9** `static` **METADATA** `pub`
- **48** `struct` **CrtReflectionEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **50** `impl` **Effect** — *impl Effect for CrtReflectionEffect*
- **51** `fn` **apply**
- **107** `fn` **metadata**
- **127** `fn` **normalized_coords**
- **153** `fn` **normalize_bg**
- **195** `mod` **tests**

### `src/effects/builtin/crt_ruby.rs`

- **8** `static` **METADATA** `pub`
- **24** `struct` **CtrRubyEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **26** `impl` **Effect** — *impl Effect for CtrRubyEffect*
- **27** `fn` **apply**
- **30** `fn` **metadata**

### `src/effects/builtin/crt_scan_glitch.rs`

- **8** `static` **METADATA** `pub`
- **24** `struct` **CtrScanGlitchEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **26** `impl` **Effect** — *impl Effect for CtrScanGlitchEffect*
- **27** `fn` **apply**
- **30** `fn` **metadata**

### `src/effects/builtin/crt_underlay.rs`

- **8** `static` **METADATA** `pub`
- **26** `struct` **CtrUnderlayEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **28** `impl` **Effect** — *impl Effect for CtrUnderlayEffect*
- **29** `fn` **apply**
- **32** `fn` **metadata**

### `src/effects/builtin/cutout.rs`

- **11** `static` **METADATA** `pub`
- **84** `struct` **CutoutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **86** `impl` **Effect** — *impl Effect for CutoutEffect*
- **87** `fn` **apply**
- **200** `fn` **metadata**
- **256** `struct` **CutoutCacheKey**
- **270** `struct` **CutoutCacheEntry**
- **478** `mod` **tests**

### `src/effects/builtin/devour.rs`

- **13** `static` **METADATA** `pub`
- **26** `fn` **noise**
- **31** `struct` **DevourOutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **33** `impl` **Effect** — *impl Effect for DevourOutEffect*
- **34** `fn` **apply**
- **138** `fn` **metadata**

### `src/effects/builtin/fade.rs`

- **11** `static` **METADATA_FADE_IN** `pub`
- **22** `static` **METADATA_FADE_OUT** `pub`
- **33** `struct` **FadeInEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **35** `impl` **Effect** — *impl Effect for FadeInEffect*
- **36** `fn` **apply**
- **56** `fn` **metadata**
- **62** `struct` **FadeOutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **64** `impl` **Effect** — *impl Effect for FadeOutEffect*
- **65** `fn` **apply**
- **91** `fn` **metadata**

### `src/effects/builtin/fade_to_black.rs`

- **11** `static` **METADATA** `pub`
- **22** `struct` **FadeToBlackEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **24** `impl` **Effect** — *impl Effect for FadeToBlackEffect*
- **25** `fn` **apply**
- **62** `fn` **metadata**

### `src/effects/builtin/glitch.rs`

- **10** `static` **METADATA** `pub`
- **27** `fn` **noise**
- **37** `struct` **GlitchOutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **39** `impl` **Effect** — *impl Effect for GlitchOutEffect*
- **40** `fn` **apply**
- **209** `fn` **metadata**

### `src/effects/builtin/lightning.rs`

- **13** `static` **METADATA_LIGHTNING_FLASH** `pub`
- **34** `static` **METADATA_LIGHTNING_BRANCH** `pub`
- **53** `static` **METADATA_LIGHTNING_FBM** `pub`
- **64** `static` **METADATA_LIGHTNING_OPTICAL_80S** `pub`
- **74** `static` **METADATA_LIGHTNING_GROWTH** `pub`
- **91** `static` **METADATA_LIGHTNING_AMBIENT** `pub`
- **109** `static` **METADATA_LIGHTNING_NATURAL** `pub`
- **127** `static` **METADATA_TESLA_ORB** `pub`
- **280** `enum` **LightningOrientation**
- **285** `impl` **LightningOrientation** — *impl LightningOrientation*
- **286** `fn` **from_params**
- **440** `struct` **LightningFlashEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **442** `impl` **Effect** — *impl Effect for LightningFlashEffect*
- **443** `fn` **apply**
- **468** `fn` **metadata**
- **474** `struct` **LightningGrowthEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **476** `impl` **Effect** — *impl Effect for LightningGrowthEffect*
- **477** `fn` **apply**
- **658** `fn` **metadata**
- **664** `struct` **LightningBranchEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **666** `impl` **Effect** — *impl Effect for LightningBranchEffect*
- **667** `fn` **apply**
- **853** `fn` **metadata**
- **862** `struct` **LightningOptical80sEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **864** `impl` **Effect** — *impl Effect for LightningOptical80sEffect*
- **865** `fn` **apply**
- **966** `fn` **metadata**
- **972** `struct` **LightningFbmEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **974** `impl` **Effect** — *impl Effect for LightningFbmEffect*
- **975** `fn` **apply**
- **1042** `fn` **metadata**
- **1048** `struct` **LightningNaturalEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **1050** `impl` **Effect** — *impl Effect for LightningNaturalEffect*
- **1051** `fn` **apply**
- **1308** `fn` **metadata**
- **1315** `struct` **LightningAmbientEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **1317** `impl` **Effect** — *impl Effect for LightningAmbientEffect*
- **1318** `fn` **apply**
- **1358** `fn` **metadata**
- **1364** `struct` **TeslaOrbEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **1366** `impl` **Effect** — *impl Effect for TeslaOrbEffect*
- **1367** `fn` **apply**
- **1493** `fn` **metadata**
- **1499** `mod` **tests**

### `src/effects/builtin/mod.rs`

- **3** `mod` **artifact** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-pipeline/src/lib.rs`
- **4** `mod` **blur** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/scene/model.rs`
- **5** `mod` **brighten** `pub`<br>↳ used in: `engine-core/src/effects/builtin/brighten.rs`
- **6** `mod` **clear_to_colour** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **7** `mod` **crt_burn_in** `pub`
- **8** `mod` **crt_distort** `pub`
- **9** `mod` **crt_on** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/power_off.rs`
- **10** `mod` **crt_reflection** `pub`
- **11** `mod` **crt_ruby** `pub`
- **12** `mod` **crt_scan_glitch** `pub`
- **13** `mod` **crt_underlay** `pub`
- **14** `mod` **cutout** `pub`<br>↳ used in: `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/scene/model.rs`
- **15** `mod` **devour** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-core/src/effects/builtin/devour.rs`
- **16** `mod` **fade** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/metadata.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine/src/scene3d_format.rs`, `engine/src/splash.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **17** `mod` **fade_to_black** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **18** `mod` **glitch** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`
- **19** `mod` **lightning** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/model.rs`
- **20** `mod` **neon_edge_glow** `pub`
- **21** `mod` **posterize** `pub`<br>↳ used in: `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/scene/model.rs`
- **22** `mod` **power_off** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`
- **23** `mod` **scanlines** `pub`<br>↳ used in: `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`
- **24** `mod` **shake** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/scene/model.rs`
- **25** `mod` **shatter** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-core/src/effects/builtin/shatter.rs`
- **26** `mod` **shine** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-authoring/src/compile/scene.rs`, `engine-core/src/effects/builtin/shine.rs`
- **27** `mod` **terminal_crt** `pub`
- **28** `mod` **whiteout** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-core/src/effects/builtin/whiteout.rs`
- **62** `type` **EffectConstructor** `pub`
- **64** `struct` **BuiltinEffectDefinition** `pub`
- **69** `static` **BUILTIN_EFFECTS** `pub`<br>↳ used in: `engine-core/src/effects/mod.rs`
- **210** `fn` **builtin_names** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-core/src/authoring/catalog.rs`, `tools/schema-gen/src/main.rs`
- **223** `mod` **tests**

### `src/effects/builtin/neon_edge_glow.rs`

- **16** `static` **METADATA** `pub`
- **54** `struct` **NeonEdgeGlowEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **56** `impl` **Effect** — *impl Effect for NeonEdgeGlowEffect*
- **57** `fn` **apply**
- **169** `fn` **metadata**

### `src/effects/builtin/posterize.rs`

- **8** `static` **METADATA** `pub`
- **29** `struct` **PosterizeEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **31** `impl` **Effect** — *impl Effect for PosterizeEffect*
- **33** `fn` **apply**
- **57** `fn` **metadata**
- **82** `mod` **tests**

### `src/effects/builtin/power_off.rs`

- **12** `static` **METADATA** `pub`
- **23** `struct` **PowerOffEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **25** `impl` **Effect** — *impl Effect for PowerOffEffect*
- **26** `fn` **apply**
- **67** `fn` **metadata**

### `src/effects/builtin/scanlines.rs`

- **9** `static` **METADATA** `pub`
- **20** `struct` **ScanlinesEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **22** `impl` **Effect** — *impl Effect for ScanlinesEffect*
- **23** `fn` **apply**
- **37** `fn` **metadata**

### `src/effects/builtin/shake.rs`

- **6** `static` **METADATA** `pub`
- **23** `struct` **ScreenShakeEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **25** `impl` **Effect** — *impl Effect for ScreenShakeEffect*
- **26** `fn` **apply**
- **67** `fn` **metadata**
- **73** `mod` **tests**

### `src/effects/builtin/shatter.rs`

- **10** `static` **METADATA** `pub`
- **21** `fn` **noise**
- **30** `struct` **ShatterGlitchEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **32** `impl` **Effect** — *impl Effect for ShatterGlitchEffect*
- **33** `fn` **apply**
- **270** `fn` **metadata**

### `src/effects/builtin/shine.rs`

- **11** `static` **METADATA** `pub`
- **54** `struct` **ShineEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **56** `impl` **Effect** — *impl Effect for ShineEffect*
- **57** `fn` **apply**
- **123** `fn` **metadata**

### `src/effects/builtin/terminal_crt.rs`

- **13** `static` **METADATA** `pub`
- **70** `struct` **TerminalCrtEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **72** `impl` **Effect** — *impl Effect for TerminalCrtEffect*
- **73** `fn` **apply**
- **161** `fn` **metadata**
- **181** `fn` **normalized_coords**
- **214** `fn` **rand01**
- **218** `fn` **scale_colour**
- **229** `mod` **tests**

### `src/effects/builtin/whiteout.rs`

- **8** `static` **METADATA** `pub`
- **19** `struct` **WhiteoutEffect** `pub`<br>↳ used in: `engine-core/src/effects/builtin/mod.rs`
- **21** `impl` **Effect** — *impl Effect for WhiteoutEffect*
- **22** `fn` **apply**
- **45** `fn` **metadata**

### `src/effects/effect.rs`

- **7** `struct` **Region** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/prepared_frame.rs`, `engine/src/splash.rs`
- **14** `impl` **Region** — *impl Region*
- **16** `fn` **full** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/scene3d_format.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/strategy/diff.rs`, `engine-game/src/components.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-scene-runtime/src/access.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`
- **26** `fn` **row** `pub`
- **37** `trait` **Effect** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **44** `fn` **metadata**
- **50** `fn` **apply**
- **54** `struct` **EffectTargetMask** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`
- **56** `impl` **EffectTargetMask** — *impl EffectTargetMask*
- **57** `const` **SCENE** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine/src/bench.rs`, `engine/src/systems/scene_lifecycle.rs`
- **58** `const` **LAYER** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`
- **59** `const` **SPRITE** `pub`
- **60** `const` **SPRITE_TEXT** `pub`
- **61** `const` **SPRITE_BITMAP** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_reflection.rs`
- **62** `const` **ANY** `pub`
- **70** `const` **fn** `pub`
- **74** `const` **fn** `pub`

### `src/effects/metadata.rs`

- **12** `enum` **ParamControl** `pub`<br>↳ used in: `editor/src/ui/components/effects_preview.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/mod.rs`, `tools/schema-gen/src/main.rs`
- **33** `impl` **ParamControl** — *impl ParamControl*
- **34** `const` **fn** `pub`
- **50** `const` **fn** `pub`
- **64** `fn` **is_slider** `pub`
- **68** `const` **fn** `pub`
- **81** `struct` **ParamMetadata** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/mod.rs`
- **92** `impl` **ParamMetadata** — *impl ParamMetadata*
- **94** `const` **fn** `pub`
- **130** `struct` **EffectMetadata** `pub`<br>↳ used in: `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`
- **147** `impl` **EffectMetadata** — *impl EffectMetadata*
- **149** `fn` **slider_params** `pub`
- **154** `fn` **all_params** `pub`
- **161** `const` **fn** `pub`
- **183** `const` **fn** `pub`
- **198** `const` **fn** `pub`
- **212** `const` **fn** `pub`
- **228** `const` **EASINGS** `pub`
- **230** `const` **P_EASING** `pub`<br>↳ used in: `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`
- **237** `const` **P_INTENSITY** `pub`<br>↳ used in: `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/whiteout.rs`
- **246** `const` **P_SPEED** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/shine.rs`
- **255** `const` **P_STRIKES** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **264** `const` **P_THICKNESS** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **273** `const` **P_GLOW** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **274** `const` **P_ORIENTATION** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **281** `const` **P_OCTAVES** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`
- **291** `static` **META_UNKNOWN** `pub`<br>↳ used in: `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`
- **302** `mod` **tests**

### `src/effects/mod.rs`

- **1** `mod` **builtin** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/strategy/effect_factory.rs`, `rust-os/src/exec/builtins.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **2** `mod` **effect** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/mod.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/utils/math.rs`, `engine-core/src/effects/utils/mod.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/easing.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-core/src/strategy/mod.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-pipeline/src/strategies/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **3** `mod` **metadata** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/compare.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`
- **4** `mod` **utils** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`
- **15** `struct` **EffectDispatcher** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `tools/schema-gen/src/main.rs`
- **19** `impl` **EffectDispatcher** — *impl EffectDispatcher*
- **20** `fn` **new** `pub`
- **34** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`
- **47** `fn` **supports** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/effect.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/terminal_caps.rs`, `engine-terminal/src/lib.rs`, `engine/src/scene3d_format.rs`
- **51** `fn` **supports_target_kind** `pub`<br>↳ used in: `engine-mod/src/startup/checks/effect_registry.rs`
- **63** `fn` **metadata** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/compare.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`
- **71** `fn` **builtin_names** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-core/src/authoring/catalog.rs`, `tools/schema-gen/src/main.rs`
- **76** `impl` **Default** — *impl Default for EffectDispatcher*
- **84** `fn` **shared_dispatcher** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine/src/splash.rs`, `tools/schema-gen/src/main.rs`
- **90** `fn` **apply_effect** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/systems/postfx/mod.rs`

### `src/effects/utils/color.rs`

- **4** `fn` **colour_to_rgb** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`
- **28** `fn` **lerp_colour** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`
- **43** `mod` **tests**

### `src/effects/utils/math.rs`

- **4** `const` **TICK_MS** `pub`<br>↳ used in: `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`
- **6** `const` **PHASE_POWER_ON** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **8** `const` **PHASE_WHITE_FLASH** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **10** `const` **PHASE_SCAN_END** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **12** `const` **PHASE_SCAN_START** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **14** `const` **PHASE_BOOT** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **17** `fn` **smoothstep** `pub`<br>↳ used in: `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`
- **23** `fn` **phase_progress** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`
- **31** `mod` **tests**

### `src/effects/utils/mod.rs`

- **4** `mod` **color** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/theme.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-sdl2/src/color_convert.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render-terminal/src/strategy/flush_trait.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/vector_overlay.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/style.rs`
- **6** `mod` **math** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-physics/src/lib.rs`
- **8** `mod` **noise** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/kernel/modem.rs`

### `src/effects/utils/noise.rs`

- **2** `fn` **crt_hash** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`
- **11** `mod` **tests**

### `src/game_object.rs`

- **5** `enum` **GameObjectKind** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`
- **19** `struct` **GameObject** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`

### `src/game_state.rs`

- **15** `struct` **GameState** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/level_state.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`
- **19** `impl` **GameState** — *impl GameState*
- **21** `fn` **new** `pub`
- **28** `fn` **with_initial_data** `pub`
- **39** `fn` **get** `pub`
- **48** `fn` **set** `pub`
- **93** `fn` **has** `pub`
- **103** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **141** `fn` **push** `pub`
- **195** `fn` **snapshot** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/prepared_frame.rs`, `engine/src/systems/scene_lifecycle.rs`
- **200** `fn` **clear** `pub`
- **207** `impl` **Default** — *impl Default for GameState*
- **214** `mod` **tests**

### `src/level_state.rs`

- **12** `struct` **LevelStore**
- **22** `struct` **LevelState** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/level_state.rs`, `engine/src/systems/behavior.rs`
- **26** `impl` **LevelState** — *impl LevelState*
- **28** `fn` **new** `pub`
- **38** `fn` **register_level** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/level_state.rs`
- **54** `fn` **ids** `pub`
- **62** `fn` **has_level** `pub`
- **70** `fn` **select** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/input/keys.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/metadata.rs`, `engine-runtime/src/lib.rs`, `engine/src/level_state.rs`, `engine/src/systems/scene_lifecycle.rs`
- **86** `fn` **current_id** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine/src/level_state.rs`
- **94** `fn` **active_snapshot** `pub`
- **103** `fn` **get** `pub`
- **113** `fn` **set** `pub`
- **124** `fn` **has** `pub`
- **135** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **146** `fn` **push** `pub`
- **157** `impl` **Default** — *impl Default for LevelState*
- **283** `mod` **tests**

### `src/lib.rs`

- **5** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **7** `mod` **animations** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine/src/lib.rs`
- **9** `mod` **asset_cache** `pub`<br>↳ used in: `engine-compositor/src/obj_loader.rs`, `engine-core/src/asset_source.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/font_loader.rs`
- **11** `mod` **assets** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/context.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/generic.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **13** `mod` **authoring** `pub`<br>↳ used in: `editor/src/io/indexer.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine/src/lib.rs`, `tools/schema-gen/src/main.rs`
- **15** `mod` **buffer** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-audio/src/audio.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-debug/src/log.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/present.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/vector_overlay.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/mesh.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `rust-os/src/kernel/journal.rs`
- **17** `mod` **color** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/theme.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-sdl2/src/color_convert.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render-terminal/src/strategy/flush_trait.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/vector_overlay.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/style.rs`
- **19** `mod` **effects** `pub`<br>↳ used in: `editor/src/domain/asset_index.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/mod.rs`, `editor/src/io/indexer.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/effects/utils/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-game/src/components.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **21** `mod` **game_object** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/lib.rs`
- **23** `mod` **game_state** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/level_state.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`
- **25** `mod` **level_state** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/level_state.rs`, `engine/src/systems/behavior.rs`
- **27** `mod` **logging** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/main.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/logging.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/audio_sequencer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **29** `mod` **markup** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/color.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/lib.rs`, `rust-os/src/screen_buffer.rs`, `rust-os/src/style.rs`
- **31** `mod` **scene** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/domain/scene_index.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/lib.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-animation/src/animator.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/menu.rs`, `engine-animation/src/provider.rs`, `engine-asset/src/lib.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/access.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/mod.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_object.rs`, `engine-core/src/game_state.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-debug/src/log.rs`, `engine-error/src/lib.rs`, `engine-events/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-game/src/game_object.rs`, `engine-game/src/game_state.rs`, `engine-game/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/context.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **34** `mod` **scene_runtime_types** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/lib.rs`
- **36** `mod` **strategy** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-game/src/strategy.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **38** `mod` **world** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/access.rs`, `engine-audio/src/access.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/strategy.rs`, `engine-render-terminal/src/renderer.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `engine/src/world.rs`
- **41** `mod` **access_tests**
- **42** `mod` **asset_source** `pub`<br>↳ used in: `engine-asset/src/source_loader.rs`, `engine-compositor/src/obj_loader.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`

### `src/logging.rs`

- **16** `enum` **LogLevel** `pub`
- **23** `impl` **LogLevel** — *impl LogLevel*
- **36** `struct` **LogOverlayLine** `pub`
- **44** `struct` **RunLoggerConfig** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`
- **55** `struct` **RunLogInfo** `pub`
- **64** `struct` **RunLogger**
- **71** `impl` **RunLogger** — *impl RunLogger*
- **125** `fn` **resolve_enabled** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`
- **142** `fn` **init_run_logger** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`
- **181** `fn` **install_panic_hook** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`
- **193** `fn` **is_enabled** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`, `engine-debug/src/lib.rs`
- **198** `fn` **run_log_info** `pub`
- **203** `fn` **tail_recent** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **214** `fn` **log** `pub`
- **221** `fn` **debug** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-debug/src/access.rs`, `engine-debug/src/lib.rs`, `engine-debug/src/log.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine/src/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **226** `fn` **info** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/main.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/warmup.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/audio_sequencer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/session.rs`
- **231** `fn` **warn** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `editor/src/state/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine/src/audio_sequencer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **236** `fn` **error** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/io/yaml.rs`, `editor/src/main.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/log.rs`, `engine-error/src/lib.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/runner.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/lib.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/image_loader.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/main.rs`, `tools/devtool/src/main.rs`, `tools/sound-server/src/main.rs`
- **315** `mod` **tests**

### `src/markup.rs`

- **6** `struct` **Span** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/status_bar.rs`
- **20** `fn` **parse_spans** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`
- **73** `fn` **strip_markup** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-scene-runtime/src/terminal_shell.rs`
- **105** `mod` **tests**

### `src/scene/color.rs`

- **14** `enum` **TermColour** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`
- **29** `fn` **parse_colour_str** `pub`<br>↳ used in: `engine-core/src/markup.rs`
- **103** `impl` **From** — *impl From<&TermColour> for Color*

### `src/scene/easing.rs`

- **6** `enum` **Easing** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`
- **14** `impl` **Easing** — *impl Easing*
- **18** `fn` **apply** `pub`<br>↳ used in: `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/hosts/registry.rs`

### `src/scene/metadata.rs`

- **55** `static` **SCENE_FIELDS** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`
- **255** `static` **LAYER_FIELDS** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`
- **319** `static` **SPRITE_FIELDS** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`
- **1204** `static` **OBJECT_FIELDS** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`
- **1268** `mod` **tests**

### `src/scene/mod.rs`

- **7** `mod` **color** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/theme.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/utils/color.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/strategy/diff.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-sdl2/src/color_convert.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-render-terminal/src/strategy/flush_trait.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/vector_overlay.rs`, `engine-vector/src/lib.rs`, `engine/src/bench.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/style.rs`
- **9** `mod` **easing** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/model.rs`, `engine/src/scene3d_format.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **11** `mod` **metadata** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/compare.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/lib.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`
- **13** `mod` **model** `pub`<br>↳ used in: `editor/src/domain/mod.rs`, `editor/src/io/fs_scan.rs`, `engine-3d/src/lib.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render/src/overlay.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/gpu/mesh.rs`, `tools/schema-gen/src/main.rs`
- **15** `mod` **sprite** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/obj_prerender.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/layout/mod.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/render/mod.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/vector_overlay.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **17** `mod` **template** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/template.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `tools/schema-gen/src/main.rs`
- **19** `mod` **ui_theme** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`

### `src/scene/model.rs`

- **13** `enum` **SceneRenderedMode** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-policy/src/lib.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/systems/scene_lifecycle.rs`
- **30** `struct` **Animation** `pub`<br>↳ used in: `engine-animation/src/animator.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/sprite.rs`
- **41** `struct` **BehaviorSpec** `pub`<br>↳ used in: `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/lib.rs`
- **50** `struct` **BehaviorParams** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`
- **102** `struct` **Effect** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **117** `enum` **EffectTargetKind** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-mod/src/startup/checks/effect_registry.rs`
- **127** `impl` **EffectTargetKind** — *impl EffectTargetKind*
- **128** `const` **fn** `pub`
- **145** `struct` **EffectParams** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine/src/splash.rs`
- **259** `struct` **Step** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-core/src/scene/mod.rs`, `engine/src/scene_pipeline.rs`
- **267** `impl` **Step** — *impl Step*
- **269** `fn` **duration_ms** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/scene3d_format.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`
- **278** `enum` **StageTrigger** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-core/src/scene/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **287** `struct` **Stage** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-core/src/scene/mod.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **297** `impl` **Stage** — *impl Stage*
- **299** `fn` **duration_ms** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-3d/src/scene3d_format.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-render/src/image_loader.rs`, `engine/src/image_loader.rs`, `engine/src/scene3d_format.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`
- **306** `struct` **SceneStages** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **317** `struct` **SceneAudio** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **328** `struct` **MenuOption** `pub`<br>↳ used in: `engine-animation/src/menu.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **339** `struct` **SceneInput** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`
- **351** `impl` **SceneInput** — *impl SceneInput*
- **353** `fn` **builtin_profiles** `pub`<br>↳ used in: `engine-core/src/authoring/catalog.rs`
- **361** `enum` **UiPersistence** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **373** `struct` **SceneUi** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-core/src/scene/mod.rs`
- **384** `impl` **Default** — *impl Default for SceneUi*
- **397** `struct` **ObjViewerControls** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **404** `struct` **TerminalSizeTesterControls** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **441** `enum` **TerminalShellMode** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/engine_io.rs`
- **453** `enum` **TerminalShellOutput** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **458** `impl` **TerminalShellOutput** — *impl TerminalShellOutput*
- **459** `fn` **lines** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/io/recent.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/lib.rs`, `engine-io/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/types.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/rasterizer/types.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/modem.rs`, `rust-os/src/kernel/unit_of_work.rs`, `rust-os/src/main.rs`, `rust-os/src/protocol.rs`, `rust-os/src/screen_buffer.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/scaffold.rs`, `tools/sound-server/src/main.rs`
- **468** `struct` **TerminalShellCommand** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **477** `struct` **TerminalShellSidecarControls** `pub`
- **486** `struct` **TerminalShellControls** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`
- **577** `struct` **AudioCue** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine-events/src/lib.rs`
- **587** `struct` **LayerStages** `pub`<br>↳ used in: `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/render/common.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/effect_registry.rs`
- **599** `struct` **Layer** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/game_object.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-game/src/game_object.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`
- **629** `struct` **Scene** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/domain/scene_index.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/mod.rs`, `engine-animation/src/systems.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/game_object.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/template.rs`, `engine-debug/src/log.rs`, `engine-game/src/game_object.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/context.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/lib.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/warmup.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`
- **666** `impl` **Scene** — *impl Scene*
- **669** `fn` **on_enter_duration_ms** `pub`<br>↳ used in: `engine-authoring/src/validate/mod.rs`
- **675** `mod` **tests**

### `src/scene/sprite.rs`

- **6** `enum` **HorizontalAlign** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-compositor/src/layout/area.rs`, `engine-core/src/scene/mod.rs`, `engine/src/splash.rs`
- **14** `enum` **VerticalAlign** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-compositor/src/layout/area.rs`, `engine-core/src/scene/mod.rs`, `engine/src/splash.rs`
- **23** `enum` **TextTransform** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **34** `struct` **Glow** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/metadata.rs`
- **85** `enum` **FlexDirection** `pub`<br>↳ used in: `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/measure.rs`, `engine-core/src/scene/mod.rs`
- **94** `enum` **SpriteSizePreset** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-core/src/scene/mod.rs`, `engine-render-policy/src/lib.rs`
- **101** `impl` **SpriteSizePreset** — *impl SpriteSizePreset*
- **102** `const` **fn** `pub`
- **111** `const` **fn** `pub`
- **120** `const` **fn** `pub`
- **130** `impl` **TryFrom** — *impl TryFrom<u8> for SpriteSizePreset*
- **157** `enum` **Sprite** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/sidebar/effects.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/metadata.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/scene3d_format.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/main.rs`
- **671** `impl` **Sprite** — *impl Sprite*
- **672** `fn` **id** `pub`
- **685** `fn` **z_index** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-scene-runtime/src/construction.rs`, `engine/src/systems/compositor/mod.rs`, `tools/devtool/src/scaffold.rs`
- **698** `fn` **stages** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/ui/components/start_screen.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/schema-gen/src/main.rs`
- **711** `fn` **grid_position** `pub`<br>↳ used in: `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`
- **773** `fn` **walk_recursive** `pub`<br>↳ used in: `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`
- **790** `fn` **behaviors** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/scene_lifecycle.rs`
- **803** `fn` **hide_on_leave** `pub`<br>↳ used in: `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/scene/metadata.rs`
- **816** `fn` **appear_at_ms** `pub`<br>↳ used in: `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/metadata.rs`
- **829** `fn` **disappear_at_ms** `pub`<br>↳ used in: `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/metadata.rs`
- **858** `fn` **animations** `pub`<br>↳ used in: `editor/src/state/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine/src/lib.rs`
- **873** `mod` **tests**

### `src/scene/template.rs`

- **7** `fn` **expand_scene_templates** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`

### `src/scene/ui_theme.rs`

- **5** `struct` **WindowThemeStyle** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **16** `struct` **ScrollListThemeStyle** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **24** `struct` **UiThemeStyle** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **48** `fn` **normalize_theme_key** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **57** `fn` **resolve_ui_theme** `pub`<br>↳ used in: `engine-core/src/scene/mod.rs`
- **130** `fn` **resolve_ui_theme_or_default** `pub`<br>↳ used in: `engine-authoring/src/document/scene.rs`, `engine-core/src/scene/mod.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **135** `mod` **tests**

### `src/scene_runtime_types.rs`

- **12** `struct` **TargetResolver** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`
- **19** `impl` **TargetResolver** — *impl TargetResolver*
- **20** `fn` **new** `pub`
- **29** `fn` **from_parts** `pub`<br>↳ used in: `engine-render/src/image_loader.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`
- **44** `fn` **scene_object_id** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-scene-runtime/src/terminal_shell.rs`
- **49** `fn` **resolve_alias** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **53** `fn` **register_alias** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **58** `fn` **aliases_snapshot** `pub`
- **63** `fn` **layer_object_id** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **68** `fn` **sprite_object_id** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/lib.rs`
- **76** `fn` **effect_region** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`
- **90** `fn` **register_layer** `pub`
- **94** `fn` **register_sprite** `pub`
- **111** `struct` **ObjectRuntimeState** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **117** `impl` **Default** — *impl Default for ObjectRuntimeState*
- **129** `struct` **RawKeyEvent** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **142** `struct` **SidecarIoFrameState** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **151** `struct` **ObjCameraState** `pub`<br>↳ used in: `engine-compositor/src/compositor.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`

### `src/strategy/diff.rs`

- **12** `trait` **DiffStrategy** `pub`<br>↳ used in: `engine-core/src/strategy/mod.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`
- **13** `fn` **diff_into**
- **17** `struct` **FullScanDiff** `pub`<br>↳ used in: `engine-core/src/strategy/mod.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`
- **19** `impl` **DiffStrategy** — *impl DiffStrategy for FullScanDiff*
- **21** `fn` **diff_into**
- **29** `struct` **DirtyRegionDiff** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-core/src/strategy/mod.rs`, `engine-pipeline/src/strategies/mod.rs`
- **31** `impl` **DiffStrategy** — *impl DiffStrategy for DirtyRegionDiff*
- **33** `fn` **diff_into**
- **43** `struct` **RowSkipDiff** `pub`<br>↳ used in: `engine-core/src/buffer.rs`, `engine-core/src/strategy/mod.rs`, `engine-pipeline/src/strategies/mod.rs`
- **45** `impl` **DiffStrategy** — *impl DiffStrategy for RowSkipDiff*
- **47** `fn` **diff_into**

### `src/strategy/effect_factory.rs`

- **10** `trait` **ModEffectFactory** `pub`<br>↳ used in: `engine-core/src/strategy/mod.rs`

### `src/strategy/mod.rs`

- **2** `mod` **diff** `pub`<br>↳ used in: `app/src/main.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/strategy/diff.rs`, `engine-io/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/present.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/main.rs`
- **4** `mod` **effect_factory** `pub`

### `src/world.rs`

- **7** `struct` **World** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/lib.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/prepared_frame.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `rust-os/src/commands/net.rs`
- **12** `impl` **World** — *impl World*
- **14** `fn` **new** `pub`
- **22** `fn` **register** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-authoring/src/compile/scene.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`
- **28** `fn` **register_scoped** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`
- **34** `fn` **get** `pub`
- **42** `fn` **get_mut** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/access.rs`, `engine-audio/src/access.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/access.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-persistence/src/lib.rs`, `engine-render/src/font_loader.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine/src/game_loop.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/services.rs`, `engine/src/systems/audio_sequencer.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/gameplay_events.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `rust-os/src/kernel/mail.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **54** `fn` **clear_scoped** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/scene_pipeline.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **59** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **70** `fn` **with_ref_and_mut** `pub`
- **98** `impl` **Default** — *impl Default for World*

---

## engine-debug

### `src/access.rs`

- **8** `trait` **DebugAccess** `pub`
- **9** `fn` **debug_features**
- **11** `fn` **debug_log**
- **18** `impl` **DebugAccess** — *impl DebugAccess for World*
- **19** `fn` **debug_features**
- **25** `fn` **debug_log**

### `src/lib.rs`

- **3** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **4** `mod` **log** `pub`
- **5** `mod` **profiling** `pub`<br>↳ used in: `engine-debug/src/profiling.rs`
- **15** `struct` **FpsCounter** `pub`<br>↳ used in: `engine-debug/src/access.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/services.rs`
- **19** `impl` **Default** — *impl Default for FpsCounter*
- **27** `struct` **SystemTimings** `pub`<br>↳ used in: `engine-debug/src/access.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/services.rs`
- **36** `enum` **DebugOverlayMode** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine/src/systems/scene_lifecycle.rs`
- **44** `struct` **DebugFeatures** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-debug/src/access.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`
- **53** `impl` **DebugFeatures** — *impl DebugFeatures*
- **55** `fn` **from_enabled** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine/src/lib.rs`
- **70** `fn` **from_env** `pub`
- **91** `struct` **ProcessStats** `pub`<br>↳ used in: `engine-debug/src/access.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/services.rs`
- **100** `impl` **Default** — *impl Default for ProcessStats*
- **113** `impl` **ProcessStats** — *impl ProcessStats*
- **115** `fn` **tick** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/utils/math.rs`, `engine-game/src/gameplay.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`

### `src/log.rs`

- **9** `enum` **DebugSeverity** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **15** `impl` **DebugSeverity** — *impl DebugSeverity*
- **16** `fn` **label** `pub`<br>↳ used in: `editor/src/domain/effect_params.rs`, `editor/src/state/effects_browser.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/icons.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/gameplay.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/mesh.rs`, `engine/src/gpu/render.rs`, `engine/src/systems/scene_lifecycle.rs`
- **24** `fn` **is_error** `pub`
- **31** `struct` **DebugLogEntry** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **43** `impl` **DebugLogEntry** — *impl DebugLogEntry*
- **45** `fn` **display_line** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **66** `struct` **DebugLogBuffer** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-debug/src/access.rs`, `engine-debug/src/lib.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`
- **75** `impl` **DebugLogBuffer** — *impl DebugLogBuffer*
- **76** `fn` **new** `pub`
- **85** `fn` **push** `pub`
- **97** `fn` **push_error** `pub`
- **114** `fn` **push_info** `pub`
- **131** `fn` **push_warn** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`
- **148** `fn` **recent** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/io/recent.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/start_screen.rs`, `engine-core/src/buffer.rs`, `engine-core/src/logging.rs`, `engine-render-terminal/src/renderer.rs`
- **157** `fn` **is_empty** `pub`
- **161** `fn` **clear_errors** `pub`
- **172** `trait` **DiagnosticSink** `pub`
- **176** `impl` **DiagnosticSink** — *impl DiagnosticSink for DebugLogBuffer*
- **183** `struct` **NullSink** `pub`
- **185** `impl` **DiagnosticSink** — *impl DiagnosticSink for NullSink*

### `src/profiling.rs`

- **14** `struct` **TimingMarker** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **25** `struct` **ProfilingFrame** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **33** `struct` **Profiler** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **44** `impl` **Default** — *impl Default for Profiler*
- **55** `impl` **Profiler** — *impl Profiler*
- **58** `fn` **set_enabled** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **64** `fn` **is_enabled** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`, `engine-debug/src/lib.rs`
- **70** `fn` **begin** `pub`
- **80** `fn` **end** `pub`
- **103** `fn` **mark** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `tools/schema-gen/src/main.rs`
- **118** `fn` **finish_frame** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **129** `fn` **reset** `pub`<br>↳ used in: `engine-animation/src/animator.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/strategy/diff.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/skip.rs`
- **135** `fn` **stats** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `rust-os/src/vfs/mod.rs`
- **166** `struct` **ProfileStats** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **174** `struct` **ProfileEntry** `pub`
- **180** `impl` **ProfileEntry** — *impl ProfileEntry*
- **182** `fn` **avg_us** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`
- **194** `fn` **export_flamegraph_stacks** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **222** `struct` **ProfileSpan** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **227** `impl` **ProfileSpan** — *impl ProfileSpan*
- **229** `fn` **new** `pub`
- **238** `impl` **Drop** — *impl Drop for ProfileSpan*
- **255** `fn` **begin_span** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **261** `fn` **end_span** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **267** `fn` **mark** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `tools/schema-gen/src/main.rs`
- **272** `fn` **get_stats** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **277** `fn` **finish_frame** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **282** `fn` **set_enabled** `pub`<br>↳ used in: `engine-debug/src/lib.rs`
- **287** `fn` **is_enabled** `pub`<br>↳ used in: `editor/src/ui/components/scenes_preview.rs`, `engine-debug/src/lib.rs`
- **292** `mod` **tests**

---

## engine-error

### `src/lib.rs`

- **7** `enum` **EngineError** `pub`<br>↳ used in: `app/src/main.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/obj_loader.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/runner.rs`, `engine-render/src/image_loader.rs`, `engine/src/game_loop.rs`, `engine/src/image_loader.rs`, `engine/src/lib.rs`, `engine/src/scene_loader.rs`

---

## engine-events

### `src/access.rs`

- **7** `trait` **EventAccess** `pub`
- **8** `fn` **events**
- **12** `impl` **EventAccess** — *impl EventAccess for World*
- **13** `fn` **events**

### `src/input_backend.rs`

- **6** `trait` **InputBackend** `pub`<br>↳ used in: `engine-events/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`
- **8** `fn` **poll_events**

### `src/key.rs`

- **7** `enum` **KeyCode** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `engine-animation/src/menu.rs`, `engine-events/src/lib.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/scene_lifecycle.rs`
- **28** `impl` **fmt** — *impl fmt::Display for KeyCode*
- **55** `struct` **KeyModifiers** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `engine-events/src/lib.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/scene_lifecycle.rs`
- **57** `impl` **KeyModifiers** — *impl KeyModifiers*
- **58** `const` **NONE** `pub`<br>↳ used in: `editor/src/input/keys.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/scene_lifecycle.rs`
- **59** `const` **SHIFT** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/scene_lifecycle.rs`
- **60** `const` **CONTROL** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/scene_lifecycle.rs`
- **61** `const` **ALT** `pub`
- **64** `fn` **contains** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/cutscene.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/terminal_caps.rs`, `engine-physics/src/lib.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine-terminal/src/lib.rs`, `engine/src/pipelines/mod.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/kernel/users.rs`, `rust-os/src/state.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **69** `impl` **std** — *impl std::ops::BitOr for KeyModifiers*
- **76** `impl` **std** — *impl std::ops::BitOrAssign for KeyModifiers*
- **84** `struct` **KeyEvent** `pub`<br>↳ used in: `editor/src/input/keys.rs`, `editor/src/state/scene_run.rs`, `engine-animation/src/menu.rs`, `engine-events/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/scene_lifecycle.rs`
- **89** `impl` **KeyEvent** — *impl KeyEvent*
- **90** `fn` **new** `pub`

### `src/lib.rs`

- **3** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **4** `mod` **input_backend** `pub`<br>↳ used in: `engine/src/lib.rs`
- **5** `mod` **key** `pub`
- **12** `enum` **EngineEvent** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-events/src/input_backend.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **27** `struct` **EventQueue** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-events/src/access.rs`, `engine/src/lib.rs`, `engine/src/services.rs`
- **31** `impl` **EventQueue** — *impl EventQueue*
- **33** `fn` **new** `pub`
- **38** `fn` **push** `pub`
- **43** `fn` **drain** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-behavior/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/screen_buffer.rs`
- **48** `fn` **is_empty** `pub`

---

## engine-frame

### `src/lib.rs`

- **17** `struct` **FrameTicket** `pub`<br>↳ used in: `engine/src/prepared_frame.rs`
- **24** `impl` **FrameTicket** — *impl FrameTicket*
- **27** `fn` **matches_generation** `pub`
- **33** `fn` **is_newer_than** `pub`
- **43** `fn` **is_acceptable** `pub`
- **49** `mod` **tests**

---

## engine-game

### `src/collision.rs`

- **7** `struct` **CollisionStrategies** `pub`<br>↳ used in: `engine-game/src/lib.rs`, `engine/src/lib.rs`, `engine/src/systems/collision.rs`
- **14** `enum` **BroadphaseKind** `pub`<br>↳ used in: `engine-game/src/lib.rs`
- **20** `enum` **NarrowphaseKind** `pub`<br>↳ used in: `engine-game/src/lib.rs`
- **26** `enum` **WrapStrategy** `pub`<br>↳ used in: `engine-game/src/lib.rs`, `engine/src/systems/collision.rs`
- **34** `struct` **CollisionHit** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/gameplay_events.rs`
- **39** `fn` **collision_system** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/components.rs`

- **6** `struct` **Transform2D** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/visual_sync.rs`
- **14** `struct` **PhysicsBody2D** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine/src/systems/ship_controller.rs`
- **25** `impl` **Default** — *impl Default for PhysicsBody2D*
- **39** `enum` **ColliderShape** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/lib.rs`
- **44** `impl` **Default** — *impl Default for ColliderShape*
- **51** `struct` **Collider2D** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`
- **57** `impl` **Default** — *impl Default for Collider2D*
- **68** `struct` **VisualBinding** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine/src/systems/visual_sync.rs`
- **73** `impl` **VisualBinding** — *impl VisualBinding*
- **75** `fn` **all_visual_ids** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/systems/gameplay.rs`
- **88** `enum` **DespawnVisual** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **93** `impl` **Default** — *impl Default for DespawnVisual*
- **100** `struct` **Lifetime** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine/src/systems/gameplay.rs`
- **110** `struct` **EntityTimers** `pub`<br>↳ used in: `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`
- **118** `struct` **WrapBounds** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine/src/systems/gameplay.rs`
- **125** `impl` **WrapBounds** — *impl WrapBounds*
- **126** `fn` **new** `pub`
- **132** `fn` **wrap_x** `pub`<br>↳ used in: `engine-game/src/gameplay.rs`
- **141** `fn` **wrap_y** `pub`<br>↳ used in: `engine-game/src/gameplay.rs`
- **157** `struct` **TopDownShipController** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/gameplay.rs`, `engine/src/systems/ship_controller.rs`
- **177** `impl` **TopDownShipController** — *impl TopDownShipController*
- **179** `fn` **new** `pub`
- **193** `fn` **set_turn** `pub`
- **198** `fn` **set_thrust** `pub`
- **204** `fn` **heading_vector** `pub`<br>↳ used in: `engine/src/systems/ship_controller.rs`
- **211** `fn` **heading_radians** `pub`<br>↳ used in: `engine/src/systems/ship_controller.rs`
- **216** `impl` **Default** — *impl Default for TopDownShipController*
- **241** `enum` **GameplayEvent** `pub`<br>↳ used in: `engine-game/src/collision.rs`, `engine-game/src/gameplay.rs`

### `src/game_object.rs`

- **5** `enum` **GameObjectKind** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`
- **18** `struct` **GameObject** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/lib.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`

### `src/game_state.rs`

- **15** `struct` **GameState** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/level_state.rs`, `engine-game/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`
- **19** `impl` **GameState** — *impl GameState*
- **21** `fn` **new** `pub`
- **28** `fn` **with_initial_data** `pub`
- **39** `fn` **get** `pub`
- **48** `fn` **set** `pub`
- **93** `fn` **has** `pub`
- **103** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **141** `fn` **push** `pub`
- **195** `fn` **snapshot** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/prepared_frame.rs`, `engine/src/systems/scene_lifecycle.rs`
- **200** `fn` **clear** `pub`
- **207** `impl` **Default** — *impl Default for GameState*
- **214** `mod` **tests**

### `src/gameplay.rs`

- **16** `struct` **GameplayEntity** `pub`<br>↳ used in: `engine-game/src/lib.rs`
- **24** `struct` **GameplayStore**
- **47** `impl` **Default** — *impl Default for GameplayStore*
- **77** `struct` **GameplayWorld** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/lib.rs`, `engine-game/src/strategy.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/lib.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/visual_sync.rs`
- **81** `impl` **GameplayWorld** — *impl GameplayWorld*
- **83** `fn` **new** `pub`
- **90** `fn` **clear** `pub`
- **97** `fn` **count** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/components.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/generic.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/systems/engine_io.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/mail.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/style.rs`
- **109** `fn` **spawn** `pub`<br>↳ used in: `engine-core/src/level_state.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`
- **136** `fn` **despawn** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **162** `fn` **register_child** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **170** `fn` **despawn_children** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **181** `fn` **exists** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/schema/mod.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/buffer.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-game/src/game_state.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/context.rs`, `engine-persistence/src/lib.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/world.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **189** `fn` **kind_of** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **197** `fn` **tags** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/color.rs`, `rust-os/src/style.rs`
- **209** `fn` **ids** `pub`
- **217** `fn` **query_kind** `pub`
- **230** `fn` **count_kind** `pub`
- **235** `fn` **first_kind** `pub`
- **240** `fn` **query_tag** `pub`
- **257** `fn` **count_tag** `pub`
- **262** `fn` **first_tag** `pub`
- **267** `fn` **get_entity** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **275** `fn` **data** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-capture/src/compare.rs`, `engine-compositor/src/access.rs`, `engine-core/src/buffer.rs`, `engine-core/src/game_object.rs`, `engine-core/src/game_state.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/game_object.rs`, `engine-game/src/game_state.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/types.rs`, `engine-render/src/vector_overlay.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/gpu/mesh.rs`, `engine/src/gpu/render.rs`, `engine/src/rasterizer/types.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/vfs/seed.rs`
- **284** `fn` **set_many** `pub`
- **301** `fn` **set_transform** `pub`<br>↳ used in: `engine-game/src/strategy.rs`, `engine/src/systems/ship_controller.rs`
- **312** `fn` **transform** `pub`<br>↳ used in: `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/strategy.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`, `engine/src/scene3d_format.rs`, `engine/src/systems/ship_controller.rs`, `engine/src/systems/visual_sync.rs`
- **319** `fn` **set_physics** `pub`<br>↳ used in: `engine-game/src/strategy.rs`, `engine/src/systems/ship_controller.rs`
- **330** `fn` **physics** `pub`<br>↳ used in: `engine-game/src/components.rs`, `engine-game/src/strategy.rs`, `engine/src/game_loop.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/ship_controller.rs`
- **337** `fn` **set_collider** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **348** `fn` **collider** `pub`<br>↳ used in: `engine-game/src/collision.rs`
- **355** `fn` **set_lifetime** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **366** `fn` **lifetime** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/buffer_pool.rs`, `engine/src/game_loop.rs`, `engine/src/systems/gameplay.rs`
- **373** `fn` **set_visual** `pub`
- **384** `fn` **visual** `pub`<br>↳ used in: `app/src/main.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/capture.rs`, `engine-core/src/effects/builtin/mod.rs`, `engine-core/src/effects/utils/math.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/components.rs`, `engine/src/lib.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/visual_sync.rs`, `tools/schema-gen/src/main.rs`
- **391** `fn` **add_visual** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **407** `fn` **ids_with_physics** `pub`<br>↳ used in: `engine-game/src/strategy.rs`
- **414** `fn` **ids_with_lifetime** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **421** `fn` **ids_with_colliders** `pub`<br>↳ used in: `engine-game/src/collision.rs`
- **428** `fn` **ids_with_visual_binding** `pub`<br>↳ used in: `engine/src/systems/visual_sync.rs`
- **435** `fn` **remove_lifetime** `pub`
- **444** `fn` **cooldown_start** `pub`
- **452** `fn` **cooldown_ready** `pub`
- **461** `fn` **cooldown_remaining** `pub`
- **471** `fn` **status_add** `pub`
- **479** `fn` **status_has** `pub`
- **488** `fn` **status_remaining** `pub`
- **497** `fn` **ids_with_timers** `pub`
- **503** `fn` **tick_timers** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **521** `fn` **tick_world_timers** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **541** `fn` **after_ms** `pub`
- **547** `fn` **timer_fired** `pub`
- **553** `fn` **cancel_timer** `pub`
- **561** `fn` **set_wrap_bounds** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **568** `fn` **wrap_bounds_for** `pub`
- **573** `fn` **remove_wrap_bounds** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **579** `fn` **ids_with_wrap** `pub`
- **585** `fn` **apply_wrap** `pub`<br>↳ used in: `engine/src/systems/gameplay.rs`
- **601** `fn` **attach_controller** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **609** `fn` **controller** `pub`<br>↳ used in: `engine-authoring/src/compile/scene.rs`, `engine-behavior/src/lib.rs`, `engine-game/src/components.rs`, `engine/src/systems/gameplay.rs`, `engine/src/systems/ship_controller.rs`
- **615** `fn` **with_controller** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/systems/ship_controller.rs`
- **629** `fn` **ids_with_controller** `pub`<br>↳ used in: `engine/src/systems/ship_controller.rs`
- **635** `fn` **remove_controller** `pub`
- **642** `fn` **emit_event** `pub`<br>↳ used in: `engine-game/src/collision.rs`
- **651** `fn` **poll_events** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine/src/game_loop.rs`
- **670** `fn` **clear_events** `pub`
- **677** `fn` **get** `pub`
- **686** `fn` **set** `pub`
- **697** `fn` **has** `pub`
- **708** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **719** `fn` **push** `pub`
- **732** `fn` **rand_i** `pub`
- **740** `fn` **rand_seed** `pub`
- **748** `fn` **set_world_bounds** `pub`
- **754** `fn` **world_bounds** `pub`
- **760** `fn` **enable_wrap_bounds** `pub`
- **771** `fn` **tag_add** `pub`
- **779** `fn` **tag_remove** `pub`
- **786** `fn` **tag_has** `pub`
- **792** `impl` **Default** — *impl Default for GameplayWorld*
- **927** `mod` **tests**

### `src/lib.rs`

- **9** `mod` **game_object** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/lib.rs`
- **10** `mod` **game_state** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/level_state.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`
- **11** `mod` **gameplay** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/strategy.rs`, `engine-physics/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/gameplay.rs`
- **12** `mod` **components** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/ui/components/sidebar/mod.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/obj_loader.rs`, `engine-core/src/effects/utils/color.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/systems/gameplay.rs`
- **13** `mod` **strategy** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-game/src/strategy.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **14** `mod` **collision** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-game/src/collision.rs`, `engine-physics/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/systems/collision.rs`

### `src/strategy.rs`

- **6** `struct` **GameplayStrategies** `pub`<br>↳ used in: `engine-game/src/lib.rs`, `engine/src/lib.rs`, `engine/src/systems/gameplay.rs`
- **10** `impl` **Default** — *impl Default for GameplayStrategies*
- **18** `trait` **PhysicsIntegrationStrategy** `pub`<br>↳ used in: `engine-game/src/lib.rs`
- **19** `fn` **step**
- **24** `struct` **SimpleEulerIntegration** `pub`<br>↳ used in: `engine-game/src/lib.rs`
- **26** `impl` **PhysicsIntegrationStrategy** — *impl PhysicsIntegrationStrategy for SimpleEulerIntegration*
- **27** `fn` **step**

---

## engine-io

### `src/lib.rs`

- **17** `enum` **IoRequest** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/main.rs`, `rust-os/src/protocol.rs`
- **48** `enum` **IoEvent** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/main.rs`, `rust-os/src/protocol.rs`, `rust-os/src/screen_buffer.rs`
- **87** `trait` **SidecarTransport** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **91** `fn` **try_drain_events**
- **93** `fn` **is_alive**
- **95** `fn` **kill**
- **99** `enum` **EngineIoError** `pub`
- **115** `struct` **SidecarProcess** `pub`
- **125** `struct` **TcpSidecar** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **131** `impl` **TcpSidecar** — *impl TcpSidecar*
- **132** `fn` **spawn** `pub`<br>↳ used in: `engine-core/src/level_state.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`
- **168** `fn` **send** `pub`
- **175** `fn` **try_drain_events** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **190** `fn` **is_alive** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **197** `fn` **kill** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/proc.rs`
- **204** `impl` **SidecarProcess** — *impl SidecarProcess*
- **205** `fn` **spawn** `pub`<br>↳ used in: `engine-core/src/level_state.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`
- **238** `fn` **send** `pub`
- **245** `fn` **try_drain_events** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **260** `fn` **is_alive** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **267** `fn` **kill** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/proc.rs`
- **274** `impl` **SidecarTransport** — *impl SidecarTransport for TcpSidecar*
- **278** `fn` **try_drain_events**
- **281** `fn` **is_alive**
- **284** `fn` **kill**
- **289** `impl` **SidecarTransport** — *impl SidecarTransport for SidecarProcess*
- **293** `fn` **try_drain_events**
- **296** `fn` **is_alive**
- **299** `fn` **kill**
- **306** `struct` **NullTransport** `pub`
- **308** `impl` **SidecarTransport** — *impl SidecarTransport for NullTransport*
- **312** `fn` **try_drain_events**
- **315** `fn` **is_alive**
- **318** `fn` **kill**
- **388** `mod` **tests**

---

## engine-mod

### `src/lib.rs`

- **6** `mod` **output_backend** `pub`<br>↳ used in: `app/src/main.rs`, `engine-authoring/src/schema/mod.rs`, `engine/src/lib.rs`
- **7** `mod` **startup** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/main.rs`, `editor/src/ui/icons.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-debug/src/log.rs`, `engine-error/src/lib.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/boot.rs`
- **8** `mod` **terminal_caps** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/game_loop.rs`
- **24** `fn` **load_mod_manifest** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`

### `src/output_backend.rs`

- **7** `enum` **StartupOutputSetting** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine/src/lib.rs`
- **13** `impl` **StartupOutputSetting** — *impl StartupOutputSetting*
- **15** `fn` **parse** `pub`
- **27** `fn` **from_manifest** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`

### `src/startup/check.rs`

- **9** `trait` **StartupCheck** `pub`<br>↳ used in: `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/mod.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`

### `src/startup/checks/action_map.rs`

- **20** `struct` **ActionMapCheck** `pub`<br>↳ used in: `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`
- **22** `impl` **StartupCheck** — *impl StartupCheck for ActionMapCheck*
- **128** `mod` **tests**

### `src/startup/checks/audio_sequencer.rs`

- **22** `struct` **AudioSequencerCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`
- **24** `impl` **StartupCheck** — *impl StartupCheck for AudioSequencerCheck*
- **142** `struct` **AudioYamlFile**

### `src/startup/checks/effect_registry.rs`

- **13** `struct` **EffectRegistryCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **15** `impl` **StartupCheck** — *impl StartupCheck for EffectRegistryCheck*
- **334** `mod` **tests**

### `src/startup/checks/font_glyph_coverage.rs`

- **16** `struct` **FontGlyphCoverageCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **18** `impl` **StartupCheck** — *impl StartupCheck for FontGlyphCoverageCheck*

### `src/startup/checks/font_manifest.rs`

- **15** `struct` **FontManifestCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **17** `impl` **StartupCheck** — *impl StartupCheck for FontManifestCheck*

### `src/startup/checks/image_assets.rs`

- **13** `struct` **ImageAssetsCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **15** `impl` **StartupCheck** — *impl StartupCheck for ImageAssetsCheck*

### `src/startup/checks/level_config.rs`

- **18** `struct` **LevelConfigCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`
- **20** `impl` **StartupCheck** — *impl StartupCheck for LevelConfigCheck*
- **86** `struct` **LevelAsset**
- **259** `mod` **tests**

### `src/startup/checks/mod.rs`

- **3** `mod` **action_map**
- **4** `mod` **audio_sequencer**
- **5** `mod` **effect_registry**
- **6** `mod` **font_glyph_coverage**
- **7** `mod` **font_manifest**
- **8** `mod` **image_assets**
- **9** `mod` **level_config**
- **10** `mod` **rhai_scripts**
- **11** `mod` **scene_graph**
- **12** `mod` **terminal_requirements**

### `src/startup/checks/rhai_scripts.rs`

- **12** `struct` **RhaiScriptsCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`
- **14** `impl` **StartupCheck** — *impl StartupCheck for RhaiScriptsCheck*
- **205** `mod` **tests**

### `src/startup/checks/scene_graph.rs`

- **12** `struct` **SceneGraphCheck** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **14** `impl` **StartupCheck** — *impl StartupCheck for SceneGraphCheck*
- **162** `mod` **tests**

### `src/startup/checks/terminal_requirements.rs`

- **25** `struct` **TerminalRequirementsCheck** `pub`<br>↳ used in: `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/runner.rs`
- **27** `impl` **StartupCheck** — *impl StartupCheck for TerminalRequirementsCheck*
- **150** `mod` **tests**

### `src/startup/context.rs`

- **19** `struct` **StartupSceneFile** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/mod.rs`, `engine/src/lib.rs`
- **25** `type` **SceneLoaderFn** `pub`
- **26** `type` **FontAssetCheckerFn** `pub`
- **27** `type` **GlyphCoverageCheckerFn** `pub`
- **28** `type` **ImageAssetCheckerFn** `pub`
- **29** `type` **RhaiScriptValidatorFn** `pub`
- **33** `struct` **StartupContext** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/mod.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **48** `fn` **new** `pub`
- **69** `fn` **with_selected_output** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **75** `fn` **with_font_asset_checker** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **81** `fn` **with_glyph_coverage_checker** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **87** `fn` **with_image_asset_checker** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **93** `fn` **with_rhai_script_validator** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine/src/lib.rs`
- **101** `fn` **mod_source** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/cli.rs`, `editor/src/io/indexer.rs`, `editor/src/main.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/mod.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scenes_preview.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/audio_sequencer.rs`, `engine/src/image_loader.rs`, `engine/src/level_state.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene_loader.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`
- **106** `fn` **manifest** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/asset_index.rs`, `editor/src/domain/mod.rs`, `editor/src/domain/mod_manifest.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `editor/src/state/scene_run.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/output_backend.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/terminal_caps.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/types.rs`, `engine-runtime/src/lib.rs`, `engine-terminal/src/lib.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/types.rs`, `engine/src/splash.rs`, `tools/schema-gen/src/main.rs`
- **111** `fn` **entrypoint** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/mod_manifest.rs`, `editor/src/io/fs_scan.rs`, `editor/src/io/indexer.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/start_screen.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-error/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `tools/devtool/src/scaffold.rs`
- **116** `fn` **selected_output** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`
- **121** `fn` **all_scenes** `pub`<br>↳ used in: `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`
- **134** `fn` **has_font_assets** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render/src/lib.rs`, `engine/src/lib.rs`
- **141** `fn` **font_missing_glyphs** `pub`<br>↳ used in: `engine-mod/src/startup/checks/font_glyph_coverage.rs`
- **148** `fn` **has_image_asset** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine/src/lib.rs`
- **155** `fn` **validate_rhai_script** `pub`<br>↳ used in: `engine-mod/src/startup/checks/rhai_scripts.rs`

### `src/startup/mod.rs`

- **3** `mod` **check** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/obj_prerender.rs`, `engine-animation/src/systems.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-core/src/logging.rs`, `engine-error/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine-render/src/image_loader.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/game_loop.rs`, `engine/src/image_loader.rs`, `engine/src/lib.rs`, `engine/src/systems/behavior.rs`, `rust-os/src/boot.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`
- **4** `mod` **checks** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/mod.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/lib.rs`
- **5** `mod` **context** `pub`<br>↳ used in: `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/status_bar.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/render/common.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/context.rs`, `engine-mod/src/startup/runner.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/gpu/context.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/main.rs`
- **6** `mod` **report** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`
- **7** `mod` **runner** `pub`<br>↳ used in: `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/runner.rs`

### `src/startup/report.rs`

- **5** `enum` **StartupIssueLevel** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/mod.rs`, `engine/src/lib.rs`
- **12** `struct` **StartupIssue** `pub`<br>↳ used in: `engine-mod/src/startup/mod.rs`
- **20** `struct` **StartupReport** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-mod/src/startup/mod.rs`, `engine-mod/src/startup/runner.rs`
- **24** `impl` **StartupReport** — *impl StartupReport*
- **26** `fn` **add_info** `pub`<br>↳ used in: `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`
- **35** `fn` **add_warning** `pub`<br>↳ used in: `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`
- **44** `fn` **issues** `pub`<br>↳ used in: `app/src/main.rs`, `engine-authoring/src/validate/mod.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`

### `src/startup/runner.rs`

- **15** `struct` **StartupRunner** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/check.rs`, `engine-mod/src/startup/mod.rs`, `engine/src/lib.rs`
- **19** `impl` **StartupRunner** — *impl StartupRunner*
- **21** `fn` **with_checks** `pub`<br>↳ used in: `app/src/main.rs`, `engine/src/lib.rs`
- **26** `fn` **run** `pub`
- **35** `impl` **Default** — *impl Default for StartupRunner*

### `src/terminal_caps.rs`

- **6** `struct` **TerminalCaps** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`
- **17** `struct` **TerminalRequirements** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **24** `const` **DEFAULT_TARGET_FPS** `pub`
- **25** `const` **MAX_TARGET_FPS** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **29** `struct` **TerminalViolation** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`
- **35** `impl` **TerminalCaps** — *impl TerminalCaps*
- **37** `fn` **detect** `pub`<br>↳ used in: `engine-core/src/buffer.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`
- **49** `fn` **validate** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/document/scene.rs`
- **86** `impl` **TerminalRequirements** — *impl TerminalRequirements*
- **89** `fn` **from_manifest** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **114** `fn` **target_fps_from_manifest** `pub`<br>↳ used in: `engine/src/lib.rs`
- **151** `mod` **tests**

---

## engine-persistence

### `src/lib.rs`

- **15** `struct` **PersistenceStore** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/lib.rs`, `engine/src/systems/behavior.rs`
- **20** `impl` **PersistenceStore** — *impl PersistenceStore*
- **22** `fn` **new** `pub`
- **27** `fn` **from_root** `pub`
- **38** `fn` **file_path** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/io/fs_scan.rs`, `editor/src/main.rs`, `editor/src/state/editor_pane.rs`, `engine-core/src/logging.rs`
- **43** `fn` **get** `pub`
- **49** `fn` **has** `pub`
- **57** `fn` **set** `pub`
- **69** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **81** `fn` **push** `pub`
- **93** `fn` **snapshot** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/profiling.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/prepared_frame.rs`, `engine/src/systems/scene_lifecycle.rs`
- **98** `fn` **clear** `pub`
- **108** `fn` **reload** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine/src/systems/hot_reload.rs`
- **120** `impl` **Default** — *impl Default for PersistenceStore*
- **267** `mod` **tests**

---

## engine-physics

### `src/lib.rs`

- **12** `fn` **point_in_polygon** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **21** `fn` **polygons_intersect** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **38** `fn` **segment_intersects_polygon** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **78** `mod` **tests**

---

## engine-pipeline

### `src/lib.rs`

- **7** `mod` **strategies** `pub`<br>↳ used in: `engine-core/src/strategy/mod.rs`, `engine-game/src/collision.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/strategy/flush_trait.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/collision.rs`, `engine/src/systems/gameplay.rs`
- **36** `struct` **PipelineFlags** `pub`<br>↳ used in: `engine/src/lib.rs`
- **97** `impl` **Default** — *impl Default for PipelineFlags*

### `src/strategies/display.rs`

- **6** `struct` **DisplayFrame** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine/src/strategy/mod.rs`
- **17** `trait` **DisplaySink** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/strategy/mod.rs`
- **21** `fn` **drain**

### `src/strategies/halfblock.rs`

- **4** `trait` **HalfblockPacker** `pub`<br>↳ used in: `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **16** `struct` **FullScanPacker** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **18** `impl` **HalfblockPacker** — *impl HalfblockPacker for FullScanPacker*
- **42** `struct` **DirtyRegionPacker** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`
- **44** `impl` **HalfblockPacker** — *impl HalfblockPacker for DirtyRegionPacker*

### `src/strategies/layer.rs`

- **3** `trait` **LayerCompositor** `pub`<br>↳ used in: `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **8** `struct` **ScratchLayerCompositor** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **10** `impl` **LayerCompositor** — *impl LayerCompositor for ScratchLayerCompositor*
- **19** `struct` **DirectLayerCompositor** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`
- **21** `impl` **LayerCompositor** — *impl LayerCompositor for DirectLayerCompositor*

### `src/strategies/mod.rs`

- **2** `mod` **display** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effect_params.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/main.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/status_bar.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_render.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/lib.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/lib.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/session.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **4** `mod` **halfblock** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-core/src/buffer.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`, `engine-runtime/src/lib.rs`, `engine/src/lib.rs`, `engine/src/systems/compositor/mod.rs`
- **6** `mod` **layer** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_loader.rs`, `engine/src/splash.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/kernel/disk.rs`, `rust-os/src/style.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **8** `mod` **present** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-frame/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/present.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/access.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/systems/scene_lifecycle.rs`
- **10** `mod` **skip** `pub`<br>↳ used in: `app/src/main.rs`, `engine-animation/src/systems.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/logging.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/strategy/diff.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/tokenizer.rs`
- **24** `struct` **PipelineStrategies** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **31** `impl` **PipelineStrategies** — *impl PipelineStrategies*
- **33** `fn` **new** `pub`
- **50** `fn` **from_flags** `pub`<br>↳ used in: `engine/src/lib.rs`

### `src/strategies/present.rs`

- **3** `trait` **VirtualPresenter** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`
- **8** `struct` **AlwaysPresenter** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`
- **10** `impl` **VirtualPresenter** — *impl VirtualPresenter for AlwaysPresenter*
- **21** `struct` **HashSkipPresenter** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/strategy/mod.rs`
- **23** `impl` **VirtualPresenter** — *impl VirtualPresenter for HashSkipPresenter*

### `src/strategies/skip.rs`

- **11** `trait` **FrameSkipOracle** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/mod.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/provider.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`
- **23** `struct` **AlwaysRender** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/lib.rs`, `engine/src/strategy/mod.rs`
- **25** `impl` **FrameSkipOracle** — *impl FrameSkipOracle for AlwaysRender*
- **46** `struct` **CoordinatedSkip** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine/src/lib.rs`, `engine/src/strategy/mod.rs`
- **56** `impl` **Default** — *impl Default for CoordinatedSkip*
- **69** `impl` **FrameSkipOracle** — *impl FrameSkipOracle for CoordinatedSkip*

---

## engine-render

### `src/font_loader.rs`

- **12** `fn` **load_font_assets** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **104** `struct` **FontManifestCandidate**
- **110** `impl` **FontManifestCandidate** — *impl FontManifestCandidate*
- **248** `mod` **tests**

### `src/generic.rs`

- **7** `const` **ENGINE_FALLBACK_FONT_SPEC** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **12** `const` **GENERIC_TINY_GLYPH_WIDTH** `pub`
- **13** `const` **GENERIC_TINY_GLYPH_HEIGHT** `pub`
- **14** `const` **GENERIC_TINY_GLYPH_GAP** `pub`
- **25** `fn` **generic_glyph_rows** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`
- **303** `fn` **rasterize_generic** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **347** `fn` **generic_dimensions** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **366** `fn` **generic_glyph_rows_tiny** `pub`
- **450** `fn` **rasterize_generic_tiny** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **488** `fn` **generic_dimensions_tiny** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **506** `fn` **span_width** `pub`
- **512** `fn` **span_width_tiny** `pub`
- **517** `enum` **GenericMode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/mod.rs`
- **526** `impl` **GenericMode** — *impl GenericMode*
- **527** `fn` **from_font_name** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/mod.rs`
- **545** `fn` **generic_dimensions_mode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **611** `fn` **rasterize_generic_half** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **655** `fn` **rasterize_generic_quad** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **697** `fn` **rasterize_generic_braille** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **759** `fn` **rasterize_spans_mode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`
- **787** `fn` **rasterize_spans** `pub`
- **804** `mod` **tests**

### `src/image_loader.rs`

- **17** `struct` **LoadedRgbaImage** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **23** `impl` **LoadedRgbaImage** — *impl LoadedRgbaImage*
- **25** `fn` **pixel** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effects_preview_scene.rs`, `engine-3d/src/obj_frame_cache.rs`, `engine-3d/src/obj_prerender.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/utils/noise.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/bitmap_font.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/vector_overlay.rs`, `engine-runtime/src/lib.rs`, `engine/src/lib.rs`, `engine/src/obj_frame_cache.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/splash.rs`
- **33** `fn` **from_pixels** `pub`
- **43** `struct` **LoadedAnimatedImageFrame** `pub`
- **49** `struct` **LoadedAnimatedImage** `pub`
- **56** `impl` **LoadedAnimatedImage** — *impl LoadedAnimatedImage*
- **57** `fn` **first_frame** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **61** `fn` **frame_at** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **78** `enum` **LoadedImageAsset** `pub`
- **83** `impl` **LoadedImageAsset** — *impl LoadedImageAsset*
- **84** `fn` **first_frame** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **91** `fn` **frame_at** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **98** `fn` **is_animated** `pub`
- **105** `struct` **ImageAssetAdapter**
- **107** `impl` **SourceAdapter** — *impl SourceAdapter<LoadedImageAsset> for ImageAssetAdapter*
- **199** `fn` **load_image_asset** `pub`<br>↳ used in: `engine-compositor/src/image_render.rs`
- **209** `fn` **load_rgba_image** `pub`
- **214** `fn` **has_image_asset** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine/src/lib.rs`
- **224** `mod` **tests**

### `src/lib.rs`

- **13** `mod` **font_loader**
- **14** `mod` **generic** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-animation/src/provider.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/game_state.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene3d_resolve.rs`, `tools/schema-gen/src/main.rs`
- **15** `mod` **image_loader** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/image_render.rs`
- **16** `mod` **overlay** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/state/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/model.rs`, `engine-debug/src/lib.rs`, `engine-debug/src/log.rs`, `engine-render-sdl2/src/bitmap_font.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/overlay.rs`, `engine-render/src/vector_overlay.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **17** `mod` **rasterizer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **18** `mod` **simd_text** `pub`
- **19** `mod` **types**
- **20** `mod` **vector_overlay** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **30** `enum` **RenderError** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/renderer.rs`
- **43** `enum` **ColorDepth** `pub`
- **52** `enum` **PresentMode** `pub`<br>↳ used in: `engine/src/gpu/context.rs`
- **63** `struct` **RenderCaps** `pub`
- **72** `struct` **RenderFrame** `pub`
- **83** `trait` **RenderBackend** `pub`
- **86** `fn` **present**
- **92** `fn` **shutdown**
- **99** `trait` **OutputBackend** `pub`<br>↳ used in: `engine-render-terminal/src/provider.rs`, `engine/src/lib.rs`, `engine/src/services.rs`, `engine/src/splash.rs`
- **117** `fn` **shutdown**
- **121** `trait` **DisplaySink** `pub`<br>↳ used in: `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/strategy/mod.rs`
- **126** `fn` **flush**
- **129** `fn` **drain**
- **133** `mod` **tests**

### `src/overlay.rs`

- **14** `struct` **OverlayLine** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`
- **21** `impl` **OverlayLine** — *impl OverlayLine*
- **22** `fn` **new** `pub`
- **31** `fn` **with_alpha** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`
- **46** `struct` **OverlayData** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`
- **51** `impl` **OverlayData** — *impl OverlayData*
- **52** `fn` **is_empty** `pub`

### `src/rasterizer.rs`

- **48** `fn` **rasterize_cached** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/lib.rs`
- **72** `fn` **rasterize** `pub`<br>↳ used in: `engine-compositor/src/obj_render.rs`, `engine-render/src/lib.rs`
- **159** `fn` **has_font_assets** `pub`<br>↳ used in: `app/src/main.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render/src/lib.rs`, `engine/src/lib.rs`
- **168** `fn` **missing_glyphs** `pub`<br>↳ used in: `app/src/main.rs`, `engine-render/src/lib.rs`, `engine/src/lib.rs`
- **219** `fn` **blit** `pub`<br>↳ used in: `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render/src/lib.rs`
- **230** `mod` **tests**

### `src/simd_text.rs`

- **19** `struct` **GlyphBatch** `pub`<br>↳ used in: `engine-render/src/lib.rs`
- **32** `impl` **GlyphBatch** — *impl GlyphBatch*
- **35** `fn` **with_capacity** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/buffer_pool.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/scene/template.rs`, `engine-debug/src/log.rs`, `engine-debug/src/profiling.rs`, `engine-persistence/src/lib.rs`, `engine-physics/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/image_loader.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/bench.rs`, `engine/src/gpu/render.rs`, `engine/src/image_loader.rs`, `engine/src/lib.rs`, `rust-os/src/style.rs`
- **47** `fn` **reset** `pub`<br>↳ used in: `engine-animation/src/animator.rs`, `engine-core/src/buffer.rs`, `engine-core/src/color.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/strategy/diff.rs`, `engine-game/src/gameplay.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/skip.rs`
- **57** `fn` **len** `pub`
- **63** `fn` **is_empty** `pub`
- **70** `fn` **stage_glyph_placement** `pub`<br>↳ used in: `engine-render/src/lib.rs`
- **99** `fn` **rasterize_staged_glyphs** `pub`<br>↳ used in: `engine-render/src/lib.rs`
- **299** `mod` **tests**

### `src/types.rs`

- **8** `struct` **GlyphManifest** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/font_loader.rs`
- **14** `struct` **ManifestGlyph** `pub`
- **23** `struct` **LoadedGlyph** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/font_loader.rs`
- **31** `struct` **LoadedFont** `pub`<br>↳ used in: `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/font_loader.rs`
- **36** `impl` **LoadedFont** — *impl LoadedFont*
- **38** `fn` **advance_and_height** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/rasterizer/mod.rs`

### `src/vector_overlay.rs`

- **9** `struct` **VectorPrimitive** `pub`<br>↳ used in: `engine-compositor/src/lib.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-render/src/lib.rs`
- **22** `struct` **VectorOverlay** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/lib.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`
- **28** `impl` **VectorOverlay** — *impl VectorOverlay*
- **29** `fn` **is_empty** `pub`

---

## engine-render-policy

### `src/lib.rs`

- **8** `const` **ENGINE_DEFAULT_FONT_SPEC** `pub`
- **12** `fn` **resolve_renderer_mode** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`
- **22** `fn` **resolve_font_spec** `pub`
- **77** `fn` **resolve_text_font_spec** `pub`<br>↳ used in: `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`
- **150** `mod` **tests**

---

## engine-render-sdl2

### `src/bitmap_font.rs`

- **9** `const` **GLYPH_WIDTH** `pub`
- **10** `const` **GLYPH_HEIGHT** `pub`
- **14** `fn` **glyph** `pub`<br>↳ used in: `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/icons.rs`, `engine-asset/src/repositories.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/buffer.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/types.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/rasterizer/types.rs`
- **504** `mod` **tests**

### `src/color_convert.rs`

- **3** `fn` **to_sdl_color** `pub`

### `src/input.rs`

- **7** `struct` **Sdl2InputBackend** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`
- **11** `impl` **Sdl2InputBackend** — *impl Sdl2InputBackend*
- **12** `fn` **from_client** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`
- **17** `impl` **InputBackend** — *impl InputBackend for Sdl2InputBackend*
- **18** `fn` **poll_events**
- **36** `mod` **tests**

### `src/lib.rs`

- **1** `mod` **bitmap_font** `pub`
- **2** `mod` **color_convert** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`
- **3** `mod` **input** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-animation/src/menu.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/model.rs`, `engine-events/src/input_backend.rs`, `engine-events/src/lib.rs`, `engine-game/src/components.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/input.rs`, `engine-render/src/font_loader.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/ship_controller.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`
- **4** `mod` **renderer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/log.rs`, `engine-pipeline/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`
- **5** `mod` **runtime**

### `src/renderer.rs`

- **11** `const` **DEFAULT_PIXEL_SCALE** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`
- **12** `const` **LOGICAL_CELL_WIDTH** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`
- **13** `const` **LOGICAL_CELL_HEIGHT** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`
- **15** `struct` **Sdl2Backend** `pub`<br>↳ used in: `engine/src/lib.rs`
- **24** `struct` **BackendProfile**
- **34** `impl` **BackendProfile** — *impl BackendProfile*
- **83** `impl` **Sdl2Backend** — *impl Sdl2Backend*
- **84** `fn` **new_pair** `pub`<br>↳ used in: `engine/src/lib.rs`
- **117** `fn` **request**
- **129** `fn` **set_splash_mode** `pub`<br>↳ used in: `engine/src/lib.rs`
- **136** `impl` **OutputBackend** — *impl OutputBackend for Sdl2Backend*
- **183** `fn` **shutdown**
- **191** `impl` **Drop** — *impl Drop for Sdl2Backend*
- **198** `mod` **tests**

### `src/runtime.rs`

- **18** `type` **CellPatch** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`
- **20** `enum` **RuntimeCommand** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/renderer.rs`
- **34** `enum` **RuntimeResponse** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/renderer.rs`
- **39** `struct` **Sdl2RuntimeClient** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/renderer.rs`
- **44** `fn` **sdl_profile_enabled** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`
- **59** `struct` **RuntimeProfile**
- **71** `impl` **RuntimeProfile** — *impl RuntimeProfile*
- **144** `impl` **Sdl2RuntimeClient** — *impl Sdl2RuntimeClient*
- **145** `fn` **spawn** `pub`<br>↳ used in: `engine-core/src/level_state.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/splash.rs`, `engine/src/systems/engine_io.rs`
- **179** `fn` **disconnected_for_tests** `pub`<br>↳ used in: `engine-render-sdl2/src/renderer.rs`
- **188** `fn` **request** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-io/src/lib.rs`, `engine-render-sdl2/src/input.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `rust-os/src/hosts/registry.rs`, `tools/sound-server/src/main.rs`
- **469** `struct` **DirtyPixelRect**
- **477** `impl` **DirtyPixelRect** — *impl DirtyPixelRect*
- **844** `fn` **map_keycode** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`
- **883** `fn` **map_modifiers** `pub`<br>↳ used in: `engine-render-sdl2/src/input.rs`
- **1119** `mod` **tests**

---

## engine-render-terminal

### `src/color_convert.rs`

- **42** `fn` **to_crossterm** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`
- **75** `fn` **from_crossterm** `pub`<br>↳ used in: `engine/src/splash.rs`
- **100** `fn` **diffs_to_crossterm** `pub`
- **110** `mod` **tests**

### `src/input.rs`

- **7** `struct` **TerminalInputBackend** `pub`<br>↳ used in: `engine/src/lib.rs`
- **9** `impl` **TerminalInputBackend** — *impl TerminalInputBackend*
- **10** `fn` **new** `pub`
- **15** `impl` **InputBackend** — *impl InputBackend for TerminalInputBackend*
- **16** `fn` **poll_events**
- **57** `fn` **crossterm_key_to_engine** `pub`<br>↳ used in: `editor/src/app.rs`
- **98** `fn` **is_debug_fast_forward_toggle** `pub`<br>↳ used in: `engine/src/game_loop.rs`

### `src/lib.rs`

- **6** `mod` **color_convert** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`
- **7** `mod` **input** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-animation/src/menu.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/document/mod.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/layout/tracks.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/model.rs`, `engine-events/src/input_backend.rs`, `engine-events/src/lib.rs`, `engine-game/src/components.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-terminal/src/input.rs`, `engine-render/src/font_loader.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/ship_controller.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`
- **8** `mod` **provider** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-audio/src/systems_audio.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/services.rs`, `engine/src/world.rs`
- **9** `mod` **rasterizer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/scene/sprite.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **10** `mod` **renderer** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_format.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/log.rs`, `engine-pipeline/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-runtime/src/lib.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`
- **11** `mod` **strategy** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/lib.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/buffer.rs`, `engine-game/src/strategy.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/compositor/mod.rs`

### `src/provider.rs`

- **11** `trait` **RendererProvider** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/renderer.rs`
- **12** `fn` **buffer**
- **14** `fn` **runtime_settings**
- **15** `fn` **debug_features**
- **16** `fn` **debug_log**
- **17** `fn` **animator**
- **27** `fn` **restore_front_to_back**
- **29** `fn` **vector_overlay**

### `src/rasterizer/generic.rs`

- **6** `const` **GENERIC_TINY_GLYPH_WIDTH** `pub`
- **7** `const` **GENERIC_TINY_GLYPH_HEIGHT** `pub`
- **8** `const` **GENERIC_TINY_GLYPH_GAP** `pub`
- **19** `fn` **generic_glyph_rows** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`
- **297** `fn` **rasterize_generic** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **341** `fn` **generic_dimensions** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`
- **360** `fn` **generic_glyph_rows_tiny** `pub`
- **444** `fn` **rasterize_generic_tiny** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **482** `fn` **generic_dimensions_tiny** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **500** `fn` **span_width** `pub`
- **506** `fn` **span_width_tiny** `pub`
- **511** `enum` **GenericMode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/mod.rs`
- **520** `impl` **GenericMode** — *impl GenericMode*
- **521** `fn` **from_font_name** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/rasterizer/mod.rs`
- **539** `fn` **generic_dimensions_mode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **605** `fn` **rasterize_generic_half** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **649** `fn` **rasterize_generic_quad** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **691** `fn` **rasterize_generic_braille** `pub`<br>↳ used in: `engine-render/src/rasterizer.rs`, `engine/src/rasterizer/mod.rs`
- **753** `fn` **rasterize_spans_mode** `pub`<br>↳ used in: `engine-compositor/src/text_render.rs`
- **781** `fn` **rasterize_spans** `pub`
- **798** `mod` **tests**

### `src/rasterizer/mod.rs`

- **1** `mod` **generic** `pub`<br>↳ used in: `engine-3d/src/scene3d_resolve.rs`, `engine-animation/src/provider.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/game_state.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/bench.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene3d_resolve.rs`, `tools/schema-gen/src/main.rs`

### `src/renderer.rs`

- **17** `struct` **TerminalRenderer** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine/src/lib.rs`
- **35** `impl` **TerminalRenderer** — *impl TerminalRenderer*
- **36** `fn` **new** `pub`
- **40** `fn` **new_with_async** `pub`<br>↳ used in: `engine/src/lib.rs`
- **67** `fn` **clear_black** `pub`
- **86** `fn` **reset_console** `pub`
- **96** `fn` **shutdown** `pub`<br>↳ used in: `engine-core/src/effects/builtin/power_off.rs`, `engine-pipeline/src/strategies/display.rs`, `engine/src/lib.rs`, `rust-os/src/exec/pipeline.rs`, `tools/sound-server/src/main.rs`
- **110** `impl` **OutputBackend** — *impl OutputBackend for TerminalRenderer*
- **158** `fn` **shutdown**
- **163** `impl` **Drop** — *impl Drop for TerminalRenderer*
- **169** `impl` **TerminalRenderer** — *impl TerminalRenderer*
- **263** `fn` **renderer_system** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine/src/game_loop.rs`, `engine/src/prepared_frame.rs`
- **641** `struct` **Viewport**
- **729** `fn` **resolve_color** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/strategy/flush.rs`
- **749** `fn` **flush_batched** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render-terminal/src/strategy/flush.rs`
- **796** `macro_rules!` **emit_run**
- **856** `mod` **tests**

### `src/strategy/display.rs`

- **7** `struct` **SyncDisplaySink** `pub`<br>↳ used in: `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/strategy/mod.rs`, `engine/src/strategy/mod.rs`
- **9** `impl` **DisplaySink** — *impl DisplaySink for SyncDisplaySink*
- **17** `fn` **drain**
- **22** `struct` **AsyncDisplaySink** `pub`<br>↳ used in: `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/mod.rs`, `engine/src/strategy/mod.rs`
- **27** `impl` **AsyncDisplaySink** — *impl AsyncDisplaySink*
- **28** `fn` **new** `pub`
- **45** `impl` **DisplaySink** — *impl DisplaySink for AsyncDisplaySink*
- **52** `fn` **drain**
- **60** `impl` **Default** — *impl Default for AsyncDisplaySink*

### `src/strategy/flush.rs`

- **6** `struct` **NaiveFlusher** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/strategy/mod.rs`, `engine/src/strategy/mod.rs`
- **8** `impl` **TerminalFlusher** — *impl TerminalFlusher for NaiveFlusher*
- **9** `fn` **flush**
- **32** `struct` **AnsiBatchFlusher** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/mod.rs`, `engine/src/lib.rs`, `engine/src/strategy/mod.rs`
- **34** `impl` **TerminalFlusher** — *impl TerminalFlusher for AnsiBatchFlusher*
- **35** `fn` **flush**

### `src/strategy/flush_trait.rs`

- **10** `trait` **TerminalFlusher** `pub`<br>↳ used in: `engine-render-terminal/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/mod.rs`, `engine/src/strategy/mod.rs`
- **11** `fn` **flush**

### `src/strategy/mod.rs`

- **1** `mod` **display** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effect_params.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/main.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/status_bar.rs`, `engine-asset/src/source_loader.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_render.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/lib.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/strategy/display.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/lib.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/session.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/devtool/src/schema.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **2** `mod` **flush** `pub`<br>↳ used in: `app/src/main.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/buffer.rs`, `engine-core/src/logging.rs`, `engine-io/src/lib.rs`, `engine-persistence/src/lib.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/display.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/bench.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`, `engine/src/strategy/mod.rs`, `rust-os/src/main.rs`, `tools/sound-server/src/main.rs`
- **3** `mod` **flush_trait** `pub`<br>↳ used in: `engine-render-terminal/src/strategy/flush.rs`

---

## engine-runtime

### `src/access.rs`

- **7** `trait` **RuntimeAccess** `pub`
- **8** `fn` **runtime_settings**
- **11** `impl` **RuntimeAccess** — *impl RuntimeAccess for World*
- **12** `fn` **runtime_settings**

### `src/lib.rs`

- **1** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **9** `enum` **PresentationPolicy** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`
- **15** `type` **VirtualPolicy** `pub`
- **17** `impl` **Default** — *impl Default for PresentationPolicy*
- **24** `enum` **RenderSize** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine/src/lib.rs`, `engine/src/runtime_settings.rs`, `engine/src/systems/scene_lifecycle.rs`
- **29** `impl` **Default** — *impl Default for RenderSize*
- **38** `impl` **RenderSize** — *impl RenderSize*
- **39** `fn` **resolve** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/compile/scene.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/services.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/scaffold.rs`
- **46** `fn` **matches_output** `pub`<br>↳ used in: `engine/src/runtime_settings.rs`
- **50** `fn` **fixed** `pub`<br>↳ used in: `engine-authoring/src/package/mod.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/scene/metadata.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`
- **59** `struct` **BufferLayout** `pub`<br>↳ used in: `engine/src/runtime_settings.rs`
- **67** `struct` **RuntimeSettings** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-compositor/src/access.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-terminal/src/provider.rs`, `engine-render-terminal/src/renderer.rs`, `engine-runtime/src/access.rs`, `engine/src/lib.rs`, `engine/src/runtime_settings.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`
- **80** `struct` **PresentationLayout** `pub`
- **89** `impl` **Default** — *impl Default for RuntimeSettings*
- **101** `impl` **RuntimeSettings** — *impl RuntimeSettings*
- **102** `fn` **from_manifest** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **187** `fn` **resolved_render_size** `pub`
- **191** `fn` **buffer_layout** `pub`<br>↳ used in: `engine/src/runtime_settings.rs`
- **203** `fn` **render_size_matches_output** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/systems/scene_lifecycle.rs`
- **207** `fn` **fixed_render_size** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine-render-terminal/src/renderer.rs`
- **211** `fn` **resolved_virtual_size** `pub`
- **216** `fn` **compute_presentation_layout** `pub`<br>↳ used in: `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/renderer.rs`
- **339** `fn` **parse_render_size_str** `pub`
- **347** `fn` **parse_virtual_size_str** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`, `engine/src/runtime_settings.rs`
- **352** `mod` **tests**

---

## engine-scene-runtime

### `src/access.rs`

- **16** `trait` **SceneRuntimeAccess** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine/src/services.rs`
- **18** `fn` **scene_runtime**
- **25** `impl` **SceneRuntimeAccess** — *impl SceneRuntimeAccess for dyn Any*
- **26** `fn` **scene_runtime**

### `src/behavior_runner.rs`

- **13** `impl` **SceneRuntime** — *impl SceneRuntime*
- **16** `fn` **update_behaviors** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **269** `fn` **apply_behavior_commands** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`
- **555** `fn` **attach_default_behaviors** `pub`<br>↳ used in: `engine-scene-runtime/src/construction.rs`
- **564** `fn` **attach_declared_behaviors** `pub`<br>↳ used in: `engine-scene-runtime/src/construction.rs`
- **617** `fn` **has_pending_bindings** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **627** `fn` **apply_mod_behavior_registry** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **646** `impl` **SceneRuntime** — *impl SceneRuntime*

### `src/camera_3d.rs`

- **3** `impl` **SceneRuntime** — *impl SceneRuntime*
- **4** `fn` **adjust_obj_scale** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`
- **22** `fn` **toggle_obj_surface_mode** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`
- **51** `fn` **toggle_obj_orbit** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`
- **82** `fn` **is_obj_orbit_active** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **92** `fn` **apply_obj_camera_pan** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **103** `fn` **apply_obj_camera_look** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **113** `fn` **obj_camera_state** `pub`
- **120** `fn` **set_obj_last_mouse_pos** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **129** `fn` **obj_last_mouse_pos** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`

### `src/construction.rs`

- **3** `impl` **SceneRuntime** — *impl SceneRuntime*
- **11** `fn` **new** `pub`

### `src/lib.rs`

- **17** `mod` **access** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-animation/src/access.rs`, `engine-animation/src/provider.rs`, `engine-animation/src/systems.rs`, `engine-audio/src/access.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/provider.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-events/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/image_loader.rs`, `engine-runtime/src/access.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/image_loader.rs`, `engine/src/services.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/world.rs`, `rust-os/src/commands/net.rs`
- **18** `mod` **behavior_runner** `pub`
- **19** `mod` **camera_3d** `pub`
- **20** `mod` **construction** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **21** `mod` **lifecycle_controls** `pub`
- **22** `mod` **materialization** `pub`<br>↳ used in: `engine-authoring/src/document/object.rs`, `engine-core/src/scene_runtime_types.rs`
- **23** `mod` **object_graph** `pub`
- **24** `mod` **terminal_shell** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-scene-runtime/src/construction.rs`
- **25** `mod` **ui_focus** `pub`
- **56** `struct` **SceneRuntime** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-scene-runtime/src/access.rs`, `engine/src/lib.rs`, `engine/src/prepared_frame.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`
- **94** `struct` **TerminalShellState**
- **107** `struct` **PanelLayoutSpec**
- **115** `struct` **TextLayoutSpec**
- **122** `struct` **ObjSpritePropertySnapshot**
- **136** `struct` **UiTextEvent**
- **142** `struct` **UiRuntimeState**
- **156** `struct` **ObjectBehaviorRuntime**
- **161** `struct` **BehaviorBinding**
- **167** `mod` **tests**

### `src/lifecycle_controls.rs`

- **13** `enum` **TerminalShellRoute** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine/src/systems/scene_lifecycle.rs`
- **20** `impl` **SceneRuntime** — *impl SceneRuntime*
- **21** `fn` **terminal_size_presets** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **37** `fn` **handle_terminal_shell_lifecycle_keys** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **53** `fn` **apply_obj_viewer_key_presses** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **123** `fn` **apply_obj_viewer_mouse_moves** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`

### `src/materialization.rs`

- **3** `impl` **SceneRuntime** — *impl SceneRuntime*
- **4** `fn` **text_sprite_content** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **13** `fn` **object_text_snapshot** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **35** `fn` **object_props_snapshot** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **114** `fn` **set_text_sprite_content** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`
- **126** `fn` **set_text_sprite_font** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **134** `fn` **set_text_sprite_fg_colour** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **146** `fn` **set_text_sprite_bg_colour** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **158** `fn` **set_obj_sprite_property** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **171** `fn` **set_vector_sprite_property** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **184** `fn` **set_scene3d_sprite_frame** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **192** `fn` **set_image_sprite_frame_index** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **209** `fn` **object_alias_candidates** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **222** `fn` **apply_text_property_for_target** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **258** `fn` **find_text_layout_recursive** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/ui_focus.rs`
- **710** `fn` **parse_term_colour** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **759** `fn` **json_value_to_f32** `pub`

### `src/object_graph.rs`

- **14** `impl` **SceneRuntime** — *impl SceneRuntime*
- **16** `fn` **scene** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/effect_params.rs`, `editor/src/domain/effects_catalog.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/domain/scene_index.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/lib.rs`, `engine-3d/src/obj_prerender.rs`, `engine-3d/src/scene3d_atlas.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-animation/src/animator.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/menu.rs`, `engine-animation/src/provider.rs`, `engine-asset/src/lib.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/document/value.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/image_render.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/area.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/obj_prerender.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/systems/postfx/pass_ruby_crt.rs`, `engine-compositor/src/systems/postfx/pass_scan_glitch.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-compositor/src/text_render.rs`, `engine-compositor/src/warmup.rs`, `engine-core/src/access.rs`, `engine-core/src/animations/animation.rs`, `engine-core/src/animations/builtin/float.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/authoring/mod.rs`, `engine-core/src/effects/builtin/artifact.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_burn_in.rs`, `engine-core/src/effects/builtin/crt_distort.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/crt_reflection.rs`, `engine-core/src/effects/builtin/crt_ruby.rs`, `engine-core/src/effects/builtin/crt_scan_glitch.rs`, `engine-core/src/effects/builtin/crt_underlay.rs`, `engine-core/src/effects/builtin/cutout.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/neon_edge_glow.rs`, `engine-core/src/effects/builtin/posterize.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/effects/builtin/whiteout.rs`, `engine-core/src/effects/effect.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_object.rs`, `engine-core/src/game_state.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-debug/src/log.rs`, `engine-error/src/lib.rs`, `engine-events/src/lib.rs`, `engine-frame/src/lib.rs`, `engine-game/src/game_object.rs`, `engine-game/src/game_state.rs`, `engine-game/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-mod/src/startup/context.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/halfblock.rs`, `engine-pipeline/src/strategies/layer.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/renderer.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/game_loop.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/runtime_settings.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/scene_pipeline.rs`, `engine/src/splash.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/hot_reload.rs`, `engine/src/systems/mod.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_sync.rs`, `engine/src/systems/warmup.rs`, `tools/devtool/src/cli.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **20** `fn` **set_scene_rendered_mode** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`
- **25** `fn` **root_id** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`
- **30** `fn` **behavior_count** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **35** `fn` **object** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/factory.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/game_object.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-game/src/game_object.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/prepared_frame.rs`, `engine/src/scene3d_format.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/visual_sync.rs`, `tools/schema-gen/src/main.rs`
- **40** `fn` **objects** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-asset/src/scene_compiler.rs`, `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/repository/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-core/src/animations/params.rs`, `engine-core/src/game_state.rs`, `engine-core/src/scene/model.rs`, `engine-game/src/game_state.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/gpu/context.rs`, `engine/src/scene3d_format.rs`, `engine/src/systems/hot_reload.rs`, `tools/schema-gen/src/main.rs`
- **44** `fn` **object_count** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`
- **49** `fn` **object_state** `pub`<br>↳ used in: `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/lib.rs`
- **53** `fn` **object_states_snapshot** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **67** `fn` **object_kind_snapshot** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **71** `fn` **obj_camera_states_snapshot** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **84** `fn` **effective_object_state** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`
- **112** `fn` **effective_object_states_snapshot** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **139** `fn` **target_resolver** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/render/containers.rs`, `engine-compositor/src/scene_compositor.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`
- **143** `fn` **target_resolver_arc** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`
- **147** `fn` **build_target_resolver** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`
- **166** `fn` **set_object_regions** `pub`<br>↳ used in: `engine/src/systems/compositor/mod.rs`

### `src/terminal_shell.rs`

- **12** `impl` **TerminalShellState** — *impl TerminalShellState*
- **13** `fn` **new** `pub`
- **180** `impl` **SceneRuntime** — *impl SceneRuntime*
- **181** `fn` **has_terminal_shell** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`
- **185** `fn` **terminal_shell_controls_snapshot** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **193** `fn` **terminal_push_output** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/engine_io.rs`
- **206** `fn` **terminal_clear_output** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/engine_io.rs`
- **218** `fn` **terminal_set_prompt_prefix** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **225** `fn` **terminal_set_prompt_masked** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **232** `fn` **terminal_shell_back_requested** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **249** `fn` **handle_terminal_shell_keys** `pub`<br>↳ used in: `engine-scene-runtime/src/lifecycle_controls.rs`
- **368** `fn` **sidecar_mark_screen_full** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **383** `fn` **sidecar_push_custom_event** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **388** `fn` **sync_terminal_shell_sprites** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`
- **713** `fn` **wrap_text_to_width** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`
- **806** `enum` **WrapToken**

### `src/ui_focus.rs`

- **3** `impl` **SceneRuntime** — *impl SceneRuntime*
- **4** `fn` **ui_theme_id** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`
- **8** `fn` **ui_theme_style** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`
- **13** `fn` **set_last_raw_key** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`
- **18** `fn` **clear_last_raw_key** `pub`<br>↳ used in: `engine/src/game_loop.rs`, `engine/src/systems/scene_lifecycle.rs`
- **23** `fn` **set_key_down** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **31** `fn` **set_key_up** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **39** `fn` **clear_keys_down** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **44** `fn` **keys_down_snapshot** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/scene_lifecycle.rs`
- **48** `fn` **focused_ui_target_id** `pub`<br>↳ used in: `engine-scene-runtime/src/behavior_runner.rs`
- **58** `fn` **handle_ui_focus_keys** `pub`<br>↳ used in: `engine/src/systems/scene_lifecycle.rs`
- **83** `fn` **ui_last_submit_snapshot** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **93** `fn` **ui_last_change_snapshot** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`
- **103** `fn` **last_raw_key_snapshot** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`
- **107** `fn` **reset_frame_state** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`
- **114** `fn` **initialize_ui_state** `pub`<br>↳ used in: `engine-scene-runtime/src/construction.rs`
- **154** `fn` **is_ui_target_focused** `pub`<br>↳ used in: `engine-scene-runtime/src/terminal_shell.rs`
- **160** `fn` **resolve_text_layout** `pub`<br>↳ used in: `engine-scene-runtime/src/terminal_shell.rs`
- **193** `fn` **find_panel_layout_recursive** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`
- **247** `fn` **set_panel_height_recursive** `pub`<br>↳ used in: `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`

---

## engine-terminal

### `src/lib.rs`

- **6** `struct` **TerminalCaps** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`
- **17** `struct` **TerminalRequirements** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **24** `const` **DEFAULT_TARGET_FPS** `pub`
- **25** `const` **MAX_TARGET_FPS** `pub`<br>↳ used in: `engine/src/game_loop.rs`
- **29** `struct` **TerminalViolation** `pub`<br>↳ used in: `engine-mod/src/startup/checks/terminal_requirements.rs`
- **35** `impl` **TerminalCaps** — *impl TerminalCaps*
- **37** `fn` **detect** `pub`<br>↳ used in: `engine-core/src/buffer.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`
- **49** `fn` **validate** `pub`<br>↳ used in: `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/document/scene.rs`
- **86** `impl` **TerminalRequirements** — *impl TerminalRequirements*
- **89** `fn` **from_manifest** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/scene_run.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/terminal_requirements.rs`, `engine/src/lib.rs`
- **114** `fn` **target_fps_from_manifest** `pub`<br>↳ used in: `engine/src/lib.rs`
- **151** `mod` **tests**

---

## engine-vector

### `src/lib.rs`

- **8** `struct` **VectorBounds** `pub`
- **16** `fn` **bounds** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/measure.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/buffer.rs`, `engine-game/src/components.rs`, `engine-game/src/gameplay.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine/src/image_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/systems/compositor/mod.rs`
- **38** `fn` **draw_polyline** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`
- **93** `fn` **fill_polygon** `pub`<br>↳ used in: `engine-compositor/src/sprite_renderer.rs`
- **168** `fn` **point_in_polygon** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **194** `fn` **polygons_intersect** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **234** `fn` **segment_intersects_polygon** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`
- **307** `mod` **tests**

---

## mods

---

## rust-os

### `src/app_host.rs`

- **14** `enum` **ActiveApp**
- **20** `struct` **AppHost** `pub`<br>↳ used in: `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/lib.rs`, `rust-os/src/main.rs`
- **36** `impl` **AppHost** — *impl AppHost*
- **37** `fn` **new** `pub`
- **60** `fn` **handle** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-render/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/main.rs`, `tools/sound-server/src/main.rs`

### `src/app_stack.rs`

- **4** `struct` **PendingLine** `pub`
- **10** `struct` **AppStack** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **14** `impl` **AppStack** — *impl AppStack*
- **15** `fn` **new** `pub`
- **22** `fn` **enqueue** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **34** `fn` **drain_ready** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/kernel/mod.rs`
- **47** `fn` **has_pending** `pub`<br>↳ used in: `engine/src/systems/behavior.rs`, `rust-os/src/app_host.rs`

### `src/apps/ftp.rs`

- **8** `enum` **FtpState** `pub`
- **13** `struct` **FtpApp** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/mod.rs`
- **20** `impl` **FtpApp** — *impl FtpApp*
- **21** `fn` **new** `pub`
- **30** `fn` **prompt** `pub`<br>↳ used in: `app/src/main.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/output_backend.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/screen_buffer.rs`
- **38** `fn` **handle_input** `pub`<br>↳ used in: `rust-os/src/app_host.rs`

### `src/apps/mail.rs`

- **6** `struct` **MailApp** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/mod.rs`
- **10** `impl` **MailApp** — *impl MailApp*
- **11** `fn` **new** `pub`
- **17** `fn` **prompt** `pub`<br>↳ used in: `app/src/main.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/output_backend.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/screen_buffer.rs`
- **21** `fn` **on_enter** `pub`<br>↳ used in: `editor/src/ui/components/start_screen.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `tools/devtool/src/scaffold.rs`
- **68** `fn` **handle_input** `pub`<br>↳ used in: `rust-os/src/app_host.rs`

### `src/apps/mod.rs`

- **1** `mod` **ftp** `pub`
- **2** `mod` **mail** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/vfs/seed.rs`
- **3** `mod` **shell** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/cli.rs`, `editor/src/domain/effects_preview_scene.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/game_state.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-game/src/game_state.rs`, `engine-persistence/src/lib.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/lifecycle_controls.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/lib.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`, `rust-os/src/kernel/users.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`

### `src/apps/shell.rs`

- **9** `struct` **ShellApp** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/mod.rs`
- **13** `impl` **ShellApp** — *impl ShellApp*
- **14** `fn` **new** `pub`
- **20** `fn` **prompt** `pub`<br>↳ used in: `app/src/main.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-mod/src/output_backend.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/screen_buffer.rs`
- **36** `fn` **handle_input** `pub`<br>↳ used in: `rust-os/src/app_host.rs`

### `src/boot.rs`

- **5** `fn` **build_boot_steps** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **15** `macro_rules!` **step**

### `src/commands/fs.rs`

- **5** `struct` **LsCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **6** `impl` **Command** — *impl Command for LsCmd*
- **7** `fn` **execute**
- **111** `struct` **CatCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **112** `impl` **Command** — *impl Command for CatCmd*
- **113** `fn` **execute**
- **152** `struct` **CpCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **153** `impl` **Command** — *impl Command for CpCmd*
- **154** `fn` **execute**
- **170** `struct` **MvCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **171** `impl` **Command** — *impl Command for MvCmd*
- **172** `fn` **execute**
- **186** `struct` **RmCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **187** `impl` **Command** — *impl Command for RmCmd*
- **188** `fn` **execute**
- **215** `struct` **RmdirCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **216** `impl` **Command** — *impl Command for RmdirCmd*
- **217** `fn` **execute**
- **238** `struct` **MkdirCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **239** `impl` **Command** — *impl Command for MkdirCmd*
- **240** `fn` **execute**
- **263** `struct` **ChmodCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **264** `impl` **Command** — *impl Command for ChmodCmd*
- **265** `fn` **execute**
- **303** `struct` **FileCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **304** `impl` **Command** — *impl Command for FileCmd*
- **305** `fn` **execute**

### `src/commands/misc.rs`

- **21** `struct` **FortuneCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **22** `impl` **Command** — *impl Command for FortuneCmd*
- **23** `fn` **execute**
- **38** `struct` **ManCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **39** `impl` **Command** — *impl Command for ManCmd*
- **40** `fn` **execute**
- **86** `struct` **HelpCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **87** `impl` **Command** — *impl Command for HelpCmd*
- **88** `fn` **execute**
- **103** `struct` **ClearCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **104** `impl` **Command** — *impl Command for ClearCmd*
- **105** `fn` **execute**
- **111** `struct` **MailCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **112** `impl` **Command** — *impl Command for MailCmd*
- **113** `fn` **execute**

### `src/commands/mod.rs`

- **5** `mod` **fs** `pub`
- **6** `mod` **misc** `pub`
- **7** `mod` **net** `pub`
- **8** `mod` **proc** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/vfs/seed.rs`
- **9** `mod` **sys** `pub`
- **10** `mod` **text** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/state/mod.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/theme.rs`, `engine-3d/src/scene3d_format.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-compositor/src/text_render.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/builtin/brighten.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/sprite.rs`, `engine-debug/src/profiling.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/context.rs`, `engine-persistence/src/lib.rs`, `engine-render-policy/src/lib.rs`, `engine-render-sdl2/src/bitmap_font.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render/src/generic.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/types.rs`, `engine-runtime/src/lib.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/bench.rs`, `engine/src/level_state.rs`, `engine/src/lib.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/rasterizer/types.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/boot.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/unit_of_work.rs`, `rust-os/src/protocol.rs`, `rust-os/src/style.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **12** `trait` **Command** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-io/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `tools/devtool/src/main.rs`
- **13** `fn` **execute**
- **16** `struct` **CommandRegistry** `pub`<br>↳ used in: `rust-os/src/exec/pipeline.rs`
- **20** `impl` **CommandRegistry** — *impl CommandRegistry*
- **21** `fn` **build** `pub`
- **79** `fn` **get** `pub`

### `src/commands/net.rs`

- **5** `struct` **PingCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **6** `impl` **Command** — *impl Command for PingCmd*
- **7** `fn` **execute**
- **88** `struct` **PingWithRegistry** `pub`
- **93** `fn` **ping_with_registry** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`
- **185** `struct` **NslookupCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **186** `impl` **Command** — *impl Command for NslookupCmd*
- **187** `fn` **execute**
- **224** `struct` **NetstatCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **225** `impl` **Command** — *impl Command for NetstatCmd*
- **226** `fn` **execute**
- **258** `struct` **IfconfigCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **259** `impl` **Command** — *impl Command for IfconfigCmd*
- **260** `fn` **execute**
- **270** `struct` **FtpCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **271** `impl` **Command** — *impl Command for FtpCmd*
- **272** `fn` **execute**
- **282** `struct` **FingerCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **283** `impl` **Command** — *impl Command for FingerCmd*
- **284** `fn` **execute**
- **325** `struct` **TelnetCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **326** `impl` **Command** — *impl Command for TelnetCmd*
- **327** `fn` **execute**

### `src/commands/proc.rs`

- **5** `struct` **PsCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **6** `impl` **Command** — *impl Command for PsCmd*
- **7** `fn` **execute**
- **40** `struct` **KillCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **41** `impl` **Command** — *impl Command for KillCmd*
- **42** `fn` **execute**

### `src/commands/sys.rs`

- **5** `struct` **DfCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **6** `impl` **Command** — *impl Command for DfCmd*
- **7** `fn` **execute**
- **28** `struct` **MountCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **29** `impl` **Command** — *impl Command for MountCmd*
- **30** `fn` **execute**
- **40** `struct` **DateCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **41** `impl` **Command** — *impl Command for DateCmd*
- **42** `fn` **execute**
- **55** `struct` **UnameCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **56** `impl` **Command** — *impl Command for UnameCmd*
- **57** `fn` **execute**
- **95** `struct` **HostnameCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **96** `impl` **Command** — *impl Command for HostnameCmd*
- **97** `fn` **execute**
- **113** `struct` **WhoamiCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **114** `impl` **Command** — *impl Command for WhoamiCmd*
- **115** `fn` **execute**
- **120** `struct` **WhoCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **121** `impl` **Command** — *impl Command for WhoCmd*
- **122** `fn` **execute**
- **133** `struct` **WCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **134** `impl` **Command** — *impl Command for WCmd*
- **135** `fn` **execute**
- **170** `struct` **IdCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **171** `impl` **Command** — *impl Command for IdCmd*
- **172** `fn` **execute**
- **186** `struct` **SyncCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **187** `impl` **Command** — *impl Command for SyncCmd*
- **188** `fn` **execute**
- **193** `struct` **DmesgCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **194** `impl` **Command** — *impl Command for DmesgCmd*
- **195** `fn` **execute**
- **219** `struct` **LastCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **220** `impl` **Command** — *impl Command for LastCmd*
- **221** `fn` **execute**

### `src/commands/text.rs`

- **5** `struct` **EchoCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **6** `impl` **Command** — *impl Command for EchoCmd*
- **7** `fn` **execute**
- **13** `struct` **GrepCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **14** `impl` **Command** — *impl Command for GrepCmd*
- **15** `fn` **execute**
- **51** `struct` **HeadCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **52** `impl` **Command** — *impl Command for HeadCmd*
- **53** `fn` **execute**
- **82** `struct` **TailCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **83** `impl` **Command** — *impl Command for TailCmd*
- **84** `fn` **execute**
- **115** `struct` **WcCmd** `pub`<br>↳ used in: `rust-os/src/commands/mod.rs`
- **116** `impl` **Command** — *impl Command for WcCmd*
- **117** `fn` **execute**

### `src/difficulty.rs`

- **2** `enum` **Difficulty** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/lib.rs`, `rust-os/src/main.rs`
- **10** `impl` **Difficulty** — *impl Difficulty*
- **11** `fn` **from_label** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/main.rs`
- **24** `struct` **MachineSpec** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/boot.rs`, `rust-os/src/hardware.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/mount.rs`, `rust-os/src/kernel/process.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/lib.rs`, `rust-os/src/main.rs`
- **39** `impl` **MachineSpec** — *impl MachineSpec*
- **40** `fn` **from_difficulty** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/main.rs`

### `src/exec/builtins.rs`

- **4** `fn` **try_handle** `pub`<br>↳ used in: `rust-os/src/exec/pipeline.rs`

### `src/exec/mod.rs`

- **1** `mod` **builtins** `pub`<br>↳ used in: `rust-os/src/exec/pipeline.rs`
- **2** `mod` **pipeline** `pub`<br>↳ used in: `engine-authoring/src/compile/mod.rs`, `engine-authoring/src/lib.rs`, `engine-authoring/src/package/mod.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/lib.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/strategy/mod.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/mod.rs`, `engine-mod/src/startup/report.rs`, `engine-mod/src/startup/runner.rs`, `engine-pipeline/src/lib.rs`, `engine-pipeline/src/strategies/mod.rs`, `engine-pipeline/src/strategies/skip.rs`, `engine-render/src/simd_text.rs`, `engine/src/bench.rs`, `engine/src/gpu/context.rs`, `engine/src/gpu/render.rs`, `engine/src/lib.rs`, `engine/src/pipelines/mod.rs`, `engine/src/prepared_frame.rs`, `engine/src/scene_pipeline.rs`, `engine/src/strategy/mod.rs`, `engine/src/systems/prerender.rs`, `engine/src/systems/scene3d_prerender.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/warmup.rs`, `rust-os/src/apps/shell.rs`
- **3** `mod` **tokenizer** `pub`<br>↳ used in: `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`

### `src/exec/pipeline.rs`

- **9** `struct` **MinixPipeline** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`, `rust-os/src/exec/mod.rs`
- **13** `impl` **MinixPipeline** — *impl MinixPipeline*
- **14** `fn` **new** `pub`
- **21** `fn` **execute** `pub`<br>↳ used in: `editor/src/app.rs`, `engine-core/src/scene/model.rs`, `engine-render-terminal/src/renderer.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/apps/shell.rs`

### `src/exec/tokenizer.rs`

- **4** `struct` **SimpleCommand** `pub`<br>↳ used in: `rust-os/src/exec/mod.rs`
- **13** `fn` **tokenize** `pub`<br>↳ used in: `rust-os/src/exec/mod.rs`, `rust-os/src/exec/pipeline.rs`

### `src/hardware.rs`

- **6** `struct` **HardwareProfile** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/modem.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/kernel/syscall.rs`
- **49** `impl` **HardwareProfile** — *impl HardwareProfile*
- **50** `fn` **from_spec** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`

### `src/hosts/mod.rs`

- **1** `mod` **registry** `pub`<br>↳ used in: `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-behavior/src/registry/mod.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/systems/postfx/pass_crt.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/effects/builtin/clear_to_colour.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/devour.rs`, `engine-core/src/effects/builtin/fade_to_black.rs`, `engine-core/src/effects/builtin/power_off.rs`, `engine-core/src/effects/builtin/scanlines.rs`, `engine-core/src/effects/builtin/shine.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/scene/mod.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/strategy/effect_factory.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine/src/systems/behavior.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/exec/pipeline.rs`

### `src/hosts/registry.rs`

- **4** `enum` **HostKind** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`, `rust-os/src/hosts/mod.rs`
- **11** `struct` **RemoteHost** `pub`<br>↳ used in: `rust-os/src/hosts/mod.rs`
- **24** `impl` **RemoteHost** — *impl RemoteHost*
- **62** `struct` **RemoteHostIndex** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/hosts/mod.rs`
- **66** `impl` **RemoteHostIndex** — *impl RemoteHostIndex*
- **67** `fn` **build** `pub`
- **84** `fn` **resolve** `pub`<br>↳ used in: `app/src/main.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/scene_compiler.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-authoring/src/compile/scene.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/buffer.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-render-policy/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_loader.rs`, `engine/src/services.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/scaffold.rs`
- **88** `fn` **is_known** `pub`

### `src/kernel/clock.rs`

- **2** `const` **EPOCH_YEAR** `pub`
- **3** `const` **EPOCH_MONTH** `pub`
- **4** `const` **EPOCH_DAY** `pub`
- **5** `const` **EPOCH_HOUR** `pub`
- **6** `const` **EPOCH_MINUTE** `pub`
- **8** `struct` **SimulatedClock** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **12** `impl` **SimulatedClock** — *impl SimulatedClock*
- **13** `fn` **new** `pub`
- **17** `fn` **advance** `pub`<br>↳ used in: `engine-animation/src/systems.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-render/src/types.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/rasterizer/types.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/kernel/mod.rs`
- **21** `fn` **uptime_ms** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/state.rs`
- **26** `fn` **now_str** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`
- **45** `fn` **time_str** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`
- **55** `fn` **uptime_str** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`

### `src/kernel/disk.rs`

- **3** `struct` **SimulatedDisk** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **8** `impl` **SimulatedDisk** — *impl SimulatedDisk*
- **9** `fn` **new** `pub`
- **16** `fn` **record_read** `pub`
- **19** `fn` **record_write** `pub`

### `src/kernel/events.rs`

- **4** `struct` **ScheduledEvent** `pub`
- **11** `impl` **PartialEq** — *impl PartialEq for ScheduledEvent*
- **16** `impl` **Eq** — *impl Eq for ScheduledEvent {}*
- **18** `impl` **PartialOrd** — *impl PartialOrd for ScheduledEvent*
- **24** `impl` **Ord** — *impl Ord for ScheduledEvent*
- **34** `struct` **KernelEventQueue** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **39** `impl` **KernelEventQueue** — *impl KernelEventQueue*
- **40** `fn` **new** `pub`
- **47** `fn` **schedule_at** `pub`
- **63** `fn` **schedule_after** `pub`
- **73** `fn` **drain_ready** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/kernel/mod.rs`

### `src/kernel/journal.rs`

- **1** `struct` **Journal** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/syscall.rs`
- **5** `impl` **Journal** — *impl Journal*
- **6** `fn` **new** `pub`
- **12** `fn` **append** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-core/src/logging.rs`, `engine/src/splash.rs`, `rust-os/src/app_host.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/cli.rs`, `tools/sound-server/src/main.rs`
- **16** `fn` **all** `pub`
- **21** `fn` **append_anomaly** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`
- **27** `fn` **append_clock_drift** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`
- **33** `fn` **append_inode_anomaly** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`

### `src/kernel/mail.rs`

- **3** `struct` **MailSpool** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **7** `impl` **MailSpool** — *impl MailSpool*
- **8** `fn` **new** `pub`
- **35** `fn` **list** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/scene_index.rs`, `editor/src/io/recent.rs`, `editor/src/state/filters.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/selection.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-mod/src/startup/runner.rs`, `engine-mod/src/terminal_caps.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`, `engine-terminal/src/lib.rs`, `engine-vector/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_pipeline.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/exec/tokenizer.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`
- **39** `fn` **get** `pub`
- **43** `fn` **mark_read** `pub`<br>↳ used in: `rust-os/src/apps/mail.rs`
- **49** `fn` **deliver** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **60** `fn` **unread_count** `pub`<br>↳ used in: `rust-os/src/app_host.rs`

### `src/kernel/mod.rs`

- **1** `mod` **clock** `pub`<br>↳ used in: `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-debug/src/lib.rs`, `rust-os/src/boot.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/journal.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **2** `mod` **disk** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/io/yaml.rs`, `editor/src/state/watch.rs`, `engine-3d/src/scene3d_format.rs`, `engine-asset/src/repositories.rs`, `engine-persistence/src/lib.rs`, `engine/src/scene3d_format.rs`, `engine/src/scene_pipeline.rs`, `rust-os/src/boot.rs`, `rust-os/src/kernel/disk.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/vfs/seed.rs`, `tools/schema-gen/src/main.rs`
- **3** `mod` **events** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/commands.rs`, `editor/src/state/scene_run.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-events/src/input_backend.rs`, `engine-events/src/lib.rs`, `engine-game/src/collision.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-io/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-render-sdl2/src/input.rs`, `engine-render-sdl2/src/runtime.rs`, `engine-render-terminal/src/input.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/services.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/engine_io.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/app_host.rs`, `rust-os/src/kernel/events.rs`, `rust-os/src/main.rs`
- **4** `mod` **journal** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`
- **5** `mod` **mail** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/vfs/seed.rs`
- **6** `mod` **modem** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/kernel/modem.rs`, `rust-os/src/vfs/seed.rs`
- **7** `mod` **mount** `pub`<br>↳ used in: `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`
- **8** `mod` **network** `pub`<br>↳ used in: `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/vfs/seed.rs`
- **9** `mod` **process** `pub`<br>↳ used in: `app/src/main.rs`, `engine-audio/src/audio.rs`, `engine-audio/src/lib.rs`, `engine-authoring/src/schema/mod.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/effects/builtin/terminal_crt.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-debug/src/lib.rs`, `engine-debug/src/log.rs`, `engine-io/src/lib.rs`, `engine-render/src/image_loader.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/image_loader.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/kernel/process.rs`, `tools/devtool/src/main.rs`, `tools/schema-gen/src/main.rs`
- **10** `mod` **resources** `pub`<br>↳ used in: `engine-animation/src/provider.rs`, `engine-behavior/src/registry/provider.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/provider.rs`, `engine-core/src/access.rs`, `engine-core/src/access_tests.rs`, `engine-core/src/game_state.rs`, `engine-core/src/world.rs`, `engine-debug/src/access.rs`, `engine-game/src/game_state.rs`, `engine-render/src/lib.rs`, `engine/src/scene_pipeline.rs`, `engine/src/services.rs`, `engine/src/splash.rs`, `engine/src/systems/scene_lifecycle.rs`, `rust-os/src/kernel/mail.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/vfs/seed.rs`
- **11** `mod` **unit_of_work** `pub`<br>↳ used in: `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/boot.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/modem.rs`
- **12** `mod` **users** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`, `rust-os/src/kernel/users.rs`, `rust-os/src/vfs/seed.rs`, `tools/schema-gen/src/main.rs`
- **29** `struct` **Kernel** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/vfs/seed.rs`
- **46** `impl` **Kernel** — *impl Kernel*
- **47** `fn` **new** `pub`
- **81** `fn` **tick** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-animation/src/systems.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/effects/utils/math.rs`, `engine-game/src/gameplay.rs`, `engine/src/audio_sequencer.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`
- **92** `fn` **uptime_ms** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/state.rs`

### `src/kernel/modem.rs`

- **13** `struct` **SimulatedModem** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **21** `impl` **SimulatedModem** — *impl SimulatedModem*
- **22** `fn` **new** `pub`
- **33** `fn` **dial** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`
- **60** `fn` **hangup** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`
- **83** `fn` **apply_noise** `pub`

### `src/kernel/mount.rs`

- **4** `struct` **MountEntry** `pub`
- **13** `struct` **MountTable** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **17** `impl` **MountTable** — *impl MountTable*
- **18** `fn` **new** `pub`
- **46** `fn` **get_mounts** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`
- **50** `fn` **get_mount** `pub`

### `src/kernel/network.rs`

- **3** `struct` **SimulatedNetwork** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **8** `impl` **SimulatedNetwork** — *impl SimulatedNetwork*
- **9** `fn` **new** `pub`
- **16** `fn` **connect** `pub`<br>↳ used in: `engine-io/src/lib.rs`, `rust-os/src/commands/net.rs`
- **23** `fn` **close** `pub`
- **27** `fn` **is_connected** `pub`

### `src/kernel/process.rs`

- **4** `struct` **SimulatedProcessTable** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **10** `impl` **SimulatedProcessTable** — *impl SimulatedProcessTable*
- **11** `fn` **new** `pub`
- **19** `fn` **boot_system_processes** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **46** `fn` **fork** `pub`<br>↳ used in: `rust-os/src/hosts/registry.rs`
- **68** `fn` **exit_pid** `pub`
- **72** `fn` **kill** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/proc.rs`
- **82** `fn` **list** `pub`<br>↳ used in: `editor/src/domain/effects_catalog.rs`, `editor/src/domain/scene_index.rs`, `editor/src/io/recent.rs`, `editor/src/state/filters.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/selection.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/start_screen.rs`, `engine-3d/src/scene3d_resolve.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/mod.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/effects/metadata.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/markup.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/model.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-mod/src/startup/runner.rs`, `engine-mod/src/terminal_caps.rs`, `engine-render-terminal/src/rasterizer/generic.rs`, `engine-render/src/generic.rs`, `engine-terminal/src/lib.rs`, `engine-vector/src/lib.rs`, `engine/src/scene3d_resolve.rs`, `engine/src/scene_pipeline.rs`, `engine/tests/frame_regression.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/exec/tokenizer.rs`, `rust-os/src/kernel/syscall.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`
- **86** `fn` **get** `pub`
- **91** `fn` **inject_phantom_process** `pub`<br>↳ used in: `rust-os/src/commands/proc.rs`
- **108** `fn` **remove_phantom_process** `pub`<br>↳ used in: `rust-os/src/commands/proc.rs`

### `src/kernel/resources.rs`

- **8** `enum` **SpindleState** `pub`
- **14** `struct` **DiskController** `pub`
- **20** `impl` **DiskController** — *impl DiskController*
- **21** `fn` **new** `pub`
- **29** `fn` **acquire** `pub`
- **50** `fn` **release** `pub`<br>↳ used in: `app/src/main.rs`, `editor/src/app.rs`, `editor/src/state/scenes_browser.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-debug/src/profiling.rs`, `rust-os/src/kernel/clock.rs`
- **54** `fn` **update_spindle_state** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **68** `fn` **latency_for_read** `pub`
- **79** `struct` **RamAllocator** `pub`
- **85** `impl` **RamAllocator** — *impl RamAllocator*
- **86** `fn` **new** `pub`
- **94** `fn` **available_kb** `pub`<br>↳ used in: `rust-os/src/kernel/syscall.rs`
- **98** `fn` **alloc** `pub`<br>↳ used in: `rust-os/src/kernel/syscall.rs`
- **110** `fn` **free** `pub`<br>↳ used in: `app/src/main.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine/src/lib.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/vfs/seed.rs`
- **118** `struct` **FdEntry** `pub`
- **125** `enum` **FdMode** `pub`
- **131** `struct` **FdTable** `pub`
- **138** `impl` **FdTable** — *impl FdTable*
- **139** `fn` **new** `pub`
- **173** `fn` **alloc** `pub`<br>↳ used in: `rust-os/src/kernel/syscall.rs`
- **183** `fn` **open_file** `pub`
- **196** `fn` **close_fd** `pub`
- **202** `fn` **free** `pub`<br>↳ used in: `app/src/main.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine/src/lib.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/vfs/seed.rs`
- **209** `struct` **CacheEntry**
- **213** `struct` **BufferCache** `pub`
- **220** `impl` **BufferCache** — *impl BufferCache*
- **221** `fn` **new** `pub`
- **230** `fn` **lookup** `pub`<br>↳ used in: `engine-3d/src/scene3d_atlas.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/scene3d_atlas.rs`, `engine-core/src/scene/sprite.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-game/src/components.rs`, `engine-render/src/rasterizer.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine/src/rasterizer/mod.rs`, `engine/src/scene_loader.rs`
- **243** `fn` **insert** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `editor/src/state/effects_browser.rs`, `engine-animation/src/menu.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-authoring/src/package/mod.rs`, `engine-authoring/src/schema/mod.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/compositor.rs`, `engine-compositor/src/effect_applicator.rs`, `engine-compositor/src/layer_compositor.rs`, `engine-compositor/src/layout/flex.rs`, `engine-compositor/src/layout/grid.rs`, `engine-compositor/src/obj_loader.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/prerender.rs`, `engine-compositor/src/render/common.rs`, `engine-compositor/src/scene3d_prerender.rs`, `engine-compositor/src/sprite_renderer.rs`, `engine-core/src/animations/mod.rs`, `engine-core/src/asset_cache.rs`, `engine-core/src/effects/mod.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/template.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-core/src/world.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/effect_registry.rs`, `engine-mod/src/startup/checks/font_glyph_coverage.rs`, `engine-mod/src/startup/checks/font_manifest.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-persistence/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render/src/font_loader.rs`, `engine-render/src/rasterizer.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/construction.rs`, `engine-scene-runtime/src/materialization.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/rasterizer/font_loader.rs`, `engine/src/rasterizer/mod.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **262** `fn` **invalidate** `pub`<br>↳ used in: `engine-render-terminal/src/renderer.rs`, `engine/src/systems/scene_lifecycle.rs`
- **267** `fn` **len** `pub`
- **274** `struct` **CpuScheduler** `pub`
- **279** `impl` **CpuScheduler** — *impl CpuScheduler*
- **280** `fn` **new** `pub`
- **287** `fn` **add** `pub`
- **293** `fn` **remove** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/cutscene.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-debug/src/log.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/systems/compositor/mod.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/kernel/network.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/edit.rs`, `tools/sound-server/src/main.rs`
- **297** `fn` **schedule** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/kernel/modem.rs`
- **306** `fn` **run_queue_len** `pub`<br>↳ used in: `rust-os/src/kernel/syscall.rs`
- **314** `enum` **SocketState** `pub`
- **322** `struct` **SocketEntry** `pub`
- **330** `struct` **NetworkController** `pub`
- **337** `impl` **NetworkController** — *impl NetworkController*
- **338** `fn` **new** `pub`
- **347** `fn` **connect** `pub`<br>↳ used in: `engine-io/src/lib.rs`, `rust-os/src/commands/net.rs`
- **370** `fn` **send** `pub`
- **380** `fn` **recv** `pub`
- **392** `fn` **enqueue_recv** `pub`
- **398** `fn` **close** `pub`
- **406** `fn` **active_count** `pub`<br>↳ used in: `rust-os/src/kernel/syscall.rs`
- **413** `fn` **list_sockets** `pub`
- **420** `struct` **ResourceState** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/syscall.rs`
- **431** `impl` **ResourceState** — *impl ResourceState*
- **432** `fn` **new** `pub`

### `src/kernel/syscall.rs`

- **5** `enum` **SyscallKind** `pub`
- **38** `impl` **SyscallKind** — *impl SyscallKind*
- **39** `fn` **is_disk** `pub`
- **46** `fn` **is_net** `pub`
- **54** `struct` **MinixSyscallGate** `pub`
- **56** `impl` **MinixSyscallGate** — *impl MinixSyscallGate*
- **58** `fn` **can_satisfy** `pub`
- **88** `fn` **debit** `pub`
- **106** `fn` **credit** `pub`
- **124** `fn` **latency_for** `pub`

### `src/kernel/unit_of_work.rs`

- **5** `struct` **ScheduledLine** `pub`<br>↳ used in: `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/boot.rs`, `rust-os/src/exec/pipeline.rs`
- **11** `struct` **UnitOfWork** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/modem.rs`
- **21** `fn` **new** `pub`
- **33** `fn` **schedule** `pub`<br>↳ used in: `engine/src/systems/engine_io.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/kernel/modem.rs`
- **42** `fn` **print** `pub`<br>↳ used in: `app/src/main.rs`, `engine-behavior/src/lib.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`
- **47** `fn` **drain** `pub`<br>↳ used in: `editor/src/state/scene_run.rs`, `engine-behavior/src/lib.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/game_loop.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/screen_buffer.rs`
- **51** `fn` **request_exit** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/exec/builtins.rs`

### `src/kernel/users.rs`

- **2** `struct` **PasswdEntry** `pub`
- **12** `struct` **GroupEntry** `pub`
- **18** `struct` **UserDatabase** `pub`<br>↳ used in: `rust-os/src/kernel/mod.rs`
- **23** `impl` **UserDatabase** — *impl UserDatabase*
- **24** `fn` **new** `pub`
- **94** `fn` **get_user** `pub`
- **98** `fn` **all_users** `pub`
- **102** `fn` **get_group** `pub`
- **106** `fn` **get_group_by_name** `pub`
- **110** `fn` **groups_for_user** `pub`<br>↳ used in: `rust-os/src/commands/sys.rs`

### `src/lib.rs`

- **1** `mod` **app_host** `pub`
- **2** `mod` **app_stack** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **3** `mod` **apps** `pub`<br>↳ used in: `engine-io/src/lib.rs`, `rust-os/src/app_host.rs`
- **4** `mod` **boot** `pub`<br>↳ used in: `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/utils/math.rs`, `engine-core/src/level_state.rs`, `engine-core/src/scene/model.rs`, `rust-os/src/app_host.rs`, `rust-os/src/boot.rs`, `rust-os/src/commands/sys.rs`
- **5** `mod` **commands** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/commands.rs`, `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-audio/src/audio.rs`, `engine-audio/src/lib.rs`, `engine-audio/src/systems_audio.rs`, `engine-behavior/src/lib.rs`, `engine-core/src/authoring/catalog.rs`, `engine-core/src/scene/model.rs`, `engine-render-terminal/src/renderer.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine/src/gpu/render.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/scene_lifecycle.rs`, `engine/src/systems/visual_binding.rs`, `engine/src/systems/visual_sync.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/exec/tokenizer.rs`
- **6** `mod` **difficulty** `pub`<br>↳ used in: `engine-core/src/level_state.rs`, `engine-io/src/lib.rs`, `engine/src/systems/engine_io.rs`, `rust-os/src/app_host.rs`, `rust-os/src/boot.rs`, `rust-os/src/difficulty.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hardware.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/mount.rs`, `rust-os/src/kernel/process.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/main.rs`
- **7** `mod` **exec** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`
- **8** `mod` **hardware** `pub`<br>↳ used in: `rust-os/src/hardware.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/modem.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/kernel/syscall.rs`
- **9** `mod` **hosts** `pub`<br>↳ used in: `engine-core/src/scene/model.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/vfs/seed.rs`
- **10** `mod` **kernel** `pub`<br>↳ used in: `engine-compositor/src/systems/postfx/glow.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-core/src/effects/builtin/blur.rs`, `engine-core/src/scene/model.rs`, `rust-os/src/app_host.rs`, `rust-os/src/app_stack.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/boot.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/journal.rs`, `rust-os/src/kernel/mail.rs`, `rust-os/src/kernel/mod.rs`, `rust-os/src/kernel/modem.rs`, `rust-os/src/kernel/process.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/vfs/seed.rs`
- **11** `mod` **protocol** `pub`<br>↳ used in: `engine-io/src/lib.rs`, `rust-os/src/app_host.rs`, `rust-os/src/commands/misc.rs`
- **12** `mod` **screen_buffer** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **13** `mod` **session** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-game/src/game_state.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hosts/registry.rs`, `rust-os/src/kernel/unit_of_work.rs`, `rust-os/src/session.rs`
- **14** `mod` **state** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/filters.rs`, `editor/src/state/mod.rs`, `editor/src/state/selection.rs`, `editor/src/state/watch.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/header.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/components/status_bar.rs`, `editor/src/ui/icons.rs`, `editor/src/ui/mod.rs`, `editor/src/ui/theme.rs`, `engine-animation/src/animator.rs`, `engine-animation/src/lib.rs`, `engine-animation/src/systems.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/object.rs`, `engine-authoring/src/validate/mod.rs`, `engine-behavior/src/lib.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/access.rs`, `engine-compositor/src/render/common.rs`, `engine-core/src/access.rs`, `engine-core/src/buffer.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-core/src/lib.rs`, `engine-core/src/logging.rs`, `engine-core/src/scene_runtime_types.rs`, `engine-debug/src/lib.rs`, `engine-game/src/components.rs`, `engine-game/src/game_state.rs`, `engine-game/src/gameplay.rs`, `engine-game/src/lib.rs`, `engine-persistence/src/lib.rs`, `engine-render/src/lib.rs`, `engine-render/src/overlay.rs`, `engine-render/src/simd_text.rs`, `engine-scene-runtime/src/access.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/camera_3d.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/object_graph.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `engine-scene-runtime/src/ui_focus.rs`, `engine/src/audio_sequencer.rs`, `engine/src/level_state.rs`, `engine/src/systems/behavior.rs`, `engine/src/systems/hot_reload.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/hardware.rs`, `rust-os/src/kernel/mail.rs`, `rust-os/src/kernel/process.rs`, `rust-os/src/kernel/resources.rs`, `rust-os/src/kernel/syscall.rs`, `rust-os/src/kernel/unit_of_work.rs`, `rust-os/src/session.rs`, `rust-os/src/vfs/mod.rs`
- **15** `mod` **style** `pub`<br>↳ used in: `editor/src/domain/preview_renderer.rs`, `editor/src/ui/components/cutscene_preview.rs`, `editor/src/ui/components/editor.rs`, `editor/src/ui/components/effects_preview.rs`, `editor/src/ui/components/help.rs`, `editor/src/ui/components/preview.rs`, `editor/src/ui/components/scene_run.rs`, `editor/src/ui/components/scenes_preview.rs`, `editor/src/ui/components/sidebar/effects.rs`, `editor/src/ui/components/sidebar/explorer.rs`, `editor/src/ui/components/sidebar/icons.rs`, `editor/src/ui/components/sidebar/placeholder.rs`, `editor/src/ui/components/start_screen.rs`, `editor/src/ui/theme.rs`, `engine-authoring/src/document/scene.rs`, `engine-behavior/src/lib.rs`, `engine-compositor/src/systems/postfx/pass_underlay.rs`, `engine-compositor/src/systems/postfx/registry.rs`, `engine-core/src/color.rs`, `engine-core/src/effects/builtin/glitch.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/shake.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-core/src/scene/color.rs`, `engine-core/src/scene/metadata.rs`, `engine-core/src/scene/ui_theme.rs`, `engine-game/src/components.rs`, `engine-io/src/lib.rs`, `engine-physics/src/lib.rs`, `engine-render-terminal/src/color_convert.rs`, `engine-render-terminal/src/renderer.rs`, `engine-render-terminal/src/strategy/flush.rs`, `engine-scene-runtime/src/behavior_runner.rs`, `engine-scene-runtime/src/lib.rs`, `engine-scene-runtime/src/materialization.rs`, `engine/src/lib.rs`, `engine/src/splash.rs`, `engine/src/systems/mod.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/screen_buffer.rs`, `tools/schema-gen/src/main.rs`
- **16** `mod` **vfs** `pub`

### `src/protocol.rs`

- **4** `fn` **emit_line** `pub`
- **11** `fn` **out** `pub`
- **15** `fn` **set_prompt** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **19** `fn` **set_masked** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **23** `fn` **clear** `pub`

### `src/screen_buffer.rs`

- **4** `struct` **ScreenBuffer** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **14** `impl` **ScreenBuffer** — *impl ScreenBuffer*
- **15** `fn` **new** `pub`
- **27** `fn` **set_viewport** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **32** `fn` **set_prompt** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **36** `fn` **set_input_line** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **45** `fn` **append** `pub`<br>↳ used in: `engine-audio/src/audio.rs`, `engine-authoring/src/compile/scene.rs`, `engine-authoring/src/document/scene.rs`, `engine-core/src/logging.rs`, `engine/src/splash.rs`, `rust-os/src/app_host.rs`, `rust-os/src/vfs/mod.rs`, `tools/devtool/src/cli.rs`, `tools/sound-server/src/main.rs`
- **58** `fn` **commit_input_line** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **65** `fn` **send_frame** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **81** `fn` **clear** `pub`

### `src/session.rs`

- **3** `struct` **UserSession** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/unit_of_work.rs`
- **12** `impl` **UserSession** — *impl UserSession*
- **13** `fn` **new** `pub`
- **27** `fn` **cwd** `pub`
- **31** `fn` **set_cwd** `pub`<br>↳ used in: `rust-os/src/exec/builtins.rs`
- **36** `fn` **resolve_path** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/text.rs`, `rust-os/src/exec/builtins.rs`
- **49** `fn` **display_cwd** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`
- **59** `fn` **push_history** `pub`<br>↳ used in: `rust-os/src/exec/pipeline.rs`
- **67** `fn` **normalize**

### `src/state.rs`

- **2** `enum` **SessionMode** `pub`<br>↳ used in: `rust-os/src/app_host.rs`
- **10** `struct` **QuestState** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/apps/shell.rs`, `rust-os/src/exec/pipeline.rs`, `rust-os/src/kernel/unit_of_work.rs`
- **20** `impl` **Default** — *impl Default for QuestState*
- **34** `impl` **QuestState** — *impl QuestState*
- **35** `fn` **anomaly_count** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/exec/pipeline.rs`
- **39** `fn` **note_anomaly** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`
- **47** `fn` **decay_tier** `pub`<br>↳ used in: `rust-os/src/commands/fs.rs`, `rust-os/src/commands/proc.rs`, `rust-os/src/commands/sys.rs`
- **58** `struct` **MailMessage** `pub`<br>↳ used in: `rust-os/src/kernel/mail.rs`
- **68** `struct` **ProcessEntry** `pub`<br>↳ used in: `rust-os/src/kernel/process.rs`
- **81** `struct` **FileStat** `pub`<br>↳ used in: `rust-os/src/vfs/mod.rs`
- **91** `struct` **MachineState** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/lib.rs`
- **104** `impl` **MachineState** — *impl MachineState*
- **105** `fn` **new** `pub`
- **120** `fn` **has_account** `pub`
- **128** `impl` **Default** — *impl Default for MachineState*

### `src/style.rs`

- **5** `const` **PROMPT_USER** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`
- **6** `const` **PROMPT_HOST** `pub`<br>↳ used in: `rust-os/src/apps/ftp.rs`, `rust-os/src/apps/shell.rs`
- **7** `const` **PROMPT_PATH** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`
- **8** `const` **ERROR** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-render-terminal/src/renderer.rs`, `rust-os/src/apps/shell.rs`
- **9** `const` **WARN** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **10** `const` **INFO** `pub`<br>↳ used in: `engine-core/src/logging.rs`, `engine-debug/src/log.rs`, `engine-render-terminal/src/renderer.rs`
- **11** `const` **BRIGHT** `pub`
- **12** `const` **BOOT_KEYWORD** `pub`
- **13** `const` **BOOT_OK** `pub`<br>↳ used in: `rust-os/src/apps/shell.rs`
- **14** `const` **ANOMALY** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`
- **15** `const` **DIM** `pub`
- **17** `fn` **fg** `pub`
- **22** `fn` **strip** `pub`<br>↳ used in: `rust-os/src/screen_buffer.rs`
- **37** `fn` **visible_len** `pub`<br>↳ used in: `engine-core/src/effects/builtin/lightning.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `rust-os/src/screen_buffer.rs`

### `src/vfs/mod.rs`

- **4** `mod` **seed** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-game/src/gameplay.rs`, `engine/src/splash.rs`
- **9** `struct` **Vfs** `pub`
- **14** `impl` **Vfs** — *impl Vfs*
- **15** `fn` **new** `pub`
- **24** `fn` **mkdir** `pub`<br>↳ used in: `engine-behavior/src/registry/mod.rs`, `engine/src/systems/hot_reload.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/misc.rs`, `rust-os/src/commands/mod.rs`, `rust-os/src/vfs/seed.rs`
- **37** `fn` **write_file** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`, `rust-os/src/vfs/seed.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **58** `fn` **write_file_with_perms** `pub`<br>↳ used in: `rust-os/src/vfs/seed.rs`
- **65** `fn` **read_file** `pub`<br>↳ used in: `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/commands/net.rs`, `rust-os/src/commands/sys.rs`, `rust-os/src/commands/text.rs`
- **70** `fn` **exists** `pub`<br>↳ used in: `editor/src/domain/effects_preview_scene.rs`, `editor/src/domain/preview_renderer.rs`, `editor/src/io/fs_scan.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/mod.rs`, `editor/src/state/start_screen.rs`, `editor/src/ui/components/effects_preview.rs`, `engine-asset/src/repositories.rs`, `engine-authoring/src/schema/mod.rs`, `engine-capture/src/capture.rs`, `engine-compositor/src/systems/postfx/pass_burn_in.rs`, `engine-compositor/src/systems/postfx/pass_crt_distort.rs`, `engine-core/src/asset_source.rs`, `engine-core/src/buffer.rs`, `engine-core/src/game_state.rs`, `engine-core/src/level_state.rs`, `engine-game/src/game_state.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/action_map.rs`, `engine-mod/src/startup/checks/image_assets.rs`, `engine-mod/src/startup/context.rs`, `engine-persistence/src/lib.rs`, `engine/src/systems/compositor/mod.rs`, `engine/src/systems/engine_io.rs`, `engine/src/world.rs`, `rust-os/src/app_host.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `tools/devtool/src/edit.rs`, `tools/devtool/src/fs_utils.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`, `tools/schema-gen/src/main.rs`
- **74** `fn` **is_dir** `pub`<br>↳ used in: `editor/src/io/fs_scan.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/start_screen.rs`, `engine-asset/src/repositories.rs`, `engine-audio-sequencer/src/lib.rs`, `engine-audio/src/audio.rs`, `engine-authoring/src/schema/mod.rs`, `engine-core/src/logging.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine-mod/src/startup/checks/rhai_scripts.rs`, `engine-mod/src/startup/checks/scene_graph.rs`, `engine/src/audio_sequencer.rs`, `engine/src/systems/hot_reload.rs`, `rust-os/src/commands/fs.rs`, `tools/devtool/src/fs_utils.rs`, `tools/schema-gen/src/main.rs`, `tools/sound-server/src/main.rs`
- **79** `fn` **is_file** `pub`<br>↳ used in: `editor/src/state/cutscene.rs`, `engine-asset/src/repositories.rs`, `engine-audio/src/audio.rs`, `engine-mod/src/lib.rs`, `engine-mod/src/startup/checks/audio_sequencer.rs`, `engine-mod/src/startup/checks/level_config.rs`, `engine/src/audio_sequencer.rs`, `engine/src/systems/hot_reload.rs`, `rust-os/src/commands/fs.rs`, `tools/sound-server/src/main.rs`
- **84** `fn` **stat** `pub`<br>↳ used in: `engine-debug/src/lib.rs`, `engine/src/bench.rs`, `rust-os/src/apps/ftp.rs`, `rust-os/src/commands/fs.rs`, `rust-os/src/kernel/syscall.rs`
- **88** `fn` **stat_mut** `pub`<br>↳ used in: `rust-os/src/commands/fs.rs`
- **92** `fn` **delete** `pub`<br>↳ used in: `engine-animation/src/menu.rs`, `rust-os/src/apps/mail.rs`, `rust-os/src/commands/fs.rs`
- **99** `fn` **readdir** `pub`<br>↳ used in: `rust-os/src/commands/fs.rs`
- **125** `fn` **copy_file** `pub`<br>↳ used in: `rust-os/src/commands/fs.rs`
- **138** `fn` **move_file** `pub`<br>↳ used in: `rust-os/src/commands/fs.rs`
- **147** `fn` **inject_data_bleed** `pub`
- **160** `fn` **normalize** `pub`<br>↳ used in: `editor/src/io/recent.rs`, `engine-authoring/src/compile/mod.rs`

### `src/vfs/seed.rs`

- **3** `fn` **seed** `pub`<br>↳ used in: `engine-behavior/src/lib.rs`, `engine-compositor/src/obj_render.rs`, `engine-compositor/src/systems/postfx/glow.rs`, `engine-core/src/effects/builtin/crt_on.rs`, `engine-core/src/effects/builtin/lightning.rs`, `engine-core/src/effects/builtin/shatter.rs`, `engine-game/src/gameplay.rs`, `engine/src/splash.rs`

---

## tools

### `devtool/src/cli.rs`

- **6** `struct` **Cli** `pub`
- **12** `enum` **Command** `pub`<br>↳ used in: `editor/src/app.rs`, `editor/src/input/keys.rs`, `editor/src/state/cutscene.rs`, `editor/src/state/editor_pane.rs`, `editor/src/state/effects_browser.rs`, `editor/src/state/mod.rs`, `editor/src/state/project_explorer.rs`, `editor/src/state/scene_run.rs`, `editor/src/state/scenes_browser.rs`, `editor/src/state/start_screen.rs`, `engine-io/src/lib.rs`, `engine-scene-runtime/src/terminal_shell.rs`, `tools/devtool/src/main.rs`
- **29** `enum` **CreateCommand** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **43** `enum` **SchemaCommand** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **51** `enum` **EditCommand** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **57** `struct` **NewModArgs** `pub`<br>↳ used in: `tools/devtool/src/scaffold.rs`
- **69** `struct` **NewSceneArgs** `pub`
- **84** `struct` **NewLayerArgs** `pub`<br>↳ used in: `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **99** `struct` **NewSpriteArgs** `pub`<br>↳ used in: `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **132** `struct` **NewEffectArgs** `pub`<br>↳ used in: `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **153** `struct` **EditSpriteArgs** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`, `tools/devtool/src/main.rs`
- **187** `struct` **SchemaTargetArgs** `pub`<br>↳ used in: `tools/devtool/src/fs_utils.rs`

### `devtool/src/edit.rs`

- **13** `struct` **EditedSprite** `pub`
- **19** `fn` **edit_sprite** `pub`<br>↳ used in: `tools/devtool/src/main.rs`

### `devtool/src/fs_utils.rs`

- **7** `fn` **find_repo_root** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **19** `fn` **ensure_mod_exists** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`, `tools/devtool/src/scaffold.rs`
- **29** `fn` **write_file** `pub`<br>↳ used in: `rust-os/src/commands/net.rs`, `rust-os/src/vfs/seed.rs`, `tools/devtool/src/main.rs`, `tools/devtool/src/scaffold.rs`
- **44** `fn` **resolve_mod_roots** `pub`<br>↳ used in: `tools/devtool/src/main.rs`

### `devtool/src/main.rs`

- **1** `mod` **cli**
- **2** `mod` **edit**
- **3** `mod` **fs_utils**
- **4** `mod` **scaffold**
- **5** `mod` **schema**
- **114** `mod` **tests**

### `devtool/src/scaffold.rs`

- **9** `fn` **create_mod_scaffold** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **41** `fn` **create_scene_scaffold** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **69** `fn` **create_layer_scaffold** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **90** `struct` **CreatedSprite** `pub`
- **97** `fn` **create_sprite_scaffold** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **144** `fn` **create_effect_scaffold** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **163** `fn` **default_scene_id** `pub`<br>↳ used in: `tools/devtool/src/main.rs`
- **173** `fn` **human_title** `pub`
- **338** `fn` **find_target_layer** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`
- **357** `fn` **find_sprite_index** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`
- **416** `fn` **load_yaml_document_with_header** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`
- **425** `fn` **write_yaml_document_with_header** `pub`<br>↳ used in: `tools/devtool/src/edit.rs`

### `devtool/src/schema.rs`

- **7** `fn` **sync_fragment_for_mod** `pub`<br>↳ used in: `tools/devtool/src/main.rs`

### `schema-gen/src/main.rs`

- **13** `struct` **Cli**
- **38** `fn` **resolve_mod_roots**
- **59** `fn` **sync_fragment_for_mod**
- **172** `struct` **ParamEntry**
- **363** `mod` **tests**

### `sound-server/src/main.rs`

- **12** `struct` **Cli**
- **26** `enum` **SoundServerRequest**
- **36** `enum` **SoundServerResponse**
- **85** `struct` **AudioPlayer**
- **92** `impl` **AudioPlayer** — *impl AudioPlayer*
