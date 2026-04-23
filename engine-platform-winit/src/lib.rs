use std::collections::VecDeque;

use engine_events::EngineEvent;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::window::CursorGrabMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformWindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl PlatformWindowConfig {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorState {
    pub locked: bool,
    pub hidden: bool,
}

impl CursorState {
    pub fn new(locked: bool, hidden: bool) -> Self {
        Self { locked, hidden }
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            locked: false,
            hidden: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorUpdate {
    pub grab_mode: CursorGrabMode,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowLifecycleState {
    pub width: u32,
    pub height: u32,
    pub focused: bool,
    pub cursor: CursorState,
}

impl WindowLifecycleState {
    pub fn from_config(config: &PlatformWindowConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
            focused: true,
            cursor: CursorState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformRuntimeEvent {
    Resized { width: u32, height: u32 },
    FocusGained,
    FocusLost,
    RelativeMouseDelta { delta_x: f64, delta_y: f64 },
}

impl PlatformRuntimeEvent {
    pub fn to_engine_event(self) -> Option<EngineEvent> {
        match self {
            Self::Resized { width, height } => Some(EngineEvent::OutputResized {
                width: width.min(u16::MAX as u32) as u16,
                height: height.min(u16::MAX as u32) as u16,
            }),
            Self::FocusLost => Some(EngineEvent::InputFocusLost),
            Self::FocusGained | Self::RelativeMouseDelta { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WinitPlatformRuntime {
    frame_counter: u64,
    event_polls: u64,
    state: WindowLifecycleState,
    pending_cursor_update: Option<CursorUpdate>,
    events: VecDeque<PlatformRuntimeEvent>,
}

impl WinitPlatformRuntime {
    pub fn new(config: PlatformWindowConfig) -> Self {
        Self {
            frame_counter: 0,
            event_polls: 0,
            state: WindowLifecycleState::from_config(&config),
            pending_cursor_update: None,
            events: VecDeque::new(),
        }
    }

    pub fn run_frame(&mut self) -> u64 {
        self.frame_counter += 1;
        self.frame_counter
    }

    pub fn poll_events(&mut self) -> Vec<PlatformRuntimeEvent> {
        self.event_polls += 1;
        self.events.drain(..).collect()
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    pub fn event_polls(&self) -> u64 {
        self.event_polls
    }

    pub fn lifecycle_state(&self) -> WindowLifecycleState {
        self.state
    }

    pub fn cursor_state(&self) -> CursorState {
        self.state.cursor
    }

    pub fn set_cursor_locked(&mut self, locked: bool) {
        if self.state.cursor.locked != locked {
            self.state.cursor.locked = locked;
            self.sync_pending_cursor_update();
        }
    }

    pub fn set_cursor_hidden(&mut self, hidden: bool) {
        if self.state.cursor.hidden != hidden {
            self.state.cursor.hidden = hidden;
            self.sync_pending_cursor_update();
        }
    }

    pub fn take_pending_cursor_update(&mut self) -> Option<CursorUpdate> {
        self.pending_cursor_update.take()
    }

    pub fn push_window_event(&mut self, event: &WindowEvent) -> Option<PlatformRuntimeEvent> {
        let translated = translate_window_event(event)?;
        self.apply_runtime_event(translated);
        Some(translated)
    }

    pub fn push_device_event(&mut self, event: &DeviceEvent) -> Option<PlatformRuntimeEvent> {
        let translated = translate_device_event(event)?;
        self.apply_runtime_event(translated);
        Some(translated)
    }

    fn apply_runtime_event(&mut self, event: PlatformRuntimeEvent) {
        match event {
            PlatformRuntimeEvent::Resized { width, height } => {
                self.state.width = width;
                self.state.height = height;
            }
            PlatformRuntimeEvent::FocusGained => {
                self.state.focused = true;
            }
            PlatformRuntimeEvent::FocusLost => {
                self.state.focused = false;
            }
            PlatformRuntimeEvent::RelativeMouseDelta { .. } => {}
        }
        self.events.push_back(event);
    }

    fn sync_pending_cursor_update(&mut self) {
        let grab_mode = if self.state.cursor.locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        self.pending_cursor_update = Some(CursorUpdate {
            grab_mode,
            visible: !self.state.cursor.hidden,
        });
    }
}

pub fn translate_window_event(event: &WindowEvent) -> Option<PlatformRuntimeEvent> {
    match event {
        WindowEvent::Resized(PhysicalSize { width, height }) => Some(PlatformRuntimeEvent::Resized {
            width: *width,
            height: *height,
        }),
        WindowEvent::Focused(true) => Some(PlatformRuntimeEvent::FocusGained),
        WindowEvent::Focused(false) => Some(PlatformRuntimeEvent::FocusLost),
        _ => None,
    }
}

pub fn translate_device_event(event: &DeviceEvent) -> Option<PlatformRuntimeEvent> {
    match event {
        DeviceEvent::MouseMotion { delta } => Some(PlatformRuntimeEvent::RelativeMouseDelta {
            delta_x: delta.0,
            delta_y: delta.1,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        translate_device_event, translate_window_event, CursorGrabMode, PlatformRuntimeEvent,
        PlatformWindowConfig, WinitPlatformRuntime,
    };
    use engine_events::EngineEvent;
    use winit::{
        dpi::PhysicalSize,
        event::{DeviceEvent, WindowEvent},
    };

    fn test_config() -> PlatformWindowConfig {
        PlatformWindowConfig::new("Shell Engine", 1280, 720)
    }

    #[test]
    fn window_config_construction_is_stable() {
        let config = test_config();
        assert_eq!(config.title, "Shell Engine");
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    #[test]
    fn new_runtime_has_zeroed_counters_and_initialized_state() {
        let runtime = WinitPlatformRuntime::new(test_config());
        let state = runtime.lifecycle_state();
        assert_eq!(runtime.frame_counter(), 0);
        assert_eq!(runtime.event_polls(), 0);
        assert_eq!(state.width, 1280);
        assert_eq!(state.height, 720);
        assert!(state.focused);
        assert!(!state.cursor.locked);
        assert!(!state.cursor.hidden);
    }

    #[test]
    fn run_frame_is_deterministic() {
        let mut runtime = WinitPlatformRuntime::new(test_config());
        assert_eq!(runtime.run_frame(), 1);
        assert_eq!(runtime.run_frame(), 2);
        assert_eq!(runtime.frame_counter(), 2);
    }

    #[test]
    fn translate_window_event_maps_resize() {
        let event = WindowEvent::Resized(PhysicalSize::new(1920, 1080));
        assert_eq!(
            translate_window_event(&event),
            Some(PlatformRuntimeEvent::Resized {
                width: 1920,
                height: 1080
            })
        );
    }

    #[test]
    fn translate_window_event_maps_focus() {
        assert_eq!(
            translate_window_event(&WindowEvent::Focused(true)),
            Some(PlatformRuntimeEvent::FocusGained)
        );
        assert_eq!(
            translate_window_event(&WindowEvent::Focused(false)),
            Some(PlatformRuntimeEvent::FocusLost)
        );
    }

    #[test]
    fn translate_device_event_maps_relative_mouse_delta() {
        let event = DeviceEvent::MouseMotion { delta: (3.5, -2.0) };
        assert_eq!(
            translate_device_event(&event),
            Some(PlatformRuntimeEvent::RelativeMouseDelta {
                delta_x: 3.5,
                delta_y: -2.0
            })
        );
    }

    #[test]
    fn runtime_queues_translated_events_and_updates_state() {
        let mut runtime = WinitPlatformRuntime::new(test_config());
        runtime.push_window_event(&WindowEvent::Focused(false));
        runtime.push_window_event(&WindowEvent::Resized(PhysicalSize::new(800, 600)));
        runtime.push_device_event(&DeviceEvent::MouseMotion { delta: (1.0, 2.0) });

        let events = runtime.poll_events();
        assert_eq!(
            events,
            vec![
                PlatformRuntimeEvent::FocusLost,
                PlatformRuntimeEvent::Resized {
                    width: 800,
                    height: 600
                },
                PlatformRuntimeEvent::RelativeMouseDelta {
                    delta_x: 1.0,
                    delta_y: 2.0
                }
            ]
        );
        assert_eq!(runtime.event_polls(), 1);
        assert_eq!(runtime.lifecycle_state().width, 800);
        assert_eq!(runtime.lifecycle_state().height, 600);
        assert!(!runtime.lifecycle_state().focused);
    }

    #[test]
    fn cursor_state_updates_emit_pending_cursor_update() {
        let mut runtime = WinitPlatformRuntime::new(test_config());
        runtime.set_cursor_locked(true);
        runtime.set_cursor_hidden(true);

        let update = runtime
            .take_pending_cursor_update()
            .expect("cursor update should be pending");
        assert_eq!(update.grab_mode, CursorGrabMode::Locked);
        assert!(!update.visible);
        assert!(runtime.take_pending_cursor_update().is_none());
    }

    #[test]
    fn platform_runtime_event_to_engine_event_maps_supported_variants() {
        assert!(matches!(
            PlatformRuntimeEvent::FocusLost.to_engine_event(),
            Some(EngineEvent::InputFocusLost)
        ));
        assert!(matches!(
            PlatformRuntimeEvent::Resized {
                width: 70000,
                height: 42
            }
            .to_engine_event(),
            Some(EngineEvent::OutputResized {
                width: u16::MAX,
                height: 42
            })
        ));
        assert!(PlatformRuntimeEvent::FocusGained
            .to_engine_event()
            .is_none());
    }
}
