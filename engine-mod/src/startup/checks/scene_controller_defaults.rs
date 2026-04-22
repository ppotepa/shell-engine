//! Validates scene controller-defaults rollout, preset readiness, and resolved bootstrap visibility.

use engine_behavior::catalog::ModCatalogs;
use engine_core::scene::model::{CelestialClockSource, CelestialFrame, SceneWorldModel};
use engine_core::scene::Scene;
use engine_error::EngineError;
use serde_yaml::{Mapping, Value};
use std::path::Path;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;
use super::architecture_snapshot::collect_mod_snapshot;

/// Startup check that validates `controller-defaults` rollout and warns about
/// scenes that still depend on transitional camera authoring.
pub struct SceneControllerDefaultsCheck;

#[derive(Debug, Clone, Default)]
struct AuthoredCameraSurface {
    camera_rig_present: bool,
    raw_camera_input_blocks: Vec<&'static str>,
}

impl AuthoredCameraSurface {
    fn camera_rig_only(&self) -> bool {
        self.camera_rig_present && self.raw_camera_input_blocks.is_empty()
    }
}

fn string_key(value: &str) -> Value {
    Value::String(value.to_string())
}

fn mapping_has_non_null(map: &Mapping, aliases: &[&str]) -> bool {
    aliases.iter().any(|alias| {
        map.get(string_key(alias))
            .is_some_and(|value| !matches!(value, Value::Null))
    })
}

fn authored_scene_mapping(mod_source: &Path, scene_path: &str) -> Option<Mapping> {
    if !mod_source.is_dir() {
        return None;
    }
    let relative = scene_path.trim_start_matches('/');
    let raw = std::fs::read_to_string(mod_source.join(relative)).ok()?;
    serde_yaml::from_str::<Mapping>(&raw).ok()
}

fn authored_camera_surface(mod_source: &Path, scene_path: &str) -> AuthoredCameraSurface {
    let Some(root) = authored_scene_mapping(mod_source, scene_path) else {
        return AuthoredCameraSurface::default();
    };
    let camera_rig_present = root
        .get(string_key("camera-rig"))
        .or_else(|| root.get(string_key("camera_rig")))
        .is_some();
    let raw_camera_input_blocks = root
        .get(string_key("input"))
        .and_then(Value::as_mapping)
        .map(|input| {
            let mut blocks = Vec::new();
            if mapping_has_non_null(input, &["obj-viewer", "obj_viewer"]) {
                blocks.push("input.obj-viewer");
            }
            if mapping_has_non_null(input, &["orbit-camera", "orbit_camera"]) {
                blocks.push("input.orbit-camera");
            }
            if mapping_has_non_null(input, &["free-look-camera", "free_look_camera"]) {
                blocks.push("input.free-look-camera");
            }
            blocks
        })
        .unwrap_or_default();

    AuthoredCameraSurface {
        camera_rig_present,
        raw_camera_input_blocks,
    }
}

fn route_label(hint: Option<&str>) -> &'static str {
    let Some(hint) = hint
        .map(str::trim)
        .filter(|hint| !hint.is_empty() && *hint != "-")
    else {
        return "not-requested";
    };
    if hint.contains("pending runtime preset registry") {
        "pending-registry"
    } else if hint.contains("pending runtime bootstrap resolver") {
        "pending-resolver"
    } else {
        "resolved"
    }
}

