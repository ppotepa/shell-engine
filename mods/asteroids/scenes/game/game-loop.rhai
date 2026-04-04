// ============================================================================
// ASTEROIDS — Full Realistic RCS Simulation v3.1
// ============================================================================
// AUTO-BRAKE SYSTEM:
//   - After 30ms of no input, auto-stabilization activates (fast response)
//   - PHASE 1: Stop rotation (fire RCS opposite to angular velocity)
//   - PHASE 2: Stop linear motion (fire thrusters opposite to velocity)
//   - Velocity projected onto LOCAL ship axes each frame
//   - Thruster selection based on velocity components in ship frame
// ============================================================================
if type_of(local) == "()" { local = #{}; }

let WORLD_W = 640.0;
let WORLD_H = 360.0;
let SHIP_X = 320.0;
let SHIP_Y = 180.0;

// ======================== PHYSICS CONSTANTS ================================
// Angular dynamics
let ANGULAR_ACCEL = 5.5;        // rad/s² when RCS fires (faster)
let ANGULAR_MAX = 7.0;          // max angular velocity rad/s
let ANGULAR_DEADBAND = 0.10;    // below this, consider "stable" (rad/s)

// Linear dynamics  
let LINEAR_DEADBAND = 2.5;      // below this speed, consider "stopped"

// Auto-brake activation (3x faster = 100ms / 3 ≈ 33ms)
let AUTO_BRAKE_DELAY_MS = 30.0; // wait 30ms after last input before auto-braking

// Timing (3x faster response)
let DT = 0.016;                 // 16ms frame time
let IGNITION_DELAY_MS = 8.0;    // spool time before full thrust (was 24)
let IGNITION_RAMP_MS = 12.0;    // time to reach full intensity (was 36)

// ======================== HELPER FUNCTIONS =================================

fn absf(x) {
    if x < 0.0 { -x } else { x }
}

fn signf(x) {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

fn sqrtf(x) {
    if x <= 0.0 { return 0.0; }
    let guess = x / 2.0;
    for i in 0..8 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

fn normalize(x, y) {
    let len = sqrtf(x * x + y * y);
    if len < 0.0001 { return #{ x: 0.0, y: 0.0 }; }
    #{ x: x / len, y: y / len }
}

fn dot(ax, ay, bx, by) {
    ax * bx + ay * by
}

fn keep_asteroids_alive(world) {
    let WORLD_W = 640.0;
    let WORLD_H = 360.0;
    if world.count_kind("asteroid") > 0 { return; }
    for i in 0..6 {
        let side = i % 4;
        let x = if side == 0 { -32.0 } else if side == 1 { WORLD_W + 32.0 } else { 20.0 + rand() * (WORLD_W - 40.0) };
        let y = if side == 2 { -32.0 } else if side == 3 { WORLD_H + 32.0 } else { 20.0 + rand() * (WORLD_H - 40.0) };
        let vx = if side == 0 { 60.0 } else if side == 1 { -60.0 } else { (rand() - 0.5) * 24.0 };
        let vy = if side == 2 { 60.0 } else if side == 3 { -60.0 } else { (rand() - 0.5) * 24.0 };
        world.spawn_prefab(if i % 2 == 0 { "asteroid-large" } else { "asteroid-medium" }, #{
            x: x, y: y, vx: vx, vy: vy
        });
    }
}

fn ensure_ship(world) {
    let WORLD_W = 640.0;
    let WORLD_H = 360.0;
    let SHIP_X = 320.0;
    let SHIP_Y = 180.0;
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    let ship_id = world.spawn_prefab("ship", #{
        x: SHIP_X,
        y: SHIP_Y,
        heading: 0.0
    });
    if ship_id > 0 {
        world.enable_wrap(ship_id, 0.0, WORLD_W, 0.0, WORLD_H);
    }
    ship_id
}

// ======================== EMISSION FUNCTIONS ===============================

// Emit with intensity scaling based on ignition state
fn emit_rcs_intensity(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed,
        ttl_ms: ttl_ms,
        fg: color,
        radius: radius,
        spread: (rand() - 0.5) * jitter
    });
    if fx > 0 {
        world.tag_add(fx, "thruster-particle");
    }
    fx
}

