// ============================================================================
// ASTEROIDS — Full Realistic RCS Simulation v3.1
// ============================================================================
// AUTO-BRAKE SYSTEM:
//   - After 30ms of no input, auto-stabilization activates (fast response)
//   - PHASE 1: Stop rotation (fire RCS opposite to angular velocity)
//   - PHASE 2: Stop linear motion (fire thrusters opposite to velocity)
//   - Velocity projected onto LOCAL ship axes each frame
//   - Thruster selection based on velocity components in ship frame
//
// GAMEPLAY:
//   SPACE/Fire  — shoot bullets
//   Arrows/WASD — thrust and rotate
//   Asteroids split on hit (large → 2 medium, medium → 2 small, small → destroyed)
//   Ship collides with asteroids → lose life, respawn
//   Score: large=20, medium=50, small=100
// ============================================================================
if type_of(local) == "()" { local = #{}; }

let WORLD_W = 640.0;
let WORLD_H = 360.0;
let SHIP_X = 320.0;
let SHIP_Y = 180.0;

// ======================== PALETTE SETUP ====================================
// Read active palette once per frame; fall back to defaults if not loaded.
let PAL_BG           = palette.get("bg");           if PAL_BG           == "" { PAL_BG           = "#060620"; }
let PAL_SHIP         = palette.get("ship");         if PAL_SHIP         == "" { PAL_SHIP         = "white"; }
let PAL_ASTEROID     = palette.get("asteroid");     if PAL_ASTEROID     == "" { PAL_ASTEROID     = "white"; }
let PAL_BULLET       = palette.get("bullet");       if PAL_BULLET       == "" { PAL_BULLET       = "yellow"; }
let PAL_DEBRIS       = palette.get("debris");       if PAL_DEBRIS       == "" { PAL_DEBRIS       = "gray"; }
let PAL_HUD_LABEL    = palette.get("hud_label");    if PAL_HUD_LABEL    == "" { PAL_HUD_LABEL    = "#2e5eaa"; }
let PAL_HUD_VALUE    = palette.get("hud_value");    if PAL_HUD_VALUE    == "" { PAL_HUD_VALUE    = "#5bc0be"; }
let PAL_GAMEOVER     = palette.get("gameover");     if PAL_GAMEOVER     == "" { PAL_GAMEOVER     = "#f6ae2d"; }
let PAL_GAMEOVER_SUB = palette.get("gameover_sub"); if PAL_GAMEOVER_SUB == "" { PAL_GAMEOVER_SUB = "#c2a83e"; }

let PAL_P_THRUST_INIT  = palette.get("particle_thruster_init"); if PAL_P_THRUST_INIT  == "" { PAL_P_THRUST_INIT  = "white"; }
let PAL_P_HOT_INIT     = palette.get("particle_hot_init");      if PAL_P_HOT_INIT     == "" { PAL_P_HOT_INIT     = "#66FFFF"; }
let PAL_P_HOT_CORE     = palette.get("particle_hot_core_init"); if PAL_P_HOT_CORE     == "" { PAL_P_HOT_CORE     = "#AAFFFF"; }
let PAL_P_BRAKE_INIT   = palette.get("particle_brake_init");    if PAL_P_BRAKE_INIT   == "" { PAL_P_BRAKE_INIT   = "white"; }
let PAL_P_CORRECT      = palette.get("particle_correction");    if PAL_P_CORRECT      == "" { PAL_P_CORRECT      = "#AACCFF"; }
let PAL_P_PULSE        = palette.get("particle_pulse");         if PAL_P_PULSE        == "" { PAL_P_PULSE        = "#00FFFF"; }

// Apply scene background and HUD label colors every frame
scene.set_bg(PAL_BG);

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

fn keep_asteroids_alive(world, asteroid_col) {
    if world.count_kind("asteroid") > 0 { return; }
    spawn_wave(world, 1, asteroid_col);
}

fn spawn_wave(world, wave, asteroid_col) {
    let b = world.world_bounds();
    let WORLD_W = b["max_x"];
    let WORLD_H = b["max_y"];
    let count = 3 + wave;
    for i in 0..count {
        let side = i % 4;
        let x = if side == 0 { -32.0 } else if side == 1 { WORLD_W + 32.0 } else { 20.0 + rand() * (WORLD_W - 40.0) };
        let y = if side == 2 { -32.0 } else if side == 3 { WORLD_H + 32.0 } else { 20.0 + rand() * (WORLD_H - 40.0) };
        if absf(x - WORLD_W * 0.5) < 60.0 && absf(y - WORLD_H * 0.5) < 60.0 {
            x = if rand() > 0.5 { -32.0 } else { WORLD_W + 32.0 };
        }
        let base_speed = 30.0 + to_float(wave) * 5.0;
        let vx = (rand() - 0.5) * base_speed * 2.0;
        let vy = (rand() - 0.5) * base_speed * 2.0;
        if absf(vx) < 10.0 { vx = if vx >= 0.0 { 10.0 } else { -10.0 }; }
        if absf(vy) < 10.0 { vy = if vy >= 0.0 { 10.0 } else { -10.0 }; }
        let id = world.spawn_prefab("asteroid-large", #{
            x: x, y: y, vx: vx, vy: vy
        });
        if id > 0 {
            world.tag_add(id, "asteroid");
            world.tag_add(id, "asteroid-large");
            world.enable_wrap(id, 0.0, WORLD_W, 0.0, WORLD_H);
            world.set(id, "style.fg", asteroid_col);
        }
    }
}

