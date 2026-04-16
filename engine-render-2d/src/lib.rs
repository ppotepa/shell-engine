//! 2D sprite rendering primitives shared by the engine compositor.
//!
//! This crate owns text and image rasterization logic. Higher-level scene
//! assembly stays in `engine-compositor`.

pub mod image;
pub mod text;

pub use image::{image_sprite_dimensions, render_image_content};
pub use text::{dim_colour, render_text_content, text_sprite_dimensions, ClipRect};
