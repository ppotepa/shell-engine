// Asteroids ship RCS module.
//
// Handles: input → engine AngularBody, ignition ramp VFX state, emission calls.
// Physics (angular vel + auto-brake) live in the engine AngularBody component.
// This module is purely about visual effects and game-feel on top of that.
//
// Usage in game-loop.rhai:
//   import "rcs" as rcs;
//   rcs::update_ship_controls(world, input, ship_id, local, pal);

// ======================== EMIT HELPERS =====================================

fn emit_rcs_intensity(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#FFEECC","#FFCC88","#FF9944","#CC6622","#662211","#221108"];
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * jitter,
        color_ramp: ramp, radius_max: radius, radius_min: 0
    })
}

fn emit_hot_engine(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#CCFFFF","#66FFFF","#00FFCC","#00CC99","#008866","#004433"];
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * jitter,
        color_ramp: ramp, radius_max: radius, radius_min: 0
    })
}

fn emit_rcs_directed(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius, local_x, local_y) {
    let speed = base_speed * ignition_factor;
    let jitter = spread * 0.35;
    let ramp = ["#FFEECC","#FFCC88","#FF9944","#CC6622","#662211","#221108"];
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * jitter,
        emission_local_x: local_x, emission_local_y: local_y,
        color_ramp: ramp, radius_max: radius, radius_min: 0
    })
}

fn emit_brake_particle(world, ship_id, emitter, speed, ttl_ms, intensity, radius) {
    let ramp = ["#DDDDDD","#BBBBBB","#888888","#555555","#333333","#111111"];
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed * intensity, ttl_ms: ttl_ms, fg: "white", radius: radius,
        spread: (rand() - 0.5) * 0.03,
        color_ramp: ramp, radius_max: radius, radius_min: 1
    })
}

fn emit_final_burst_particle(world, ship_id, emitter, speed, ttl_ms) {
    let ramp = ["#AAFFFF","#88DDDD","#66BBBB","#448888","#225555","#112233"];
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed * (0.8 + rand() * 0.3), ttl_ms: ttl_ms,
        fg: "white", radius: 1, spread: (rand() - 0.5) * 0.20,
        color_ramp: ramp, radius_max: 1, radius_min: 1
    })
}

// ======================== BURST COUNT SCALING ==============================

