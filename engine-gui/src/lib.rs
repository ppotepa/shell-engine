//! `engine-gui` — declarative GUI widget model, hit-testing, and runtime state.
//!
//! # Responsibilities
//! - Define the [`GuiControl`] trait and concrete control types (Slider, Button, Toggle, Panel).
//! - Track per-widget runtime state ([`GuiRuntimeState`], [`GuiWidgetState`]).
//! - Process input events ([`engine_events::InputEvent`]) and update state ([`GuiSystem`]).
//!
//! # Non-responsibilities
//! - Rendering — handled by engine-compositor via Panel/Vector/Text sprites.
//! - Rhai scripting — handled by engine-behavior's `ScriptGuiApi`.
//! - Layout resolution — handled by Taffy inside engine-compositor.

pub mod control;
pub mod events;
pub mod state;
pub mod system;
pub mod widget;

/// Re-exported for consumers that previously imported from this crate.
pub use engine_events::MouseButton;
pub use state::{GuiRuntimeState, GuiWidgetState};
pub use system::GuiSystem;

// New trait-based API.
pub use control::{
    ButtonControl, GuiControl, PanelControl, SliderControl, ToggleControl, VisualSync, WidgetRect,
};

// Legacy enum — kept for backward compatibility during migration.
pub use widget::GuiWidgetDef;

/// Deprecated alias kept for backward compatibility.
#[deprecated(since = "0.0.0", note = "Use engine_events::InputEvent instead")]
pub use events::GuiInputEvent;
