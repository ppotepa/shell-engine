//! `PreparedFrame` — the output of the render thread, ready for backend presentation.
//!
//! The render pipeline has three explicit phases:
//!
//! 1. **Submit** (`try_offload_postfx_to_render_thread`): simulation sends a `RenderFrameRequest`
//!    (snapshot + buffers + ticket) to the render thread.
//!
//! 2. **Accept** (`apply_render_thread_result`): main thread polls for a `PreparedFrame`,
//!    validates its `FrameTicket` against the current ticket, and discards stale results.
//!    When accepted, the frame's buffers are staged back into `World`.
//!
//! 3. **Present** (`renderer_system`): main thread flushes the staged buffers to the active backend.
//!
//! `PreparedFrame` is the contract between phases 1→2 and 2→3. It owns the render buffers
//! and carries the ticket + object regions needed by the simulation thread on the next frame.

use crate::buffer::Buffer;
use crate::effects::Region;
use crate::frame_ticket::FrameTicket;
use std::collections::HashMap;

/// A fully composited and postfx-applied frame, ready for backend presentation.
///
/// Produced by the render thread after compositor → postfx. Validated by ticket
/// before being staged into World for `renderer_system`.
pub struct PreparedFrame {
    /// Rendered frame buffer.
    pub buffer: Buffer,
    /// Identity token echoed from the `RenderFrameRequest` — used for stale-frame rejection.
    pub ticket: FrameTicket,
    /// Object bounding regions computed during compositor pass.
    /// Applied to `SceneRuntime` on accept so hit-testing uses fresh layout next frame.
    pub object_regions: HashMap<String, Region>,
}