// Emit with explicit direction override
fn emit_rcs_directed(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius, local_x, local_y) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed,
        ttl_ms: ttl_ms,
        fg: color,
        radius: radius,
        spread: (rand() - 0.5) * jitter,
        emission_local_x: local_x,
        emission_local_y: local_y
    });
    if fx > 0 {
        world.tag_add(fx, "thruster-particle");
    }
    fx
}

// ======================== THRUSTER AUTHORITY ===============================
// Returns 0.0-1.0 indicating how aligned thruster is with desired brake direction

fn thruster_authority(thruster_local_x, thruster_local_y, brake_local_x, brake_local_y) {
    // Normalize thruster direction
    let t = normalize(thruster_local_x, thruster_local_y);
    let b = normalize(brake_local_x, brake_local_y);
    let d = dot(t["x"], t["y"], b["x"], b["y"]);
    // Only positive contribution counts
    if d < 0.0 { 0.0 } else { d }
}

// Get ship's velocity in local frame (for braking direction)
fn get_velocity_local(world, ship, ship_id) {
    let phys = world.physics(ship_id);
    if type_of(phys) != "map" { return #{ local_x: 0.0, local_y: 0.0, speed: 0.0 }; }
    let vx = phys["vx"] ?? 0.0;
    let vy = phys["vy"] ?? 0.0;
    let speed = sqrtf(vx * vx + vy * vy);
    if speed < 0.01 { return #{ local_x: 0.0, local_y: 0.0, speed: 0.0 }; }

    let heading = ship.heading_vector();
    let hx = heading["x"] ?? 0.0;
    let hy = heading["y"] ?? -1.0;
    
    // Ship local frame: forward = -heading, right = perpendicular
    let right_x = -hy;
    let right_y = hx;
    let fwd_x = hx;
    let fwd_y = hy;
    
    // Project velocity onto local axes
    let local_fwd = dot(vx, vy, fwd_x, fwd_y);
    let local_right = dot(vx, vy, right_x, right_y);
    
    #{
        local_x: local_right,   // positive = moving right
        local_y: -local_fwd,    // positive = moving backward (in ship coords)
        speed: speed
    }
}

// ======================== PARTICLE RENDERING ===============================

fn update_thruster_particles(world) {
    // Thruster particles: white → orange → red → dark (heat)
    for id in world.query_tag("thruster-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }

        let life = p.lifetime_fraction();
        let base_r = p.get_i("/radius", 1);
        let base_r = clamp_i(base_r, 1, 3);

        let r = if life > 0.75 {
            clamp_i(base_r + 1, 1, 4)
        } else if life > 0.45 {
            base_r
        } else if life > 0.20 {
            1
        } else {
            0
        };
        let dash_r = if r <= 0 { 0 } else { max(r, 2) };
        p.set_radius(dash_r);

        // Heat color ramp: white → orange → red → dark
        let fg = if life > 0.85 {
            "white"
        } else if life > 0.70 {
            "#FFEECC"
        } else if life > 0.55 {
            "#FFCC88"
        } else if life > 0.40 {
            "#FF9944"
        } else if life > 0.25 {
            "#CC6622"
        } else if life > 0.12 {
            "#662211"
        } else {
            "#221108"
        };
        p.set_fg(fg);
    }
}

fn update_brake_particles(world) {
    // Brake particles: white → gray → black (cold exhaust, no heat)
    for id in world.query_tag("brake-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }

        let life = p.lifetime_fraction();
        let base_r = p.get_i("/radius", 1);
        let base_r = clamp_i(base_r, 1, 3);

        // Shrink over time
        let r = if life > 0.80 {
            clamp_i(base_r + 1, 1, 4)
        } else if life > 0.55 {
            base_r
        } else if life > 0.30 {
            1
        } else {
            1
        };
        p.set_radius(max(r, 1));

        // Cold color ramp: white → silver → gray → charcoal → black
        let fg = if life > 0.85 {
            "white"
        } else if life > 0.70 {
            "#DDDDDD"
        } else if life > 0.55 {
            "#BBBBBB"
        } else if life > 0.40 {
            "#888888"
        } else if life > 0.25 {
            "#555555"
        } else if life > 0.12 {
            "#333333"
        } else {
            "#111111"
        };
        p.set_fg(fg);
    }
}

// ======================== BURST COUNT SCALING ==============================

fn rotation_burst_count(angular_vel) {
    let av = absf(angular_vel);
    if av > 4.5 { return 6; }
    if av > 3.0 { return 5; }
    if av > 2.0 { return 4; }
    if av > 1.0 { return 3; }
    if av > 0.4 { return 2; }
    1
}

fn brake_burst_count(speed) {
    if speed > 180.0 { return 7; }
    if speed > 120.0 { return 5; }
    if speed > 60.0 { return 4; }
    if speed > 30.0 { return 3; }
    if speed > 10.0 { return 2; }
    1
}

// ======================== IGNITION STATE ===================================

fn calc_ignition_factor(ignition_ms, delay_ms, ramp_ms) {
    if ignition_ms < delay_ms { return 0.0; }
    let active_time = ignition_ms - delay_ms;
    if active_time >= ramp_ms { return 1.0; }
    active_time / ramp_ms
}

// ======================== COUPLED RCS FOR ROTATION =========================
// Proper torque couple: two thrusters firing in opposite directions

fn fire_rotation_couple(world, ship_id, direction, angular_vel, ignition_factor) {
    // direction: -1.0 = rotate left (CCW), +1.0 = rotate right (CW)
    // For CCW rotation: fire port thruster outward (left), starboard-bow area would fire right
    // For CW rotation: fire starboard thruster outward (right), port-bow area would fire left
    
    let burst_count = rotation_burst_count(angular_vel);
    let intensity = 0.4 + absf(angular_vel) * 0.15;
    if intensity > 1.0 { intensity = 1.0; }
    
    let eff_factor = ignition_factor * intensity;
    
    // Primary thruster (rear corner)
    let primary_emitter = if direction < 0.0 { "ship.starboard" } else { "ship.port" };
    // Secondary thruster (opposite side, creates torque couple)
    let secondary_emitter = if direction < 0.0 { "ship.port" } else { "ship.starboard" };
    
    for i in 0..burst_count {
        let speed_var = to_float(i) * 8.0;
        // Primary: fires in its natural direction (outward from hull)
        emit_rcs_intensity(world, ship_id, primary_emitter, 
            95.0 + speed_var, 180, "white", 0.08, eff_factor, 1);
        
        // Secondary: fires at reduced rate (rear thruster pair effect)
        if i % 2 == 0 {
            emit_rcs_intensity(world, ship_id, secondary_emitter,
                70.0 + speed_var * 0.6, 140, "white", 0.06, eff_factor * 0.5, 1);
        }
    }
    
    // Bow thruster assists with lateral component
    if burst_count > 2 {
        for i in 0..(burst_count / 2) {
            // Bow fires perpendicular to help with rotation
            let bow_dir_x = if direction < 0.0 { 1.0 } else { -1.0 };
            emit_rcs_directed(world, ship_id, "ship.bow",
                65.0 + to_float(i) * 4.0, 120, "white", 0.04, eff_factor * 0.6, 1,
                bow_dir_x, 0.0);
        }
    }
}

// ======================== MAIN ENGINE THRUST ===============================

fn fire_main_engine(world, ship_id, thrust_ms, ignition_factor) {
    let intensity = 0.5 + ignition_factor * 0.5;
    
    // Dense exhaust plume
    for i in 0..6 {
        let speed = 140.0 + to_float(i) * 18.0;
        let color = if i < 2 { "white" } else if i < 4 { "#FFCC66" } else { "#FF8833" };
        emit_rcs_intensity(world, ship_id, "ship.main", speed, 380, color, 0.25, intensity, 2);
    }
    for i in 0..4 {
        let speed = 110.0 + to_float(i) * 12.0;
        emit_rcs_intensity(world, ship_id, "ship.main", speed, 280, "white", 0.18, intensity, 1);
    }
}

// ======================== AUTO-BRAKE: ROTATION =============================
// Fires RCS thrusters OPPOSITE to angular velocity direction

fn fire_auto_rotation_brake(world, ship_id, angular_vel, ignition_factor) {
    let av = absf(angular_vel);
    if av < 0.05 { return; }
    
    // Brake direction: opposite to rotation
    // Rotating LEFT (angular_vel < 0) → fire PORT to push back RIGHT
    // Rotating RIGHT (angular_vel > 0) → fire STARBOARD to push back LEFT
    let brake_emitter = if angular_vel < 0.0 { "ship.port" } else { "ship.starboard" };
    let opposite_emitter = if angular_vel < 0.0 { "ship.starboard" } else { "ship.port" };
    
    // Burst count proportional to angular velocity
    let burst_count = rotation_burst_count(av);
    
    // Intensity scales with angular velocity
    let intensity = 0.4 + av * 0.12;
    if intensity > 1.0 { intensity = 1.0; }
    let eff_factor = ignition_factor * intensity;
    
    // Primary brake thruster (fires outward)
    for i in 0..burst_count {
        let speed_base = 85.0 + av * 15.0;
        emit_rcs_intensity(world, ship_id, brake_emitter,
            speed_base + to_float(i) * 6.0, 160, "white", 0.06, eff_factor, 1);
    }
    
    // Secondary thruster (torque couple - fires at reduced rate)
    for i in 0..(burst_count / 2 + 1) {
        emit_rcs_intensity(world, ship_id, opposite_emitter,
            65.0 + to_float(i) * 4.0, 120, "white", 0.04, eff_factor * 0.4, 1);
    }
    
    // BOW assists with lateral component for torque
    let bow_dir_x = if angular_vel < 0.0 { -1.0 } else { 1.0 };
    for i in 0..(burst_count / 2) {
        emit_rcs_directed(world, ship_id, "ship.bow",
            55.0 + to_float(i) * 3.0, 100, "white", 0.03, eff_factor * 0.5, 1,
            bow_dir_x, 0.0);
    }
}

// ======================== AUTO-BRAKE: LINEAR ===============================
// Project world velocity onto ship's LOCAL axes each frame
// ACTUALLY APPLY BRAKING FORCE + visual particles
// Emission intensity PROPORTIONAL to velocity component being braked

fn fire_auto_linear_brake(world, ship, ship_id, ignition_factor) {
    // 1. Get velocity in WORLD coordinates
    let phys = world.physics(ship_id);
    if type_of(phys) != "map" { return; }
    let vx = phys["vx"] ?? 0.0;
    let vy = phys["vy"] ?? 0.0;
    let speed = sqrtf(vx * vx + vy * vy);
    if speed < 2.0 { return; }  // deadband
    
    // 2. Calculate braking deceleration (SLOWER - was 120, now 45)
    let brake_power = 45.0 * ignition_factor;  // units/s² deceleration
    let dt = 0.016;
    
    // Deceleration vector = -normalize(velocity) * brake_power * dt
    let dvx = -(vx / speed) * brake_power * dt;
    let dvy = -(vy / speed) * brake_power * dt;
    
    // Apply braking impulse to physics
    ship.physics.add_velocity(dvx, dvy);
    
    // 3. Get ship heading for thruster selection
    let h = ship.heading_vector();
    let fwd_x = h["x"] ?? 0.0;
    let fwd_y = h["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    // 4. Project velocity onto local axes (absolute values of components)
    let vel_fwd = vx * fwd_x + vy * fwd_y;
    let vel_right = vx * right_x + vy * right_y;
    let abs_vel_fwd = absf(vel_fwd);
    let abs_vel_right = absf(vel_right);
    
    // 5. Burst count PROPORTIONAL to the component being braked (not total speed)
    // Higher component = more particles from that thruster
    
    // BOW: moving forward (vel_fwd > 0)
    if vel_fwd > 0.0 {
        let ratio = vel_fwd / speed;  // 0..1
        // Burst count based on forward velocity component
        let component_bursts = component_burst_count(abs_vel_fwd);
        let intensity = 0.4 + ratio * 0.6;  // scales with ratio
        
        for i in 0..component_bursts {
            emit_brake_particle(world, ship_id, "ship.bow",
                70.0 + abs_vel_fwd * 0.4 + to_float(i) * 4.0, 200,
                intensity * ignition_factor, 1);
        }
    }
    
    // MAIN: moving backward (vel_fwd < 0)
    if vel_fwd < 0.0 {
        let ratio = -vel_fwd / speed;
        let component_bursts = component_burst_count(abs_vel_fwd);
        let intensity = 0.4 + ratio * 0.6;
        
        for i in 0..component_bursts {
            emit_brake_particle(world, ship_id, "ship.main",
                85.0 + abs_vel_fwd * 0.45 + to_float(i) * 5.0, 240,
                intensity * ignition_factor, 2);
        }
    }
    
    // STARBOARD: moving right (vel_right > 0)
    if vel_right > 0.0 {
        let ratio = vel_right / speed;
        let component_bursts = component_burst_count(abs_vel_right);
        let intensity = 0.4 + ratio * 0.6;
        
        for i in 0..component_bursts {
            emit_brake_particle(world, ship_id, "ship.starboard",
                65.0 + abs_vel_right * 0.35 + to_float(i) * 3.0, 170,
                intensity * ignition_factor, 1);
        }
    }
    
    // PORT: moving left (vel_right < 0)
    if vel_right < 0.0 {
        let ratio = -vel_right / speed;
        let component_bursts = component_burst_count(abs_vel_right);
        let intensity = 0.4 + ratio * 0.6;
        
        for i in 0..component_bursts {
            emit_brake_particle(world, ship_id, "ship.port",
                65.0 + abs_vel_right * 0.35 + to_float(i) * 3.0, 170,
                intensity * ignition_factor, 1);
        }
    }
}

// Burst count based on velocity COMPONENT (not total speed)
fn component_burst_count(component_vel) {
    if component_vel > 150.0 { return 8; }
    if component_vel > 100.0 { return 6; }
    if component_vel > 60.0 { return 5; }
    if component_vel > 35.0 { return 4; }
    if component_vel > 15.0 { return 3; }
    if component_vel > 5.0 { return 2; }
    1
}

// Emit brake particle with white color, tagged for white→black fade
fn emit_brake_particle(world, ship_id, emitter, speed, ttl_ms, intensity, radius) {
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed * intensity,
        ttl_ms: ttl_ms,
        fg: "white",
        radius: radius,
        spread: (rand() - 0.5) * 0.03
    });
    if fx > 0 {
        world.tag_add(fx, "brake-particle");
    }
    fx
}

