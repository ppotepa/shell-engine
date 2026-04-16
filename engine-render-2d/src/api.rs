use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::effects::Region;
use engine_core::scene::{Layer, Sprite};
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use std::collections::HashMap;

/// Render input for one layer pass.
pub struct Render2dInput<'a> {
    pub layer_idx: usize,
    pub layer: &'a Layer,
    pub scene_w: u16,
    pub scene_h: u16,
    pub asset_root: Option<&'a AssetRoot>,
    pub target_resolver: Option<&'a TargetResolver>,
    pub object_regions: &'a mut HashMap<String, Region>,
    pub root_origin_x: i32,
    pub root_origin_y: i32,
    pub object_states: &'a HashMap<String, ObjectRuntimeState>,
    pub scene_elapsed_ms: u64,
    pub current_stage: &'a SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub is_pixel_backend: bool,
    pub default_font: Option<&'a str>,
}

/// Seam between composition and 2D sprite rendering.
pub trait Render2dPipeline {
    fn render(&self, input: Render2dInput<'_>, target: &mut engine_core::buffer::Buffer);
}

/// Returns `true` when any sprite in the layer tree uses appear/disappear timing.
///
/// Compositors can use this to decide whether scratch composition is required
/// without interpreting authored sprite fields directly.
pub fn layer_has_timed_sprites(layer: &Layer) -> bool {
    sprites_have_timed_visibility(&layer.sprites)
}

/// Returns `true` when any sprite in the slice (including nested container children)
/// uses appear/disappear timing.
pub fn sprites_have_timed_visibility(sprites: &[Sprite]) -> bool {
    sprites.iter().any(sprite_has_timed_visibility)
}

fn sprite_has_timed_visibility(sprite: &Sprite) -> bool {
    if sprite.appear_at_ms().is_some() || sprite.disappear_at_ms().is_some() {
        return true;
    }
    match sprite {
        Sprite::Panel { children, .. }
        | Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. } => sprites_have_timed_visibility(children),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::layer_has_timed_sprites;
    use engine_core::scene::Layer;

    #[test]
    fn detects_nested_timed_sprite_visibility() {
        let layer: Layer = serde_yaml::from_str(
            r#"
name: timed-layer
sprites:
  - type: panel
    children:
      - type: text
        content: "hello"
        appear_at_ms: 25
"#,
        )
        .expect("layer should parse");

        assert!(layer_has_timed_sprites(&layer));
    }

    #[test]
    fn false_when_no_sprite_uses_timing() {
        let layer: Layer = serde_yaml::from_str(
            r#"
name: static-layer
sprites:
  - type: text
    content: "hello"
"#,
        )
        .expect("layer should parse");

        assert!(!layer_has_timed_sprites(&layer));
    }
}
