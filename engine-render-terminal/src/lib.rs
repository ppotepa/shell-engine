//! Terminal-based render backend implementation for Shell Quest.
//!
//! Provides the core renderer system that outputs buffered frames to the terminal
//! using ANSI escape codes and crossterm.

pub mod color_convert;
pub mod input;
pub mod provider;
pub mod rasterizer;
pub mod renderer;
pub mod strategy;

pub use provider::RendererProvider;
pub use renderer::{flush_batched, renderer_system, resolve_color, TerminalRenderer};
pub use strategy::{
    AnsiBatchFlusher, AsyncDisplaySink, NaiveFlusher, SyncDisplaySink, TerminalFlusher,
};
