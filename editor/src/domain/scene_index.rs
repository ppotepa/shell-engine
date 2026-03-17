//! Scene path index for a mod project.

/// Indexed list of discovered scene file paths relative to the mod root.
#[derive(Debug, Clone, Default)]
pub struct SceneIndex {
    pub scene_paths: Vec<String>,
}
