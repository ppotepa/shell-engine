use crate::logging;
use crate::{world::World, EngineError, RenderBackendKind};
use engine_events::InputBackend;
#[cfg(feature = "hardware-presenter")]
use engine_events::{KeyCode, KeyEvent, KeyModifiers, MouseButton};
use engine_render::RendererBackend;

#[derive(Debug)]
pub(crate) struct BackendBootstrapParams {
    pub(crate) selected_backend: RenderBackendKind,
    pub(crate) ui_size: (u16, u16),
}

pub(crate) struct BootstrappedBackends {
    pub(crate) input_backend: Box<dyn InputBackend>,
    pub(crate) active_backend: RenderBackendKind,
}

pub(crate) fn render_backend_name(kind: RenderBackendKind) -> &'static str {
    match kind {
        RenderBackendKind::Software => "software",
        RenderBackendKind::Hardware => "hardware",
    }
}

pub(crate) fn runtime_output_label(kind: RenderBackendKind) -> &'static str {
    match kind {
        RenderBackendKind::Software => "software",
        RenderBackendKind::Hardware => "hardware",
    }
}

pub(crate) fn backend_unavailable_message(kind: RenderBackendKind) -> &'static str {
    match kind {
        RenderBackendKind::Software => {
            "software render backend is unavailable: SDL2 runtime bootstrap has been removed; select the hardware backend"
        }
        RenderBackendKind::Hardware => {
            if cfg!(feature = "hardware-presenter") {
                "hardware render backend is unavailable"
            } else {
                "hardware render backend is unavailable: engine was built without the `hardware-presenter` feature"
            }
        }
    }
}

pub(crate) fn bootstrap_runtime_backends(
    world: &mut World,
    params: BackendBootstrapParams,
) -> Result<BootstrappedBackends, EngineError> {
    match params.selected_backend {
        RenderBackendKind::Software => Err(selection_error(RenderBackendKind::Software)),
        RenderBackendKind::Hardware => bootstrap_hardware_backend(world, params.ui_size),
    }
}

fn bootstrap_hardware_backend(
    world: &mut World,
    ui_size: (u16, u16),
) -> Result<BootstrappedBackends, EngineError> {
    #[cfg(feature = "hardware-presenter")]
    {
        let presenter = engine_render_wgpu::WgpuPresenter::new(ui_size)
            .map_err(|error| EngineError::Render(std::io::Error::other(error.to_string())))?;
        let input = HardwarePresenterInputBackend::new(presenter.input_queue());
        world.register(Box::new(presenter) as Box<dyn RendererBackend>);
        return Ok(BootstrappedBackends {
            input_backend: Box::new(input),
            active_backend: RenderBackendKind::Hardware,
        });
    }

    #[cfg(not(feature = "hardware-presenter"))]
    {
        let _ = (world, ui_size);
        Err(selection_error(RenderBackendKind::Hardware))
    }
}

fn selection_error(kind: RenderBackendKind) -> EngineError {
    let message = backend_unavailable_message(kind);
    logging::error(
        "engine.runtime",
        format!(
            "render backend selection failed: backend={} reason={}",
            render_backend_name(kind),
            message
        ),
    );
    EngineError::Render(std::io::Error::other(message))
}

#[cfg(feature = "hardware-presenter")]
struct HardwarePresenterInputBackend {
    queue: engine_render_wgpu::WgpuInputQueue,
}

#[cfg(feature = "hardware-presenter")]
impl HardwarePresenterInputBackend {
    fn new(queue: engine_render_wgpu::WgpuInputQueue) -> Self {
        Self { queue }
    }

    fn map_modifiers(modifiers: engine_render_wgpu::WgpuInputModifiers) -> KeyModifiers {
        let mut mapped = KeyModifiers::NONE;
        if modifiers.shift {
            mapped |= KeyModifiers::SHIFT;
        }
        if modifiers.control {
            mapped |= KeyModifiers::CONTROL;
        }
        if modifiers.alt {
            mapped |= KeyModifiers::ALT;
        }
        mapped
    }

