//! Compositor and PostFX systems for rendering frame composition and effects.
//!
//! This crate provides:
//! - Compositor: layer composition, halfblock rendering
//! - PostFX: post-processing effects (CRT distort, bloom, etc.)
//! - CompositorProvider trait for decoupling from engine's World type

pub mod provider;

pub use provider::CompositorProvider;