fn spawn_medium_asteroids(world, x, y, parent_vx, parent_vy, asteroid_col) {
    let b = world.world_bounds();
    let WORLD_W = b["max_x"];
    let WORLD_H = b["max_y"];
    for i in 0..2 {
        let speed = 40.0 + rand() * 30.0;
        let vx = parent_vx * 0.5 + speed * (if i == 0 { 1.0 } else { -1.0 }) * 0.707;
        let vy = parent_vy * 0.5 + speed * (if i == 0 { 0.707 } else { -0.707 });
        let id = world.spawn_prefab("asteroid-medium", #{
            x: x, y: y, vx: vx, vy: vy
        });
        if id > 0 {
            world.tag_add(id, "asteroid");
            world.tag_add(id, "asteroid-medium");
            world.enable_wrap(id, 0.0, WORLD_W, 0.0, WORLD_H);
            world.set(id, "style.fg", asteroid_col);
        }
    }
}

fn spawn_small_asteroids(world, x, y, parent_vx, parent_vy, asteroid_col) {
    let b = world.world_bounds();
    let WORLD_W = b["max_x"];
    let WORLD_H = b["max_y"];
    for i in 0..2 {
        let speed = 55.0 + rand() * 25.0;
        let vx = parent_vx * 0.4 + speed * (if i == 0 { 1.0 } else { -1.0 });
        let vy = parent_vy * 0.4 + speed * (if i == 0 { 0.5 } else { -0.5 });
        let id = world.spawn_prefab("asteroid-small", #{
            x: x, y: y, vx: vx, vy: vy
        });
        if id > 0 {
            world.tag_add(id, "asteroid");
            world.tag_add(id, "asteroid-small");
            world.enable_wrap(id, 0.0, WORLD_W, 0.0, WORLD_H);
            world.set(id, "style.fg", asteroid_col);
        }
    }
}

fn ensure_ship(world, ship_col) {
    let b = world.world_bounds();
    let WORLD_W = b["max_x"];
    let WORLD_H = b["max_y"];
    let SHIP_X = WORLD_W * 0.5;
    let SHIP_Y = WORLD_H * 0.5;
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    let ship_id = world.spawn_prefab("ship", #{
        x: SHIP_X,
        y: SHIP_Y,
        heading: 0.0
    });
    if ship_id > 0 {
        world.tag_add(ship_id, "player");
        world.enable_wrap(ship_id, 0.0, WORLD_W, 0.0, WORLD_H);
        world.set(ship_id, "style.fg", ship_col);
        world.rcs_attach(ship_id, #{});
    }
    ship_id
}

