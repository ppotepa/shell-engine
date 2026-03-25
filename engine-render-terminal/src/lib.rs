//! Terminal-based render backend implementation for Shell Quest.
//!
//! Provides the core renderer system that outputs buffered frames to the terminal
//! using ANSI escape codes and crossterm.

pub mod provider;
pub mod renderer;
pub mod rasterizer;
pub mod strategy;

pub use provider::RendererProvider;
pub use renderer::{TerminalRenderer, renderer_system, resolve_color, flush_batched};
pub use strategy::{AnsiBatchFlusher, NaiveFlusher, AsyncDisplaySink, SyncDisplaySink};
