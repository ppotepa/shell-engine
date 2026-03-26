//! Audio backend abstraction and Rodio-based audio playback for the engine.
//!
//! Provides:
//! - `AudioBackend` trait for pluggable audio implementations
//! - `RodioAudioBackend` in-process WAV/MP3 playback
//! - `NullAudioBackend` for tests and headless runs
//! - `AudioProvider` trait for integration with any container (engine's World, etc)
//! - `audio_system()` to flush commands each frame

pub mod audio;
pub mod systems_audio;
pub mod access;

pub use audio::{AudioBackend, AudioCommand, AudioRuntime, NullAudioBackend, RodioAudioBackend};
pub use systems_audio::{audio_system, AudioProvider};