// ======================== SETTLING PUFF ====================================
// Random micro-corrections when nearly stopped

fn fire_settling_puff(world, ship_id, ignition_factor) {
    let emitters = ["ship.bow", "ship.port", "ship.starboard", "ship.main"];
    let idx_f = rand() * 3.99;
    let idx = if idx_f < 1.0 { 0 } else if idx_f < 2.0 { 1 } else if idx_f < 3.0 { 2 } else { 3 };
    let emitter = emitters[idx];
    emit_rcs_intensity(world, ship_id, emitter, 25.0, 50, "white", 0.01, ignition_factor * 0.25, 1);
}

// ======================== FINAL STABILIZATION BURST ========================
// When velocity is very low (about to stop), fire ALL thrusters simultaneously
// in appropriate directions for dramatic final correction effect
// Uses thin, light particles for micro-puff feel

fn emit_final_burst_particle(world, ship_id, emitter, speed, ttl_ms) {
    // Medium-visible particles - between thin brake and full thrust
    // More visible but not as big as regular thrust
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed * (0.7 + rand() * 0.4),  // variable speed
        ttl_ms: ttl_ms,
        fg: "white",
        radius: 2,  // medium size (thrust=2-3, brake=1)
        spread: (rand() - 0.5) * 0.12  // moderate spread
    });
    if fx > 0 {
        world.tag_add(fx, "final-burst-particle");
    }
    fx
}

