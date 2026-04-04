// ============================================================================
// ASTEROIDS — Realistic RCS Simulation v4.0
// ============================================================================
// PHYSICS MODEL: EMISSION → REACTION → MOTION (Newton's 3rd Law)
//
//   1. INPUT (key press)
//   2. THRUSTER FIRES (spool time ~8ms)
//   3. PARTICLES EMIT (mass ejected in direction D)
//   4. REACTION FORCE (F = -ṁ·v, opposite to D)
//   5. ACCELERATION (a = F/m)
//   6. VELOCITY/ROTATION CHANGES (effect)
//
// KEY PRINCIPLE: Emission is the CAUSE, motion is the EFFECT
// BRAKING: Emit in direction of motion → reaction slows ship
// ============================================================================
if type_of(local) == "()" { local = #{}; }

let WORLD_W = 640.0;
let WORLD_H = 360.0;
let SHIP_X = 320.0;
let SHIP_Y = 180.0;

// ======================== PHYSICS CONSTANTS ================================
// Thruster physics - emission creates force
let THRUST_FORCE = 180.0;       // main engine thrust (units/s²)
let RCS_FORCE = 85.0;           // RCS thruster force (units/s²)
let TORQUE_ARM = 12.0;          // distance from center to RCS (affects rotation)

// Angular dynamics
let ANGULAR_ACCEL = 5.5;        // rad/s² from RCS torque
let ANGULAR_MAX = 7.0;          // max angular velocity rad/s
let ANGULAR_DEADBAND = 0.08;    // below this, consider "stable" (rad/s)

// Linear dynamics  
let LINEAR_DEADBAND = 1.5;      // below this speed, consider "stopped"

// Auto-brake activation
let AUTO_BRAKE_DELAY_MS = 25.0; // wait before auto-braking

// Timing
let DT = 0.016;                 // 16ms frame time
let IGNITION_DELAY_MS = 6.0;    // spool time before thrust
let IGNITION_RAMP_MS = 10.0;    // time to reach full intensity

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
// CORE PRINCIPLE: Emit particles → Apply reaction force
// Emission direction is OPPOSITE to resulting motion

// Emit thruster particles and return reaction force magnitude
fn emit_thruster(world, ship_id, emitter, intensity, particle_count, tag) {
    // intensity: 0.0-1.0, affects speed and count
    let base_speed = 80.0 + intensity * 60.0;
    let ttl = 120 + (intensity * 100.0);
    let ttl_i = if ttl < 120.0 { 120 } else if ttl > 280.0 { 280 } else { ttl };
    
    for i in 0..particle_count {
        let speed_var = base_speed + to_float(i) * 12.0 + rand() * 20.0;
        let fx = world.emit(emitter, ship_id, #{
            kind: "fx",
            template: "debris",
            speed: speed_var * intensity,
            ttl_ms: ttl_i,
            fg: "white",
            radius: if i < 2 { 2 } else { 1 },
            spread: (rand() - 0.5) * 0.08
        });
        if fx > 0 {
            world.tag_add(fx, tag);
        }
    }
    intensity  // return intensity as force factor
}

// Emit with specific direction override
fn emit_thruster_directed(world, ship_id, emitter, intensity, particle_count, tag, local_x, local_y) {
    let base_speed = 75.0 + intensity * 55.0;
    let ttl = 110 + (intensity * 90.0);
    let ttl_i = if ttl < 110.0 { 110 } else if ttl > 250.0 { 250 } else { ttl };
    
    for i in 0..particle_count {
        let speed_var = base_speed + to_float(i) * 10.0 + rand() * 18.0;
        let fx = world.emit(emitter, ship_id, #{
            kind: "fx",
            template: "debris",
            speed: speed_var * intensity,
            ttl_ms: ttl_i,
            fg: "white",
            radius: if i < 2 { 2 } else { 1 },
            spread: (rand() - 0.5) * 0.06,
            emission_local_x: local_x,
            emission_local_y: local_y
        });
        if fx > 0 {
            world.tag_add(fx, tag);
        }
    }
    intensity
}