fn rotation_burst_count(av) {
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

fn component_burst_count(component_vel) {
    if component_vel > 150.0 { return 8; }
    if component_vel > 100.0 { return 6; }
    if component_vel > 60.0 { return 5; }
    if component_vel > 35.0 { return 4; }
    if component_vel > 15.0 { return 3; }
    if component_vel > 5.0 { return 2; }
    1
}

// ======================== IGNITION RAMP ====================================

fn igf(ms, delay, ramp) {
    if ms < delay { return 0.0; }
    let t = (ms - delay) / ramp;
    if t > 1.0 { 1.0 } else { t }
}

// ======================== EMISSION: ROTATION ================================

fn fire_rotation_couple(world, ship_id, direction, angular_vel, ignition_factor) {
    let burst_count = rotation_burst_count(absf(angular_vel));
    let intensity = (0.4 + absf(angular_vel) * 0.15).min(1.0);
    let eff_factor = ignition_factor * intensity;

    let primary   = if direction < 0.0 { "ship.starboard" } else { "ship.port" };
    let secondary = if direction < 0.0 { "ship.port" }      else { "ship.starboard" };

    for i in 0..burst_count {
        let speed_var = to_float(i) * 8.0;
        emit_rcs_intensity(world, ship_id, primary,   95.0 + speed_var,        180, "white", 0.08, eff_factor,       1);
        if i % 2 == 0 {
            emit_rcs_intensity(world, ship_id, secondary, 70.0 + speed_var * 0.6, 140, "white", 0.06, eff_factor * 0.5, 1);
        }
    }
    if burst_count > 2 {
        let bow_dir_x = if direction < 0.0 { 1.0 } else { -1.0 };
        for i in 0..(burst_count / 2) {
            emit_rcs_directed(world, ship_id, "ship.bow", 65.0 + to_float(i) * 4.0, 120, "white", 0.04, eff_factor * 0.6, 1, bow_dir_x, 0.0);
        }
    }
}

fn fire_auto_rotation_brake(world, ship_id, angular_vel, ignition_factor) {
    let av = absf(angular_vel);
    if av < 0.05 { return; }

    let brake_emitter    = if angular_vel < 0.0 { "ship.port" }      else { "ship.starboard" };
    let opposite_emitter = if angular_vel < 0.0 { "ship.starboard" } else { "ship.port" };
    let burst_count = rotation_burst_count(av);
    let intensity   = (0.4 + av * 0.12).min(1.0);
    let eff_factor  = ignition_factor * intensity;

    for i in 0..burst_count {
        emit_rcs_intensity(world, ship_id, brake_emitter, 85.0 + av * 15.0 + to_float(i) * 6.0, 160, "white", 0.06, eff_factor, 1);
    }
    for i in 0..(burst_count / 2 + 1) {
        emit_rcs_intensity(world, ship_id, opposite_emitter, 65.0 + to_float(i) * 4.0, 120, "white", 0.04, eff_factor * 0.4, 1);
    }
    let bow_dir_x = if angular_vel < 0.0 { -1.0 } else { 1.0 };
    for i in 0..(burst_count / 2) {
        emit_rcs_directed(world, ship_id, "ship.bow", 55.0 + to_float(i) * 3.0, 100, "white", 0.03, eff_factor * 0.5, 1, bow_dir_x, 0.0);
    }
}

// ======================== EMISSION: MAIN ENGINE ============================

fn thrust_curve(thrust_ms, base_heat) {
    let t = (thrust_ms / 2000.0).min(1.0);
    let heat = base_heat + (1.0 - base_heat) * t * t;
    heat.min(1.0).max(0.0)
}

fn fire_main_engine_with_velocity(world, ship_id, thrust_ms, ignition_factor, speed, max_speed, pal) {
    let intensity     = 0.5 + ignition_factor * 0.5;
    let velocity_ratio = (speed / max_speed).min(1.0);
    let base_heat     = velocity_ratio * 0.6;
    let heat          = thrust_curve(thrust_ms, base_heat);

    let trail_mult = (1.0 - heat * 0.95).max(0.0);
    let base_ttl   = (380.0 * trail_mult).max(19.0);
    let core_ttl   = (280.0 * trail_mult).max(14.0);
    let speed_mult = 1.0 + heat * 0.5;

    let transition_pulse = heat > 0.45 && heat < 0.52;
    let power_surge      = heat >= 0.50 && heat < 0.65;
    let afterburner      = heat >= 0.85;
    let use_hot          = heat > 0.5;

    if transition_pulse {
        for i in 0..6 {
            emit_hot_engine(world, ship_id, "ship.main", 200.0 + to_float(i) * 30.0, 350.0, pal["pulse"], 0.15, 1.0, 2);
        }
    }

    let shape_var  = if afterburner { 0.7 + rand() * 0.6 } else { 1.0 };
    let blink      = if afterburner && rand() < 0.15 { 0.4 + rand() * 0.3 } else { 1.0 };
    let base_outer = if heat < 0.5 { 6.0 } else { 8.0 };
    let base_inner = if heat < 0.5 { 4.0 } else { 5.0 };
    let outer_count_f = base_outer * shape_var;
    let inner_count_f = base_inner * shape_var;
    let outer_count = if outer_count_f < 4.0 { 4 } else if outer_count_f < 6.0 { 5 } else if outer_count_f < 8.0 { 7 } else if outer_count_f < 10.0 { 9 } else { 11 };
    let inner_count = if inner_count_f < 3.0 { 2 } else if inner_count_f < 5.0 { 4 } else if inner_count_f < 7.0 { 6 } else { 7 };
    let outer_radius = if power_surge { 3 } else if use_hot { 1 } else { 2 };
    let inner_radius = if power_surge { 2 } else { 1 };
    let base_spread  = if power_surge { 0.35 } else if use_hot { 0.12 } else { 0.25 };
    let spread       = base_spread * (0.8 + rand() * 0.4);
    let ttl_var      = if afterburner { 0.85 + rand() * 0.3 } else { 1.0 };
    let ttl_boost    = if use_hot { 1.5 * ttl_var } else { 1.0 };
    let eff_intensity = intensity * blink;

    for i in 0..outer_count {
        let pspeed = (140.0 + to_float(i) * 15.0) * speed_mult * (0.9 + rand() * 0.2);
        let ttl    = ((base_ttl - to_float(i) * base_ttl * 0.03) * ttl_boost).max(25.0);
        let color  = if use_hot { pal["hot_init"] } else { pal["thrust_init"] };
        if use_hot { emit_hot_engine(world, ship_id, "ship.main", pspeed, ttl, color, spread, eff_intensity, outer_radius); }
        else       { emit_rcs_intensity(world, ship_id, "ship.main", pspeed, ttl, color, spread, eff_intensity, outer_radius); }
    }
    for i in 0..inner_count {
        let pspeed = (110.0 + to_float(i) * 10.0) * speed_mult * (0.9 + rand() * 0.2);
        let ttl    = ((core_ttl - to_float(i) * core_ttl * 0.03) * ttl_boost).max(20.0);
        let color  = if use_hot { pal["hot_core"] } else { pal["thrust_init"] };
        if use_hot { emit_hot_engine(world, ship_id, "ship.main", pspeed, ttl, color, spread * 0.5, eff_intensity, inner_radius); }
        else       { emit_rcs_intensity(world, ship_id, "ship.main", pspeed, ttl, color, 0.18, eff_intensity, inner_radius); }
    }
}

// ======================== EMISSION: LINEAR BRAKE ===========================

fn fire_auto_linear_brake(world, ship, ship_id, ignition_factor) {
    let phys = world.physics(ship_id);
    if type_of(phys) != "map" { return; }
    let vx = phys["vx"] ?? 0.0;
    let vy = phys["vy"] ?? 0.0;
    let speed = sqrtf(vx * vx + vy * vy);
    if speed < 2.0 { return; }

    let brake_power = 45.0 * ignition_factor;
    let dt = 0.016;
    ship.physics.add_velocity(-(vx / speed) * brake_power * dt, -(vy / speed) * brake_power * dt);

    let h      = ship.heading_vector();
    let fwd_x  = h["x"] ?? 0.0;
    let fwd_y  = h["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y  = fwd_x;

    let vel_fwd   = vx * fwd_x + vy * fwd_y;
    let vel_right = vx * right_x + vy * right_y;
    let abs_fwd   = absf(vel_fwd);
    let abs_right = absf(vel_right);

    if vel_fwd > 0.0 {
        let intensity = 0.4 + (vel_fwd / speed) * 0.6;
        for i in 0..component_burst_count(abs_fwd) {
            emit_brake_particle(world, ship_id, "ship.bow", 70.0 + abs_fwd * 0.4 + to_float(i) * 4.0, 200, intensity * ignition_factor, 1);
        }
    }
    if vel_fwd < 0.0 {
        let intensity = 0.4 + (-vel_fwd / speed) * 0.6;
        for i in 0..component_burst_count(abs_fwd) {
            emit_brake_particle(world, ship_id, "ship.main", 85.0 + abs_fwd * 0.45 + to_float(i) * 5.0, 240, intensity * ignition_factor, 2);
        }
    }
    if vel_right > 0.0 {
        let intensity = 0.4 + (vel_right / speed) * 0.6;
        for i in 0..component_burst_count(abs_right) {
            emit_brake_particle(world, ship_id, "ship.starboard", 65.0 + abs_right * 0.35 + to_float(i) * 3.0, 170, intensity * ignition_factor, 1);
        }
    }
    if vel_right < 0.0 {
        let intensity = 0.4 + (-vel_right / speed) * 0.6;
        for i in 0..component_burst_count(abs_right) {
            emit_brake_particle(world, ship_id, "ship.port", 65.0 + abs_right * 0.35 + to_float(i) * 3.0, 170, intensity * ignition_factor, 1);
        }
    }
}

fn fire_settling_puff(world, ship_id, ignition_factor) {
    let emitters = ["ship.bow", "ship.port", "ship.starboard", "ship.main"];
    let idx_f = rand() * 3.99;
    let idx = if idx_f < 1.0 { 0 } else if idx_f < 2.0 { 1 } else if idx_f < 3.0 { 2 } else { 3 };
    emit_rcs_intensity(world, ship_id, emitters[idx], 25.0, 50, "white", 0.01, ignition_factor * 0.25, 1);
}

fn fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel, wave_num) {
    let intensity    = if wave_num == 0 { 1.0 } else if wave_num == 1 { 0.6 } else { 0.2 };
    let base_bursts_f = if wave_num == 0 { 8.0 + rand() * 2.0 } else if wave_num == 1 { 5.0 + rand() } else { 2.0 };
    let base_bursts  = if base_bursts_f < 2.5 { 2 } else if base_bursts_f < 5.5 { 5 } else if base_bursts_f < 7.0 { 6 } else if base_bursts_f < 9.0 { 8 } else { 10 };
    let h      = ship.heading_vector();
    let fwd_x  = h["x"] ?? 0.0;
    let fwd_y  = h["y"] ?? -1.0;
    let right_x = -fwd_y;
    let right_y  = fwd_x;
    let vel_fwd   = vx * fwd_x + vy * fwd_y;
    let vel_right = vx * right_x + vy * right_y;
    let speed_mult = 0.5 + intensity * 0.5;

    if vel_fwd > 0.1   { for i in 0..base_bursts { emit_final_burst_particle(world, ship_id, "ship.bow",       (40.0 + to_float(i) * 8.0)  * speed_mult, 180); } }
    if vel_fwd < -0.1  { for i in 0..base_bursts { emit_final_burst_particle(world, ship_id, "ship.main",      (50.0 + to_float(i) * 10.0) * speed_mult, 200); } }
    if vel_right > 0.1 { for i in 0..base_bursts { emit_final_burst_particle(world, ship_id, "ship.starboard", (35.0 + to_float(i) * 7.0)  * speed_mult, 160); } }
    if vel_right < -0.1{ for i in 0..base_bursts { emit_final_burst_particle(world, ship_id, "ship.port",      (35.0 + to_float(i) * 7.0)  * speed_mult, 160); } }

    if wave_num < 2 && absf(angular_vel) > 0.01 {
        let rot_bursts_f = if wave_num == 0 { 5.0 + rand() * 2.0 } else { 3.0 };
        let rot_bursts   = if rot_bursts_f < 3.5 { 3 } else if rot_bursts_f < 5.5 { 5 } else { 6 };
        let e1 = if angular_vel > 0.0 { "ship.starboard" } else { "ship.port" };
        let e2 = if angular_vel > 0.0 { "ship.port" }      else { "ship.starboard" };
        for i in 0..rot_bursts {
            emit_final_burst_particle(world, ship_id, e1, (30.0 + to_float(i) * 6.0) * speed_mult, 140);
            emit_final_burst_particle(world, ship_id, e2, (25.0 + to_float(i) * 5.0) * speed_mult, 120);
        }
    }

    let speed = sqrtf(vx * vx + vy * vy);
    if speed > 0.1 {
        let brake_power = 60.0 * intensity;
        let dt = 0.016;
        ship.physics.add_velocity(-(vx / speed) * brake_power * dt, -(vy / speed) * brake_power * dt);
    }
}

// ======================== MAIN CONTROL UPDATE ==============================
// Called every frame when ship is alive.
// Physics (angular vel, auto-brake) is handled by the engine AngularBody system.
// This function: feeds input, reads state, manages VFX ramp timers, emits particles.

fn update_ship_controls(world, input, ship_id, local, pal) {
    if ship_id <= 0 { return; }
    let ship = world.entity(ship_id);
    if !ship.exists() { return; }

    // ── Read inputs ──────────────────────────────────────────────────────────
    let turn_left  = input.action_down("turn_left");
    let turn_right = input.action_down("turn_right");
    let thrusting  = input.action_down("thrust");
    let input_turn = if turn_left && !turn_right { -1.0 } else if turn_right && !turn_left { 1.0 } else { 0.0 };
    let rotation_input_active = input_turn != 0.0;
    let linear_input_active   = thrusting || rotation_input_active;

    // ── Drive engine components ──────────────────────────────────────────────
    world.set_angular_input(ship_id, input_turn);
    ship.set_thrust(thrusting);

    // ── Read physics state (engine-owned) ────────────────────────────────────
    let angular_vel = world.angular_vel(ship_id);
    let phys  = world.physics(ship_id);
    let vx    = if type_of(phys) == "map" { phys["vx"] ?? 0.0 } else { 0.0 };
    let vy    = if type_of(phys) == "map" { phys["vy"] ?? 0.0 } else { 0.0 };
    let speed = sqrtf(vx * vx + vy * vy);
    let still_rotating = absf(angular_vel) > 0.10;
    let still_moving   = speed > 2.5;

    // ── VFX ramp state (mod-owned, stored in local.rcs) ──────────────────────
    let rcs = local.rcs ?? #{};
    let thrust_ignition_ms  = rcs["thrust_ignition_ms"]  ?? 0.0;
    let no_input_ms         = rcs["no_input_ms"]         ?? 0.0;
    let brake_ignition_ms   = rcs["brake_ignition_ms"]   ?? 0.0;
    let auto_brake_phase    = rcs["auto_brake_phase"]    ?? "idle";
    let final_burst_waves   = rcs["final_burst_waves"]   ?? 0;
    let final_burst_timer_ms = rcs["final_burst_timer_ms"] ?? 0.0;
    let final_burst_triggered = rcs["final_burst_triggered"] ?? false;

    if linear_input_active { no_input_ms = 0.0; } else { no_input_ms += 16.0; }

    // ── Thrust ignition ramp ─────────────────────────────────────────────────
    let thrust_factor = 0.0;
    if thrusting {
        thrust_ignition_ms += 16.0;
        thrust_factor = igf(thrust_ignition_ms, 8.0, 12.0);
    } else {
        thrust_ignition_ms = 0.0;
    }

    // ── Rotation ignition factor — derived from angular_vel magnitude ────────
    let rot_factor = (absf(angular_vel) / 7.0).min(1.0);

    // ── Brake ignition ramp ──────────────────────────────────────────────────
    let linear_brake_active = no_input_ms >= 30.0 && still_moving && !still_rotating;
    let brake_factor = 0.0;

    if still_rotating && !rotation_input_active {
        brake_ignition_ms += 16.0;
        brake_factor = igf(brake_ignition_ms, 8.0 * 0.3, 12.0 * 0.5);
        auto_brake_phase = "rotation";
    } else if linear_brake_active {
        brake_ignition_ms += 16.0;
        brake_factor = igf(brake_ignition_ms + no_input_ms, 8.0 * 0.5, 12.0 * 0.8);
        auto_brake_phase = "linear";
    } else if !linear_input_active && !still_rotating && !still_moving {
        brake_ignition_ms = 0.0;
        auto_brake_phase  = "stopped";
    } else if thrusting && !rotation_input_active && !still_rotating {
        auto_brake_phase = "thrusting";
    } else if rotation_input_active {
        brake_ignition_ms = 0.0;
        auto_brake_phase  = "idle";
    }

    // ── Final stabilization burst tracking ──────────────────────────────────
    let final_burst_fired    = false;
    let final_burst_wave_num = 0;

    if linear_brake_active && speed < 15.0 && speed > 2.5 {
        if !final_burst_triggered {
            final_burst_triggered = true;
            final_burst_waves     = 0;
            final_burst_timer_ms  = 0.0;
        }
        if final_burst_waves < 3 {
            final_burst_timer_ms += 16.0;
            if final_burst_timer_ms >= 150.0 {
                final_burst_fired    = true;
                final_burst_wave_num = final_burst_waves;
                final_burst_waves   += 1;
                final_burst_timer_ms = 0.0;
            }
        }
    } else if !still_moving && !still_rotating {
        final_burst_triggered = false;
        final_burst_waves     = 0;
        final_burst_timer_ms  = 0.0;
    }
    if linear_input_active {
        final_burst_triggered = false;
        final_burst_waves     = 0;
        final_burst_timer_ms  = 0.0;
    }

    // ── Course correction ────────────────────────────────────────────────────
    if thrusting && speed > 5.0 && !rotation_input_active {
        let h  = ship.heading_vector();
        let hx = h["x"] ?? 0.0;
        let hy = h["y"] ?? -1.0;
        let cross    = hx * vy - hy * vx;
        let dot_val  = hx * vx + hy * vy;
        let drift    = absf(cross) / speed;
        if dot_val > 0.0 && drift > 0.15 {
            let emitter  = if cross > 0.0 { "ship.port" } else { "ship.starboard" };
            let strength = (drift * 0.5).min(0.4);
            emit_rcs_intensity(world, ship_id, emitter, 60.0, 120, pal["correct"], 0.15, strength, 1);
        }
    }

    // ── Emit particles based on ramp state ───────────────────────────────────
    let MAX_SPEED = 280.0;
    if thrusting && thrust_factor > 0.0 {
        fire_main_engine_with_velocity(world, ship_id, thrust_ignition_ms, thrust_factor, speed, MAX_SPEED, pal);
    }
    if rot_factor > 0.1 {
        fire_rotation_couple(world, ship_id, input_turn, angular_vel, rot_factor);
    }
    if brake_factor > 0.1 {
        if auto_brake_phase == "rotation" {
            fire_auto_rotation_brake(world, ship_id, angular_vel, brake_factor);
        } else if auto_brake_phase == "linear" {
            fire_auto_linear_brake(world, ship, ship_id, brake_factor);
        }
    }
    if final_burst_fired {
        fire_final_stabilization_burst(world, ship, ship_id, vx, vy, angular_vel, final_burst_wave_num);
    }
    if auto_brake_phase == "stopped" && rand() < 0.05 {
        fire_settling_puff(world, ship_id, 0.3);
    }

    // ── Persist VFX state ────────────────────────────────────────────────────
    local.rcs = #{
        thrust_ignition_ms:   thrust_ignition_ms,
        no_input_ms:          no_input_ms,
        brake_ignition_ms:    brake_ignition_ms,
        auto_brake_phase:     auto_brake_phase,
        final_burst_waves:    final_burst_waves,
        final_burst_timer_ms: final_burst_timer_ms,
        final_burst_triggered: final_burst_triggered,
    };
}
