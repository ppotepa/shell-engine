use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use engine_core::logging;

use crate::debug_features::DebugFeatures;
use crate::scene::Scene;
use crate::scene_loader::SceneLoader;
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use engine_animation::Animator;
use crate::world::World;

const DEFAULT_POLL_INTERVAL_MS: u64 = 350;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SceneTreeSignature {
    yaml_file_count: u64,
    yaml_total_size: u64,
    content_hash: u64,
}

#[derive(Debug)]
pub struct DebugSceneHotReloadState {
    last_poll: Instant,
    poll_interval: Duration,
    last_signature: Option<SceneTreeSignature>,
}

impl DebugSceneHotReloadState {
    fn new(last_signature: Option<SceneTreeSignature>) -> Self {
        Self {
            last_poll: Instant::now(),
            poll_interval: Duration::from_millis(DEFAULT_POLL_INTERVAL_MS),
            last_signature,
        }
    }
}

pub fn debug_scene_hot_reload_system(world: &mut World) {
    let debug_enabled = world
        .get::<DebugFeatures>()
        .map(|debug| debug.enabled)
        .unwrap_or(false);
    if !debug_enabled {
        return;
    }

    let Some(asset_root) = world.asset_root() else {
        return;
    };
    let mod_source = asset_root.mod_source().to_path_buf();
    if !mod_source.is_dir() {
        return;
    }
    let watched_roots = [
        mod_source.join("scenes"),
        mod_source.join("objects"),
        mod_source.join("stages"),
    ];
    if watched_roots.iter().all(|root| !root.is_dir()) {
        return;
    }

    if world.get::<DebugSceneHotReloadState>().is_none() {
        let baseline = scan_yaml_tree_signature(&watched_roots).ok();
        world.register(DebugSceneHotReloadState::new(baseline));
        return;
    }

    let should_poll = world
        .get::<DebugSceneHotReloadState>()
        .map(|state| state.last_poll.elapsed() >= state.poll_interval)
        .unwrap_or(false);
    if !should_poll {
        return;
    }

    let signature = match scan_yaml_tree_signature(&watched_roots) {
        Ok(sig) => sig,
        Err(error) => {
            logging::warn(
                "engine.hot-reload",
                format!("failed to scan YAML files for changes: {error}"),
            );
            return;
        }
    };
    let changed = {
        let Some(state) = world.get_mut::<DebugSceneHotReloadState>() else {
            return;
        };
        state.last_poll = Instant::now();
        let changed = signatures_differ(state.last_signature, signature);
        state.last_signature = Some(signature);
        changed
    };
    if !changed {
        return;
    }

    let Some(active_scene_id) = world.scene_runtime().map(|runtime| runtime.scene().id.clone()) else {
        return;
    };

    let refreshed_loader = match SceneLoader::new(mod_source.clone()) {
        Ok(loader) => loader,
        Err(error) => {
            logging::warn(
                "engine.hot-reload",
                format!("failed to refresh scene index: {error}"),
            );
            return;
        }
    };
    world.register(refreshed_loader);

    let reloaded_scene = match world.scene_loader() {
        Some(loader) => match loader.load_by_id(&active_scene_id) {
            Ok(scene) => scene,
            Err(error) => {
                logging::warn(
                    "engine.hot-reload",
                    format!(
                        "detected scene change but could not reload active scene id={active_scene_id}: {error}"
                    ),
                );
                return;
            }
        },
        None => return,
    };

    apply_virtual_size_override(world, &reloaded_scene);
    world.clear_scoped();
    world.register_scoped(SceneRuntime::new(reloaded_scene));
    world.register_scoped(Animator::new());
    logging::info(
        "engine.hot-reload",
        format!("reloaded active scene in debug mode: id={active_scene_id}"),
    );
}

fn signatures_differ(previous: Option<SceneTreeSignature>, next: SceneTreeSignature) -> bool {
    previous.map(|sig| sig != next).unwrap_or(false)
}

fn scan_yaml_tree_signature(roots: &[std::path::PathBuf]) -> std::io::Result<SceneTreeSignature> {
    let mut hasher = DefaultHasher::new();
    let mut signature = SceneTreeSignature::default();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        scan_yaml_tree_signature_recursive(root, root, &mut signature, &mut hasher)?;
    }
    signature.content_hash = hasher.finish();
    Ok(signature)
}

