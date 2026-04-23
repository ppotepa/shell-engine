//! Hardware presenter for the `winit + wgpu` runtime path.
//!
//! Stage-2 implementation: this backend owns a real window runtime thread and
//! presents frames through an internal wgpu-native lifecycle stub.

use engine_core::buffer::Buffer;
use engine_platform_winit::PlatformWindowConfig;
use engine_render::{
    FrameSubmission, HardwareRendererBackend, OverlayData, RenderBackendKind, RenderError,
    RendererBackend, VectorOverlay,
};
use std::fmt::{Display, Formatter};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode as WinitKeyCode, ModifiersState, PhysicalKey};
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::{Window, WindowId};

#[derive(Debug, Clone, Copy)]
enum RuntimeCommand {
    SetClearColor([u8; 4]),
    Shutdown,
}

/// Surface status exposed by the wgpu presenter runtime lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuSurfaceStatus {
    Presented,
    Lost,
    Outdated,
    Timeout,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeSurfaceConfig {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeInitConfig {
    window_title: String,
    surface: RuntimeSurfaceConfig,
}

impl RuntimeInitConfig {
    fn from_platform(config: &PlatformWindowConfig) -> Self {
        Self {
            window_title: config.title.clone(),
            surface: RuntimeSurfaceConfig {
                width: config.width,
                height: config.height,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeResizeRequest {
    surface: RuntimeSurfaceConfig,
}

impl RuntimeResizeRequest {
    fn from_physical_size(size: PhysicalSize<u32>) -> Self {
        Self {
            surface: RuntimeSurfaceConfig {
                width: size.width.max(1),
                height: size.height.max(1),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeSurfaceError {
    Lost,
    Outdated,
    Timeout,
    OutOfMemory,
    Internal,
}

impl Display for RuntimeSurfaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeSurfaceError::Lost => write!(f, "surface lost"),
            RuntimeSurfaceError::Outdated => write!(f, "surface outdated"),
            RuntimeSurfaceError::Timeout => write!(f, "surface timeout"),
            RuntimeSurfaceError::OutOfMemory => write!(f, "surface out of memory"),
            RuntimeSurfaceError::Internal => write!(f, "surface internal error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeSurfaceInitError {
    InvalidSize,
}

impl Display for RuntimeSurfaceInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeSurfaceInitError::InvalidSize => write!(f, "surface dimensions must be non-zero"),
        }
    }
}

#[derive(Debug)]
struct WgpuNativeSurface {
    config: RuntimeSurfaceConfig,
    frame: Vec<u8>,
}

impl WgpuNativeSurface {
    fn new(config: RuntimeSurfaceConfig) -> Result<Self, RuntimeSurfaceInitError> {
        if config.width == 0 || config.height == 0 {
            return Err(RuntimeSurfaceInitError::InvalidSize);
        }
        Ok(Self {
            config,
            frame: vec![0; (config.width * config.height * 4) as usize],
        })
    }

    fn frame_mut(&mut self) -> &mut [u8] {
        self.frame.as_mut_slice()
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.config = RuntimeSurfaceConfig {
            width: width.max(1),
            height: height.max(1),
        };
        self.frame
            .resize((self.config.width * self.config.height * 4) as usize, 0);
    }

    fn present(&mut self) -> Result<(), RuntimeSurfaceError> {
        if let Ok(simulated) = std::env::var("SHELL_ENGINE_WGPU_SURFACE_ERROR") {
            let error = match simulated.to_ascii_lowercase().as_str() {
                "lost" => RuntimeSurfaceError::Lost,
                "outdated" => RuntimeSurfaceError::Outdated,
                "timeout" => RuntimeSurfaceError::Timeout,
                "oom" | "out_of_memory" => RuntimeSurfaceError::OutOfMemory,
                _ => RuntimeSurfaceError::Internal,
            };
            return Err(error);
        }
        if self.config.width == 0 || self.config.height == 0 {
            return Err(RuntimeSurfaceError::Internal);
        }
        Ok(())
    }
}

/// Minimal keyboard subset shared from the winit runtime thread to the engine runtime thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuInputKey {
    Char(char),
    Esc,
    Up,
    Down,
    Left,
    Right,
}

/// Mouse button subset shared from the winit runtime thread to the engine runtime thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuInputMouseButton {
    Left,
    Right,
    Middle,
}

/// Modifier state snapshot attached to input events generated by the runtime thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WgpuInputModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

/// Input events emitted by the winit runtime thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WgpuInputEvent {
    Quit,
    FocusLost,
    MouseMoved {
        x: f32,
        y: f32,
    },
    MouseButtonDown {
        button: WgpuInputMouseButton,
        x: f32,
        y: f32,
        modifiers: WgpuInputModifiers,
    },
    MouseButtonUp {
        button: WgpuInputMouseButton,
        x: f32,
        y: f32,
        modifiers: WgpuInputModifiers,
    },
    MouseWheel {
        delta_y: f32,
        modifiers: WgpuInputModifiers,
    },
    KeyDown {
        key: WgpuInputKey,
        repeat: bool,
        modifiers: WgpuInputModifiers,
    },
    KeyUp {
        key: WgpuInputKey,
        modifiers: WgpuInputModifiers,
    },
}

/// Thread-safe queue used to bridge runtime-thread input into engine polling.
#[derive(Debug, Clone, Default)]
pub struct WgpuInputQueue {
    inner: Arc<Mutex<VecDeque<WgpuInputEvent>>>,
}

impl WgpuInputQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, event: WgpuInputEvent) {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(event);
        }
    }

    pub fn drain(&self) -> Vec<WgpuInputEvent> {
        if let Ok(mut queue) = self.inner.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug)]