// ======================== THRUSTER AUTHORITY ===============================
// Returns 0.0-1.0 indicating how aligned thruster is with desired brake direction

fn thruster_authority(thruster_local_x, thruster_local_y, brake_local_x, brake_local_y) {
    let t = normalize(thruster_local_x, thruster_local_y);
    let b = normalize(brake_local_x, brake_local_y);
    let d = dot(t["x"], t["y"], b["x"], b["y"]);
    if d < 0.0 { 0.0 } else { d }
}

// Get ship's velocity in world and local frame
fn get_ship_velocity(world, ship, ship_id) {
    let phys = world.physics(ship_id);
    if type_of(phys) != "map" { 
        return #{ vx: 0.0, vy: 0.0, speed: 0.0, vel_fwd: 0.0, vel_right: 0.0 }; 
    }
    let vx = phys["vx"] ?? 0.0;
    let vy = phys["vy"] ?? 0.0;
    let speed = sqrtf(vx * vx + vy * vy);
    
    let heading = ship.heading_vector();
    let fwd_x = heading["x"] ?? 0.0;
    let fwd_y = heading["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    // Project velocity onto local axes
    let vel_fwd = dot(vx, vy, fwd_x, fwd_y);
    let vel_right = dot(vx, vy, right_x, right_y);
    
    #{
        vx: vx,
        vy: vy,
        speed: speed,
        vel_fwd: vel_fwd,     // positive = moving forward
        vel_right: vel_right  // positive = moving right
    }
}

// ======================== PARTICLE RENDERING ===============================

fn update_thrust_particles(world) {
    // Main thrust: white → yellow → orange → red (hot exhaust)
    for id in world.query_tag("thrust-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }
        
        let life = p.lifetime_fraction();
        let r = if life > 0.7 { 2 } else if life > 0.4 { 2 } else { 1 };
        p.set_radius(r);
        
        let fg = if life > 0.85 {
            "white"
        } else if life > 0.70 {
            "#FFFFAA"
        } else if life > 0.55 {
            "#FFDD66"
        } else if life > 0.40 {
            "#FFAA33"
        } else if life > 0.25 {
            "#DD6622"
        } else {
            "#882211"
        };
        p.set_fg(fg);
    }
}

fn update_rcs_particles(world) {
    // RCS: white → pale → silver → gray (cold gas)
    for id in world.query_tag("rcs-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }
        
        let life = p.lifetime_fraction();
        let r = if life > 0.6 { 2 } else { 1 };
        p.set_radius(r);
        
        let fg = if life > 0.80 {
            "white"
        } else if life > 0.60 {
            "#DDDDEE"
        } else if life > 0.40 {
            "#AAAAAA"
        } else if life > 0.20 {
            "#666666"
        } else {
            "#333333"
        };
        p.set_fg(fg);
    }
}

fn update_brake_particles(world) {
    // Brake: white → blue-white → silver → gray (reverse thrust)
    for id in world.query_tag("brake-particle") {
        let p = world.entity(id);
        if !p.exists() { continue; }
        
        let life = p.lifetime_fraction();
        let r = if life > 0.65 { 2 } else { 1 };
        p.set_radius(r);
        
        let fg = if life > 0.82 {
            "white"
        } else if life > 0.65 {
            "#DDEEFF"
        } else if life > 0.48 {
            "#AACCDD"
        } else if life > 0.30 {
            "#7799AA"
        } else {
            "#445566"
        };
        p.set_fg(fg);
    }
}

// ======================== THRUSTER ACTIONS =================================
// Each action: EMIT particles → APPLY reaction force
// This is the core of the realistic physics model

fn calc_ignition_factor(ignition_ms, delay_ms, ramp_ms) {
    if ignition_ms < delay_ms { return 0.0; }
    let active_time = ignition_ms - delay_ms;
    if active_time >= ramp_ms { return 1.0; }
    active_time / ramp_ms
}

// Particle count scales with intensity
fn intensity_to_particles(intensity) {
    if intensity > 0.9 { return 6; }
    if intensity > 0.7 { return 5; }
    if intensity > 0.5 { return 4; }
    if intensity > 0.3 { return 3; }
    if intensity > 0.1 { return 2; }
    1
}