// ======================== EMISSION FUNCTIONS ===============================

// Emit with intensity scaling based on ignition state
fn emit_rcs_intensity(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#FFEECC","#FFCC88","#FF9944","#CC6622","#662211","#221108"];
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed,
        ttl_ms: ttl_ms,
        fg: color,
        radius: radius,
        spread: (rand() - 0.5) * jitter,
        color_ramp: ramp,
        radius_max: radius,
        radius_min: 0
    });
    fx
}

// Emit HOT engine particle - uses teal color ramp instead of orange
fn emit_hot_engine(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#CCFFFF","#66FFFF","#00FFCC","#00CC99","#008866","#004433"];
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed,
        ttl_ms: ttl_ms,
        fg: color,
        radius: radius,
        spread: (rand() - 0.5) * jitter,
        color_ramp: ramp,
        radius_max: radius,
        radius_min: 0
    });
    fx
}

// Emit with explicit direction override
fn emit_rcs_directed(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius, local_x, local_y) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#FFEECC","#FFCC88","#FF9944","#CC6622","#662211","#221108"];
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed,
        ttl_ms: ttl_ms,
        fg: color,
        radius: radius,
        spread: (rand() - 0.5) * jitter,
        emission_local_x: local_x,
        emission_local_y: local_y,
        color_ramp: ramp,
        radius_max: radius,
        radius_min: 0
    });
    fx
}

// ======================== THRUSTER AUTHORITY ===============================
// Returns 0.0-1.0 indicating how aligned thruster is with desired brake direction

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
// Heat-up effect with configurable thrust curve f(x)
// x = thrust_ms, f(x) = heat level 0.0 to 1.0
// At heat=1.0: trail shrinks to 5% (95% reduction), color → neon teal
// Velocity determines starting heat: fast ship = warm engine

// Thrust curve function - modify this to change heat-up behavior
// Returns 0.0 (cold) to 1.0 (max heat)
// base_heat = starting heat based on velocity (0.0 to 0.6)
fn thrust_curve(thrust_ms, base_heat) {
    // Quadratic ease-in curve for natural acceleration feel
    let t = thrust_ms / 2000.0;  // 2 seconds to full heat from base
    if t > 1.0 { t = 1.0; }
    
    // Start from base_heat (velocity-based), ramp to 1.0
    let heat = base_heat + (1.0 - base_heat) * t * t;
    if heat < 0.0 { heat = 0.0; }
    if heat > 1.0 { heat = 1.0; }
    heat
}

