//! Grid container layout resolution built on CSS-like track sizing.

use crate::assets::AssetRoot;
use crate::scene::{SceneRenderedMode, Sprite};

use super::area::GridCellRect;
use super::measure::measure_sprite_for_layout;
use super::tracks::{parse_track_spec, resolve_track_sizes, span_size, track_start, TrackSpec};

/// Computes child rectangles for a grid container.
pub(crate) fn compute_grid_cells(
    columns: &[String],
    rows: &[String],
    children: &[Sprite],
    container_w: u16,
    container_h: u16,
    gap_x: u16,
    gap_y: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Vec<(usize, GridCellRect)> {
    let col_specs: Vec<TrackSpec> = if columns.is_empty() {
        vec![TrackSpec::Fr(1)]
    } else {
        columns.iter().map(|c| parse_track_spec(c)).collect()
    };
    let row_specs: Vec<TrackSpec> = if rows.is_empty() {
        vec![TrackSpec::Fr(1)]
    } else {
        rows.iter().map(|r| parse_track_spec(r)).collect()
    };

    let mut col_auto_reqs: Vec<(usize, u16)> = Vec::new();
    let mut row_auto_reqs: Vec<(usize, u16)> = Vec::new();
    let mut placements: Vec<(usize, usize, usize, usize, usize)> =
        Vec::with_capacity(children.len());

    for (idx, child) in children.iter().enumerate() {
        let (row, col, row_span, col_span) = child.grid_position();
        let col_idx = (col as usize)
            .saturating_sub(1)
            .min(col_specs.len().saturating_sub(1));
        let row_idx = (row as usize)
            .saturating_sub(1)
            .min(row_specs.len().saturating_sub(1));
        let col_span_clamped = (col_span as usize)
            .max(1)
            .min(col_specs.len().saturating_sub(col_idx));
        let row_span_clamped = (row_span as usize)
            .max(1)
            .min(row_specs.len().saturating_sub(row_idx));

        let (pref_w, pref_h) = measure_sprite_for_layout(child, inherited_mode, asset_root);
        if col_span_clamped == 1 {
            col_auto_reqs.push((col_idx, pref_w));
        }
        if row_span_clamped == 1 {
            row_auto_reqs.push((row_idx, pref_h));
        }
        placements.push((idx, col_idx, row_idx, col_span_clamped, row_span_clamped));
    }

    let col_sizes = resolve_track_sizes(&col_specs, container_w, gap_x, &col_auto_reqs);
    let row_sizes = resolve_track_sizes(&row_specs, container_h, gap_y, &row_auto_reqs);

    placements
        .into_iter()
        .map(|(idx, col_idx, row_idx, col_span, row_span)| {
            let x = track_start(&col_sizes, gap_x, col_idx);
            let y = track_start(&row_sizes, gap_y, row_idx);
            let width = span_size(&col_sizes, gap_x, col_idx, col_span).max(1);
            let height = span_size(&row_sizes, gap_y, row_idx, row_span).max(1);
            (idx, GridCellRect { x, y, width, height })
        })
        .collect()
}
