// Asteroids ship RCS VFX module — original-style 4-emitter pipeline.
//
// Pure VFX dispatcher: no physics state, no AngularBody/ThrusterRamp/LinearBrake.
// All inputs come from the orbital state machine in game-loop.rhai.
//
// Emitters: ship.main (rear), ship.bow (nose), ship.port (left), ship.starboard (right)
//   thrust    → main (multi-phase ignition A→B→C→D)
//   yaw left  → port primary + starboard secondary + bow side assist
//   yaw right → starboard primary + port secondary + bow side assist
//   brake rot → opposite pair + bow counter-assist
//   reverse   → bow
//
// Usage in game-loop.rhai:
//   import "rcs" as rcs;
//   rcs::update_orbital_vfx_original_style(world, ship_id, thrust_axis, yaw_axis,
//       yaw_rate, thrust_hold_ms, thrust_release_ms, vfwd, vright, speed, pal);

// ======================== EMIT HELPERS =====================================

fn emit_rcs_intensity(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius) {
    let speed = base_speed * ignition_factor;
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * spread * 0.35,
        radius_max: radius, radius_min: 0
    })
}

fn emit_hot_engine(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius, pal) {
    let speed = base_speed * ignition_factor;
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * spread * 0.35,
        color_ramp: pal["ramp_hot_engine"],
        radius_max: radius, radius_min: 0
    })
}

fn emit_rcs_directed(world, ship_id, emitter, base_speed, ttl_ms, color, spread, ignition_factor, radius, local_x, local_y) {
    let speed = base_speed * ignition_factor;
    world.emit(emitter, ship_id, #{
        kind: "fx", template: "debris",
        speed: speed, ttl_ms: ttl_ms, fg: color, radius: radius,
        spread: (rand() - 0.5) * spread * 0.35,
        emission_local_x: local_x, emission_local_y: local_y,
        radius_max: radius, radius_min: 0
    })
}

// ======================== ORBITAL ORIGINAL_STYLE PIPELINE ====================
// Keep the original 4-emitter Asteroids feel, driven by orbital state.

fn clamp01(x) {
    if x < 0.0 { return 0.0; }
    if x > 1.0 { return 1.0; }
    x
}

fn abs_turn_bursts(av) {
    if av > 0.90 { return 6; }
    if av > 0.70 { return 5; }
    if av > 0.50 { return 4; }
    if av > 0.30 { return 3; }
    if av > 0.12 { return 2; }
    1
}

fn abs_speed_bursts(comp) {
    let a = absf(comp);
    if a > 5.0 { return 7; }
    if a > 3.5 { return 5; }
    if a > 2.0 { return 4; }
    if a > 1.0 { return 3; }
    if a > 0.35 { return 2; }
    1
}

// turn_axis (from game-loop.rhai yaw_axis):
//   < 0.0 => turn right  (turn_right action → yaw_axis = -1.0)
//   > 0.0 => turn left   (turn_left action  → yaw_axis = +1.0)
fn fire_rotation_couple_original_style(world, ship_id, turn_axis, yaw_rate, pal) {
    let av = absf(yaw_rate).max(absf(turn_axis) * 0.25);
    let bursts = abs_turn_bursts(av);
    let core = (0.22 + av * 0.58).min(1.0);
    let ttl = if av > 0.55 { 170 } else { 130 };
    let spd = if av > 0.55 { 84.0 } else { 62.0 };

    if turn_axis < 0.0 {
        for i in 0..bursts {
            let s = spd + to_float(i) * 6.0;
            emit_rcs_intensity(world, ship_id, "ship.port", s, ttl, pal["correct"], 0.08, core, 1);
            if i % 2 == 0 {
                emit_rcs_intensity(world, ship_id, "ship.starboard", s * 0.78, ttl - 10, pal["correct"], 0.07, core * 0.55, 1);
            }
        }

        if av > 0.28 {
            emit_rcs_directed(world, ship_id, "ship.bow", 52.0 + av * 20.0, 95, pal["correct"], 0.06, (0.16 + av * 0.18).min(0.42), 1, 1.0, 0.0);
        }
    } else if turn_axis > 0.0 {
        for i in 0..bursts {
            let s = spd + to_float(i) * 6.0;
            emit_rcs_intensity(world, ship_id, "ship.starboard", s, ttl, pal["correct"], 0.08, core, 1);
            if i % 2 == 0 {
                emit_rcs_intensity(world, ship_id, "ship.port", s * 0.78, ttl - 10, pal["correct"], 0.07, core * 0.55, 1);
            }
        }

        if av > 0.28 {
            emit_rcs_directed(world, ship_id, "ship.bow", 52.0 + av * 20.0, 95, pal["correct"], 0.06, (0.16 + av * 0.18).min(0.42), 1, -1.0, 0.0);
        }
    }
}

