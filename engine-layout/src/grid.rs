//! Grid container layout resolution built on CSS-like track sizing.

use engine_core::assets::AssetRoot;
use engine_core::scene::Sprite;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use taffy::geometry::Line;
use taffy::prelude::{
    auto, fr, length, line, span, AlignSelf, AvailableSpace, Dimension, Display, JustifySelf, Size,
    Style, TaffyTree, TrackSizingFunction,
};

use super::area::GridCellRect;
use super::tracks::{parse_track_spec, TrackSpec};

thread_local! {
    static GRID_LAYOUT_CACHE: RefCell<HashMap<u64, Vec<(usize, GridCellRect)>>> =
        RefCell::new(HashMap::new());
}

/// Flush cached grid layouts (call on scene change).
#[allow(dead_code)]
pub(crate) fn invalidate_grid_cache() {
    GRID_LAYOUT_CACHE.with(|c| c.borrow_mut().clear());
}

#[allow(clippy::too_many_arguments)]
fn grid_cache_key(
    measure_sprite: &impl Fn(&Sprite, Option<&AssetRoot>) -> (u16, u16),
    columns: &[String],
    rows: &[String],
    children: &[Sprite],
    container_w: u16,
    container_h: u16,
    gap_x: u16,
    gap_y: u16,
    asset_root: Option<&AssetRoot>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    container_w.hash(&mut h);
    container_h.hash(&mut h);
    gap_x.hash(&mut h);
    gap_y.hash(&mut h);
    columns.hash(&mut h);
    rows.hash(&mut h);
    for child in children {
        let (pw, ph) = measure_sprite(child, asset_root);
        pw.hash(&mut h);
        ph.hash(&mut h);
        let (row, col, rs, cs) = child.grid_position();
        row.hash(&mut h);
        col.hash(&mut h);
        rs.hash(&mut h);
        cs.hash(&mut h);
    }
    h.finish()
}

/// Computes child rectangles for a grid container.
#[allow(clippy::too_many_arguments)]
pub fn compute_grid_cells(
    columns: &[String],
    rows: &[String],
    children: &[Sprite],
    container_w: u16,
    container_h: u16,
    gap_x: u16,
    gap_y: u16,
    asset_root: Option<&AssetRoot>,
    measure_sprite: &impl Fn(&Sprite, Option<&AssetRoot>) -> (u16, u16),
) -> Vec<(usize, GridCellRect)> {
    // OPT-6: Cache hit returns previous result without TaffyTree rebuild.
    let cache_key = grid_cache_key(
        measure_sprite,
        columns,
        rows,
        children,
        container_w,
        container_h,
        gap_x,
        gap_y,
        asset_root,
    );
    let cached = GRID_LAYOUT_CACHE.with(|c| c.borrow().get(&cache_key).cloned());
    if let Some(result) = cached {
        return result;
    }

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
        let (pref_w, pref_h) = measure_sprite(child, asset_root);
        let min_width = if col_span_clamped == 1 && matches!(col_specs[col_idx], TrackSpec::Auto) {
            length(pref_w.max(1) as f32)
        } else {
            Dimension::Auto
        };
        let min_height = if row_span_clamped == 1 && matches!(row_specs[row_idx], TrackSpec::Auto) {
            length(pref_h.max(1) as f32)
        } else {
            Dimension::Auto
        };
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: auto(),
                    height: auto(),
                },
                min_size: Size {
                    width: min_width,
                    height: min_height,
                },
                // Preserve legacy compositor semantics: every child gets the full
                // resolved grid cell area, and sprite-local alignment (`at`) is
                // applied inside that cell by our renderer.
                justify_self: Some(JustifySelf::Stretch),
                align_self: Some(AlignSelf::Stretch),
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
            .expect("taffy: failed to allocate grid child node");
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
        .expect("taffy: failed to create grid root node");

    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(container_w.max(1) as f32),
                height: AvailableSpace::Definite(container_h.max(1) as f32),
            },
        )
        .expect("taffy: failed to compute grid layout");

    let mut out = Vec::with_capacity(child_nodes.len());
    for (idx, node) in child_nodes.iter().enumerate() {
        let layout = taffy
            .layout(*node)
            .expect("taffy: missing computed layout for grid child");
        let width = layout.size.width.round().max(1.0) as u16;
        let height = layout.size.height.round().max(1.0) as u16;
        let x = layout.location.x.max(0.0).floor() as u16;
        let y = layout.location.y.max(0.0).floor() as u16;
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

    GRID_LAYOUT_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(cache_key, out.clone());
    });
    out
}

#[cfg(test)]
mod tests {
    use super::compute_grid_cells;
    use engine_core::assets::AssetRoot;
    use engine_core::scene::Sprite;

    fn measure_sprite(sprite: &Sprite, _asset_root: Option<&AssetRoot>) -> (u16, u16) {
        match sprite {
            Sprite::Grid { width, height, .. } => (width.unwrap_or(1), height.unwrap_or(1)),
            _ => (1, 1),
        }
    }

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
            None,
            &measure_sprite,
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
