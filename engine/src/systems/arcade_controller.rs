//! Arcade controller system: manages discrete heading rotation and thrust acceleration.
//!
//! Each frame, for entities with ArcadeController:
//! 1. Apply turn accumulation (discrete heading updates)
//! 2. Update Transform2D heading to match controller heading
//! 3. If thrusting, calculate thrust vector and update PhysicsBody2D acceleration
//! 4. Clamp velocity to max_speed if configured
//!
//! This system runs BEFORE physics integration so that the calculated acceleration
//! affects the next frame's velocity.

use engine_game::GameplayWorld;

/// Run arcade controller logic for all entities with controllers.
///
/// Must be called BEFORE the physics integration step each frame.
/// Updates heading based on turn input, syncs to Transform2D, and applies thrust.
///
/// When an entity also has an [`AngularBody`] component, that component is
/// authoritative for heading. In that case this system skips the discrete turn
/// steps and reads `Transform2D.heading` directly for the thrust vector, so the
/// two systems cooperate rather than fight.
pub fn arcade_controller_system(world: &GameplayWorld, dt_ms: u64) {
    let controller_ids: Vec<u64> = world.ids_with_controller();

    for id in controller_ids {
        let Some(mut controller) = world.controller(id) else {
            continue;
        };

        // AngularBody (if present) owns heading — skip discrete turn steps.
        let has_angular_body = world.angular_body(id).is_some();

        if !has_angular_body {
            // Apply turn accumulation (frame-rate independent rotation)
            controller.turn_accumulator += dt_ms as u32;
            let heading_bits = controller.heading_bits as i32;

            while controller.turn_accumulator >= controller.turn_step_ms {
                match controller.turn_direction {
                    -1 => {
                        controller.current_heading =
                            (controller.current_heading + heading_bits - 1) % heading_bits;
                    }
                    1 => {
                        controller.current_heading =
                            (controller.current_heading + 1) % heading_bits;
                    }
                    _ => {}
                }
                controller.turn_accumulator -= controller.turn_step_ms;
            }

            // Sync heading to Transform2D
            if let Some(mut xf) = world.transform(id) {
                xf.heading = controller.heading_radians();
                let _ = world.set_transform(id, xf);
            }
        }

        // Apply thrust acceleration — use xf.heading when AngularBody is authoritative.
        if controller.is_thrusting {
            let (thrust_x, thrust_y) = if has_angular_body {
                let heading = world.transform(id).map(|xf| xf.heading).unwrap_or(0.0);
                (heading.sin(), -heading.cos())
            } else {
                controller.heading_vector()
            };

            let accel_x = thrust_x * controller.thrust_power;
            let accel_y = thrust_y * controller.thrust_power;

            if let Some(mut body) = world.physics(id) {
                body.ax = accel_x;
                body.ay = accel_y;
                let _ = world.set_physics(id, body);
            }
        } else {
            if let Some(mut body) = world.physics(id) {
                body.ax = 0.0;
                body.ay = 0.0;
                let _ = world.set_physics(id, body);
            }
        }

        if !has_angular_body {
            let _ = world.with_controller(id, |ctrl| {
                *ctrl = controller;
            });
        }
    }
}