struct RuntimeThread {
    tx: Sender<RuntimeCommand>,
    join: Option<JoinHandle<()>>,
}

impl RuntimeThread {
    fn spawn(
        config: PlatformWindowConfig,
        input_queue: WgpuInputQueue,
    ) -> Result<Self, RenderError> {
        let (tx, rx) = mpsc::channel::<RuntimeCommand>();
        let join = std::thread::Builder::new()
            .name("shell-engine-wgpu-runtime".to_string())
            .spawn(move || run_window_runtime(config, rx, input_queue))
            .map_err(|error| RenderError::InitFailed(error.to_string()))?;

        Ok(Self {
            tx,
            join: Some(join),
        })
    }

    fn send(&self, command: RuntimeCommand) -> Result<(), RenderError> {
        self.tx
            .send(command)
            .map_err(|error| RenderError::PresentFailed(error.to_string()))
    }

    fn shutdown(mut self) -> Result<(), RenderError> {
        let _ = self.send(RuntimeCommand::Shutdown);
        if let Some(join) = self.join.take() {
            join.join()
                .map_err(|_| RenderError::ShutdownFailed("runtime thread panicked".to_string()))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RuntimeApp {
    config: PlatformWindowConfig,
    rx: Receiver<RuntimeCommand>,
    input_queue: WgpuInputQueue,
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    surface: Option<WgpuNativeSurface>,
    clear_color: [u8; 4],
    should_exit: bool,
    cursor_pos: (f32, f32),
    modifiers: ModifiersState,
}

impl RuntimeApp {
    fn new(
        config: PlatformWindowConfig,
        rx: Receiver<RuntimeCommand>,
        input_queue: WgpuInputQueue,
    ) -> Self {
        Self {
            config,
            rx,
            input_queue,
            window: None,
            window_id: None,
            surface: None,
            clear_color: [16, 18, 24, 255],
            should_exit: false,
            cursor_pos: (0.0, 0.0),
            modifiers: ModifiersState::default(),
        }
    }

    fn modifiers_snapshot(&self) -> WgpuInputModifiers {
        WgpuInputModifiers {
            shift: self.modifiers.shift_key(),
            control: self.modifiers.control_key(),
            alt: self.modifiers.alt_key(),
        }
    }

    fn map_button(button: WinitMouseButton) -> Option<WgpuInputMouseButton> {
        match button {
            WinitMouseButton::Left => Some(WgpuInputMouseButton::Left),
            WinitMouseButton::Right => Some(WgpuInputMouseButton::Right),
            WinitMouseButton::Middle => Some(WgpuInputMouseButton::Middle),
            _ => None,
        }
    }

    fn map_key(physical_key: PhysicalKey) -> Option<WgpuInputKey> {
        match physical_key {
            PhysicalKey::Code(WinitKeyCode::KeyW) => Some(WgpuInputKey::Char('w')),
            PhysicalKey::Code(WinitKeyCode::KeyA) => Some(WgpuInputKey::Char('a')),
            PhysicalKey::Code(WinitKeyCode::KeyS) => Some(WgpuInputKey::Char('s')),
            PhysicalKey::Code(WinitKeyCode::KeyD) => Some(WgpuInputKey::Char('d')),
            PhysicalKey::Code(WinitKeyCode::Escape) => Some(WgpuInputKey::Esc),
            PhysicalKey::Code(WinitKeyCode::ArrowUp) => Some(WgpuInputKey::Up),
            PhysicalKey::Code(WinitKeyCode::ArrowDown) => Some(WgpuInputKey::Down),
            PhysicalKey::Code(WinitKeyCode::ArrowLeft) => Some(WgpuInputKey::Left),
            PhysicalKey::Code(WinitKeyCode::ArrowRight) => Some(WgpuInputKey::Right),
            _ => None,
        }
    }

    fn handle_redraw(&mut self, event_loop: &ActiveEventLoop) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        for pixel in surface.frame_mut().chunks_exact_mut(4) {
            pixel.copy_from_slice(&self.clear_color);
        }

        let render_status = match surface.present() {
            Ok(()) => WgpuSurfaceStatus::Presented,
            Err(error) => Self::classify_surface_status(&error),
        };
        self.handle_surface_status(render_status, event_loop);
    }

    fn build_init_config(&self) -> RuntimeInitConfig {
        RuntimeInitConfig::from_platform(&self.config)
    }

    fn create_surface(
        init_config: &RuntimeInitConfig,
        _window: Arc<Window>,
    ) -> Result<WgpuNativeSurface, RuntimeSurfaceInitError> {
        WgpuNativeSurface::new(init_config.surface)
    }

    fn handle_resize(&mut self, request: RuntimeResizeRequest) {
        if let Some(surface) = self.surface.as_mut() {
            surface.resize_surface(request.surface.width, request.surface.height);
        }
    }

    fn classify_surface_status(error: &RuntimeSurfaceError) -> WgpuSurfaceStatus {
        match error {
            RuntimeSurfaceError::Lost => WgpuSurfaceStatus::Lost,
            RuntimeSurfaceError::Outdated => WgpuSurfaceStatus::Outdated,
            RuntimeSurfaceError::Timeout => WgpuSurfaceStatus::Timeout,
            RuntimeSurfaceError::OutOfMemory | RuntimeSurfaceError::Internal => {
                WgpuSurfaceStatus::Fatal
            }
        }
    }

    fn handle_surface_status(
        &mut self,
        status: WgpuSurfaceStatus,
        event_loop: &ActiveEventLoop,
    ) {
        if status == WgpuSurfaceStatus::Presented {
            return;
        }

        self.should_exit = true;
        self.input_queue.push(WgpuInputEvent::Quit);
        event_loop.exit();
    }
}

impl ApplicationHandler for RuntimeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let init_config = self.build_init_config();
        let attributes = Window::default_attributes()
            .with_title(init_config.window_title.clone())
            .with_inner_size(LogicalSize::new(
                init_config.surface.width as f64,
                init_config.surface.height as f64,
            ));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("[wgpu-runtime] window creation failed: {error}");
                self.should_exit = true;
                event_loop.exit();
                return;
            }
        };

        let surface = match Self::create_surface(&init_config, window.clone()) {
            Ok(surface) => surface,
            Err(error) => {
                eprintln!("[wgpu-runtime] wgpu surface init failed: {error}");
                self.should_exit = true;
                event_loop.exit();
                return;
            }
        };

        self.window_id = Some(window.id());
        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.should_exit = true;
                self.input_queue.push(WgpuInputEvent::Quit);
                event_loop.exit();
            }
            WindowEvent::Focused(false) => {
                self.input_queue.push(WgpuInputEvent::FocusLost);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
                self.input_queue.push(WgpuInputEvent::MouseMoved {
                    x: self.cursor_pos.0,
                    y: self.cursor_pos.1,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = Self::map_button(button) {
                    let modifiers = self.modifiers_snapshot();
                    match state {
                        ElementState::Pressed => {
                            self.input_queue.push(WgpuInputEvent::MouseButtonDown {
                                button,
                                x: self.cursor_pos.0,
                                y: self.cursor_pos.1,
                                modifiers,
                            })
                        }
                        ElementState::Released => {
                            self.input_queue.push(WgpuInputEvent::MouseButtonUp {
                                button,
                                x: self.cursor_pos.0,
                                y: self.cursor_pos.1,
                                modifiers,
                            })
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y as f32,
                };
                self.input_queue.push(WgpuInputEvent::MouseWheel {
                    delta_y,
                    modifiers: self.modifiers_snapshot(),
                });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = Self::map_key(event.physical_key) {
                    let modifiers = self.modifiers_snapshot();
                    match event.state {
                        ElementState::Pressed => self.input_queue.push(WgpuInputEvent::KeyDown {
                            key,
                            repeat: event.repeat,
                            modifiers,
                        }),
                        ElementState::Released => self
                            .input_queue
                            .push(WgpuInputEvent::KeyUp { key, modifiers }),
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(RuntimeResizeRequest::from_physical_size(size));
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw(event_loop);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        while let Ok(command) = self.rx.try_recv() {
            match command {
                RuntimeCommand::SetClearColor(color) => self.clear_color = color,
                RuntimeCommand::Shutdown => {
                    self.should_exit = true;
                }
            }
        }

        if self.should_exit {
            event_loop.exit();
            return;
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn run_window_runtime(
    config: PlatformWindowConfig,
    rx: Receiver<RuntimeCommand>,
    input_queue: WgpuInputQueue,
) {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "windows")]
    {
        builder.with_any_thread(true);
    }
    let event_loop = match builder.build() {
        Ok(loop_handle) => loop_handle,
        Err(error) => {
            eprintln!("[wgpu-runtime] event loop init failed: {error}");
            return;
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = RuntimeApp::new(config, rx, input_queue);
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("[wgpu-runtime] event loop run failed: {error}");
    }
}

/// Hardware presenter used by the engine runtime.
#[derive(Debug)]
pub struct WgpuPresenter {
    output_size: (u16, u16),
    last_submission: Option<FrameSubmission>,
    overlay_line_count: usize,
    vector_primitive_count: usize,
    cleared_frames: usize,
    clipboard: Option<String>,
    runtime: Option<RuntimeThread>,
    runtime_error: Option<String>,
    runtime_spawn_enabled: bool,
    input_queue: WgpuInputQueue,
}

impl WgpuPresenter {
    /// Create a new hardware presenter for the requested output size.
    pub fn new(output_size: (u16, u16)) -> Result<Self, RenderError> {
        if output_size.0 == 0 || output_size.1 == 0 {
            return Err(RenderError::InitFailed(
                "output size must be non-zero".to_string(),
            ));
        }

        Ok(Self {
            output_size,
            last_submission: None,
            overlay_line_count: 0,
            vector_primitive_count: 0,
            cleared_frames: 0,
            clipboard: None,
            runtime: None,
            runtime_error: None,
            runtime_spawn_enabled: !cfg!(test),
            input_queue: WgpuInputQueue::new(),
        })
    }

    /// Return the most recent backend-neutral submission seen by this presenter.
    pub fn last_submission(&self) -> Option<FrameSubmission> {
        self.last_submission
    }

    /// Total overlay lines submitted to this presenter.
    pub fn overlay_line_count(&self) -> usize {
        self.overlay_line_count
    }

    /// Total vector primitives submitted to this presenter.
    pub fn vector_primitive_count(&self) -> usize {
        self.vector_primitive_count
    }

    /// Number of times `clear` was called.
    pub fn cleared_frames(&self) -> usize {
        self.cleared_frames
    }

    /// Exposes the thread-safe input queue consumed by engine input backends.
    pub fn input_queue(&self) -> WgpuInputQueue {
        self.input_queue.clone()
    }

    fn ensure_runtime(&mut self) -> Result<(), RenderError> {
        if !self.runtime_spawn_enabled {
            return Ok(());
        }
        if self.runtime.is_some() {
            return Ok(());
        }

        let config = PlatformWindowConfig::new(
            "Shell Engine Hardware Backend",
            self.output_size.0 as u32,
            self.output_size.1 as u32,
        );
        match RuntimeThread::spawn(config, self.input_queue.clone()) {
            Ok(runtime) => {
                self.runtime = Some(runtime);
                Ok(())
            }
            Err(error) => {
                self.runtime_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    fn send_runtime_command(&mut self, command: RuntimeCommand) -> Result<(), RenderError> {
        self.ensure_runtime()?;
        if let Some(runtime) = self.runtime.as_ref() {
            runtime.send(command)
        } else {
            Ok(())
        }
    }

    fn submission_color(submission: &FrameSubmission) -> [u8; 4] {
        match (submission.world.ready, submission.ui.ready) {
            (true, true) => [34, 92, 44, 255],
            (true, false) => [28, 48, 96, 255],
            (false, true) => [96, 60, 24, 255],
            (false, false) => [28, 30, 38, 255],
        }
    }
}

impl RendererBackend for WgpuPresenter {
    fn present_frame(&mut self, _buffer: &Buffer) {
        let _ = self.send_runtime_command(RuntimeCommand::SetClearColor([20, 22, 30, 255]));
    }

    fn backend_kind(&self) -> RenderBackendKind {
        RenderBackendKind::Hardware
    }

    fn submit_frame(&mut self, submission: &FrameSubmission) -> Result<(), RenderError> {
        self.last_submission = Some(*submission);
        self.send_runtime_command(RuntimeCommand::SetClearColor(Self::submission_color(submission)))
    }

    fn present_overlay(&mut self, overlay: &OverlayData) {
        self.overlay_line_count += overlay.lines.len();
    }

    fn present_vectors(&mut self, vectors: &VectorOverlay) {
        self.vector_primitive_count += vectors.primitives.len();
    }

    fn output_size(&self) -> (u16, u16) {
        self.output_size
    }

    fn copy_to_clipboard(&mut self, text: &str) -> bool {
        self.clipboard = Some(text.to_string());
        true
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        self.cleared_frames += 1;
        self.send_runtime_command(RuntimeCommand::SetClearColor([0, 0, 0, 255]))
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown()?;
        }
        Ok(())
    }
}

impl HardwareRendererBackend for WgpuPresenter {
    fn submit_frame(&mut self, submission: &FrameSubmission) -> Result<(), RenderError> {
        RendererBackend::submit_frame(self, submission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::color::Color;
    use engine_render::{
        HardwareRendererBackend, OverlayData, PreparedOverlay, PreparedUi, PreparedWorld,
        PresentationBackend, PresentMode, VectorOverlay,
    };

    #[test]
    fn renderer_backend_reports_hardware_kind_and_size() {
        let presenter = WgpuPresenter::new((1280, 720)).expect("presenter");
        assert_eq!(
            RendererBackend::backend_kind(&presenter),
            RenderBackendKind::Hardware
        );
        assert_eq!(RendererBackend::output_size(&presenter), (1280, 720));
    }

    #[test]
    fn renderer_backend_present_frame_is_safe_noop() {
        let mut presenter = WgpuPresenter::new((800, 600)).expect("presenter");
        let mut buffer = Buffer::new(2, 2);
        buffer.fill(Color::Black);
        RendererBackend::present_frame(&mut presenter, &buffer);
        assert_eq!(presenter.last_submission(), None);
    }

    #[test]
    fn renderer_backend_submit_frame_stores_submission() {
        let mut presenter = WgpuPresenter::new((800, 600)).expect("presenter");
        let submission = FrameSubmission {
            output_size: (800, 600),
            present_mode: PresentMode::Mailbox,
            world: PreparedWorld { ready: true },
            ui: PreparedUi { ready: false },
            overlay: PreparedOverlay {
                ready: true,
                line_count: 2,
                primitive_count: 3,
            },
        };
        RendererBackend::submit_frame(&mut presenter, &submission).expect("submission frame");
        assert_eq!(presenter.last_submission(), Some(submission));
    }

    #[test]
    fn presenter_tracks_staging_calls() {
        let mut presenter = WgpuPresenter::new((800, 600)).expect("presenter");
        let overlay = OverlayData::default();
        let vectors = VectorOverlay::default();
        PresentationBackend::present_overlay(&mut presenter, &overlay);
        PresentationBackend::present_vectors(&mut presenter, &vectors);
        assert!(PresentationBackend::copy_to_clipboard(
            &mut presenter,
            "test"
        ));

        assert_eq!(presenter.overlay_line_count(), 0);
        assert_eq!(presenter.vector_primitive_count(), 0);
    }

    #[test]
    fn renderer_backend_clear_and_shutdown_are_ok() {
        let mut presenter = WgpuPresenter::new((320, 200)).expect("presenter");
        assert!(RendererBackend::clear(&mut presenter).is_ok());
        assert_eq!(presenter.cleared_frames(), 1);
        assert!(RendererBackend::shutdown(&mut presenter).is_ok());
    }

    #[test]
    fn renderer_backend_zero_size_is_rejected() {
        let err = WgpuPresenter::new((0, 1080)).expect_err("must fail");
        match err {
            RenderError::InitFailed(message) => {
                assert!(message.contains("non-zero"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn hardware_renderer_backend_submit_frame_path_is_supported() {
        let mut presenter = WgpuPresenter::new((640, 360)).expect("presenter");
        let submission = FrameSubmission {
            output_size: (640, 360),
            present_mode: PresentMode::VSync,
            world: PreparedWorld { ready: false },
            ui: PreparedUi { ready: true },
            overlay: PreparedOverlay {
                ready: true,
                line_count: 0,
                primitive_count: 0,
            },
        };
        HardwareRendererBackend::submit_frame(&mut presenter, &submission).expect("submit");
        assert_eq!(presenter.last_submission(), Some(submission));
    }

    #[test]
    fn input_queue_round_trip() {
        let queue = WgpuInputQueue::new();
        queue.push(WgpuInputEvent::FocusLost);
        let events = queue.drain();
        assert_eq!(events, vec![WgpuInputEvent::FocusLost]);
    }

    #[test]
    fn runtime_init_config_maps_platform_window_config() {
        let platform = PlatformWindowConfig::new("Test Window", 1024, 768);
        let init = RuntimeInitConfig::from_platform(&platform);
        assert_eq!(init.window_title, "Test Window");
        assert_eq!(
            init.surface,
            RuntimeSurfaceConfig {
                width: 1024,
                height: 768,
            }
        );
    }

    #[test]
    fn runtime_resize_request_clamps_zero_dimensions() {
        let request = RuntimeResizeRequest::from_physical_size(PhysicalSize::new(0, 0));
        assert_eq!(
            request.surface,
            RuntimeSurfaceConfig {
                width: 1,
                height: 1,
            }
        );
    }

    #[test]
    fn surface_status_classifies_surface_error_variants() {
        let lost = RuntimeSurfaceError::Lost;
        let outdated = RuntimeSurfaceError::Outdated;
        let timeout = RuntimeSurfaceError::Timeout;
        let oom = RuntimeSurfaceError::OutOfMemory;

        assert_eq!(
            RuntimeApp::classify_surface_status(&lost),
            WgpuSurfaceStatus::Lost
        );
        assert_eq!(
            RuntimeApp::classify_surface_status(&outdated),
            WgpuSurfaceStatus::Outdated
        );
        assert_eq!(
            RuntimeApp::classify_surface_status(&timeout),
            WgpuSurfaceStatus::Timeout
        );
        assert_eq!(
            RuntimeApp::classify_surface_status(&oom),
            WgpuSurfaceStatus::Fatal
        );
    }

    #[test]
    fn surface_status_classifies_non_surface_errors_as_fatal() {
        let error = RuntimeSurfaceError::Internal;
        assert_eq!(
            RuntimeApp::classify_surface_status(&error),
            WgpuSurfaceStatus::Fatal
        );
    }
}