fn scene_bootstrap_selection_hint(
    scene: &Scene,
    catalogs: Option<&ModCatalogs>,
    authored_camera: &AuthoredCameraSurface,
) -> String {
    let camera = scene
        .controller_defaults
        .camera_preset
        .as_deref()
        .map(|preset| {
            resolved_camera_registry_note(scene, catalogs, authored_camera, preset)
                .unwrap_or_else(|| format!("pending runtime preset registry for `{preset}`"))
        })
        .unwrap_or_else(|| "-".to_string());
    let player = scene
        .controller_defaults
        .player_preset
        .as_ref()
        .map(|preset| {
            resolved_player_registry_note(catalogs, preset).unwrap_or_else(|| {
                if catalogs.is_some() {
                    format!("pending runtime preset registry for `{preset}`")
                } else {
                    format!(
                        "bootstrap target selection via controlled gameplay entity for `{preset}`"
                    )
                }
            })
        })
        .unwrap_or_else(|| "-".to_string());
    let ui = scene
        .controller_defaults
        .ui_preset
        .as_ref()
        .map(|preset| {
            resolved_ui_registry_note(catalogs, preset)
                .unwrap_or_else(|| format!("pending runtime preset registry for `{preset}`"))
        })
        .unwrap_or_else(|| "-".to_string());
    let frame = resolved_reference_frame_hint(scene).unwrap_or_else(|| "-".to_string());
    let spawn = scene
        .controller_defaults
        .spawn_preset
        .as_ref()
        .map(|preset| {
            resolved_reference_frame_hint(scene)
                .map(|hint| format!("{hint} (spawn-preset=`{preset}`)"))
                .unwrap_or_else(|| format!("pending runtime bootstrap resolver for `{preset}`"))
        })
        .unwrap_or_else(|| "-".to_string());
    let gravity_selection = scene
        .controller_defaults
        .gravity_preset
        .as_ref()
        .map(|preset| {
            scene
                .celestial
                .focus_body
                .as_deref()
                .map(|body| format!("point(body={body}) (gravity-preset=`{preset}`)"))
                .unwrap_or_else(|| format!("pending runtime bootstrap resolver for `{preset}`"))
        })
        .unwrap_or_else(|| "-".to_string());
    let surface = scene
        .controller_defaults
        .surface_preset
        .as_ref()
        .map(|preset| {
            resolved_reference_frame_hint(scene)
                .map(|hint| format!("{hint} (surface-preset=`{preset}`)"))
                .unwrap_or_else(|| format!("pending runtime bootstrap resolver for `{preset}`"))
        })
        .unwrap_or_else(|| "-".to_string());
    let atmosphere = scene
        .celestial
        .focus_body
        .as_deref()
        .map(|body| format!("body={body}"))
        .unwrap_or_else(|| "-".to_string());
    let gravity_default = scene
        .celestial
        .focus_body
        .as_deref()
        .map(|body| format!("point(body={body})"))
        .unwrap_or_else(|| "-".to_string());
    let clock = match scene.celestial.clock_source {
        CelestialClockSource::Scene => "scene",
        CelestialClockSource::Campaign => "campaign",
        CelestialClockSource::Fixed => "fixed",
    };
    let mut notes = Vec::new();
    if scene.world_model == SceneWorldModel::Celestial3D && scene.celestial.focus_body.is_none() {
        notes.push("celestial world-model without focus-body binding".to_string());
    }
    if let Some(preset) = scene.controller_defaults.camera_preset.as_deref() {
        if resolved_camera_registry_note(scene, catalogs, authored_camera, preset).is_none() {
            notes.push(format!(
                "camera-preset=`{preset}` is pending runtime preset registry"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.ui_preset.as_deref() {
        if resolved_ui_registry_note(catalogs, value).is_none() {
            notes.push(format!(
                "ui-preset=`{value}` is pending runtime preset registry"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.player_preset.as_deref() {
        if catalogs.is_some() && resolved_player_registry_note(catalogs, value).is_none() {
            notes.push(format!(
                "player-preset=`{value}` is pending runtime preset registry"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.spawn_preset.as_deref() {
        if resolved_reference_frame_hint(scene).is_none() {
            notes.push(format!(
                "spawn-preset=`{value}` is pending runtime bootstrap resolver"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.gravity_preset.as_deref() {
        if scene.celestial.focus_body.is_none() {
            notes.push(format!(
                "gravity-preset=`{value}` is pending runtime bootstrap resolver"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.surface_preset.as_deref() {
        if resolved_reference_frame_hint(scene).is_none() {
            notes.push(format!(
                "surface-preset=`{value}` is pending runtime bootstrap resolver"
            ));
        }
    }

    format!(
        "scene={} render_space={:?} world_model={:?} clock={} selection[camera={},player={},ui={},spawn={},gravity={},surface={}] routes[camera={},player={},ui={},spawn={},gravity={},surface={}] defaults[frame={},gravity={},atmo={}] notes={}",
        scene.id,
        scene.space,
        scene.world_model,
        clock,
        camera,
        player,
        ui,
        spawn,
        gravity_selection,
        surface,
        route_label(Some(&camera)),
        route_label(Some(&player)),
        route_label(Some(&ui)),
        route_label(Some(&spawn)),
        route_label(Some(&gravity_selection)),
        route_label(Some(&surface)),
        frame,
        gravity_default,
        atmosphere,
        if notes.is_empty() {
            "-".to_string()
        } else {
            notes.join(" | ")
        },
    )
}

fn resolved_reference_frame_hint(scene: &Scene) -> Option<String> {
    if scene.world_model != SceneWorldModel::Celestial3D || scene.celestial.focus_body.is_none() {
        return None;
    }

    let mode = if matches!(
        scene.controller_defaults.spawn_preset.as_deref(),
        Some("planetary-surface")
    ) || matches!(
        scene.controller_defaults.surface_preset.as_deref(),
        Some("local-horizon" | "planetary-surface")
    ) || matches!(scene.celestial.frame, CelestialFrame::SurfaceLocal)
    {
        "local-horizon"
    } else {
        "celestial-body"
    };

    Some(format!(
        "{mode}(body={})",
        scene.celestial.focus_body.as_deref().unwrap_or("-")
    ))
}

fn resolved_camera_registry_note(
    scene: &Scene,
    catalogs: Option<&ModCatalogs>,
    authored_camera: &AuthoredCameraSurface,
    preset: &str,
) -> Option<String> {
    let normalized = preset.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "obj-viewer" if authored_camera.camera_rig_only() => {
            Some("camera-rig.obj-viewer canonical route".to_string())
        }
        "orbit-camera" if authored_camera.camera_rig_only() => {
            Some("camera-rig.orbit-camera canonical route".to_string())
        }
        "free-look-camera" if authored_camera.camera_rig_only() => {
            Some("camera-rig.free-look-camera canonical route".to_string())
        }
        "surface-free-look" if authored_camera.camera_rig_only() => Some(
            "camera-rig.free-look-camera canonical route (surface-mode via camera-rig.surface.mode=locked)"
                .to_string(),
        ),
        "obj-viewer" if scene.input.obj_viewer.is_some() => {
            Some("legacy input.obj-viewer compatibility route".to_string())
        }
        "orbit-camera" if scene.input.orbit_camera.is_some() => {
            Some("legacy input.orbit-camera compatibility route".to_string())
        }
        "free-look-camera" if scene.input.free_look_camera.is_some() => {
            Some("legacy input.free-look-camera compatibility route".to_string())
        }
        "surface-free-look"
            if scene
                .input
                .free_look_camera
                .as_ref()
                .is_some_and(|controls| controls.surface_mode) =>
        {
            Some("legacy input.free-look-camera compatibility route (surface-mode)".to_string())
        }
        _ => catalogs
            .and_then(|catalogs| catalogs.presets.camera(preset))
            .map(|preset_def| {
                format!(
                    "catalog camera preset `{}` -> {}(target={})",
                    preset,
                    preset_def.controller_kind.as_deref().unwrap_or("-"),
                    preset_def.target.as_deref().unwrap_or("-"),
                )
            }),
    }
}

fn resolved_ui_registry_note(catalogs: Option<&ModCatalogs>, preset: &str) -> Option<String> {
    catalogs
        .and_then(|catalogs| catalogs.presets.ui(preset))
        .map(|preset_def| {
            format!(
                "catalog ui preset `{}` -> layout={}",
                preset,
                preset_def.layout.as_deref().unwrap_or("-"),
            )
        })
}

fn resolved_player_registry_note(catalogs: Option<&ModCatalogs>, preset: &str) -> Option<String> {
    let preset_def = catalogs.and_then(|catalogs| catalogs.presets.player(preset))?;
    let mut actions = Vec::new();
    if preset_def
        .config
        .get("controlled")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        actions.push("controlled gameplay entity".to_string());
    }
    if preset_def.components.is_some() {
        actions.push("bootstrap assembly".to_string());
    }
    if let Some(controller_type) = preset_def
        .controller
        .as_ref()
        .map(|controller| controller.controller_type.as_str())
    {
        actions.push(format!("controller={controller_type}"));
    }
    if let Some(input_profile) = preset_def.input_profile.as_deref() {
        actions.push(format!("input={input_profile}"));
    }
    let action_summary = if actions.is_empty() {
        "metadata only".to_string()
    } else {
        actions.join(" + ")
    };
    Some(format!(
        "catalog player preset `{preset}` -> {action_summary}"
    ))
}

impl StartupCheck for SceneControllerDefaultsCheck {
    fn name(&self) -> &'static str {
        "scene-controller-defaults"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let scenes = ctx.all_scenes()?;
        let catalogs = ModCatalogs::load_from_directory(&ctx.mod_source().join("catalogs")).ok();
        let mut scenes_with_defaults = 0usize;
        let mut warning_count = 0usize;

        for scene_file in scenes {
            let scene = &scene_file.scene;
            let authored_camera = authored_camera_surface(ctx.mod_source(), &scene_file.path);
            let legacy_camera_blocks = legacy_camera_blocks(scene, &authored_camera);
            let has_controller_defaults = scene.controller_defaults != Default::default();
            if has_controller_defaults {
                scenes_with_defaults += 1;
            }

            if has_controller_defaults || scene.world_model == SceneWorldModel::Celestial3D {
                report.add_info(
                    self.name(),
                    format!(
                        "resolved bootstrap: {}",
                        scene_bootstrap_selection_hint(scene, catalogs.as_ref(), &authored_camera)
                    ),
                );
            }

            if scene.world_model == SceneWorldModel::Celestial3D
                && scene.celestial.focus_body.is_none()
            {
                warning_count += 1;
                report.add_warning(
                    self.name(),
                    format!(
                        "scene `{}` uses `world-model: celestial-3d` without a focus-body binding; resolved bootstrap cannot derive the default reference frame, gravity, or atmosphere yet",
                        scene_file.path
                    ),
                );
            }

            if has_controller_defaults && !legacy_camera_blocks.is_empty() {
                warning_count += 1;
                report.add_warning(
                    self.name(),
                    format!(
                        "scene `{}` mixes `controller-defaults` with legacy camera authoring ({}) — `controller-defaults` is canonical and legacy `input.*camera` blocks are transitional",
                        scene_file.path,
                        legacy_camera_blocks.join(", ")
                    ),
                );
            }

            if let Some(message) = camera_policy_world_model_warning(scene, &scene_file.path) {
                warning_count += 1;
                report.add_warning(self.name(), message);
            }

            if let Some(message) = clock_source_runtime_note(scene, &scene_file.path) {
                report.add_info(self.name(), message);
            }

            let unresolved = unresolved_defaults(
                scene,
                catalogs.as_ref(),
                &authored_camera,
                &legacy_camera_blocks,
            );
            if !unresolved.is_empty() {
                warning_count += 1;
                report.add_warning(
                    self.name(),
                    format!(
                        "scene `{}` has controller defaults that are not startup-resolvable yet: {}",
                        scene_file.path,
                        unresolved.join(", ")
                    ),
                );
            }
        }

        report.add_info(
            self.name(),
            format!(
                "scene controller-defaults inspected ({} scenes, {} scene(s) using controller-defaults, {} warning(s))",
                scenes.len(),
                scenes_with_defaults,
                warning_count
            ),
        );

        if let Some(snapshot) = collect_mod_snapshot(ctx.mod_source()).map_err(|details| {
            EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details,
            }
        })? {
            report.add_info(
                self.name(),
                format!("architecture snapshot: {}", snapshot.format_summary()),
            );
        } else {
            report.add_info(
                self.name(),
                "architecture snapshot skipped for non-directory mod source".to_string(),
            );
        }

        Ok(())
    }
}

fn legacy_camera_blocks(
    scene: &Scene,
    authored_camera: &AuthoredCameraSurface,
) -> Vec<&'static str> {
    if authored_camera.camera_rig_only() {
        return Vec::new();
    }
    if authored_camera.camera_rig_present {
        return authored_camera.raw_camera_input_blocks.clone();
    }

    let mut blocks = Vec::new();
    if scene.input.obj_viewer.is_some() {
        blocks.push("input.obj-viewer");
    }
    if scene.input.orbit_camera.is_some() {
        blocks.push("input.orbit-camera");
    }
    if scene.input.free_look_camera.is_some() {
        blocks.push("input.free-look-camera");
    }
    blocks
}

fn unresolved_defaults(
    scene: &Scene,
    catalogs: Option<&ModCatalogs>,
    authored_camera: &AuthoredCameraSurface,
    legacy_camera_blocks: &[&str],
) -> Vec<String> {
    let mut unresolved = Vec::new();

    if let Some(camera_preset) = scene.controller_defaults.camera_preset.as_deref() {
        if let Some(reason) =
            unresolved_camera_preset_reason(scene, catalogs, authored_camera, legacy_camera_blocks)
        {
            unresolved.push(format!("camera-preset=`{camera_preset}` ({reason})"));
        }
    }

    if let Some(value) = scene.controller_defaults.ui_preset.as_deref() {
        if resolved_ui_registry_note(catalogs, value).is_none() {
            unresolved.push(format!(
                "ui-preset=`{value}` (runtime bootstrap resolver not implemented yet)"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.player_preset.as_deref() {
        if catalogs.is_some() && resolved_player_registry_note(catalogs, value).is_none() {
            unresolved.push(format!(
                "player-preset=`{value}` (runtime preset registry not implemented yet)"
            ));
        }
    }

    let reference_frame_resolved = resolved_reference_frame_hint(scene).is_some();
    if let Some(value) = scene.controller_defaults.spawn_preset.as_deref() {
        if !reference_frame_resolved {
            unresolved.push(format!(
                "spawn-preset=`{value}` (runtime bootstrap resolver not implemented yet)"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.gravity_preset.as_deref() {
        if scene.celestial.focus_body.is_none() {
            unresolved.push(format!(
                "gravity-preset=`{value}` (runtime bootstrap resolver not implemented yet)"
            ));
        }
    }
    if let Some(value) = scene.controller_defaults.surface_preset.as_deref() {
        if !reference_frame_resolved {
            unresolved.push(format!(
                "surface-preset=`{value}` (runtime bootstrap resolver not implemented yet)"
            ));
        }
    }

    unresolved
}

fn unresolved_camera_preset_reason(
    scene: &Scene,
    catalogs: Option<&ModCatalogs>,
    authored_camera: &AuthoredCameraSurface,
    legacy_camera_blocks: &[&str],
) -> Option<String> {
    let preset = scene.controller_defaults.camera_preset.as_deref()?;
    if catalogs
        .and_then(|catalogs| catalogs.presets.camera(preset))
        .is_some()
    {
        return None;
    }
    match preset {
        "obj-viewer" if scene.input.obj_viewer.is_some() => None,
        "orbit-camera" if scene.input.orbit_camera.is_some() => None,
        "free-look-camera" if scene.input.free_look_camera.is_some() => None,
        "surface-free-look"
            if scene
                .input
                .free_look_camera
                .as_ref()
                .is_some_and(|controls| controls.surface_mode) =>
        {
            None
        }
        "obj-viewer" => Some("requires `input.obj-viewer` compatibility block".to_string()),
        "orbit-camera" => Some("requires `input.orbit-camera` compatibility block".to_string()),
        "free-look-camera" => {
            Some("requires `input.free-look-camera` compatibility block".to_string())
        }
        "surface-free-look" => Some(
            "requires `input.free-look-camera` compatibility block with `surface-mode: true`"
                .to_string(),
        ),
        _ if authored_camera.camera_rig_only() => Some(
            "no camera preset registry exists yet; current runtime still relies on normalized `camera-rig` compatibility lowering"
                .to_string(),
        ),
        _ if !legacy_camera_blocks.is_empty() => Some(format!(
            "no camera preset registry exists yet; current runtime still relies on {}",
            legacy_camera_blocks.join(", ")
        )),
        _ => Some("no camera preset registry exists yet".to_string()),
    }
}

fn camera_policy_world_model_warning(scene: &Scene, scene_path: &str) -> Option<String> {
    let preset = scene.controller_defaults.camera_preset.as_deref()?;
    match scene.world_model {
        SceneWorldModel::Planar2D => Some(format!(
            "scene `{scene_path}` declares camera policy `{preset}` while `world-model: planar-2d` is active"
        )),
        SceneWorldModel::Euclidean3D if looks_celestial_camera_policy(preset) => Some(format!(
            "scene `{scene_path}` uses camera policy `{preset}` that looks celestial-specific while `world-model: euclidean-3d` is active"
        )),
        _ => None,
    }
}

fn clock_source_runtime_note(scene: &Scene, scene_path: &str) -> Option<String> {
    if scene.world_model != SceneWorldModel::Celestial3D {
        return None;
    }

    match scene.celestial.clock_source {
        CelestialClockSource::Scene => None,
        CelestialClockSource::Campaign => Some(format!(
            "scene `{scene_path}` uses `clock-source: campaign`; resolved bootstrap records the choice and celestial runtime reads `/runtime/celestial/campaign_clock_sec` or `/runtime/celestial/campaign_clock_ms` when present"
        )),
        CelestialClockSource::Fixed => Some(format!(
            "scene `{scene_path}` uses `clock-source: fixed`; resolved bootstrap records the choice and celestial runtime reads `/runtime/celestial/fixed_clock_sec` or `/runtime/celestial/fixed_clock_ms` when present"
        )),
    }
}

fn looks_celestial_camera_policy(preset: &str) -> bool {
    let lower = preset.trim().to_ascii_lowercase();
    lower == "surface-free-look"
        || lower.contains("celestial")
        || lower.contains("cockpit")
        || lower.contains("free-flight")
        || lower.starts_with("surface-")
}

#[cfg(test)]
mod tests {
    use super::SceneControllerDefaultsCheck;
    use crate::startup::{StartupCheck, StartupContext, StartupIssueLevel, StartupReport};
    use engine_core::scene::Scene;
    use engine_error::EngineError;
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    fn run_check(scenes: Vec<crate::startup::StartupSceneFile>) -> StartupReport {
        run_check_with_setup(scenes, |_| {})
    }

    fn run_check_with_setup(
        scenes: Vec<crate::startup::StartupSceneFile>,
        setup: impl FnOnce(&std::path::Path),
    ) -> StartupReport {
        let scene_loader = move |_mod_source: &std::path::Path| -> Result<
            Vec<crate::startup::StartupSceneFile>,
            EngineError,
        > { Ok(scenes.clone()) };

        let mod_dir = tempdir().expect("temp dir");
        setup(mod_dir.path());
        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main/scene.yml\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(
            mod_dir.path(),
            &manifest,
            "/scenes/main/scene.yml",
            &scene_loader,
        );
        let mut report = StartupReport::default();

        SceneControllerDefaultsCheck
            .run(&ctx, &mut report)
            .expect("check should pass");

        report
    }

    #[test]
    fn warns_when_scene_mixes_controller_defaults_with_legacy_camera_authoring() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: mixed
title: Mixed
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: obj-viewer
input:
  obj-viewer:
    sprite_id: probe
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/mixed/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("mixes `controller-defaults` with legacy camera authoring")
        }));
    }

    #[test]
    fn does_not_warn_when_camera_rig_authoring_is_the_only_camera_source() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: camera-rig-only
title: Camera Rig Only
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: orbit-camera
input:
  orbit-camera:
    target: probe
layers: []
"#,
        )
        .expect("scene");
        let report = run_check_with_setup(
            vec![crate::startup::StartupSceneFile {
                path: "/scenes/camera-rig-only/scene.yml".to_string(),
                scene,
            }],
            |mod_root| {
                let scene_dir = mod_root.join("scenes/camera-rig-only");
                fs::create_dir_all(&scene_dir).expect("scene dir");
                fs::write(
                    scene_dir.join("scene.yml"),
                    r#"
id: camera-rig-only
title: Camera Rig Only
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: orbit-camera
camera-rig:
  orbit-camera:
    target: probe
layers: []
"#,
                )
                .expect("write scene");
            },
        );

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("mixes `controller-defaults` with legacy camera authoring")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue
                    .message
                    .contains("camera-rig.orbit-camera canonical route")
        }));
    }

    #[test]
    fn does_not_warn_when_surface_free_look_was_authored_via_camera_rig() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: camera-rig-surface-free-look
title: Camera Rig Surface Free Look
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
input:
  free-look-camera:
    surface-mode: true
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene");
        let report = run_check_with_setup(
            vec![crate::startup::StartupSceneFile {
                path: "/scenes/camera-rig-surface-free-look/scene.yml".to_string(),
                scene,
            }],
            |mod_root| {
                let scene_dir = mod_root.join("scenes/camera-rig-surface-free-look");
                fs::create_dir_all(&scene_dir).expect("scene dir");
                fs::write(
                    scene_dir.join("scene.yml"),
                    r#"
id: camera-rig-surface-free-look
title: Camera Rig Surface Free Look
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
camera-rig:
  preset: surface-free-look
  surface:
    mode: locked
  free-look-camera: {}
celestial:
  focus-body: earth
layers: []
"#,
                )
                .expect("write scene");
            },
        );

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("mixes `controller-defaults` with legacy camera authoring")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains(
                    "camera-rig.free-look-camera canonical route (surface-mode via camera-rig.surface.mode=locked)",
                )
        }));
    }

    #[test]
    fn warns_when_custom_camera_preset_has_no_runtime_resolver() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: unresolved
title: Unresolved
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: custom-orbit
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/unresolved/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("camera-preset=`custom-orbit` (no camera preset registry exists yet)")
        }));
    }

    #[test]
    fn warns_when_ui_bootstrap_default_is_not_runtime_resolvable() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: defaults
title: Defaults
controller-defaults:
  player-preset: pilot
  ui-preset: cockpit-hud
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/defaults/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains(
                    "ui-preset=`cockpit-hud` (runtime bootstrap resolver not implemented yet)",
                )
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue
                    .message
                    .contains("player=pending runtime preset registry for `pilot`")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("player-preset=`pilot`")
        }));
    }

    #[test]
    fn catalog_backed_camera_and_ui_presets_still_report_pending_registry_until_runtime_binding_exists(
    ) {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: catalog-backed-defaults
title: Catalog-backed Defaults
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: cockpit-flight
  ui-preset: flight-hud
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene");
        let report = run_check_with_setup(
            vec![crate::startup::StartupSceneFile {
                path: "/scenes/catalog-backed-defaults/scene.yml".to_string(),
                scene,
            }],
            |mod_root| {
                let catalogs_dir = mod_root.join("catalogs");
                fs::create_dir_all(&catalogs_dir).expect("catalogs dir");
                fs::write(
                    catalogs_dir.join("presets.yaml"),
                    r#"
presets:
  cameras:
    cockpit-flight:
      controller-kind: cockpit
      target: player
  ui:
    flight-hud:
      layout: cockpit
"#,
                )
                .expect("write presets");
            },
        );

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue
                    .message
                    .contains("resolved bootstrap: scene=catalog-backed-defaults")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains(
                    "selection[camera=catalog camera preset `cockpit-flight` -> cockpit(target=player)",
                )
                && issue
                    .message
                    .contains("ui=catalog ui preset `flight-hud` -> layout=cockpit")
        }));
        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("camera-preset=`cockpit-flight`")
        }));
        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("ui-preset=`flight-hud`")
        }));
    }

    #[test]
    fn startup_snapshot_stays_runtime_object_aware_for_catalog_backed_defaults() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-bootstrap