fn fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel) {
    // Get ship heading
    let h = ship.heading_vector();
    let fwd_x = h["x"] ?? 0.0;
    let fwd_y = h["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    // Project residual velocity onto local axes
    let vel_fwd = vx * fwd_x + vy * fwd_y;
    let vel_right = vx * right_x + vy * right_y;
    
    // Random burst count 3-5 particles per thruster (more visible)
    let base_bursts = 3 + (rand() * 2.5);
    let base_bursts = if base_bursts < 3.5 { 3 } else if base_bursts < 4.5 { 4 } else { 5 };
    
    // Fire ALL thrusters simultaneously for dramatic final correction
    // Each fires in direction to counter residual motion
    
    // BOW: fires if moving forward (counter forward drift)
    if vel_fwd > 0.3 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.bow", 55.0 + to_float(i) * 12.0, 140);
        }
    }
    
    // MAIN: fires if moving backward (counter backward drift)
    if vel_fwd < -0.3 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.main", 65.0 + to_float(i) * 14.0, 160);
        }
    }
    
    // STARBOARD: fires if moving right (counter rightward drift)
    if vel_right > 0.3 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.starboard", 50.0 + to_float(i) * 10.0, 130);
        }
    }
    
    // PORT: fires if moving left (counter leftward drift)
    if vel_right < -0.3 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.port", 50.0 + to_float(i) * 10.0, 130);
        }
    }
    
    // Angular stabilization: fire RCS pair opposite to residual rotation
    if absf(angular_vel) > 0.02 {
        let rot_bursts = 2 + (rand() * 2.0);
        let rot_bursts = if rot_bursts < 2.5 { 2 } else if rot_bursts < 3.5 { 3 } else { 4 };
        
        // Fire opposite to rotation direction
        let emitter1 = if angular_vel > 0.0 { "ship.starboard" } else { "ship.port" };
        let emitter2 = if angular_vel > 0.0 { "ship.port" } else { "ship.starboard" };
        
        for i in 0..rot_bursts {
            emit_final_burst_particle(world, ship_id, emitter1, 45.0 + to_float(i) * 8.0, 110);
            if rand() > 0.3 {
                emit_final_burst_particle(world, ship_id, emitter2, 38.0 + to_float(i) * 6.0, 100);
            }
        }
    }
}

