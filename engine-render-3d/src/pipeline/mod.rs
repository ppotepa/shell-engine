pub mod generated_world_sprite_spec;
pub mod obj_sprite_spec;
pub mod sprite_mapping;

pub use generated_world_sprite_spec::{
    extract_generated_world_sprite_spec, GeneratedWorldSpriteSpec,
};
pub use obj_sprite_spec::{extract_obj_sprite_spec, ObjSpriteSpec};
pub use sprite_mapping::map_sprite_to_node3d;