    fn map_mouse_button(button: engine_render_wgpu::WgpuInputMouseButton) -> MouseButton {
        match button {
            engine_render_wgpu::WgpuInputMouseButton::Left => MouseButton::Left,
            engine_render_wgpu::WgpuInputMouseButton::Right => MouseButton::Right,
            engine_render_wgpu::WgpuInputMouseButton::Middle => MouseButton::Middle,
        }
    }

    fn map_key(key: engine_render_wgpu::WgpuInputKey) -> KeyCode {
        match key {
            engine_render_wgpu::WgpuInputKey::Char(c) => KeyCode::Char(c),
            engine_render_wgpu::WgpuInputKey::Esc => KeyCode::Esc,
            engine_render_wgpu::WgpuInputKey::Up => KeyCode::Up,
            engine_render_wgpu::WgpuInputKey::Down => KeyCode::Down,
            engine_render_wgpu::WgpuInputKey::Left => KeyCode::Left,
            engine_render_wgpu::WgpuInputKey::Right => KeyCode::Right,
        }
    }

    fn map_event(event: engine_render_wgpu::WgpuInputEvent) -> engine_events::EngineEvent {
        match event {
            engine_render_wgpu::WgpuInputEvent::Quit => engine_events::EngineEvent::Quit,
            engine_render_wgpu::WgpuInputEvent::FocusLost => {
                engine_events::EngineEvent::InputFocusLost
            }
            engine_render_wgpu::WgpuInputEvent::MouseMoved { x, y } => {
                engine_events::EngineEvent::MouseMoved { x, y }
            }
            engine_render_wgpu::WgpuInputEvent::MouseButtonDown {
                button,
                x,
                y,
                modifiers,
            } => engine_events::EngineEvent::MouseButtonDown {
                button: Self::map_mouse_button(button),
                x,
                y,
                modifiers: Self::map_modifiers(modifiers),
            },
            engine_render_wgpu::WgpuInputEvent::MouseButtonUp {
                button,
                x,
                y,
                modifiers,
            } => engine_events::EngineEvent::MouseButtonUp {
                button: Self::map_mouse_button(button),
                x,
                y,
                modifiers: Self::map_modifiers(modifiers),
            },
            engine_render_wgpu::WgpuInputEvent::MouseWheel { delta_y, modifiers } => {
                engine_events::EngineEvent::MouseWheel {
                    delta_y,
                    modifiers: Self::map_modifiers(modifiers),
                }
            }
            engine_render_wgpu::WgpuInputEvent::KeyDown {
                key,
                repeat,
                modifiers,
            } => engine_events::EngineEvent::KeyDown {
                key: KeyEvent::new(Self::map_key(key), Self::map_modifiers(modifiers)),
                repeat,
            },
            engine_render_wgpu::WgpuInputEvent::KeyUp { key, modifiers } => {
                engine_events::EngineEvent::KeyUp {
                    key: KeyEvent::new(Self::map_key(key), Self::map_modifiers(modifiers)),
                }
            }
        }
    }
}

#[cfg(feature = "hardware-presenter")]
impl engine_events::InputBackend for HardwarePresenterInputBackend {
    fn poll_events(&mut self) -> Vec<engine_events::EngineEvent> {
        self.queue
            .drain()
            .into_iter()
            .map(Self::map_event)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{backend_unavailable_message, bootstrap_runtime_backends, BackendBootstrapParams};
    use crate::{world::World, RenderBackendKind};

    #[test]
    fn software_backend_reports_unavailable() {
        let mut world = World::new();
        let result = bootstrap_runtime_backends(
            &mut world,
            BackendBootstrapParams {
                selected_backend: RenderBackendKind::Software,
                ui_size: (80, 25),
            },
        );

        let error = match result {
            Ok(_) => panic!("software backend should be unavailable"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("SDL2 runtime bootstrap has been removed"),
            "error should state software backend bootstrap removal: {error}"
        );
    }

    #[test]
    fn unavailable_messages_are_stable() {
        let software = backend_unavailable_message(RenderBackendKind::Software);
        assert!(
            software.contains("removed"),
            "software message should describe removal: {software}"
        );

        let hardware = backend_unavailable_message(RenderBackendKind::Hardware);
        assert!(
            hardware.contains("hardware render backend is unavailable"),
            "hardware message should identify unavailable hardware backend: {hardware}"
        );
    }
}