// turn_axis == 0, but ship still has yaw_rate
fn fire_rotation_brake_original_style(world, ship_id, yaw_rate, pal) {
    let av = absf(yaw_rate);
    if av < 0.035 { return; }

    let bursts = abs_turn_bursts(av);
    let core = (0.18 + av * 0.42).min(0.62);
    let ttl = if av > 0.45 { 140 } else { 110 };
    let spd = if av > 0.45 { 70.0 } else { 52.0 };

    if yaw_rate < 0.0 {
        for i in 0..bursts {
            let s = spd + to_float(i) * 5.0;
            emit_rcs_intensity(world, ship_id, "ship.starboard", s, ttl, pal["brake"], 0.06, core, 1);
            if i % 2 == 0 {
                emit_rcs_intensity(world, ship_id, "ship.port", s * 0.72, ttl - 10, pal["brake"], 0.05, core * 0.45, 1);
            }
        }

        emit_rcs_directed(world, ship_id, "ship.bow", 42.0 + av * 14.0, 80, pal["brake"], 0.05, (0.12 + av * 0.12).min(0.26), 1, -1.0, 0.0);
    } else {
        for i in 0..bursts {
            let s = spd + to_float(i) * 5.0;
            emit_rcs_intensity(world, ship_id, "ship.port", s, ttl, pal["brake"], 0.06, core, 1);
            if i % 2 == 0 {
                emit_rcs_intensity(world, ship_id, "ship.starboard", s * 0.72, ttl - 10, pal["brake"], 0.05, core * 0.45, 1);
            }
        }

        emit_rcs_directed(world, ship_id, "ship.bow", 42.0 + av * 14.0, 80, pal["brake"], 0.05, (0.12 + av * 0.12).min(0.26), 1, 1.0, 0.0);
    }
}

// Translation corrections in local orbital frame:
// vfwd = nose-forward component, vright = side component.
fn fire_translation_trim_original_style(world, ship_id, thrust_axis, vfwd, vright, pal) {
    if thrust_axis > 0.0 {
        emit_rcs_intensity(world, ship_id, "ship.main", 110.0, 90, pal["thrust_init"], 0.04, 0.18, 1);
    }

    if thrust_axis < 0.0 {
        let bursts = abs_speed_bursts(vfwd);
        for i in 0..bursts {
            emit_rcs_intensity(world, ship_id, "ship.bow", 78.0 + to_float(i) * 8.0, 85, pal["brake"], 0.05, 0.22, 1);
        }
    }

    if absf(vright) > 0.18 {
        let bursts = abs_speed_bursts(vright);
        let intensity = (0.14 + absf(vright) * 0.06).min(0.36);

        if vright > 0.0 {
            for i in 0..bursts {
                emit_rcs_intensity(world, ship_id, "ship.port", 48.0 + to_float(i) * 7.0, 80, pal["correct"], 0.05, intensity, 1);
            }
        } else {
            for i in 0..bursts {
                emit_rcs_intensity(world, ship_id, "ship.starboard", 48.0 + to_float(i) * 7.0, 80, pal["correct"], 0.05, intensity, 1);
            }
        }
    }
}

