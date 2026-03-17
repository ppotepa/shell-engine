use crate::scene::{Scene, SceneDocument};

pub fn compile_scene_document(content: &str) -> Result<Scene, serde_yaml::Error> {
    let document = serde_yaml::from_str::<SceneDocument>(content)?;
    document.compile()
}

#[cfg(test)]
mod tests {
    use super::compile_scene_document;

    #[test]
    fn compiles_legacy_scene_yaml_into_runtime_scene() {
        let raw = r#"
id: intro
title: Intro
bg_colour: black
layers: []
"#;
        let scene = compile_scene_document(raw).expect("scene should compile");
        assert_eq!(scene.id, "intro");
        assert_eq!(scene.title, "Intro");
    }
}