title: Runtime Object Bootstrap
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: cockpit-flight
  ui-preset: flight-hud
runtime-objects:
  - name: pilot
    kind: runtime-object
    transform:
      space: 3d
      translation: [0.0, 1.0, 2.0]
layers: []
"#,
        )
        .expect("scene");
        let report = run_check_with_setup(
            vec![crate::startup::StartupSceneFile {
                path: "/scenes/runtime-object-bootstrap/scene.yml".to_string(),
                scene,
            }],
            |mod_root| {
                let scene_dir = mod_root.join("scenes/runtime-object-bootstrap");
                fs::create_dir_all(&scene_dir).expect("scene dir");
                fs::write(
                    scene_dir.join("scene.yml"),
                    r#"
id: runtime-object-bootstrap
title: Runtime Object Bootstrap
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: cockpit-flight
  ui-preset: flight-hud
runtime-objects:
  - name: pilot
    kind: runtime-object
    transform:
      space: 3d
      translation: [0.0, 1.0, 2.0]
layers: []
"#,
                )
                .expect("write scene");
                let catalogs_dir = mod_root.join("catalogs");
                fs::create_dir_all(&catalogs_dir).expect("catalogs dir");
                fs::write(
                    catalogs_dir.join("presets.yaml"),
                    r#"
presets:
  cameras:
    cockpit-flight:
      controller-kind: cockpit
      target: player
  ui:
    flight-hud:
      layout: cockpit
"#,
                )
                .expect("write presets");
            },
        );

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("mixes `controller-defaults` with legacy camera authoring")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains(
                    "selection[camera=catalog camera preset `cockpit-flight` -> cockpit(target=player)",
                )
                && issue
                    .message
                    .contains("ui=catalog ui preset `flight-hud` -> layout=cockpit")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains("architecture snapshot:")
                && issue
                    .message
                    .contains("controller-defaults=1 files / fields=camera-preset:1, ui-preset:1")
                && issue.message.contains("legacy camera blocks=none")
                && issue
                    .message
                    .contains("objects=0 files / 0 sequences / 0 instances")
        }));
    }

    #[test]
    fn runtime_object_scene_reports_catalog_backed_player_bootstrap_without_legacy_object_debt() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-player-bootstrap