fn fire_main_engine_with_velocity(world, ship_id, thrust_ms, ignition_factor, speed, max_speed, pal) {
    let intensity = 0.5 + ignition_factor * 0.5;
    
    // Velocity determines starting heat: 0 speed = 0 heat, max speed = 0.6 heat
    // So fast-moving ship starts with warm engine
    let velocity_ratio = speed / max_speed;
    if velocity_ratio > 1.0 { velocity_ratio = 1.0; }
    let base_heat = velocity_ratio * 0.6;  // up to 60% starting heat
    
    // Get heat from thrust curve
    let heat = thrust_curve(thrust_ms, base_heat);
    
    // Trail length: 100% → 5% (95% reduction at max heat)
    let trail_mult = 1.0 - heat * 0.95;  // 1.0 → 0.05
    let base_ttl = 380.0 * trail_mult;
    let core_ttl = 280.0 * trail_mult;
    if base_ttl < 19.0 { base_ttl = 19.0; }
    if core_ttl < 14.0 { core_ttl = 14.0; }
    
    // Speed increases with heat (more efficient burn)
    let speed_mult = 1.0 + heat * 0.5;  // faster when hot
    
    // === PHASE DETECTION ===
    let transition_pulse = heat > 0.45 && heat < 0.52;  // TEAL PULSE zone
    let power_surge = heat >= 0.50 && heat < 0.65;      // 50% wider teal
    let jet_stream = heat >= 0.65 && heat < 0.85;       // narrowing jet
    let afterburner = heat >= 0.85;                      // max power flicker
    
    // Use HOT engine particles when heat > 0.5 (teal color ramp)
    let use_hot = heat > 0.5;
    
    // === TEAL TRANSITION PULSE ===
    // When about to become teal, emit bright teal pulse particles
    if transition_pulse {
        // Emit fast teal pulse that travels through entire exhaust
        for i in 0..6 {
            let pulse_speed = 200.0 + to_float(i) * 30.0;  // fast!
            let pulse_ttl = 350.0;  // long enough to travel full trail
            emit_hot_engine(world, ship_id, "ship.main", pulse_speed, pulse_ttl, pal["pulse"], 0.15, 1.0, 2);
        }
    }
    
    // === AFTERBURNER EFFECT at max heat ===
    // Random shape variation ±30%, occasional blink
    let shape_var = if afterburner { 0.7 + rand() * 0.6 } else { 1.0 };  // 0.7 to 1.3
    let blink = if afterburner && rand() < 0.15 { 0.4 + rand() * 0.3 } else { 1.0 };  // 15% chance dim
    
    // === PARTICLE COUNT ===
    let base_outer = if heat < 0.3 { 6.0 } else if heat < 0.5 { 6.0 } else { 8.0 };
    let base_inner = if heat < 0.3 { 4.0 } else if heat < 0.5 { 4.0 } else { 5.0 };
    
    // Apply shape variation to particle count
    let outer_count_f = base_outer * shape_var;
    let inner_count_f = base_inner * shape_var;
    let outer_count = if outer_count_f < 4.0 { 4 } else if outer_count_f < 6.0 { 5 } else if outer_count_f < 8.0 { 7 } else if outer_count_f < 10.0 { 9 } else { 11 };
    let inner_count = if inner_count_f < 3.0 { 2 } else if inner_count_f < 5.0 { 4 } else if inner_count_f < 7.0 { 6 } else { 7 };
    
    // === RADIUS based on phase ===
    // Cold: 2, Power surge: 3 (50% wider!), Jet stream: 1, Afterburner: 1
    let outer_radius = if power_surge { 3 } else if use_hot { 1 } else { 2 };
    let inner_radius = if power_surge { 2 } else { 1 };
    
    // === SPREAD based on phase ===
    // Cold: 0.25, Power surge: 0.35 (50% wider!), Jet: 0.12, Afterburner: varies
    let base_spread = if power_surge { 0.35 } else if use_hot { 0.12 } else { 0.25 };
    let spread = base_spread * (0.8 + rand() * 0.4);  // ±20% spread variation
    
    // TTL boost for hot: longer visible trail, with variation
    let ttl_var = if afterburner { 0.85 + rand() * 0.3 } else { 1.0 };  // 0.85 to 1.15
    let ttl_boost = if use_hot { 1.5 * ttl_var } else { 1.0 };
    
    // Intensity affected by blink
    let eff_intensity = intensity * blink;
    
    // Dense exhaust plume - outer layer
    for i in 0..outer_count {
        let pspeed = (140.0 + to_float(i) * 15.0) * speed_mult * (0.9 + rand() * 0.2);
        let ttl = (base_ttl - to_float(i) * (base_ttl * 0.03)) * ttl_boost;
        if ttl < 25.0 { ttl = 25.0; }
        
        // Color for initial emit (will be overridden by particle update)
        let color = if use_hot { pal["hot_init"] } else { pal["thrust_init"] };
        
        if use_hot {
            emit_hot_engine(world, ship_id, "ship.main", pspeed, ttl, color, spread, eff_intensity, outer_radius);
        } else {
            emit_rcs_intensity(world, ship_id, "ship.main", pspeed, ttl, color, spread, eff_intensity, outer_radius);
        }
    }
    
    // Inner core - brighter, faster
    for i in 0..inner_count {
        let pspeed = (110.0 + to_float(i) * 10.0) * speed_mult * (0.9 + rand() * 0.2);
        let ttl = (core_ttl - to_float(i) * (core_ttl * 0.03)) * ttl_boost;
        if ttl < 20.0 { ttl = 20.0; }
        
        let core_color = if use_hot { pal["hot_core"] } else { pal["thrust_init"] };
        
        if use_hot {
            emit_hot_engine(world, ship_id, "ship.main", pspeed, ttl, core_color, spread * 0.5, eff_intensity, inner_radius);
        } else {
            emit_rcs_intensity(world, ship_id, "ship.main", pspeed, ttl, core_color, 0.18, eff_intensity, inner_radius);
        }
    }
}

