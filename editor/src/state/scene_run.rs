//! Scene run mode: play selected scene from Scenes Browser inside the editor.

use std::path::Path;

use engine::animation::{animator_system, Animator};
use engine::asset::{create_scene_repository, SceneRepository};
use engine::assets::AssetRoot;
use engine::audio::audio_system;
use engine::audio::AudioRuntime;
use engine::buffer::Buffer;
use engine::debug_features::DebugFeatures;
use engine::events::{EngineEvent, EventQueue};
use engine::runtime_settings::{RenderSize, RuntimeSettings};
use engine::scene::Scene;
use engine::scene_runtime::SceneRuntime;
use engine::systems::behavior::behavior_system;
use engine::systems::compositor::compositor_system;
use engine::systems::postfx::postfx_system;
use engine::systems::scene_lifecycle::SceneLifecycleManager;
use engine::world::World;
use engine_core::logging;
use engine_events::KeyEvent;

use crate::input::commands::Command;

use super::{AppMode, AppState, SceneRunKind, SidebarItem};

impl AppState {
    pub(super) fn start_scene_soft_run(&mut self) {
        self.start_scene_run(SceneRunKind::Soft);
    }

    pub(super) fn start_scene_hard_run(&mut self) {
        self.start_scene_run(SceneRunKind::Hard);
    }

    fn start_scene_run(&mut self, kind: SceneRunKind) {
        if self.sidebar.active != SidebarItem::Scenes {
            return;
        }
        logging::info(
            "editor.scene-run",
            format!(
                "start requested: kind={:?} mod={} cursor={}",
                kind, self.mod_source, self.scenes.scene_cursor
            ),
        );
        if self.mod_source.is_empty() {
            self.status = String::from("Scene Run: open a mod project first");
            logging::warn("editor.scene-run", "start rejected: no open mod project");
            return;
        }

        let Some(scene_path) = self.selected_scene_path().map(str::to_string) else {
            self.status = String::from("Scene Run: no scene selected");
            logging::warn("editor.scene-run", "start rejected: no selected scene");
            return;
        };
        let scene_ref = self.normalize_scene_ref_path(&scene_path);
        let scene_name = self
            .selected_scene_display_name()
            .unwrap_or_else(|| scene_ref.clone());

        let scene = match create_scene_repository(Path::new(&self.mod_source))
            .and_then(|repo| repo.load_scene(&scene_ref))
        {
            Ok(scene) => scene,
            Err(err) => {
                self.status = format!("Scene Run: cannot load scene ({err})");
                logging::error(
                    "editor.scene-run",
                    format!("failed to load scene for run: ref={scene_ref} error={err}"),
                );
                return;
            }
        };

        let mut world = World::new();
        world.register(EventQueue::new());
        world.register(Buffer::new(2, 2));
        world.register(AudioRuntime::null());
        world.register(load_runtime_settings(Path::new(&self.mod_source)));
        world.register(DebugFeatures::from_enabled(true));
        world.register(AssetRoot::new(Path::new(&self.mod_source).to_path_buf()));
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator::new());

        // Initial frame before first tick.
        behavior_system(&mut world);
        compositor_system(&mut world);
        postfx_system(&mut world);

