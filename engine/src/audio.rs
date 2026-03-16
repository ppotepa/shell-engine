#[derive(Debug, Clone, PartialEq)]
pub struct AudioCommand {
    pub cue: String,
    pub volume: Option<f32>,
}

pub trait AudioBackend: Send + Sync {
    fn play(&mut self, command: &AudioCommand);
}

#[derive(Default)]
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn play(&mut self, _command: &AudioCommand) {}
}

pub struct AudioRuntime {
    backend: Box<dyn AudioBackend + Send + Sync>,
    pending: Vec<AudioCommand>,
    played: Vec<AudioCommand>,
}

impl AudioRuntime {
    pub fn null() -> Self {
        Self {
            backend: Box::new(NullAudioBackend),
            pending: Vec::new(),
            played: Vec::new(),
        }
    }

    pub fn queue(&mut self, command: AudioCommand) {
        self.pending.push(command);
    }

    pub fn flush(&mut self) {
        for command in std::mem::take(&mut self.pending) {
            self.backend.play(&command);
            self.played.push(command);
        }
    }

    pub fn played(&self) -> &[AudioCommand] {
        &self.played
    }

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