title: Runtime Object Player Bootstrap
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
layers: []
"#,
        )
        .expect("scene");
        let report = run_check_with_setup(
            vec![crate::startup::StartupSceneFile {
                path: "/scenes/runtime-object-player-bootstrap/scene.yml".to_string(),
                scene,
            }],
            |mod_root| {
                let scene_dir = mod_root.join("scenes/runtime-object-player-bootstrap");
                fs::create_dir_all(&scene_dir).expect("scene dir");
                fs::write(
                    scene_dir.join("scene.yml"),
                    r#"
id: runtime-object-player-bootstrap
title: Runtime Object Player Bootstrap
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
layers: []
"#,
                )
                .expect("write scene");
                let catalogs_dir = mod_root.join("catalogs");
                fs::create_dir_all(&catalogs_dir).expect("catalogs dir");
                fs::write(
                    catalogs_dir.join("presets.yaml"),
                    r#"
presets:
  players:
    flight-player:
      input_profile: default-flight
      controller:
        controller_type: VehicleAssembly
      config:
        controlled: true
      components: {}
"#,
                )
                .expect("write presets");
            },
        );

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("player-preset=`flight-player`")
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains(
                    "selection[camera=-,player=catalog player preset `flight-player` -> controlled gameplay entity + bootstrap assembly + controller=VehicleAssembly + input=default-flight",
                )
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains("architecture snapshot:")
                && issue
                    .message
                    .contains("controller-defaults=1 files / fields=player-preset:1")
                && issue.message.contains("legacy camera blocks=none")
                && issue
                    .message
                    .contains("objects=0 files / 0 sequences / 0 instances / 0 repeat groups / 0 repeat instances")
        }));
    }

    #[test]
    fn warns_when_celestial_scene_has_no_focus_body_binding() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: celestial
