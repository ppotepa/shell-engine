use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Tick,
    KeyPressed(KeyCode),
    SceneLoaded { scene_id: String },
    SceneTransition { to_scene_id: String },
    TerminalResized { width: u16, height: u16 },
    Quit,
}

#[derive(Debug, Default)]
pub struct EventQueue {
    events: Vec<EngineEvent>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: EngineEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