// Legacy wrapper for compatibility
fn fire_main_engine(world, ship_id, thrust_ms, ignition_factor, pal) {
    fire_main_engine_with_velocity(world, ship_id, thrust_ms, ignition_factor, 0.0, 280.0, pal);
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
    let ramp = ["#DDDDDD","#BBBBBB","#888888","#555555","#333333","#111111"];
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed * intensity,
        ttl_ms: ttl_ms,
        fg: "white",
        radius: radius,
        spread: (rand() - 0.5) * 0.03,
        color_ramp: ramp,
        radius_max: radius,
        radius_min: 1
    });
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
    let ramp = ["#AAFFFF","#88DDDD","#66BBBB","#448888","#225555","#112233"];
    let fx = world.emit(emitter, ship_id, #{
        kind: "fx",
        template: "debris",
        speed: speed * (0.8 + rand() * 0.3),
        ttl_ms: ttl_ms,
        fg: "white",
        radius: 1,
        spread: (rand() - 0.5) * 0.20,
        color_ramp: ramp,
        radius_max: 1,
        radius_min: 1
    });
    fx
}

fn fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel, wave_num) {
    // Wave intensity: wave 0 = 100%, wave 1 = 60%, wave 2 = 20%
    // This creates a clear "big → smaller → tiny" visual sequence
    let intensity = if wave_num == 0 { 1.0 } else if wave_num == 1 { 0.6 } else { 0.2 };
    
    // Particle count scales with intensity: 8-10, 5-6, 2
    let base_bursts = if wave_num == 0 { 
        8 + (rand() * 2.0) 
    } else if wave_num == 1 { 
        5 + (rand() * 1.0) 
    } else { 
        2 
    };
    let base_bursts = if base_bursts < 2.5 { 2 } else if base_bursts < 5.5 { 5 } else if base_bursts < 7.0 { 6 } else if base_bursts < 9.0 { 8 } else { 10 };
    
    // Get ship heading
    let h = ship.heading_vector();
    let fwd_x = h["x"] ?? 0.0;
    let fwd_y = h["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y = fwd_x;
    
    // Project residual velocity onto local axes
    let vel_fwd = vx * fwd_x + vy * fwd_y;
    let vel_right = vx * right_x + vy * right_y;
    
    // Speed scales with intensity too
    let speed_mult = 0.5 + intensity * 0.5;  // 100%, 80%, 60%
    
    // BOW: fires if moving forward
    if vel_fwd > 0.1 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.bow", (40.0 + to_float(i) * 8.0) * speed_mult, 180);
        }
    }
    
    // MAIN: fires if moving backward
    if vel_fwd < -0.1 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.main", (50.0 + to_float(i) * 10.0) * speed_mult, 200);
        }
    }
    
    // STARBOARD: fires if moving right
    if vel_right > 0.1 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.starboard", (35.0 + to_float(i) * 7.0) * speed_mult, 160);
        }
    }
    
    // PORT: fires if moving left
    if vel_right < -0.1 {
        for i in 0..base_bursts {
            emit_final_burst_particle(world, ship_id, "ship.port", (35.0 + to_float(i) * 7.0) * speed_mult, 160);
        }
    }
    
    // Angular stabilization (only on first 2 waves)
    if wave_num < 2 && absf(angular_vel) > 0.01 {
        let rot_bursts = if wave_num == 0 { 5 + (rand() * 2.0) } else { 3 };
        let rot_bursts = if rot_bursts < 3.5 { 3 } else if rot_bursts < 5.5 { 5 } else { 6 };
        
        let emitter1 = if angular_vel > 0.0 { "ship.starboard" } else { "ship.port" };
        let emitter2 = if angular_vel > 0.0 { "ship.port" } else { "ship.starboard" };
        
        for i in 0..rot_bursts {
            emit_final_burst_particle(world, ship_id, emitter1, (30.0 + to_float(i) * 6.0) * speed_mult, 140);
            emit_final_burst_particle(world, ship_id, emitter2, (25.0 + to_float(i) * 5.0) * speed_mult, 120);
        }
    }
    
    // FORCE STOP: Apply braking impulse proportional to wave (stronger early)
    let speed = sqrtf(vx * vx + vy * vy);
    if speed > 0.1 {
        let brake_power = 60.0 * intensity;  // 60, 36, 12 per wave
        let dt = 0.016;
        let dvx = -(vx / speed) * brake_power * dt;
        let dvy = -(vy / speed) * brake_power * dt;
        ship.physics.add_velocity(dvx, dvy);
    }
}