title: Celestial
render-space: 3d
world-model: celestial-3d
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/celestial/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("uses `world-model: celestial-3d` without a focus-body binding")
        }));
    }

    #[test]
    fn warns_when_euclidean_scene_uses_celestial_looking_camera_policy() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: mismatch
title: Mismatch
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: celestial-orbit-inspector
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/mismatch/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains(
                    "looks celestial-specific while `world-model: euclidean-3d` is active",
                )
        }));
    }

    #[test]
    fn reports_clock_source_runtime_dependency_for_celestial_scenes() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: clocked
title: Clocked
render-space: 3d
world-model: celestial-3d
celestial:
  focus-body: earth
  clock-source: campaign
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/clocked/scene.yml".to_string(),
            scene,
        }]);

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue.message.contains("uses `clock-source: campaign`")
        }));
    }

    #[test]
    fn reports_resolved_bootstrap_summary_for_celestial_controller_defaults() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: resolved-bootstrap
title: Resolved Bootstrap
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: cockpit-flight
  player-preset: celestial-free-flight
  ui-preset: flight-hud
  spawn-preset: planetary-surface
  gravity-preset: radial-body
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
  clock-source: campaign
layers: []
"#,
        )
        .expect("scene");
        let report = run_check(vec![crate::startup::StartupSceneFile {
            path: "/scenes/resolved-bootstrap/scene.yml".to_string(),
            scene,
        }]);

        let resolved = report.issues().iter().find(|issue| {
            issue.level == StartupIssueLevel::Info
                && issue
                    .message
                    .starts_with("resolved bootstrap: scene=resolved-bootstrap")
        });
        assert!(resolved.is_some(), "expected resolved bootstrap info");
        let resolved = resolved.expect("resolved bootstrap info");
        assert!(resolved
            .message
            .contains("selection[camera=pending runtime preset registry for `cockpit-flight`"));
        assert!(resolved.message.contains(
            "routes[camera=pending-registry,player=pending-registry,ui=pending-registry,spawn=resolved,gravity=resolved,surface=resolved]"
        ));
        assert!(resolved
            .message
            .contains("player=pending runtime preset registry for `celestial-free-flight`"));
        assert!(resolved.message.contains(
            "defaults[frame=local-horizon(body=earth),gravity=point(body=earth),atmo=body=earth]"
        ));
        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && (issue.message.contains("spawn-preset=")
                    || issue.message.contains("gravity-preset=")
                    || issue.message.contains("surface-preset="))
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue
                    .message
                    .contains("player-preset=`celestial-free-flight`")
        }));
    }
}
