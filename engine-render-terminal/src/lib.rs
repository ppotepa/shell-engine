//! Terminal-based render backend implementation for Shell Quest.
//!
//! Provides the core renderer system that outputs buffered frames to the terminal
//! using ANSI escape codes and crossterm.

pub mod provider;

pub use provider::RendererProvider;
