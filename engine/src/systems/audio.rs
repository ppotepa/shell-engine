use crate::services::EngineWorldAccess;
use crate::world::World;

pub fn audio_system(world: &mut World) {
    let Some(audio_runtime) = world.audio_runtime_mut() else {
        return;
    };
    audio_runtime.flush();
}

#[cfg(test)]
mod tests {
    use super::audio_system;
    use crate::audio::{AudioCommand, AudioRuntime};
    use crate::world::World;

    #[test]
    fn audio_system_flushes_pending_commands() {
        let mut world = World::new();
        let mut runtime = AudioRuntime::null();
        runtime.queue(AudioCommand {
            cue: "thunder".to_string(),
            volume: Some(0.8),
        });
        world.register(runtime);

        audio_system(&mut world);

        let runtime = world.get::<AudioRuntime>().expect("audio runtime");
        assert_eq!(runtime.pending_len(), 0);
        assert_eq!(runtime.played().len(), 1);
    }
}
