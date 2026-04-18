use std::path::{Path, PathBuf};

use engine_core::scene::{LightingProfile, Scene, SpaceEnvironmentProfile, ViewProfile};
use serde::de::DeserializeOwned;

use crate::repositories::AssetRepository;

const LIGHTING_PROFILE_DIR: &str = "/lighting-profiles";
const SPACE_ENVIRONMENT_PROFILE_DIR: &str = "/space-environment-profiles";
const VIEW_PROFILE_DIR: &str = "/view-profiles";

pub fn hydrate_scene_view_profiles<R: AssetRepository>(
    repo: &R,
    scene: &mut Scene,
) -> Result<(), engine_error::EngineError> {
    let Some(view) = scene.view.as_mut() else {
        return Ok(());
    };

    if let Some(view_ref) = view.profile.as_deref() {
        view.resolved_view_profile_asset =
            load_profile_asset::<ViewProfile, _>(repo, view_ref, VIEW_PROFILE_DIR)?;
    }

    let lighting_ref = view.lighting_profile.as_deref().or_else(|| {
        view.resolved_view_profile_asset
            .as_ref()
            .and_then(|profile| profile.lighting_profile.as_deref())
    });
    if let Some(lighting_ref) = lighting_ref {
        view.resolved_lighting_profile_asset =
            load_profile_asset::<LightingProfile, _>(repo, lighting_ref, LIGHTING_PROFILE_DIR)?;
    }

    let environment_ref = view.space_environment_profile.as_deref().or_else(|| {
        view.resolved_view_profile_asset
            .as_ref()
            .and_then(|profile| profile.space_environment_profile.as_deref())
    });
    if let Some(environment_ref) = environment_ref {
        view.resolved_space_environment_profile_asset =
            load_profile_asset::<SpaceEnvironmentProfile, _>(
                repo,
                environment_ref,
                SPACE_ENVIRONMENT_PROFILE_DIR,
            )?;
    }

    Ok(())
}

fn load_profile_asset<T: DeserializeOwned, R: AssetRepository>(
    repo: &R,
    reference: &str,
    default_dir: &str,
) -> Result<Option<T>, engine_error::EngineError> {
    let candidates = profile_asset_candidates(reference, default_dir);
    for candidate in candidates {
        if repo.has_asset(&candidate)? {
            let bytes = repo.read_asset_bytes(&candidate)?;
            let profile = serde_yaml::from_slice::<T>(&bytes).map_err(|source| {
                engine_error::EngineError::InvalidModYaml {
                    path: PathBuf::from(candidate.as_str()),
                    source: source.into(),
                }
            })?;
            return Ok(Some(profile));
        }
    }
    Ok(None)
}

fn profile_asset_candidates(reference: &str, default_dir: &str) -> Vec<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if looks_like_explicit_profile_path(trimmed) {
        let normalized = if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        };
        let mut candidates = vec![normalized.clone()];
        if normalized.ends_with(".yml") {
            candidates.push(normalized.trim_end_matches(".yml").to_string() + ".yaml");
        } else if normalized.ends_with(".yaml") {
            candidates.push(normalized.trim_end_matches(".yaml").to_string() + ".yml");
        }
        return candidates;
    }

    vec![
        format!("{default_dir}/{trimmed}.yml"),
        format!("{default_dir}/{trimmed}.yaml"),
    ]
}

fn looks_like_explicit_profile_path(reference: &str) -> bool {
    reference.starts_with('/')
        || reference.contains('/')
        || Path::new(reference)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "yml" | "yaml"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::hydrate_scene_view_profiles;
    use crate::repositories::FsSceneRepository;
    use engine_core::scene::{Scene, SceneView};
    use std::fs;
    use tempfile::tempdir;

    fn base_scene() -> Scene {
        Scene {
            id: "test".to_string(),
            title: "Test".to_string(),
            cutscene: false,
            target_fps: None,
            space: Default::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: None,
            view: Some(SceneView {
                profile: Some("orbit-mod".to_string()),
                lighting_profile: None,
                space_environment_profile: None,
                resolved_view_profile_asset: None,
                resolved_lighting_profile_asset: None,
                resolved_space_environment_profile_asset: None,
            }),
            virtual_size_override: None,
            bg_colour: None,
            stages: Default::default(),
            behaviors: Vec::new(),
            audio: Default::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
            gui: Default::default(),
        }
    }

    #[test]
    fn hydrates_scene_view_profiles_from_mod_assets() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("view-profiles")).expect("view dir");
        fs::create_dir_all(mod_dir.join("lighting-profiles")).expect("lighting dir");
        fs::create_dir_all(mod_dir.join("space-environment-profiles")).expect("env dir");
        fs::write(
            mod_dir.join("view-profiles/orbit-mod.yml"),
            "id: orbit-mod\nlighting_profile: custom-light\nspace_environment_profile: custom-space\noverrides:\n  exposure: 0.9\n",
        )
        .expect("write view profile");
        fs::write(
            mod_dir.join("lighting-profiles/custom-light.yml"),
            "id: custom-light\nblack_level: 0.02\nexposure: 0.85\ngamma: 2.0\n",
        )
        .expect("write lighting profile");
        fs::write(
            mod_dir.join("space-environment-profiles/custom-space.yml"),
            "id: custom-space\nbackground_color: \"#010203\"\nstarfield_brightness: 0.6\n",
        )
        .expect("write environment profile");

        let repo = FsSceneRepository::new(&mod_dir);
        let mut scene = base_scene();
        hydrate_scene_view_profiles(&repo, &mut scene).expect("hydrate profiles");

        let view = scene.view.expect("view");
        assert_eq!(
            view.resolved_view_profile_asset.expect("view profile").id,
            "orbit-mod"
        );
        assert_eq!(
            view.resolved_lighting_profile_asset
                .expect("lighting profile")
                .id,
            "custom-light"
        );
        assert_eq!(
            view.resolved_space_environment_profile_asset
                .expect("environment profile")
                .background_color
                .as_deref(),
            Some("#010203")
        );
    }
}
