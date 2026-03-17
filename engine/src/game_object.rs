#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameObjectKind {
    Scene,
    Layer,
    TextSprite,
    ImageSprite,
    ObjSprite,
    GridSprite,
}

#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: String,
    pub name: String,
    pub kind: GameObjectKind,
    pub aliases: Vec<String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
}
