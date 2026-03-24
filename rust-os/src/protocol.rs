//! Re-exports engine-io types and helpers for use within rust-os.
pub use engine_io::{IoEvent, IoRequest};

pub fn emit_line(text: impl Into<String>, delay_ms: Option<u64>) -> IoEvent {
    IoEvent::EmitLine {
        text: text.into(),
        delay_ms,
    }
}

pub fn out(lines: Vec<String>) -> IoEvent {
    IoEvent::Out { lines }
}

pub fn set_prompt(text: impl Into<String>) -> IoEvent {
    IoEvent::SetPromptPrefix { text: text.into() }
}

pub fn set_masked(masked: bool) -> IoEvent {
    IoEvent::SetPromptMasked { masked }
}

pub fn clear() -> IoEvent {
    IoEvent::Clear
}
