//! Collision traits and a simple circle-circle default.

use crate::components::{Collider2D, ColliderShape, Transform2D};
use crate::GameplayWorld;

#[derive(Default)]
pub struct CollisionStrategies {
    pub broadphase: BroadphaseKind,
    pub narrowphase: NarrowphaseKind,
    pub wrap_strategy: WrapStrategy,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BroadphaseKind {
    #[default]
    BruteForce,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum NarrowphaseKind {
    #[default]
    Circle,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum WrapStrategy {
    #[default]
    None,
    Toroid { min_x: f32, max_x: f32, min_y: f32, max_y: f32 },
}

/// Simple collision result emitted to callers; future versions can add normals/impulses.
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionHit {
    pub a: u64,
    pub b: u64,
}

pub fn collision_system(world: &GameplayWorld, strategies: &CollisionStrategies) -> Vec<CollisionHit> {
    let ids = world.ids_with_colliders();
    let mut hits = Vec::new();
    for i in 0..ids.len() {
        let a_id = ids[i];
        let Some(a_col) = world.collider(a_id) else { continue };
        let Some(a_xf) = world.transform(a_id) else { continue };
        for j in (i + 1)..ids.len() {
            let b_id = ids[j];
            let Some(b_col) = world.collider(b_id) else { continue };
            let Some(b_xf) = world.transform(b_id) else { continue };
            if !layers_interact(&a_col, &b_col) {
                continue;
            }
            if intersects(&a_col.shape, &a_xf, &b_col.shape, &b_xf, strategies.wrap_strategy) {
                hits.push(CollisionHit { a: a_id, b: b_id });
            }
        }
    }
    hits
}

fn layers_interact(a: &Collider2D, b: &Collider2D) -> bool {
    (a.mask & b.layer) != 0 && (b.mask & a.layer) != 0
}

fn intersects(
    a_shape: &ColliderShape,
    a_xf: &Transform2D,
    b_shape: &ColliderShape,
    b_xf: &Transform2D,
    wrap: WrapStrategy,
) -> bool {
    match (a_shape, b_shape) {
        (ColliderShape::Circle { radius: ra }, ColliderShape::Circle { radius: rb }) => {
            circle_circle(a_xf, *ra, b_xf, *rb, wrap)
        }
        _ => false, // polygon support can be added later
    }
}

fn circle_circle(a: &Transform2D, ra: f32, b: &Transform2D, rb: f32, wrap: WrapStrategy) -> bool {
    match wrap {
        WrapStrategy::None => {
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let r = ra + rb;
            (dx * dx + dy * dy) <= r * r
        }
        WrapStrategy::Toroid { min_x, max_x, min_y, max_y } => {
            let w = max_x - min_x;
            let h = max_y - min_y;
            let mut dx = (a.x - b.x).abs();
            let mut dy = (a.y - b.y).abs();
            if dx > w * 0.5 { dx = w - dx; }
            if dy > h * 0.5 { dy = h - dy; }
            let r = ra + rb;
            (dx * dx + dy * dy) <= r * r
        }
    }
}
