use std::sync::{Arc, Mutex};

use engine_events::{EngineEvent, InputBackend};

use crate::runtime::{RuntimeCommand, RuntimeResponse, Sdl2RuntimeClient};

pub struct Sdl2InputBackend {
    client: Arc<Mutex<Sdl2RuntimeClient>>,
}

impl Sdl2InputBackend {
    pub(crate) fn from_client(client: Arc<Mutex<Sdl2RuntimeClient>>) -> Self {
        Self { client }
    }
}

impl InputBackend for Sdl2InputBackend {
    fn poll_events(&mut self) -> Vec<EngineEvent> {
        let Ok(response) = self
            .client
            .lock()
            .expect("sdl2 runtime client poisoned")
            .request(RuntimeCommand::PollInput)
        else {
            return Vec::new();
        };

        match response {
            RuntimeResponse::Input(events) => events,
            RuntimeResponse::Ack => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{map_keycode, map_modifiers};
    use engine_events::{KeyCode, KeyModifiers};
    use sdl2::keyboard::{Keycode, Mod};

    #[test]
    fn maps_ascii_keys() {
        assert_eq!(map_keycode(Keycode::A), KeyCode::Char('a'));
        assert_eq!(map_keycode(Keycode::Space), KeyCode::Char(' '));
        assert_eq!(map_keycode(Keycode::F4), KeyCode::F(4));
    }

    #[test]
    fn maps_modifier_bits() {
        let mods = map_modifiers(Mod::LCTRLMOD | Mod::LSHIFTMOD);
        assert!(mods.contains(KeyModifiers::CONTROL));
        assert!(mods.contains(KeyModifiers::SHIFT));
        assert!(!mods.contains(KeyModifiers::ALT));
    }
}
