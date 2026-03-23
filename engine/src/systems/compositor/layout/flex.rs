//! Flex container layout resolution with content-sized children.

use crate::assets::AssetRoot;
use crate::scene::{FlexDirection as SceneFlexDirection, SceneRenderedMode, Sprite};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use taffy::prelude::{
    length, AvailableSpace, Dimension, Display, FlexDirection as TaffyFlexDirection, Size, Style,
    TaffyTree,
};

use super::area::GridCellRect;
use super::measure::measure_sprite_for_layout;

thread_local! {
    static FLEX_LAYOUT_CACHE: RefCell<HashMap<u64, Vec<(usize, GridCellRect)>>> =
        RefCell::new(HashMap::new());
}

/// Flush cached flex layouts (call on scene change).
#[allow(dead_code)]
pub(crate) fn invalidate_flex_cache() {
    FLEX_LAYOUT_CACHE.with(|c| c.borrow_mut().clear());
}

fn flex_cache_key(
    children: &[Sprite],
    direction: SceneFlexDirection,
    container_w: u16,
    container_h: u16,
    gap: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    container_w.hash(&mut h);
    container_h.hash(&mut h);
    gap.hash(&mut h);
    std::mem::discriminant(&direction).hash(&mut h);
    for child in children {
        let (pw, ph) = measure_sprite_for_layout(child, inherited_mode, asset_root);
        pw.hash(&mut h);
        ph.hash(&mut h);
    }
    h.finish()
}

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
    if children.is_empty() {
        return vec![];
    }

    // OPT-6: Cache hit returns previous result without TaffyTree rebuild.
    let cache_key = flex_cache_key(children, direction, container_w, container_h, gap, inherited_mode, asset_root);
    let cached = FLEX_LAYOUT_CACHE.with(|c| c.borrow().get(&cache_key).cloned());
    if let Some(result) = cached {
        return result;
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
        let node = taffy
            .new_leaf(child_style)
            .expect("taffy: failed to allocate flex child node");
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
        .expect("taffy: failed to create flex root node");

    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(container_w.max(1) as f32),
                height: AvailableSpace::Definite(container_h.max(1) as f32),
            },
        )
        .expect("taffy: failed to compute flex layout");

    let mut out = Vec::with_capacity(child_nodes.len());
    for (idx, node) in child_nodes.iter().enumerate() {
        let layout = taffy
            .layout(*node)
            .expect("taffy: missing computed layout for flex child");
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

    FLEX_LAYOUT_CACHE.with(|c| {
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