// ======================== MAIN CONTROL UPDATE ==============================

fn update_ship_controls(world, input, ship_id, pal) {
    if ship_id <= 0 { return; }
    let ship = world.entity(ship_id);
    if !ship.exists() { return; }

    // Read inputs
    let turn_left  = input.action_down("turn_left");
    let turn_right = input.action_down("turn_right");
    let thrusting  = input.action_down("thrust");
    let input_turn = if turn_left && !turn_right { -1.0 } else if turn_right && !turn_left { 1.0 } else { 0.0 };

    // Feed inputs into the engine-managed RCS component and tick it.
    world.rcs_set_input(ship_id, thrusting, input_turn);
    world.rcs_tick(ship_id, 16);

    // Read derived outputs.
    let s = world.rcs_state(ship_id);
    let thrust_factor  = s["thrust_factor"]  ?? 0.0;
    let rot_factor     = s["rot_factor"]      ?? 0.0;
    let brake_factor   = s["brake_factor"]    ?? 0.0;
    let speed          = s["speed"]           ?? 0.0;
    let vx             = s["vx"]              ?? 0.0;
    let vy             = s["vy"]              ?? 0.0;
    let angular_vel    = s["angular_vel"]     ?? 0.0;
    let course_drift   = s["course_drift"]    ?? 0.0;
    let course_cross   = s["course_cross"]    ?? 0.0;
    let auto_brake_phase = s["auto_brake_phase"] ?? "idle";
    let final_burst_fired     = s["final_burst_fired"]     ?? false;
    let final_burst_wave_num  = s["final_burst_wave_num"]  ?? 0;
    let command_turn   = s["command_turn"]    ?? 0;

    // Drive the ArcadeController and ship physics.
    ship.set_thrust(thrusting);
    ship.set_turn(command_turn);

    // ── Main engine emission ─────────────────────────────────────────────────
    let MAX_SPEED = 280.0;
    if thrusting && thrust_factor > 0.0 {
        fire_main_engine_with_velocity(world, ship_id, thrust_factor * 1000.0, thrust_factor, speed, MAX_SPEED, pal);

        // Auto-course correction visual
        if course_drift > 0.15 {
            let emitter = if course_cross > 0.0 { "ship.port" } else { "ship.starboard" };
            let strength = (course_drift * 0.5).min(0.4);
            emit_rcs_intensity(world, ship_id, emitter, 60.0, 120, pal["correct"], 0.15, strength, 1);
        }
    }

    // ── Rotation emission ────────────────────────────────────────────────────
    if rot_factor > 0.1 {
        fire_rotation_couple(world, ship_id, input_turn, angular_vel, rot_factor);
    }

    // ── Auto-brake emission ──────────────────────────────────────────────────
    if brake_factor > 0.1 {
        if auto_brake_phase == "rotation" {
            fire_auto_rotation_brake(world, ship_id, angular_vel, brake_factor);
        } else if auto_brake_phase == "linear" {
            fire_auto_linear_brake(world, ship, ship_id, brake_factor);
        }
    }

    // ── Final stabilization burst ────────────────────────────────────────────
    if final_burst_fired {
        fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel, final_burst_wave_num);
    }

    // ── Settling puffs when fully stopped ────────────────────────────────────
    if auto_brake_phase == "stopped" && rand() < 0.05 {
        fire_settling_puff(world, ship_id, 0.3);
    }
}

