//! Flex container layout resolution with content-sized children.

use crate::assets::AssetRoot;
use crate::scene::{FlexDirection as SceneFlexDirection, SceneRenderedMode, Sprite};
use taffy::prelude::{
    length, AvailableSpace, Dimension, Display, FlexDirection as TaffyFlexDirection, Size, Style,
    TaffyTree,
};

use super::area::GridCellRect;
use super::measure::measure_sprite_for_layout;

/// Computes child rectangles for a flex container.
pub(crate) fn compute_flex_cells(
    children: &[Sprite],
    direction: SceneFlexDirection,
    container_w: u16,
    container_h: u16,
    gap: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Vec<(usize, GridCellRect)> {
    if let Some(cells) = compute_flex_cells_taffy(
        children,
        direction,
        container_w,
        container_h,
        gap,
        inherited_mode,
        asset_root,
    ) {
        return cells;
    }
    compute_flex_cells_fallback(
        children,
        direction,
        container_w,
        container_h,
        gap,
        inherited_mode,
        asset_root,
    )
}

fn compute_flex_cells_taffy(
    children: &[Sprite],
    direction: SceneFlexDirection,
    container_w: u16,
    container_h: u16,
    gap: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Option<Vec<(usize, GridCellRect)>> {
    if children.is_empty() {
        return Some(vec![]);
    }

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut child_nodes = Vec::with_capacity(children.len());
    for child in children {
        let (pref_w, pref_h) = measure_sprite_for_layout(child, inherited_mode, asset_root);
        let child_style = match direction {
            SceneFlexDirection::Column => Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: length(pref_h.max(1) as f32),
                },
                ..Default::default()
            },
            SceneFlexDirection::Row => Style {
                size: Size {
                    width: length(pref_w.max(1) as f32),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            },
        };
        let node = taffy.new_leaf(child_style).ok()?;
        child_nodes.push(node);
    }

    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: match direction {
                    SceneFlexDirection::Column => TaffyFlexDirection::Column,
                    SceneFlexDirection::Row => TaffyFlexDirection::Row,
                },
                size: Size {
                    width: length(container_w.max(1) as f32),
                    height: length(container_h.max(1) as f32),
                },
                gap: Size {
                    width: length(gap as f32),
                    height: length(gap as f32),
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

fn compute_flex_cells_fallback(
    children: &[Sprite],
    direction: SceneFlexDirection,
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
        SceneFlexDirection::Column => {
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
        SceneFlexDirection::Row => {
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

#[cfg(test)]
mod tests {
    use super::compute_flex_cells;
    use crate::scene::{FlexDirection, SceneRenderedMode, Sprite};

    fn child_block(width: u16, height: u16) -> Sprite {
        let raw = format!(
            r#"
type: grid
width: {width}
height: {height}
columns: []
rows: []
"#
        );
        serde_yaml::from_str(&raw).expect("child sprite")
    }

    #[test]
    fn flex_row_respects_measured_width_and_gap() {
        let children = vec![child_block(3, 2), child_block(4, 2)];
        let cells = compute_flex_cells(
            &children,
            FlexDirection::Row,
            20,
            6,
            2,
            SceneRenderedMode::Cell,
            None,
        );
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].1.x, 0);
        assert_eq!(cells[0].1.width, 3);
        assert_eq!(cells[0].1.height, 6);
        assert_eq!(cells[1].1.x, 5);
        assert_eq!(cells[1].1.width, 4);
        assert_eq!(cells[1].1.height, 6);
    }

    #[test]
    fn flex_column_stretches_children_to_container_width() {
        let children = vec![child_block(3, 2), child_block(4, 5)];
        let cells = compute_flex_cells(
            &children,
            FlexDirection::Column,
            12,
            20,
            1,
            SceneRenderedMode::Cell,
            None,
        );
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].1.y, 0);
        assert_eq!(cells[0].1.width, 12);
        assert_eq!(cells[0].1.height, 2);
        assert_eq!(cells[1].1.y, 3);
        assert_eq!(cells[1].1.width, 12);
        assert_eq!(cells[1].1.height, 5);
    }
}
