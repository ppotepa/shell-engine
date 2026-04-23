use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::effects::Region;
use engine_core::scene::{Layer, Sprite};
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderTargetSize {
    pub width: u16,
    pub height: u16,
}

/// Buffer-agnostic packet for a prepared 2D layer pass.
///
/// This scaffolding allows higher-level orchestration to carry prepared
/// 2D pass metadata without hard-coding any concrete render target type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreparedRender2dPacket {
    pub layer_idx: usize,
    pub scene_w: u16,
    pub scene_h: u16,
    pub root_origin_x: i32,
    pub root_origin_y: i32,
    pub scene_elapsed_ms: u64,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub is_pixel_backend: bool,
    pub ui_font_scale: f32,
    pub ui_layout_scale_x: f32,
    pub ui_layout_scale_y: f32,
    pub target_size: Option<RenderTargetSize>,
}

impl PreparedRender2dPacket {
    pub fn from_input(input: &Render2dInput<'_>) -> Self {
        Self {
            layer_idx: input.layer_idx,
            scene_w: input.scene_w,
            scene_h: input.scene_h,
            root_origin_x: input.root_origin_x,
            root_origin_y: input.root_origin_y,
            scene_elapsed_ms: input.scene_elapsed_ms,
            step_idx: input.step_idx,
            elapsed_ms: input.elapsed_ms,
            is_pixel_backend: input.is_pixel_backend,
            ui_font_scale: input.ui_font_scale,
            ui_layout_scale_x: input.ui_layout_scale_x,
            ui_layout_scale_y: input.ui_layout_scale_y,
            target_size: None,
        }
    }

    pub fn builder(input: &Render2dInput<'_>) -> PreparedRender2dPacketBuilder {
        PreparedRender2dPacketBuilder {
            packet: Self::from_input(input),
        }
    }
}

pub struct PreparedRender2dPacketBuilder {
    packet: PreparedRender2dPacket,
}

impl PreparedRender2dPacketBuilder {
    pub fn target_size(mut self, width: u16, height: u16) -> Self {
        self.packet.target_size = Some(RenderTargetSize { width, height });
        self
    }

    pub fn build(self) -> PreparedRender2dPacket {
        self.packet
    }
}

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
    pub ui_font_scale: f32,
    /// Logical-to-target layout scale on X axis.
    /// For UI layers in split-pass this is typically `final_w / world_w`.
    pub ui_layout_scale_x: f32,
    /// Logical-to-target layout scale on Y axis.
    /// For UI layers in split-pass this is typically `final_h / world_h`.
    pub ui_layout_scale_y: f32,
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
    use super::{layer_has_timed_sprites, PreparedRender2dPacket, Render2dInput};
    use engine_animation::SceneStage;
    use engine_core::effects::Region;
    use engine_core::scene::Layer;
    use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
    use std::collections::HashMap;

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

    #[test]
    fn packet_from_input_preserves_core_fields() {
        let layer: Layer = serde_yaml::from_str(
            r#"
name: packet-layer
sprites: []
"#,
        )
        .expect("layer should parse");
        let current_stage = SceneStage::default();
        let mut object_regions: HashMap<String, Region> = HashMap::new();
        let object_states: HashMap<String, ObjectRuntimeState> = HashMap::new();

        let input = Render2dInput {
            layer_idx: 3,
            layer: &layer,
            scene_w: 160,
            scene_h: 90,
            asset_root: None,
            target_resolver: Option::<&TargetResolver>::None,
            object_regions: &mut object_regions,
            root_origin_x: 12,
            root_origin_y: -4,
            object_states: &object_states,
            scene_elapsed_ms: 1500,
            current_stage: &current_stage,
            step_idx: 2,
            elapsed_ms: 120,
            is_pixel_backend: true,
            default_font: None,
            ui_font_scale: 1.25,
            ui_layout_scale_x: 1.5,
            ui_layout_scale_y: 0.8,
        };

        let packet = PreparedRender2dPacket::builder(&input)
            .target_size(320, 180)
            .build();

        assert_eq!(packet.layer_idx, 3);
        assert_eq!(packet.scene_w, 160);
        assert_eq!(packet.scene_h, 90);
        assert_eq!(packet.root_origin_x, 12);
        assert_eq!(packet.root_origin_y, -4);
        assert_eq!(packet.scene_elapsed_ms, 1500);
        assert_eq!(packet.step_idx, 2);
        assert_eq!(packet.elapsed_ms, 120);
        assert!(packet.is_pixel_backend);
        assert_eq!(packet.ui_font_scale, 1.25);
        assert_eq!(packet.ui_layout_scale_x, 1.5);
        assert_eq!(packet.ui_layout_scale_y, 0.8);
        assert_eq!(packet.target_size.map(|s| (s.width, s.height)), Some((320, 180)));
    }
}
