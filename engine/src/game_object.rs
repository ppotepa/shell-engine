//! Core scene-object types: [`GameObjectKind`] discriminant and the [`GameObject`] data record.

/// Discriminates the kind of node a [`GameObject`] represents in the scene tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameObjectKind {
    Scene,
    Layer,
    TextSprite,
    ImageSprite,
    ObjSprite,
    GridSprite,
}

/// A runtime node in the scene tree, carrying identity, kind, and parent–child relationships.
#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: String,
    pub name: String,
    pub kind: GameObjectKind,
    pub aliases: Vec<String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
}
