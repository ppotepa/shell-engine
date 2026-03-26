//! Audio system — provides the system callback to flush audio commands each frame.
//!
//! Integration: engine-audio is backend-agnostic. The engine (or any consumer)
//! calls `audio_system()` once per frame after processing commands.

use crate::audio::AudioRuntime;

/// Trait for any type that can provide mutable audio runtime access.
///
/// This allows engine-audio to remain decoupled from the main engine's World implementation.
pub trait AudioProvider: Send {
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime>;
}

/// Flushes all pending audio commands to the backend for the current frame.
///
/// This should be called once per frame after all systems have queued audio commands.
pub fn audio_system<T: AudioProvider>(provider: &mut T) {
    let Some(audio_runtime) = provider.audio_runtime_mut() else {
        return;
    };
    audio_runtime.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{AudioCommand, AudioRuntime};

    struct TestProvider {
        audio: Option<AudioRuntime>,
    }

    impl AudioProvider for TestProvider {
        fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
            self.audio.as_mut()
        }
    }

    #[test]
    fn audio_system_flushes_pending_commands() {
        let mut provider = TestProvider {
            audio: Some(AudioRuntime::null()),
        };

        if let Some(audio) = &mut provider.audio {
            audio.queue(AudioCommand {
                cue: "thunder".to_string(),
                volume: Some(0.8),
            });
        }

        audio_system(&mut provider);

        if let Some(audio) = &provider.audio {
            assert_eq!(audio.pending_len(), 0);
            assert_eq!(audio.played().len(), 1);
        }
    }
}