// ======================== INITIALIZATION ===================================

if !(local.initialized ?? false) {
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    local.score = 0;
    local.lives = 3;
    local.wave = 1;
    local.game_over = false;
    local.respawn_timer = 0.0;
    local.invincible_timer = 0.0;
    input.load_profile("default");
    scene.set("hud-gameover", "visible", false);
    scene.set("hud-gameover-sub", "visible", false);
    spawn_wave(world, 1, PAL_ASTEROID);
}

// ======================== MAIN LOOP ========================================

let score       = local.score ?? 0;
let lives       = local.lives ?? 3;
let wave        = local.wave ?? 1;
let game_over   = local.game_over ?? false;
let respawn_timer   = local.respawn_timer ?? 0.0;
let invincible_timer = local.invincible_timer ?? 0.0;

// ── HUD update ──────────────────────────────────────────────────────────────
scene.set("hud-score", "text.content", to_string(score));
scene.set("hud-lives", "text.content", to_string(lives));
scene.set("hud-wave",  "text.content", to_string(wave));
scene.set("hud-score",       "style.fg", PAL_HUD_VALUE);
scene.set("hud-lives",       "style.fg", PAL_HUD_VALUE);
scene.set("hud-wave",        "style.fg", PAL_HUD_VALUE);
scene.set("hud-label-score", "style.fg", PAL_HUD_LABEL);
scene.set("hud-label-lives", "style.fg", PAL_HUD_LABEL);
scene.set("hud-label-wave",  "style.fg", PAL_HUD_LABEL);
scene.set("hud-gameover",     "style.fg", PAL_GAMEOVER);
scene.set("hud-gameover-sub", "style.fg", PAL_GAMEOVER_SUB);

// ── Game-over state ─────────────────────────────────────────────────────────
if game_over {
    scene.set("hud-gameover", "visible", true);
    scene.set("hud-gameover-sub", "visible", true);
    if input.action_just_pressed("restart") || input.action_just_pressed("fire") {
        // Reset game
        local.score = 0;
        local.lives = 3;
        local.wave = 1;
        local.game_over = false;
        local.respawn_timer = 0.0;
        local.invincible_timer = 0.0;
        scene.set("hud-gameover", "visible", false);
        scene.set("hud-gameover-sub", "visible", false);
        // Despawn all asteroids and bullets
        for id in world.query_tag("asteroid") { world.despawn(id); }
        for id in world.query_tag("bullet") { world.despawn(id); }
        spawn_wave(world, 1, PAL_ASTEROID);
    }
    #{state: local}
} else {

// ── Respawn timer ────────────────────────────────────────────────────────────
if respawn_timer > 0.0 {
    respawn_timer = respawn_timer - 16.0;
    local.respawn_timer = respawn_timer;
    if respawn_timer <= 0.0 {
        local.invincible_timer = 2000.0;
    }
}

// ── Invincibility timer ───────────────────────────────────────────────────────
if invincible_timer > 0.0 {
    invincible_timer = invincible_timer - 16.0;
    local.invincible_timer = invincible_timer;
}

// ── Ship controls ────────────────────────────────────────────────────────────
let ship_id = if respawn_timer <= 0.0 { ensure_ship(world, PAL_SHIP) } else { 0 };
local.ship_id = ship_id;

if ship_id > 0 {
    let pal = #{
        thrust_init: PAL_P_THRUST_INIT,
        hot_init: PAL_P_HOT_INIT,
        hot_core: PAL_P_HOT_CORE,
        pulse: PAL_P_PULSE,
        correct: PAL_P_CORRECT,
    };
    update_ship_controls(world, input, ship_id, pal);

    // ── Shooting ─────────────────────────────────────────────────────────────
    if input.action_just_pressed("fire") && world.entity(ship_id).cooldown_ready("fire") {
        let ship_xf = world.transform(ship_id);
        if type_of(ship_xf) == "map" {
            let sx = ship_xf["x"] ?? 0.0;
            let sy = ship_xf["y"] ?? 0.0;
            let heading = ship_xf["heading"] ?? 0.0;
            // Ship heading: 0 = up (−y), positive = clockwise
            let bvx = -sin(heading) * 280.0;
            let bvy = -cos(heading) * 280.0;
            // Offset bullet to ship nose
            let bx = sx - sin(heading) * 17.0;
            let by = sy - cos(heading) * 17.0;
            let phys = world.physics(ship_id);
            let svx = if type_of(phys) == "map" { phys["vx"] ?? 0.0 } else { 0.0 };
            let svy = if type_of(phys) == "map" { phys["vy"] ?? 0.0 } else { 0.0 };
            let bullet_id = world.spawn_prefab("bullet", #{
                x: bx, y: by,
                vx: bvx + svx, vy: bvy + svy,
                ttl_ms: 1400
            });
            if bullet_id > 0 {
                world.tag_add(bullet_id, "bullet");
                world.tag_add(bullet_id, "player-bullet");
                world.set(bullet_id, "style.fg", PAL_BULLET);
            }
            world.entity(ship_id).cooldown_start("fire", 180);
        }
    }
}

