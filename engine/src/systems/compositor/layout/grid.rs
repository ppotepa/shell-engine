//! Grid container layout resolution built on CSS-like track sizing.

use crate::assets::AssetRoot;
use crate::scene::{SceneRenderedMode, Sprite};
use taffy::geometry::Line;
use taffy::prelude::{
    auto, fr, length, line, span, AvailableSpace, Display, Size, Style, TaffyTree,
    TrackSizingFunction,
};

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
    if let Some(cells) = compute_grid_cells_taffy(
        columns,
        rows,
        children,
        container_w,
        container_h,
        gap_x,
        gap_y,
        inherited_mode,
        asset_root,
    ) {
        return cells;
    }
    compute_grid_cells_fallback(
        columns,
        rows,
        children,
        container_w,
        container_h,
        gap_x,
        gap_y,
        inherited_mode,
        asset_root,
    )
}

fn compute_grid_cells_taffy(
    columns: &[String],
    rows: &[String],
    children: &[Sprite],
    container_w: u16,
    container_h: u16,
    gap_x: u16,
    gap_y: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Option<Vec<(usize, GridCellRect)>> {
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

    let map_track = |spec: TrackSpec| -> TrackSizingFunction {
        match spec {
            TrackSpec::Auto => auto(),
            TrackSpec::Fr(w) => fr(w as f32),
            TrackSpec::Fixed(px) => length(px as f32),
        }
    };

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut child_nodes = Vec::with_capacity(children.len());
    for child in children {
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
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: auto(),
                    height: auto(),
                },
                min_size: Size {
                    width: length(pref_w.max(1) as f32),
                    height: length(pref_h.max(1) as f32),
                },
                grid_column: Line {
                    start: line((col_idx + 1) as i16),
                    end: span(col_span_clamped as u16),
                },
                grid_row: Line {
                    start: line((row_idx + 1) as i16),
                    end: span(row_span_clamped as u16),
                },
                ..Default::default()
            })
            .ok()?;
        child_nodes.push(node);
    }

    let root = taffy
        .new_with_children(
            Style {
                display: Display::Grid,
                size: Size {
                    width: length(container_w.max(1) as f32),
                    height: length(container_h.max(1) as f32),
                },
                grid_template_columns: col_specs.iter().copied().map(map_track).collect(),
                grid_template_rows: row_specs.iter().copied().map(map_track).collect(),
                gap: Size {
                    width: length(gap_x as f32),
                    height: length(gap_y as f32),
                },
                ..Default::default()
            },
            &child_nodes,
        )
        .ok()?;

    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(container_w.max(1) as f32),
                height: AvailableSpace::Definite(container_h.max(1) as f32),
            },
        )
        .ok()?;

    let mut out = Vec::with_capacity(child_nodes.len());
    for (idx, node) in child_nodes.iter().enumerate() {
        let layout = taffy.layout(*node).ok()?;
        let width = layout.size.width.round().max(1.0) as u16;
        let height = layout.size.height.round().max(1.0) as u16;
        let x = layout.location.x.round().max(0.0) as u16;
        let y = layout.location.y.round().max(0.0) as u16;
        out.push((
            idx,
            GridCellRect {
                x,
                y,
                width,
                height,
            },
        ));
    }

    Some(out)
}

fn compute_grid_cells_fallback(
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
            (
                idx,
                GridCellRect {
                    x,
                    y,
                    width,
                    height,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::compute_grid_cells;
    use crate::scene::{SceneRenderedMode, Sprite};

    fn grid_child(width: u16, height: u16, row: u16, col: u16) -> Sprite {
        let raw = format!(
            r#"
type: grid
width: {width}
height: {height}
grid-row: {row}
grid-col: {col}
columns: []
rows: []
"#
        );
        serde_yaml::from_str(&raw).expect("child sprite")
    }

    #[test]
    fn grid_fixed_tracks_place_children_in_expected_cells() {
        let children = vec![grid_child(2, 2, 1, 1), grid_child(2, 2, 2, 2)];
        let cells = compute_grid_cells(
            &["10".to_string(), "20".to_string()],
            &["5".to_string(), "7".to_string()],
            &children,
            40,
            20,
            1,
            1,
            SceneRenderedMode::Cell,
            None,
        );
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].1.x, 0);
        assert_eq!(cells[0].1.y, 0);
        assert_eq!(cells[0].1.width, 10);
        assert_eq!(cells[0].1.height, 5);
        assert_eq!(cells[1].1.x, 11);
        assert_eq!(cells[1].1.y, 6);
        assert_eq!(cells[1].1.width, 20);
        assert_eq!(cells[1].1.height, 7);
    }
}
