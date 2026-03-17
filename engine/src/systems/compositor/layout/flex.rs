//! Flex container layout resolution with content-sized children.

use crate::assets::AssetRoot;
use crate::scene::{FlexDirection, SceneRenderedMode, Sprite};

use super::area::GridCellRect;
use super::measure::measure_sprite_for_layout;

/// Computes child rectangles for a flex container.
pub(crate) fn compute_flex_cells(
    children: &[Sprite],
    direction: FlexDirection,
    container_w: u16,
    container_h: u16,
    gap: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Vec<(usize, GridCellRect)> {
    if children.is_empty() {
        return vec![];
    }

    let measurements: Vec<(u16, u16)> = children
        .iter()
        .map(|c| measure_sprite_for_layout(c, inherited_mode, asset_root))
        .collect();

    match direction {
        FlexDirection::Column => {
            let cell_w = container_w;
            let mut y_cursor = 0u16;
            measurements
                .iter()
                .enumerate()
                .map(|(i, &(_, h))| {
                    let h = h.max(1);
                    let rect = GridCellRect {
                        x: 0,
                        y: y_cursor,
                        width: cell_w,
                        height: h,
                    };
                    y_cursor = y_cursor
                        .saturating_add(h)
                        .saturating_add(if i + 1 < children.len() { gap } else { 0 });
                    (i, rect)
                })
                .collect()
        }
        FlexDirection::Row => {
            let cell_h = container_h;
            let mut x_cursor = 0u16;
            measurements
                .iter()
                .enumerate()
                .map(|(i, &(w, _))| {
                    let w = w.max(1);
                    let rect = GridCellRect {
                        x: x_cursor,
                        y: 0,
                        width: w,
                        height: cell_h,
                    };
                    x_cursor = x_cursor
                        .saturating_add(w)
                        .saturating_add(if i + 1 < children.len() { gap } else { 0 });
                    (i, rect)
                })
                .collect()
        }
    }
}