fn fire_main_engine_original_style(world, ship_id, thrust_on, thrust_hold_ms, thrust_release_ms, speed, pal) {
    let speed_norm = (speed / 7.5).min(1.0);
    let ignite = clamp01(thrust_hold_ms / 140.0);
    let mid = clamp01((thrust_hold_ms - 110.0) / 340.0);
    let sustain = clamp01((thrust_hold_ms - 420.0) / 420.0);
    let fade = if thrust_on { 1.0 } else { 1.0 - clamp01(thrust_release_ms / 180.0) };

    if thrust_on && thrust_hold_ms < 150.0 {
        let hot = (0.35 + ignite * 0.65) * fade;
        for i in 0..5 {
            emit_hot_engine(world, ship_id, "ship.main", 118.0 + to_float(i) * 15.0, 90.0 + to_float(i) * 8.0, pal["hot_core"], 0.10, hot, 2, pal);
        }
    }

    if thrust_on && thrust_hold_ms >= 90.0 {
        let trans = (0.18 + mid * 0.62) * fade;
        for i in 0..6 {
            emit_hot_engine(world, ship_id, "ship.main", (135.0 + to_float(i) * 16.0) * (1.0 + speed_norm * 0.22), 120.0 + to_float(i) * 10.0, pal["hot_init"], 0.08, trans, 1, pal);
        }
    }

    if thrust_on && thrust_hold_ms >= 260.0 {
        let blue = (0.24 + sustain * 0.76) * fade;
        for i in 0..8 {
            emit_rcs_intensity(world, ship_id, "ship.main", (105.0 + to_float(i) * 12.0) * (1.0 + speed_norm * 0.18), 150.0 + to_float(i) * 16.0, pal["thrust_init"], 0.045, blue, 1);
        }
    }

    if !thrust_on && thrust_release_ms < 180.0 {
        let cool = 1.0 - clamp01(thrust_release_ms / 180.0);
        for i in 0..4 {
            emit_rcs_intensity(world, ship_id, "ship.main", 76.0 + to_float(i) * 10.0, 70.0 + to_float(i) * 18.0, pal["thrust_init"], 0.04, cool * 0.65, 1);
        }
    }
}

fn fire_settling_puff_original_style(world, ship_id, yaw_rate, vfwd, vright, pal) {
    if absf(yaw_rate) > 0.025 { return; }
    if absf(vfwd) > 0.30 { return; }
    if absf(vright) > 0.20 { return; }
    if rand() > 0.08 { return; }

    let pick = rand();
    if pick < 0.33 {
        emit_rcs_intensity(world, ship_id, "ship.port", 42.0, 55, pal["correct"], 0.04, 0.08, 1);
    } else if pick < 0.66 {
        emit_rcs_intensity(world, ship_id, "ship.starboard", 42.0, 55, pal["correct"], 0.04, 0.08, 1);
    } else {
        emit_rcs_intensity(world, ship_id, "ship.bow", 38.0, 48, pal["brake"], 0.04, 0.07, 1);
    }
}

fn update_orbital_vfx_original_style(world, ship_id, thrust_axis, turn_axis, yaw_rate, thrust_hold_ms, thrust_release_ms, vfwd, vright, speed, pal) {
    if ship_id <= 0 { return; }
    if !world.exists(ship_id) { return; }

    if absf(turn_axis) > 0.001 {
        fire_rotation_couple_original_style(world, ship_id, turn_axis, yaw_rate, pal);
    } else {
        fire_rotation_brake_original_style(world, ship_id, yaw_rate, pal);
    }

    fire_translation_trim_original_style(world, ship_id, thrust_axis, vfwd, vright, pal);
    fire_main_engine_original_style(world, ship_id, thrust_axis > 0.0, thrust_hold_ms, thrust_release_ms, speed, pal);
    fire_settling_puff_original_style(world, ship_id, yaw_rate, vfwd, vright, pal);
}
