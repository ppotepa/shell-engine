//! Audio command queue and backend abstraction used by the engine's audio system.

/// A single audio playback request, bundling the cue name and optional volume.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioCommand {
    pub cue: String,
    pub volume: Option<f32>,
}

/// Abstracts an audio playback sink; implement this to integrate a real audio library.
pub trait AudioBackend: Send + Sync {
    fn play(&mut self, command: &AudioCommand);
}

/// A no-op [`AudioBackend`] that silently discards all commands, used in tests and headless runs.
#[derive(Default)]
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn play(&mut self, _command: &AudioCommand) {}
}

/// Buffers [`AudioCommand`]s each frame and flushes them to the active [`AudioBackend`].
pub struct AudioRuntime {
    backend: Box<dyn AudioBackend + Send + Sync>,
    pending: Vec<AudioCommand>,
    played: Vec<AudioCommand>,
}

impl AudioRuntime {
    /// Creates an [`AudioRuntime`] backed by [`NullAudioBackend`].
    pub fn null() -> Self {
        Self {
            backend: Box::new(NullAudioBackend),
            pending: Vec::new(),
            played: Vec::new(),
        }
    }

    /// Enqueues `command` for playback on the next [`flush`](Self::flush) call.
    pub fn queue(&mut self, command: AudioCommand) {
        self.pending.push(command);
    }

    /// Sends all pending commands to the backend and moves them to the played history.
    pub fn flush(&mut self) {
        for command in std::mem::take(&mut self.pending) {
            self.backend.play(&command);
            self.played.push(command);
        }
    }

    /// Returns the slice of commands that have already been flushed to the backend.
    pub fn played(&self) -> &[AudioCommand] {
        &self.played
    }

    /// Returns the number of commands waiting to be flushed.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioCommand, AudioRuntime};

    #[test]
    fn null_audio_runtime_tracks_flushed_commands() {
        let mut runtime = AudioRuntime::null();
        runtime.queue(AudioCommand {
            cue: "beep".to_string(),
            volume: Some(1.0),
        });

        assert_eq!(runtime.pending_len(), 1);
        runtime.flush();

        assert_eq!(runtime.pending_len(), 0);
        assert_eq!(
            runtime.played(),
            &[AudioCommand {
                cue: "beep".to_string(),
                volume: Some(1.0)
            }]
        );
    }
}
