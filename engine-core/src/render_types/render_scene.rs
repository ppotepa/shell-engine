use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer2DRef {
    pub layer_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteRef {
    pub layer_index: usize,
    pub sprite_path: Vec<usize>,
    pub sprite_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport3DRef {
    pub id: Option<String>,
    pub sprite: SpriteRef,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderScene {
    pub layers_2d: Vec<Layer2DRef>,
    pub viewports_3d: Vec<Viewport3DRef>,
}