// ============================================================================
// MAIN ENGINE: Emits backward → Pushes ship forward
// ============================================================================
fn fire_main_engine(world, ship, ship_id, intensity) {
    if intensity < 0.05 { return; }
    
    // STEP 1: EMIT particles (CAUSE)
    let particles = intensity_to_particles(intensity);
    let thrust_particles = particles + 2;  // main engine is bigger
    emit_thruster(world, ship_id, "ship.main", intensity, thrust_particles, "thrust-particle");
    
    // STEP 2: APPLY reaction force (EFFECT)
    // Main emits DOWN (local +Y) → Reaction pushes UP (local -Y) → Ship moves forward
    let heading = ship.heading_vector();
    let fwd_x = heading["x"] ?? 0.0;
    let fwd_y = heading["y"] ?? -1.0;
    
    let THRUST_FORCE = 180.0;
    let DT = 0.016;
    let force = THRUST_FORCE * intensity * DT;
    
    // Apply forward acceleration
    ship.physics.add_velocity(fwd_x * force, fwd_y * force);
}

// ============================================================================
// BOW THRUSTER: Emits forward → Pushes ship backward (braking)
// ============================================================================
fn fire_bow_thruster(world, ship, ship_id, intensity) {
    if intensity < 0.05 { return; }
    
    // STEP 1: EMIT particles forward
    let particles = intensity_to_particles(intensity);
    emit_thruster(world, ship_id, "ship.bow", intensity, particles, "brake-particle");
    
    // STEP 2: APPLY reaction force backward
    let heading = ship.heading_vector();
    let fwd_x = heading["x"] ?? 0.0;
    let fwd_y = heading["y"] ?? -1.0;
    
    let RCS_FORCE = 85.0;
    let DT = 0.016;
    let force = RCS_FORCE * intensity * DT;
    
    // Apply backward deceleration (opposite to forward)
    ship.physics.add_velocity(-fwd_x * force, -fwd_y * force);
}