        self.scene_run.kind = kind;
        self.scene_run.scene_path = scene_ref;
        self.scene_run.scene_name = scene_name;
        self.scene_run.last_tick_ms = super::now_millis();
        self.scene_run.world = Some(world);
        self.mode = AppMode::SceneRun;
        let mode_label = match kind {
            SceneRunKind::Soft => "SOFT RUN",
            SceneRunKind::Hard => "RUN",
        };
        self.status = format!(
            "{mode_label}: {} | mod={} | path={} | Esc back",
            self.scene_run.scene_name, self.mod_source, self.scene_run.scene_path
        );
        logging::info(
            "editor.scene-run",
            format!(
                "started: kind={:?} scene={} path={} mod={}",
                kind, self.scene_run.scene_name, self.scene_run.scene_path, self.mod_source
            ),
        );
    }

    pub(super) fn stop_scene_run(&mut self) {
        logging::info(
            "editor.scene-run",
            format!(
                "stopping run: kind={:?} scene={} path={}",
                self.scene_run.kind, self.scene_run.scene_name, self.scene_run.scene_path
            ),
        );
        self.scene_run.world = None;
        self.scene_run.last_tick_ms = 0;
        self.scene_run.kind = SceneRunKind::Soft;
        self.mode = AppMode::Browser;
        if self.sidebar.active == SidebarItem::Scenes {
            self.status = self.scene_browser_status_message();
        } else {
            self.status = String::from("Scene Run stopped");
        }
    }

    pub fn enqueue_scene_run_key(&mut self, key: KeyEvent) {
        if self.mode != AppMode::SceneRun {
            return;
        }
        let Some(world) = self.scene_run.world.as_mut() else {
            return;
        };
        if let Some(queue) = world.get_mut::<EventQueue>() {
            queue.push(EngineEvent::KeyDown { key, repeat: false });
        }
    }

    pub fn enqueue_scene_run_resize(&mut self, width: u16, height: u16) {
        if self.mode != AppMode::SceneRun {
            return;
        }
        let Some(world) = self.scene_run.world.as_mut() else {
            return;
        };
        if let Some(queue) = world.get_mut::<EventQueue>() {
            queue.push(EngineEvent::OutputResized { width, height });
        }
    }

    pub fn ensure_scene_run_buffer_size(&mut self, width: u16, height: u16) {
        if self.mode != AppMode::SceneRun {
            return;
        }
        let Some(world) = self.scene_run.world.as_mut() else {
            return;
        };

        let target_w = width.max(2);
        let target_h = height.max(2);
        let tracks_output = world
            .get::<RuntimeSettings>()
            .map(|settings| settings.render_size_matches_output())
            .unwrap_or(true);
        if !tracks_output {
            return;
        }
        let resize_needed = world
            .get::<Buffer>()
            .map(|buffer| buffer.width != target_w || buffer.height != target_h)
            .unwrap_or(true);

        if resize_needed {
            world.register(Buffer::new(target_w, target_h));
            compositor_system(world);
            postfx_system(world);
        }
    }

    pub fn scene_run_buffer(&self) -> Option<&Buffer> {
        self.scene_run.world.as_ref()?.get::<Buffer>()
    }

    pub(super) fn tick_scene_run(&mut self, dt_secs: f32) {
        if self.mode != AppMode::SceneRun {
            return;
        }

        let Some(mut world) = self.scene_run.world.take() else {
            return;
        };

        let mut tick_ms = (dt_secs * 1000.0).round() as u64;
        if tick_ms == 0 {
            tick_ms = 1;
        }

        // Clamp very large frame skips to keep stage progression smooth.
        let mut remaining = tick_ms.min(250);
        while remaining > 0 {
            let step_ms = remaining.min(33);
            if let Some(queue) = world.get_mut::<EventQueue>() {
                queue.push(EngineEvent::Tick);
            }
            let pre_events = world
                .get_mut::<EventQueue>()
                .map(EventQueue::drain)
                .unwrap_or_default();
            let (pre_regular, pre_transitions) = split_transition_events(pre_events);
            let quit_pre = SceneLifecycleManager::process_events(&mut world, pre_regular);
            if self.scene_run.kind == SceneRunKind::Hard {
                self.apply_scene_run_transitions(&mut world, pre_transitions);
            }
            if quit_pre {
                self.stop_scene_run();
                return;
            }

            animator_system(&mut world, step_ms);
            let post_events = world
                .get_mut::<EventQueue>()
                .map(EventQueue::drain)
                .unwrap_or_default();
            let (post_regular, post_transitions) = split_transition_events(post_events);
            let quit_post = SceneLifecycleManager::process_events(&mut world, post_regular);
            if self.scene_run.kind == SceneRunKind::Hard {
                self.apply_scene_run_transitions(&mut world, post_transitions);
            }
            if quit_post {
                self.stop_scene_run();
                return;
            }

            behavior_system(&mut world);
            audio_system(&mut world);
            compositor_system(&mut world);
            postfx_system(&mut world);
            remaining -= step_ms;
        }

        self.scene_run.world = Some(world);
    }

    fn apply_scene_run_transitions(&mut self, world: &mut World, transitions: Vec<String>) {
        for to_scene_ref in transitions {
            logging::debug(
                "editor.scene-run",
                format!("hard-run transition requested: to={to_scene_ref}"),
            );
            let Some((scene, scene_ref, scene_name, scene_index)) =
                self.resolve_scene_transition(&to_scene_ref)
            else {
                logging::warn(
                    "editor.scene-run",
                    format!("hard-run transition unresolved: to={to_scene_ref}"),
                );
                continue;
            };
            world.clear_scoped();
            world.register_scoped(SceneRuntime::new(scene));
            world.register_scoped(Animator::new());

            if let Some(idx) = scene_index {
                self.scenes.scene_cursor = idx;
                self.sync_scene_preview_selection();
            }
            self.scene_run.scene_path = scene_ref;
            self.scene_run.scene_name = scene_name;
            logging::info(
                "editor.scene-run",
                format!(
                    "hard-run transition applied: scene={} path={}",
                    self.scene_run.scene_name, self.scene_run.scene_path
                ),
            );
        }
    }

    fn resolve_scene_transition(
        &self,
        to_scene_ref: &str,
    ) -> Option<(Scene, String, String, Option<usize>)> {
        let repo = create_scene_repository(Path::new(&self.mod_source)).ok()?;

        if let Ok(scene) = repo.load_scene(to_scene_ref) {
            let scene_ref = normalize_scene_ref(to_scene_ref);
            let scene_name = preferred_scene_name(&scene);
            let scene_index = self
                .index
                .scenes
                .scene_paths
                .iter()
                .position(|path| self.normalize_scene_ref_path(path) == scene_ref);
            return Some((scene, scene_ref, scene_name, scene_index));
        }

        for (idx, scene_path) in self.index.scenes.scene_paths.iter().enumerate() {
            let scene_ref = self.normalize_scene_ref_path(scene_path);
            let Ok(scene) = repo.load_scene(&scene_ref) else {
                continue;
            };
            if scene.id == to_scene_ref {
                let scene_name = preferred_scene_name(&scene);
                return Some((scene, scene_ref, scene_name, Some(idx)));
            }
        }
        None
    }

    pub(super) fn handle_scene_run_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => true,
            Command::Back => {
                self.stop_scene_run();
                false
            }
            _ => false,
        }
    }
}