fn update_final_burst_particles(world) {
    // Final burst particles: more visible, medium brightness
    // White → pale → silver → gray (slower fade than brake, more visible)
    for id in world.query_tag("final-burst-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }
        
        let life = p.lifetime_fraction();
        
        // Medium radius - shrinks near end
        let r = if life > 0.6 { 2 } else if life > 0.3 { 2 } else { 1 };
        p.set_radius(r);
        
        // Slower, more visible fade: white → pale → silver → gray → charcoal
        let fg = if life > 0.80 {
            "white"
        } else if life > 0.65 {
            "#EEEEFF"  // pale blue-white
        } else if life > 0.50 {
            "#CCCCDD"  // light silver
        } else if life > 0.35 {
            "#AAAAAA"  // silver
        } else if life > 0.20 {
            "#777777"  // gray
        } else {
            "#444444"  // charcoal
        };
        p.set_fg(fg);
    }
}

// ======================== MAIN CONTROL UPDATE ==============================

fn update_ship_controls(world, input, ship_id, state_prev) {
    // Physics constants (local to function) - 3x faster response
    let ANGULAR_ACCEL = 5.5;
    let ANGULAR_MAX = 7.0;
    let ANGULAR_DEADBAND = 0.10;
    let LINEAR_DEADBAND = 2.5;
    let AUTO_BRAKE_DELAY_MS = 30.0;
    let DT = 0.016;
    let IGNITION_DELAY_MS = 8.0;
    let IGNITION_RAMP_MS = 12.0;
    
    // Final stabilization burst thresholds
    let FINAL_BURST_SPEED_THRESHOLD = 8.0;  // trigger when speed drops below this
    let FINAL_BURST_INTERVAL_MS = 55.0;      // time between burst waves (~3-4 frames)
    let FINAL_BURST_MAX_WAVES = 3;           // 3 burst waves before full stop
    
    if ship_id <= 0 {
        return #{
            angular_vel: 0.0,
            thrust_ignition_ms: 0.0,
            rotation_ignition_ms: 0.0,
            brake_ignition_ms: 0.0,
            rotation_dir: 0.0,
            no_input_ms: 0.0,
            auto_brake_phase: "idle",
            final_burst_waves: 0,
            final_burst_timer_ms: 0.0,
            final_burst_triggered: false
        };
    }
    let ship = world.entity(ship_id);
    if !ship.exists() {
        return #{
            angular_vel: 0.0,
            thrust_ignition_ms: 0.0,
            rotation_ignition_ms: 0.0,
            brake_ignition_ms: 0.0,
            rotation_dir: 0.0,
            no_input_ms: 0.0,
            auto_brake_phase: "idle",
            final_burst_waves: 0,
            final_burst_timer_ms: 0.0,
            final_burst_triggered: false
        };
    }

    // Unpack previous state
    let angular_vel = state_prev["angular_vel"] ?? 0.0;
    let thrust_ignition_ms = state_prev["thrust_ignition_ms"] ?? 0.0;
    let rotation_ignition_ms = state_prev["rotation_ignition_ms"] ?? 0.0;
    let brake_ignition_ms = state_prev["brake_ignition_ms"] ?? 0.0;
    let prev_rotation_dir = state_prev["rotation_dir"] ?? 0.0;
    let no_input_ms = state_prev["no_input_ms"] ?? 0.0;
    let auto_brake_phase = state_prev["auto_brake_phase"] ?? "idle";
    let final_burst_waves = state_prev["final_burst_waves"] ?? 0;
    let final_burst_timer_ms = state_prev["final_burst_timer_ms"] ?? 0.0;
    let final_burst_triggered = state_prev["final_burst_triggered"] ?? false;

    // Read inputs
    let turn_left = input.action_down("turn_left");
    let turn_right = input.action_down("turn_right");
    let thrusting = input.action_down("thrust");
    let input_turn = if turn_left && !turn_right { -1.0 } else if turn_right && !turn_left { 1.0 } else { 0.0 };
    
    // SEPARATE tracking for rotation vs linear
    // Rotation auto-brake: only blocked by L/R input (NOT by thrust)
    // Linear auto-brake: blocked by thrust OR L/R
    let rotation_input_active = input_turn != 0.0;
    let linear_input_active = thrusting || input_turn != 0.0;

    // === TRACK NO-INPUT DURATION (for linear braking only) ===
    if linear_input_active {
        no_input_ms = 0.0;
    } else {
        no_input_ms = no_input_ms + 16.0;
    }
    
    // Get current velocity state
    let vel_local = get_velocity_local(world, ship, ship_id);
    let speed = vel_local["speed"];
    let still_moving = speed > LINEAR_DEADBAND;
    let still_rotating = absf(angular_vel) > ANGULAR_DEADBAND;

    // === MAIN ENGINE (player input) ===
    ship.set_thrust(thrusting);
    if thrusting {
        thrust_ignition_ms = thrust_ignition_ms + 16.0;
        let thrust_factor = calc_ignition_factor(thrust_ignition_ms, IGNITION_DELAY_MS, IGNITION_RAMP_MS);
        fire_main_engine(world, ship_id, thrust_ignition_ms, thrust_factor);
    } else {
        thrust_ignition_ms = 0.0;
    }

    // === ROTATION (player input) ===
    if rotation_input_active {
        // Player is actively turning
        let rotation_dir_changed = signf(input_turn) != signf(prev_rotation_dir) && prev_rotation_dir != 0.0;
        if rotation_dir_changed {
            rotation_ignition_ms = 0.0;
        }
        rotation_ignition_ms = rotation_ignition_ms + 16.0;
        
        let rot_factor = calc_ignition_factor(rotation_ignition_ms, IGNITION_DELAY_MS * 0.5, IGNITION_RAMP_MS * 0.7);
        
        // Apply torque
        let torque = input_turn * ANGULAR_ACCEL * rot_factor;
        angular_vel = angular_vel + torque * DT;
        
        // Clamp
        if angular_vel > ANGULAR_MAX { angular_vel = ANGULAR_MAX; }
        if angular_vel < -ANGULAR_MAX { angular_vel = -ANGULAR_MAX; }
        
        // Fire rotation thrusters
        if rot_factor > 0.1 {
            fire_rotation_couple(world, ship_id, input_turn, angular_vel, rot_factor);
        }
        
        // Reset rotation brake state
        auto_brake_phase = "idle";
    } else {
        rotation_ignition_ms = 0.0;
        
        // === AUTO-BRAKE ROTATION (even while thrusting!) ===
        if still_rotating {
            brake_ignition_ms = brake_ignition_ms + 16.0;
            let brake_factor = calc_ignition_factor(brake_ignition_ms, IGNITION_DELAY_MS * 0.3, IGNITION_RAMP_MS * 0.5);
            
            auto_brake_phase = "rotation";
            
            // Counter-torque proportional to angular velocity
            let brake_torque = -signf(angular_vel) * absf(angular_vel) * 4.5 * brake_factor;
            angular_vel = angular_vel + brake_torque * DT;
            
            // Fire RCS opposite to rotation direction
            if brake_factor > 0.1 {
                fire_auto_rotation_brake(world, ship_id, state_prev["angular_vel"] ?? angular_vel, brake_factor);
            }
            
            // Check if rotation stopped
            if absf(angular_vel) < ANGULAR_DEADBAND {
                angular_vel = 0.0;
            }
        } else {
            // Rotation is stable
            if !thrusting {
                brake_ignition_ms = 0.0;
            }
        }
    }

    // === AUTO-BRAKE LINEAR (only when no thrust AND no rotation input) ===
    let linear_brake_active = no_input_ms >= AUTO_BRAKE_DELAY_MS && still_moving && !still_rotating;
    
    // Get raw velocity for final burst calculations
    let phys = world.physics(ship_id);
    let vx = if type_of(phys) == "map" { phys["vx"] ?? 0.0 } else { 0.0 };
    let vy = if type_of(phys) == "map" { phys["vy"] ?? 0.0 } else { 0.0 };
    
    if linear_brake_active {
        auto_brake_phase = "linear";
        let linear_brake_factor = calc_ignition_factor(brake_ignition_ms + no_input_ms, IGNITION_DELAY_MS * 0.5, IGNITION_RAMP_MS * 0.8);
        
        if linear_brake_factor > 0.1 {
            fire_auto_linear_brake(world, ship, ship_id, linear_brake_factor);
        }
        
        // === FINAL STABILIZATION BURST ===
        // When velocity drops below threshold, fire dramatic multi-wave micro-bursts
        if speed < FINAL_BURST_SPEED_THRESHOLD && speed > LINEAR_DEADBAND {
            // Check if we should trigger final burst sequence
            if !final_burst_triggered {
                // Start final burst sequence
                final_burst_triggered = true;
                final_burst_waves = 0;
                final_burst_timer_ms = 0.0;
            }
            
            // Fire burst waves at intervals
            if final_burst_waves < FINAL_BURST_MAX_WAVES {
                final_burst_timer_ms = final_burst_timer_ms + 16.0;
                
                if final_burst_timer_ms >= FINAL_BURST_INTERVAL_MS {
                    // Fire a wave!
                    fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel);
                    final_burst_waves = final_burst_waves + 1;
                    final_burst_timer_ms = 0.0;
                }
            }
        }
    } else if !linear_input_active && !still_rotating && !still_moving {
        auto_brake_phase = "stopped";
        // Reset final burst state when fully stopped
        final_burst_triggered = false;
        final_burst_waves = 0;
        final_burst_timer_ms = 0.0;
        // Occasional micro-puffs for settling effect
        if rand() < 0.05 {
            fire_settling_puff(world, ship_id, 0.3);
        }
    } else if !rotation_input_active && !still_rotating && thrusting {
        // Thrusting but not rotating - don't change phase
        auto_brake_phase = "thrusting";
    }
    
    // Reset final burst when player takes control
    if linear_input_active {
        final_burst_triggered = false;
        final_burst_waves = 0;
        final_burst_timer_ms = 0.0;
    }
    
    // Apply angular velocity to ship heading
    let command_turn = if angular_vel > 0.15 { 1 } else if angular_vel < -0.15 { -1 } else { 0 };
    ship.set_turn(command_turn);

    #{
        angular_vel: angular_vel,
        thrust_ignition_ms: thrust_ignition_ms,
        rotation_ignition_ms: rotation_ignition_ms,
        brake_ignition_ms: brake_ignition_ms,
        rotation_dir: input_turn,
        no_input_ms: no_input_ms,
        auto_brake_phase: auto_brake_phase,
        final_burst_waves: final_burst_waves,
        final_burst_timer_ms: final_burst_timer_ms,
        final_burst_triggered: final_burst_triggered
    }
}