fn scan_yaml_tree_signature_recursive(
    root: &Path,
    base_root: &Path,
    signature: &mut SceneTreeSignature,
    hasher: &mut DefaultHasher,
) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let path = entry.path();
        if metadata.is_dir() {
            scan_yaml_tree_signature_recursive(&path, base_root, signature, hasher)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        if !is_yaml_path(&path) {
            continue;
        }
        let rel = path.strip_prefix(base_root).unwrap_or(path.as_path());
        rel.to_string_lossy().hash(hasher);
        let bytes = fs::read(&path)?;
        signature.yaml_file_count = signature.yaml_file_count.saturating_add(1);
        signature.yaml_total_size = signature.yaml_total_size.saturating_add(bytes.len() as u64);
        bytes.hash(hasher);
    }
    Ok(())
}

fn is_yaml_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "yml" | "yaml"))
        .unwrap_or(false)
}

fn apply_virtual_size_override(world: &mut World, scene: &Scene) {
    let Some(settings) = world.runtime_settings() else {
        return;
    };
    if !settings.use_virtual_buffer {
        return;
    }
    let Some(size_override) = scene.virtual_size_override.as_deref() else {
        return;
    };
    let Some((w, h, is_max)) = crate::runtime_settings::parse_virtual_size_str(size_override) else {
        return;
    };
    let (new_width, new_height) = if is_max {
        let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
        (term_w.max(1), term_h.max(1))
    } else {
        (w, h)
    };
    if let Some(vbuf) = world.virtual_buffer_mut() {
        if vbuf.0.width != new_width || vbuf.0.height != new_height {
            vbuf.0.resize(new_width, new_height);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{is_yaml_path, scan_yaml_tree_signature, signatures_differ, SceneTreeSignature};

    #[test]
    fn signature_difference_requires_previous_value() {
        let sig = SceneTreeSignature {
            yaml_file_count: 1,
            yaml_total_size: 8,
            content_hash: 44,
        };
        assert!(!signatures_differ(None, sig));
        assert!(!signatures_differ(Some(sig), sig));
        assert!(signatures_differ(
            Some(sig),
            SceneTreeSignature {
                content_hash: 45,
                ..sig
            }
        ));
    }

    #[test]
    fn scans_yaml_tree_recursively() {
        let temp = tempdir().expect("tempdir");
        let scenes = temp.path().join("scenes");
        fs::create_dir_all(scenes.join("pkg/layers")).expect("mkdir");
        fs::write(scenes.join("scene-a.yml"), "id: a\ntitle: A\nlayers: []\n").expect("write a");
        fs::write(scenes.join("pkg/scene.yml"), "id: b\ntitle: B\nlayers: []\n").expect("write b");
        fs::write(scenes.join("pkg/layers/main.yml"), "- name: main\nsprites: []\n")
            .expect("write layer");
        fs::write(scenes.join("pkg/layers/readme.txt"), "ignored").expect("write txt");

        let sig = scan_yaml_tree_signature(std::slice::from_ref(&scenes)).expect("signature");
        assert_eq!(sig.yaml_file_count, 3);
        assert!(sig.yaml_total_size > 0);
        assert_ne!(sig.content_hash, 0);
    }

    #[test]
    fn detects_same_size_yaml_content_changes() {
        let temp = tempdir().expect("tempdir");
        let scenes = temp.path().join("scenes");
        fs::create_dir_all(&scenes).expect("mkdir");
        let file = scenes.join("scene-a.yml");
        fs::write(&file, "id: aa\ntitle: A\n").expect("write a");
        let sig_a = scan_yaml_tree_signature(std::slice::from_ref(&scenes)).expect("signature a");
        fs::write(&file, "id: aa\ntitle: B\n").expect("write b");
        let sig_b = scan_yaml_tree_signature(std::slice::from_ref(&scenes)).expect("signature b");
        assert_eq!(sig_a.yaml_total_size, sig_b.yaml_total_size);
        assert!(signatures_differ(Some(sig_a), sig_b));
    }

    #[test]
    fn yaml_extension_detection_is_case_insensitive() {
        assert!(is_yaml_path(Path::new("/tmp/a.yml")));
        assert!(is_yaml_path(Path::new("/tmp/a.YAML")));
        assert!(!is_yaml_path(Path::new("/tmp/a.txt")));
    }
}
