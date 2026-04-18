use crate::effects::passes::halo::apply_obj_halo_from_params;
use crate::ObjRenderParams;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RgbPostPassMetrics {
    pub halo_us: f32,
}

/// Apply RGB-space post-passes in a fixed order.
///
/// Keep this function as the single orchestration seam for all RGB post effects
/// so raster stays agnostic of individual pass internals.
pub(crate) fn apply_rgb_post_passes(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    params: &ObjRenderParams,
) -> RgbPostPassMetrics {
    RgbPostPassMetrics {
        halo_us: apply_obj_halo_from_params(canvas, virtual_w, virtual_h, params),
    }
}
