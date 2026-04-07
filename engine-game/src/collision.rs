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

/// Collision result with contact normal (A→B direction).
/// normal_x/normal_y are unit vectors; both zero for polygon-polygon (normal not computed).
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionHit {
    pub a: u64,
    pub b: u64,
    /// Contact normal X component pointing from A toward B (unit vector).
    pub normal_x: f32,
    /// Contact normal Y component pointing from A toward B (unit vector).
    pub normal_y: f32,
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
        for &b_id in &ids[(i + 1)..] {
            let Some(b_col) = world.collider(b_id) else {
                continue;
            };
            let Some(b_xf) = world.transform(b_id) else {
                continue;
            };
            if !layers_interact(&a_col, &b_col) {
                continue;
            }
            if let Some((nx, ny)) = intersects_with_normal(
                &a_col.shape,
                &a_xf,
                &b_col.shape,
                &b_xf,
                strategies.wrap_strategy,
            ) {
                hits.push(CollisionHit { a: a_id, b: b_id, normal_x: nx, normal_y: ny });
                world.emit_event(GameplayEvent::CollisionEnter { a: a_id, b: b_id });
                world.emit_event(GameplayEvent::CollisionEnter { a: b_id, b: a_id });
            }
        }
    }
    hits
}

/// Apply impulse-based collision response for all rigid body hits.
/// Called after collision_system(), before physics integration writes positions.
/// Only processes pairs where both entities have a PhysicsBody2D.
/// Uses the hit normal and per-body mass/restitution for elastic/inelastic response.
pub fn apply_collision_response(world: &GameplayWorld, hits: &[CollisionHit]) {
    for hit in hits {
        let (Some(mut body_a), Some(mut body_b)) =
            (world.physics(hit.a), world.physics(hit.b))
        else {
            continue;
        };

        let nx = hit.normal_x;
        let ny = hit.normal_y;

        // Skip if normal is degenerate (polygon-polygon fallback)
        if nx == 0.0 && ny == 0.0 {
            continue;
        }

        // Relative velocity of B relative to A along the contact normal
        let rel_vx = body_b.vx - body_a.vx;
        let rel_vy = body_b.vy - body_a.vy;
        let dvn = rel_vx * nx + rel_vy * ny;

        // Already separating — skip (prevents double-application on sustained overlap)
        if dvn >= 0.0 {
            continue;
        }

        // Effective restitution: average of both bodies
        let e = (body_a.restitution + body_b.restitution) * 0.5;

        // Impulse scalar: j = -(1 + e) * dvn / (1/m_a + 1/m_b)
        // Infinite mass (mass == 0.0) means immovable.
        let inv_mass_a = if body_a.mass > 0.0 { 1.0 / body_a.mass } else { 0.0 };
        let inv_mass_b = if body_b.mass > 0.0 { 1.0 / body_b.mass } else { 0.0 };
        let inv_mass_sum = inv_mass_a + inv_mass_b;
        if inv_mass_sum == 0.0 {
            continue; // Both immovable
        }

        let j = -(1.0 + e) * dvn / inv_mass_sum;

        // Apply impulse proportional to inverse mass
        body_a.vx -= j * inv_mass_a * nx;
        body_a.vy -= j * inv_mass_a * ny;
        body_b.vx += j * inv_mass_b * nx;
        body_b.vy += j * inv_mass_b * ny;

        // Positional separation: push apart by half penetration depth to prevent tunnelling.
        // We use a small fixed push rather than computing true penetration depth.
        if let (Some(a_xf), Some(b_xf)) = (world.transform(hit.a), world.transform(hit.b)) {
            const SEPARATION: f32 = 0.5;
            let _ = world.set_transform(
                hit.a,
                Transform2D { x: a_xf.x - nx * SEPARATION, y: a_xf.y - ny * SEPARATION, heading: a_xf.heading },
            );
            let _ = world.set_transform(
                hit.b,
                Transform2D { x: b_xf.x + nx * SEPARATION, y: b_xf.y + ny * SEPARATION, heading: b_xf.heading },
            );
        }

        let _ = world.set_physics(hit.a, body_a);
        let _ = world.set_physics(hit.b, body_b);
    }
}

/// Apply bounce response for particles that have bounce > 0.0 set on their ParticlePhysics.
/// Reflects each particle's velocity along the contact normal, scaled by bounce coefficient.
pub fn apply_particle_bounce(world: &GameplayWorld, hits: &[CollisionHit]) {
    for hit in hits {
        // Only handle particle side (hit.a is always the particle in particle_collision_system)
        let Some(pp) = world.particle_physics(hit.a) else {
            continue;
        };
        if pp.bounce <= 0.0 {
            continue;
        }
        let Some(mut body) = world.physics(hit.a) else {
            continue;
        };

        let nx = hit.normal_x;
        let ny = hit.normal_y;
        if nx == 0.0 && ny == 0.0 {
            continue;
        }

        // Reflect: v' = v - 2(v·n)n, then scale by bounce coefficient
        let dot = body.vx * nx + body.vy * ny;
        body.vx = (body.vx - 2.0 * dot * nx) * pp.bounce;
        body.vy = (body.vy - 2.0 * dot * ny) * pp.bounce;
        let _ = world.set_physics(hit.a, body);
    }
}