fn load_runtime_settings(mod_root: &Path) -> RuntimeSettings {
    let manifest_path = mod_root.join("mod.yaml");
    let mut settings = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|raw| serde_yaml::from_str::<serde_yaml::Value>(&raw).ok())
        .map(|manifest| RuntimeSettings::from_manifest(&manifest))
        .unwrap_or_default();
    settings.render_size = RenderSize::MatchOutput;
    settings
}

fn split_transition_events(events: Vec<EngineEvent>) -> (Vec<EngineEvent>, Vec<String>) {
    let mut regular = Vec::with_capacity(events.len());
    let mut transitions = Vec::new();
    for event in events {
        match event {
            EngineEvent::SceneTransition { to_scene_id } => transitions.push(to_scene_id),
            other => regular.push(other),
        }
    }
    (regular, transitions)
}

fn normalize_scene_ref(scene_ref: &str) -> String {
    let normalized = scene_ref.replace('\\', "/");
    if normalized.starts_with('/') {
        normalized
    } else {
        format!("/{}", normalized.trim_start_matches('/'))
    }
}

fn preferred_scene_name(scene: &Scene) -> String {
    if !scene.title.trim().is_empty() {
        scene.title.clone()
    } else if !scene.id.trim().is_empty() {
        scene.id.clone()
    } else {
        String::from("<scene>")
    }
}