// ======================== INITIALIZATION ===================================

if !(local.initialized ?? false) {
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    local.control_state = #{
        angular_vel: 0.0,
        thrust_ignition_ms: 0.0,
        rotation_ignition_ms: 0.0,
        brake_ignition_ms: 0.0,
        rotation_dir: 0.0,
        no_input_ms: 0.0,
        auto_brake_phase: "idle",
        final_burst_waves: 0,
        final_burst_timer_ms: 0.0,
        final_burst_triggered: false
    };
    input.load_profile("default");
    scene.set("hud-gameover", "visible", false);
    keep_asteroids_alive(world);
}

// ======================== MAIN LOOP ========================================

scene.set("hud-score", "text.content", "ASTEROIDS");
scene.set("hud-lives", "text.content", "RCS v3.2");
scene.set("hud-wave", "text.content", "BURST");

let ship_id = ensure_ship(world);
local.ship_id = ship_id;

let state_prev = local.control_state ?? #{
    angular_vel: 0.0,
    thrust_ignition_ms: 0.0,
    rotation_ignition_ms: 0.0,
    brake_ignition_ms: 0.0,
    rotation_dir: 0.0,
    no_input_ms: 0.0,
    auto_brake_phase: "idle",
    final_burst_waves: 0,
    final_burst_timer_ms: 0.0,
    final_burst_triggered: false
};

let new_state = update_ship_controls(world, input, ship_id, state_prev);
local.control_state = new_state;

update_thruster_particles(world);
update_brake_particles(world);
update_final_burst_particles(world);
keep_asteroids_alive(world);

#{state: local}