// ============================================================================
// PORT THRUSTER: Emits left → Pushes ship right + CCW torque
// ============================================================================
fn fire_port_thruster(world, ship, ship_id, intensity, apply_torque) {
    if intensity < 0.05 { return 0.0; }
    
    // STEP 1: EMIT particles left
    let particles = intensity_to_particles(intensity);
    emit_thruster(world, ship_id, "ship.port", intensity, particles, "rcs-particle");
    
    // STEP 2: APPLY reaction force right + torque
    let heading = ship.heading_vector();
    let fwd_x = heading["x"] ?? 0.0;
    let fwd_y = heading["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    let RCS_FORCE = 85.0;
    let DT = 0.016;
    let force = RCS_FORCE * intensity * DT;
    
    // Apply rightward force
    ship.physics.add_velocity(right_x * force, right_y * force);
    
    // Return torque contribution (CCW = negative angular velocity)
    if apply_torque {
        let ANGULAR_ACCEL = 5.5;
        -ANGULAR_ACCEL * intensity * DT
    } else {
        0.0
    }
}

// ============================================================================
// STARBOARD THRUSTER: Emits right → Pushes ship left + CW torque
// ============================================================================
fn fire_starboard_thruster(world, ship, ship_id, intensity, apply_torque) {
    if intensity < 0.05 { return 0.0; }
    
    // STEP 1: EMIT particles right
    let particles = intensity_to_particles(intensity);
    emit_thruster(world, ship_id, "ship.starboard", intensity, particles, "rcs-particle");
    
    // STEP 2: APPLY reaction force left + torque
    let heading = ship.heading_vector();
    let fwd_x = heading["x"] ?? 0.0;
    let fwd_y = heading["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    let RCS_FORCE = 85.0;
    let DT = 0.016;
    let force = RCS_FORCE * intensity * DT;
    
    // Apply leftward force
    ship.physics.add_velocity(-right_x * force, -right_y * force);
    
    // Return torque contribution (CW = positive angular velocity)
    if apply_torque {
        let ANGULAR_ACCEL = 5.5;
        ANGULAR_ACCEL * intensity * DT
    } else {
        0.0
    }
}

// ============================================================================
// ROTATION: Fire thruster pair for torque
// LEFT rotation (CCW): STARBOARD emits → ship rotates left
// RIGHT rotation (CW): PORT emits → ship rotates right
// ============================================================================
fn fire_rotation_thrusters(world, ship, ship_id, direction, intensity) {
    // direction: -1 = left (CCW), +1 = right (CW)
    if absf(direction) < 0.1 { return 0.0; }
    if intensity < 0.05 { return 0.0; }
    
    let torque = 0.0;
    
    if direction < 0.0 {
        // Rotate LEFT (CCW): fire STARBOARD (emits right → pushes left → CCW torque)
        torque = fire_starboard_thruster(world, ship, ship_id, intensity, true);
        // Also fire BOW with lateral component for torque couple
        let particles = intensity_to_particles(intensity * 0.5);
        emit_thruster_directed(world, ship_id, "ship.bow", intensity * 0.5, particles, "rcs-particle", 1.0, 0.0);
    } else {
        // Rotate RIGHT (CW): fire PORT (emits left → pushes right → CW torque)
        torque = fire_port_thruster(world, ship, ship_id, intensity, true);
        // Also fire BOW with lateral component
        let particles = intensity_to_particles(intensity * 0.5);
        emit_thruster_directed(world, ship_id, "ship.bow", intensity * 0.5, particles, "rcs-particle", -1.0, 0.0);
    }
    
    torque
}

// ============================================================================
// AUTO-BRAKE ROTATION: Fire opposite to angular velocity
// ============================================================================
fn fire_rotation_brake(world, ship, ship_id, angular_vel, intensity) {
    if absf(angular_vel) < 0.05 { return 0.0; }
    if intensity < 0.05 { return 0.0; }
    
    // Rotating CW (angular_vel > 0) → fire STARBOARD to brake
    // Rotating CCW (angular_vel < 0) → fire PORT to brake
    let torque = 0.0;
    
    if angular_vel > 0.0 {
        // Rotating CW → fire STARBOARD (creates CCW counter-torque)
        torque = fire_starboard_thruster(world, ship, ship_id, intensity, true);
    } else {
        // Rotating CCW → fire PORT (creates CW counter-torque)
        torque = fire_port_thruster(world, ship, ship_id, intensity, true);
    }
    
    torque
}

// ============================================================================
// AUTO-BRAKE LINEAR: Fire thrusters opposite to velocity direction
// Moving forward → fire BOW (brakes forward motion)
// Moving backward → fire MAIN (brakes backward motion)
// Moving right → fire STARBOARD (brakes rightward motion)
// Moving left → fire PORT (brakes leftward motion)
// ============================================================================
fn fire_linear_brake(world, ship, ship_id, vel_fwd, vel_right, intensity) {
    if intensity < 0.05 { return; }
    
    let BRAKE_THRESHOLD = 1.5;
    
    // Brake forward motion with BOW
    if vel_fwd > BRAKE_THRESHOLD {
        let fwd_intensity = intensity * (vel_fwd / 100.0);
        if fwd_intensity > 1.0 { fwd_intensity = 1.0; }
        if fwd_intensity < 0.15 { fwd_intensity = 0.15; }
        fire_bow_thruster(world, ship, ship_id, fwd_intensity);
    }
    
    // Brake backward motion with MAIN (reduced power)
    if vel_fwd < -BRAKE_THRESHOLD {
        let back_intensity = intensity * (absf(vel_fwd) / 150.0);
        if back_intensity > 0.6 { back_intensity = 0.6; }  // cap to avoid overshoot
        if back_intensity < 0.12 { back_intensity = 0.12; }
        fire_main_engine(world, ship, ship_id, back_intensity);
    }
    
    // Brake rightward motion with STARBOARD (no torque)
    if vel_right > BRAKE_THRESHOLD {
        let right_intensity = intensity * (vel_right / 80.0);
        if right_intensity > 1.0 { right_intensity = 1.0; }
        if right_intensity < 0.15 { right_intensity = 0.15; }
        fire_starboard_thruster(world, ship, ship_id, right_intensity, false);
    }
    
    // Brake leftward motion with PORT (no torque)
    if vel_right < -BRAKE_THRESHOLD {
        let left_intensity = intensity * (absf(vel_right) / 80.0);
        if left_intensity > 1.0 { left_intensity = 1.0; }
        if left_intensity < 0.15 { left_intensity = 0.15; }
        fire_port_thruster(world, ship, ship_id, left_intensity, false);
    }
}

// ======================== MAIN CONTROL UPDATE ==============================
// REALISTIC MODEL: Emission → Reaction → Motion
// Every thruster action FIRST emits particles, THEN applies reaction force

fn update_ship_controls(world, input, ship_id, state_prev) {
    // Physics constants
    let ANGULAR_MAX = 7.0;
    let ANGULAR_DEADBAND = 0.08;
    let LINEAR_DEADBAND = 1.5;
    let AUTO_BRAKE_DELAY_MS = 25.0;
    let DT = 0.016;
    let IGNITION_DELAY_MS = 6.0;
    let IGNITION_RAMP_MS = 10.0;
    
    if ship_id <= 0 {
        return #{
            angular_vel: 0.0,
            thrust_ignition_ms: 0.0,
            rotation_ignition_ms: 0.0,
            brake_ignition_ms: 0.0,
            rotation_dir: 0.0,
            no_input_ms: 0.0
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
            no_input_ms: 0.0
        };
    }

    // Unpack previous state
    let angular_vel = state_prev["angular_vel"] ?? 0.0;
    let thrust_ignition_ms = state_prev["thrust_ignition_ms"] ?? 0.0;
    let rotation_ignition_ms = state_prev["rotation_ignition_ms"] ?? 0.0;
    let brake_ignition_ms = state_prev["brake_ignition_ms"] ?? 0.0;
    let prev_rotation_dir = state_prev["rotation_dir"] ?? 0.0;
    let no_input_ms = state_prev["no_input_ms"] ?? 0.0;

    // Read inputs
    let turn_left = input.action_down("turn_left");
    let turn_right = input.action_down("turn_right");
    let thrusting = input.action_down("thrust");
    let input_turn = if turn_left && !turn_right { -1.0 } else if turn_right && !turn_left { 1.0 } else { 0.0 };
    
    let rotation_input_active = input_turn != 0.0;
    let linear_input_active = thrusting || input_turn != 0.0;

    // Track no-input duration
    if linear_input_active {
        no_input_ms = 0.0;
    } else {
        no_input_ms = no_input_ms + 16.0;
    }
    
    // Get current velocity state
    let vel = get_ship_velocity(world, ship, ship_id);
    let speed = vel["speed"];
    let vel_fwd = vel["vel_fwd"];
    let vel_right = vel["vel_right"];
    let still_moving = speed > LINEAR_DEADBAND;
    let still_rotating = absf(angular_vel) > ANGULAR_DEADBAND;

    // === MAIN ENGINE (player input) ===
    // Press thrust → MAIN emits → ship accelerates forward
    ship.set_thrust(thrusting);
    if thrusting {
        thrust_ignition_ms = thrust_ignition_ms + 16.0;
        let thrust_factor = calc_ignition_factor(thrust_ignition_ms, IGNITION_DELAY_MS, IGNITION_RAMP_MS);
        if thrust_factor > 0.05 {
            fire_main_engine(world, ship, ship_id, thrust_factor);
        }
    } else {
        thrust_ignition_ms = 0.0;
    }

    // === ROTATION (player input) ===
    // Press L/R → PORT/STARBOARD emits → ship rotates
    if rotation_input_active {
        let rotation_dir_changed = signf(input_turn) != signf(prev_rotation_dir) && prev_rotation_dir != 0.0;
        if rotation_dir_changed {
            rotation_ignition_ms = 0.0;
        }
        rotation_ignition_ms = rotation_ignition_ms + 16.0;
        
        let rot_factor = calc_ignition_factor(rotation_ignition_ms, IGNITION_DELAY_MS * 0.5, IGNITION_RAMP_MS * 0.7);
        
        if rot_factor > 0.05 {
            // EMIT thrusters → GET torque as reaction
            let torque = fire_rotation_thrusters(world, ship, ship_id, input_turn, rot_factor);
            angular_vel = angular_vel + torque;
        }
        
        // Clamp angular velocity
        if angular_vel > ANGULAR_MAX { angular_vel = ANGULAR_MAX; }
        if angular_vel < -ANGULAR_MAX { angular_vel = -ANGULAR_MAX; }
        
        brake_ignition_ms = 0.0;
    } else {
        rotation_ignition_ms = 0.0;
        
        // === AUTO-BRAKE ROTATION ===
        // Ship is rotating but no input → emit to brake
        if still_rotating {
            brake_ignition_ms = brake_ignition_ms + 16.0;
            let brake_factor = calc_ignition_factor(brake_ignition_ms, IGNITION_DELAY_MS * 0.3, IGNITION_RAMP_MS * 0.5);
            
            // Intensity proportional to angular velocity
            let intensity = 0.3 + absf(angular_vel) / ANGULAR_MAX * 0.7;
            if intensity > 1.0 { intensity = 1.0; }
            
            if brake_factor > 0.05 {
                // EMIT opposite → GET counter-torque
                let counter_torque = fire_rotation_brake(world, ship, ship_id, angular_vel, brake_factor * intensity);
                angular_vel = angular_vel + counter_torque;
            }
            
            // Natural damping (very small - space has no friction)
            angular_vel = angular_vel * 0.995;
            
            // Snap to zero if very small
            if absf(angular_vel) < ANGULAR_DEADBAND {
                angular_vel = 0.0;
            }
        } else {
            if !thrusting {
                brake_ignition_ms = 0.0;
            }
        }
    }

    // === AUTO-BRAKE LINEAR ===
    // No input + ship moving → emit to brake
    let linear_brake_active = no_input_ms >= AUTO_BRAKE_DELAY_MS && still_moving && !still_rotating;
    
    if linear_brake_active {
        let brake_factor = calc_ignition_factor(brake_ignition_ms + no_input_ms, IGNITION_DELAY_MS * 0.5, IGNITION_RAMP_MS * 0.8);
        
        // Intensity proportional to speed
        let intensity = 0.3 + speed / 150.0 * 0.7;
        if intensity > 1.0 { intensity = 1.0; }
        
        if brake_factor > 0.05 {
            // EMIT in direction of motion → reaction brakes ship
            fire_linear_brake(world, ship, ship_id, vel_fwd, vel_right, brake_factor * intensity);
        }
    }
    
    // Apply angular velocity to ship heading
    let command_turn = if angular_vel > 0.12 { 1 } else if angular_vel < -0.12 { -1 } else { 0 };
    ship.set_turn(command_turn);

    #{
        angular_vel: angular_vel,
        thrust_ignition_ms: thrust_ignition_ms,
        rotation_ignition_ms: rotation_ignition_ms,
        brake_ignition_ms: brake_ignition_ms,
        rotation_dir: input_turn,
        no_input_ms: no_input_ms
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
        no_input_ms: 0.0
    };
    input.load_profile("default");
    scene.set("hud-gameover", "visible", false);
    keep_asteroids_alive(world);
}

// ======================== MAIN LOOP ========================================

scene.set("hud-score", "text.content", "ASTEROIDS");
scene.set("hud-lives", "text.content", "RCS v4.0");
scene.set("hud-wave", "text.content", "REAL");

let ship_id = ensure_ship(world);
local.ship_id = ship_id;

let state_prev = local.control_state ?? #{
    angular_vel: 0.0,
    thrust_ignition_ms: 0.0,
    rotation_ignition_ms: 0.0,
    brake_ignition_ms: 0.0,
    rotation_dir: 0.0,
    no_input_ms: 0.0
};

let new_state = update_ship_controls(world, input, ship_id, state_prev);
local.control_state = new_state;

update_thrust_particles(world);
update_rcs_particles(world);
update_brake_particles(world);
keep_asteroids_alive(world);

#{state: local}
