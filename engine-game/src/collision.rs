//! Collision traits and a simple circle-circle default.

use crate::components::{Collider2D, ColliderShape, GameplayEvent, Transform2D};
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
    Toroid {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    },
}

/// Simple collision result emitted to callers; future versions can add normals/impulses.
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionHit {
    pub a: u64,
    pub b: u64,
}

pub fn collision_system(
    world: &GameplayWorld,
    strategies: &CollisionStrategies,
) -> Vec<CollisionHit> {
    let ids = world.ids_with_colliders();
    let mut hits = Vec::new();
    for i in 0..ids.len() {
        let a_id = ids[i];
        let Some(a_col) = world.collider(a_id) else {
            continue;
        };
        let Some(a_xf) = world.transform(a_id) else {
            continue;
        };
        for j in (i + 1)..ids.len() {
            let b_id = ids[j];
            let Some(b_col) = world.collider(b_id) else {
                continue;
            };
            let Some(b_xf) = world.transform(b_id) else {
                continue;
            };
            if !layers_interact(&a_col, &b_col) {
                continue;
            }
            if intersects(
                &a_col.shape,
                &a_xf,
                &b_col.shape,
                &b_xf,
                strategies.wrap_strategy,
            ) {
                hits.push(CollisionHit { a: a_id, b: b_id });
                // Emit collision enter events (bidirectional for script convenience)
                world.emit_event(GameplayEvent::CollisionEnter { a: a_id, b: b_id });
                world.emit_event(GameplayEvent::CollisionEnter { a: b_id, b: a_id });
            }
        }
    }
    hits
}

/// Particle collision system: checks particles with ParticlePhysics.collision=true
/// against entities whose tags match the particle's collision_mask.
/// Returns hits for script handling (e.g. particle despawn, damage).
pub fn particle_collision_system(
    world: &GameplayWorld,
    strategies: &CollisionStrategies,
) -> Vec<CollisionHit> {
    let mut hits = Vec::new();
    
    // Get all particles with collision enabled
    let particle_ids = world.ids_with_particle_physics();
    if particle_ids.is_empty() {
        return hits;
    }
    
    // Get all potential target entities (those with colliders)
    let target_ids = world.ids_with_colliders();
    
    for p_id in &particle_ids {
        let Some(pp) = world.particle_physics(*p_id) else {
            continue;
        };
        if !pp.collision || pp.collision_mask.is_empty() {
            continue;
        }
        let Some(p_xf) = world.transform(*p_id) else {
            continue;
        };
        // Particles are treated as small circles (radius 1-2 pixels)
        let p_radius = 2.0f32;
        
        for t_id in &target_ids {
            if *t_id == *p_id {
                continue;
            }
            
            // Check if target's tags match particle's collision_mask
            let target_tags = world.tags(*t_id);
            let matches_mask = pp.collision_mask.iter().any(|mask| target_tags.contains(mask));
            if !matches_mask {
                continue;
            }
            
            let Some(t_col) = world.collider(*t_id) else {
                continue;
            };
            let Some(t_xf) = world.transform(*t_id) else {
                continue;
            };
            
            // Check intersection
            let particle_shape = ColliderShape::Circle { radius: p_radius };
            if intersects(&particle_shape, &p_xf, &t_col.shape, &t_xf, strategies.wrap_strategy) {
                hits.push(CollisionHit { a: *p_id, b: *t_id });
                world.emit_event(GameplayEvent::CollisionEnter { a: *p_id, b: *t_id });
                world.emit_event(GameplayEvent::CollisionEnter { a: *t_id, b: *p_id });
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
        (ColliderShape::Circle { radius: ra }, ColliderShape::Polygon { points: pb }) => {
            circle_polygon(a_xf, *ra, b_xf, pb)
        }
        (ColliderShape::Polygon { points: pa }, ColliderShape::Circle { radius: rb }) => {
            circle_polygon(b_xf, *rb, a_xf, pa)
        }
        (ColliderShape::Polygon { points: pa }, ColliderShape::Polygon { points: pb }) => {
            polygon_polygon(a_xf, pa, b_xf, pb)
        }
    }
}

/// Circle vs polygon: approximate circle as its center point plus a radius check
/// against the polygon boundary, using `engine_physics` for exact intersection math.
fn circle_polygon(circle_xf: &Transform2D, radius: f32, poly_xf: &Transform2D, poly_points: &[[f32; 2]]) -> bool {
    let int_points: Vec<[i32; 2]> = poly_points
        .iter()
        .map(|p| [(p[0] + poly_xf.x).round() as i32, (p[1] + poly_xf.y).round() as i32])
        .collect();
    if int_points.len() < 3 {
        return false;
    }
    let cx = circle_xf.x.round() as i32;
    let cy = circle_xf.y.round() as i32;
    let r = radius.round() as i32;

    // Point inside polygon
    if engine_physics::point_in_polygon([cx, cy], &int_points, [0, 0]) {
        return true;
    }
    // Any polygon vertex inside circle
    for p in &int_points {
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        if dx * dx + dy * dy <= r * r {
            return true;
        }
    }
    // Circle center near any polygon edge
    let n = int_points.len();
    for i in 0..n {
        let a = int_points[i];
        let b = int_points[(i + 1) % n];
        if segment_point_dist_sq(a, b, [cx, cy]) <= (r * r) as i64 {
            return true;
        }
    }
    false
}

/// Polygon vs polygon using `engine_physics` geo-backed intersection.
fn polygon_polygon(a_xf: &Transform2D, pa: &[[f32; 2]], b_xf: &Transform2D, pb: &[[f32; 2]]) -> bool {
    let pa_i32: Vec<[i32; 2]> = pa
        .iter()
        .map(|p| [(p[0] + a_xf.x).round() as i32, (p[1] + a_xf.y).round() as i32])
        .collect();
    let pb_i32: Vec<[i32; 2]> = pb
        .iter()
        .map(|p| [(p[0] + b_xf.x).round() as i32, (p[1] + b_xf.y).round() as i32])
        .collect();
    engine_physics::polygons_intersect(&pa_i32, [0, 0], &pb_i32, [0, 0])
}

/// Squared distance from point `p` to segment `[a, b]`.
fn segment_point_dist_sq(a: [i32; 2], b: [i32; 2], p: [i32; 2]) -> i64 {
    let ax = a[0] as i64;
    let ay = a[1] as i64;
    let bx = b[0] as i64;
    let by = b[1] as i64;
    let px = p[0] as i64;
    let py = p[1] as i64;
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0 {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }
    let t = ((px - ax) * dx + (py - ay) * dy).clamp(0, len_sq);
    let proj_x = ax + t * dx / len_sq;
    let proj_y = ay + t * dy / len_sq;
    let ex = px - proj_x;
    let ey = py - proj_y;
    ex * ex + ey * ey
}

fn circle_circle(a: &Transform2D, ra: f32, b: &Transform2D, rb: f32, wrap: WrapStrategy) -> bool {
    match wrap {
        WrapStrategy::None => {
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let r = ra + rb;
            (dx * dx + dy * dy) <= r * r
        }
        WrapStrategy::Toroid {
            min_x,
            max_x,
            min_y,
            max_y,
        } => {
            let w = max_x - min_x;
            let h = max_y - min_y;
            let mut dx = (a.x - b.x).abs();
            let mut dy = (a.y - b.y).abs();
            if dx > w * 0.5 {
                dx = w - dx;
            }
            if dy > h * 0.5 {
                dy = h - dy;
            }
            let r = ra + rb;
            (dx * dx + dy * dy) <= r * r
        }
    }
}