/// Particle collision system: checks particles with ParticlePhysics.collision=true
/// against entities whose tags match the particle's collision_mask.
/// Returns hits for script handling (e.g. particle despawn, damage).
pub fn particle_collision_system(
    world: &GameplayWorld,
    strategies: &CollisionStrategies,
) -> Vec<CollisionHit> {
    let mut hits = Vec::new();

    let particle_ids = world.ids_with_particle_physics();
    if particle_ids.is_empty() {
        return hits;
    }

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
        let p_radius = 2.0f32;

        for t_id in &target_ids {
            if *t_id == *p_id {
                continue;
            }

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

            let particle_shape = ColliderShape::Circle { radius: p_radius };
            if let Some((nx, ny)) = intersects_with_normal(
                &particle_shape,
                &p_xf,
                &t_col.shape,
                &t_xf,
                strategies.wrap_strategy,
            ) {
                hits.push(CollisionHit { a: *p_id, b: *t_id, normal_x: nx, normal_y: ny });
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

/// Returns `Some((nx, ny))` — the A→B contact unit normal — when shapes intersect,
/// or `None` when they do not. Falls back to `(0.0, 0.0)` for polygon-polygon.
fn intersects_with_normal(
    a_shape: &ColliderShape,
    a_xf: &Transform2D,
    b_shape: &ColliderShape,
    b_xf: &Transform2D,
    wrap: WrapStrategy,
) -> Option<(f32, f32)> {
    match (a_shape, b_shape) {
        (ColliderShape::Circle { radius: ra }, ColliderShape::Circle { radius: rb }) => {
            circle_circle_normal(a_xf, *ra, b_xf, *rb, wrap)
        }
        (ColliderShape::Circle { radius: ra }, ColliderShape::Polygon { points: pb }) => {
            if circle_polygon(a_xf, *ra, b_xf, pb) {
                Some(center_to_center_normal(a_xf, b_xf))
            } else {
                None
            }
        }
        (ColliderShape::Polygon { points: pa }, ColliderShape::Circle { radius: rb }) => {
            if circle_polygon(b_xf, *rb, a_xf, pa) {
                Some(center_to_center_normal(a_xf, b_xf))
            } else {
                None
            }
        }
        (ColliderShape::Polygon { points: pa }, ColliderShape::Polygon { points: pb }) => {
            if polygon_polygon(a_xf, pa, b_xf, pb) {
                Some(center_to_center_normal(a_xf, b_xf))
            } else {
                None
            }
        }
    }
}

/// Compute the unit normal from a_xf center toward b_xf center.
/// Falls back to (1.0, 0.0) if centers overlap exactly.
fn center_to_center_normal(a_xf: &Transform2D, b_xf: &Transform2D) -> (f32, f32) {
    let dx = b_xf.x - a_xf.x;
    let dy = b_xf.y - a_xf.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1e-6 {
        (1.0, 0.0)
    } else {
        (dx / dist, dy / dist)
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

    if engine_physics::point_in_polygon([cx, cy], &int_points, [0, 0]) {
        return true;
    }
    for p in &int_points {
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        if dx * dx + dy * dy <= r * r {
            return true;
        }
    }
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

/// Returns `Some((nx, ny))` contact normal (A→B) when circles intersect, else `None`.
fn circle_circle_normal(a: &Transform2D, ra: f32, b: &Transform2D, rb: f32, wrap: WrapStrategy) -> Option<(f32, f32)> {
    let (dx, dy) = match wrap {
        WrapStrategy::None => (b.x - a.x, b.y - a.y),
        WrapStrategy::Toroid { min_x, max_x, min_y, max_y } => {
            let w = max_x - min_x;
            let h = max_y - min_y;
            let mut dx = b.x - a.x;
            let mut dy = b.y - a.y;
            if dx.abs() > w * 0.5 { dx -= dx.signum() * w; }
            if dy.abs() > h * 0.5 { dy -= dy.signum() * h; }
            (dx, dy)
        }
    };
    let dist_sq = dx * dx + dy * dy;
    let r_sum = ra + rb;
    if dist_sq <= r_sum * r_sum {
        let dist = dist_sq.sqrt();
        if dist < 1e-6 {
            Some((1.0, 0.0))
        } else {
            Some((dx / dist, dy / dist))
        }
    } else {
        None
    }
}