// ── Wave complete check ───────────────────────────────────────────────────────
if world.count_kind("asteroid") == 0 && world.count_tag("bullet") == 0 && respawn_timer <= 0.0 {
    wave = wave + 1;
    local.wave = wave;
    spawn_wave(world, wave, PAL_ASTEROID);
}

// ── Collision handling ────────────────────────────────────────────────────────
// bullet hits asteroid
for hit in collision.enters_of("bullet") {
    let bullet_id   = hit["self"];
    let asteroid_id = hit["other"];
    if !world.tag_has(asteroid_id, "asteroid") { continue; }
    if !world.exists(bullet_id) || !world.exists(asteroid_id) { continue; }
    let axf = world.transform(asteroid_id);
    let ax = if type_of(axf) == "map" { axf["x"] ?? 0.0 } else { 0.0 };
    let ay = if type_of(axf) == "map" { axf["y"] ?? 0.0 } else { 0.0 };
    let aphys = world.physics(asteroid_id);
    let avx = if type_of(aphys) == "map" { aphys["vx"] ?? 0.0 } else { 0.0 };
    let avy = if type_of(aphys) == "map" { aphys["vy"] ?? 0.0 } else { 0.0 };

    // Score + split
    if world.tag_has(asteroid_id, "asteroid-large") {
        score = score + 20;
        world.despawn(asteroid_id);
        spawn_medium_asteroids(world, ax, ay, avx, avy, PAL_ASTEROID);
    } else if world.tag_has(asteroid_id, "asteroid-medium") {
        score = score + 50;
        world.despawn(asteroid_id);
        spawn_small_asteroids(world, ax, ay, avx, avy, PAL_ASTEROID);
    } else if world.tag_has(asteroid_id, "asteroid-small") {
        score = score + 100;
        world.despawn(asteroid_id);
    }
    local.score = score;
    world.despawn(bullet_id);
}

// ship hits asteroid
if invincible_timer <= 0.0 && ship_id > 0 {
    for hit in collision.enters_of("ship") {
        let hit_ship_id = hit["self"];
        let other_id    = hit["other"];
        if hit_ship_id != ship_id || !world.tag_has(other_id, "asteroid") { continue; }
        world.despawn(ship_id);
        local.ship_id = 0;
        ship_id = 0;
        lives = lives - 1;
        local.lives = lives;
        invincible_timer = 0.0;
        if lives <= 0 {
            local.game_over = true;
            local.lives = 0;
        } else {
            respawn_timer = 2000.0;
            local.respawn_timer = respawn_timer;
        }
        break;
    }
}

#{state: local}
} // end game_over else block
