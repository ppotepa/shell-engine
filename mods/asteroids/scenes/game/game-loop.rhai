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
import "rcs" as rcs;

if type_of(local) == "()" { local = #{}; }

let WORLD_W = 640.0;
let WORLD_H = 360.0;

// ======================== PALETTE SETUP ====================================
// Only re-read palette values when the active palette has changed.
// palette.version() is bumped by the engine every time set_active/cycle is called.
let pal_ver = palette.version();
if pal_ver != (local.pal_ver ?? -1) {
    local.pal_ver = pal_ver;

    let PAL_BG           = palette.get("bg");           if PAL_BG           == "" { PAL_BG           = "#060620"; }
    let PAL_SHIP         = palette.get("ship");         if PAL_SHIP         == "" { PAL_SHIP         = "white"; }
    let PAL_ASTEROID     = palette.get("asteroid");     if PAL_ASTEROID     == "" { PAL_ASTEROID     = "white"; }
    let PAL_BULLET       = palette.get("bullet");       if PAL_BULLET       == "" { PAL_BULLET       = "yellow"; }
    let PAL_P_THRUST_INIT  = palette.get("particle_thruster_init"); if PAL_P_THRUST_INIT  == "" { PAL_P_THRUST_INIT  = "white"; }
    let PAL_P_HOT_INIT     = palette.get("particle_hot_init");      if PAL_P_HOT_INIT     == "" { PAL_P_HOT_INIT     = "#66FFFF"; }
    let PAL_P_HOT_CORE     = palette.get("particle_hot_core_init"); if PAL_P_HOT_CORE     == "" { PAL_P_HOT_CORE     = "#AAFFFF"; }
    let PAL_P_CORRECT      = palette.get("particle_correction");    if PAL_P_CORRECT      == "" { PAL_P_CORRECT      = "#AACCFF"; }
    let PAL_P_PULSE        = palette.get("particle_pulse");         if PAL_P_PULSE        == "" { PAL_P_PULSE        = "#00FFFF"; }

    local.pal = #{
        bg:           PAL_BG,
        ship:         PAL_SHIP,
        asteroid:     PAL_ASTEROID,
        bullet:       PAL_BULLET,
        thrust_init:  PAL_P_THRUST_INIT,
        hot_init:     PAL_P_HOT_INIT,
        hot_core:     PAL_P_HOT_CORE,
        pulse:        PAL_P_PULSE,
        correct:      PAL_P_CORRECT,
        ramp_hot_engine: palette.particles("hot_engine"),
        ramp_brake:      palette.particles("brake"),
        ramp_heat_trail: palette.particles("heat_trail"),
        ramp_bullet_trail: palette.particles("bullet_trail"),
    };

    // Apply palette-driven static styles (only needed on change)
    scene.set_bg(PAL_BG);
}

let col  = local.pal ?? #{};



// Randomise initial 3-D orientation and spin speed of a freshly-spawned asteroid.
// orbit_speed range: 30–140 deg/sec — mix of slow drifters and fast tumblers.
fn apply_obj_rotation(scene, id) {
    let visual = "asteroid-" + id;
    scene.set(visual, "obj.yaw",         rand() * 360.0);
    scene.set(visual, "obj.pitch",       rand() * 360.0);
    scene.set(visual, "obj.roll",        rand() * 360.0);
    scene.set(visual, "obj.orbit_speed", 30.0 + rand() * 110.0);
}

// --- FREE FLIGHT MODE: no asteroid waves, camera follows ship ---
// Asteroids disabled — world is open for free exploration.
// fn keep_asteroids_alive(world, scene) {
//     if world.count_kind("asteroid") > 0 { return; }
//     spawn_wave(world, scene, 1);
// }

// Give an asteroid a random tumble (constant spin, no braking).
fn add_tumble(world, id, size) {
    let max_spin = if size == 3 { 1.5 } else if size == 2 { 2.5 } else { 4.0 };
    let spin = (rand() - 0.5) * max_spin * 2.0;
    if absf(spin) < 0.3 { spin = if spin >= 0.0 { 0.3 } else { -0.3 }; }
    world.angular_body_attach(id, #{
        accel: 0.0,
        max: max_spin + 1.0,
        deadband: 0.0,
        auto_brake: false,
        angular_vel: spin,
    });
}

fn spawn_wave(world, scene, wave) {
    let WORLD_W = world.world_width();
    let WORLD_H = world.world_height();
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
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-large", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 {
            add_tumble(world, id, 3);
            apply_obj_rotation(scene, id);
        }
    }
}

fn spawn_medium_asteroids(world, scene, x, y, parent_vx, parent_vy) {
    for i in 0..2 {
        let speed = 40.0 + rand() * 30.0;
        let vx = parent_vx * 0.5 + speed * (if i == 0 { 1.0 } else { -1.0 }) * 0.707;
        let vy = parent_vy * 0.5 + speed * (if i == 0 { 0.707 } else { -0.707 });
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-medium", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 {
            add_tumble(world, id, 2);
            apply_obj_rotation(scene, id);
        }
    }
}

fn spawn_small_asteroids(world, scene, x, y, parent_vx, parent_vy) {
    for i in 0..2 {
        let speed = 55.0 + rand() * 25.0;
        let vx = parent_vx * 0.4 + speed * (if i == 0 { 1.0 } else { -1.0 });
        let vy = parent_vy * 0.4 + speed * (if i == 0 { 0.5 } else { -0.5 });
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-small", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 {
            add_tumble(world, id, 1);
            apply_obj_rotation(scene, id);
        }
    }
}

// Spawn shrapnel: real physics entities that can ricochet and damage the ship.
// Evenly spread around 360° with jitter, inheriting asteroid + bullet momentum.
fn spawn_shrapnel(world, ax, ay, bvx, bvy, avx, avy, count, base_speed) {
    for i in range(0, count) {
        let angle = (i.to_float() / count.to_float()) * 6.2832 + (rand() - 0.5) * 1.2;
        let spd   = base_speed + (rand() - 0.5) * 40.0;
        let vx    = sin(angle)  * spd + avx * 0.4 + bvx * 0.2;
        let vy    = -cos(angle) * spd + avy * 0.4 + bvy * 0.2;
        let ttl   = 1200 + (rand() * 600.0).to_int();
        world.spawn_prefab("shrapnel", #{
            x: ax, y: ay, vx: vx, vy: vy, ttl_ms: ttl, heading: angle
        });
    }
}

// Shared death visual sequence — hull pieces + shockwave + shake + whiteout flash.
// Call BEFORE despawning the ship so ship_id is still a valid entity.
fn do_ship_death_fx(world, effects, col, ship_id) {
    world.emit("ship.hull_pieces", ship_id, #{ count: 22, speed:  95.0, spread: 6.28, fg: col.ship ?? "white" });
    world.emit("ship.hull_pieces", ship_id, #{ count:  9, speed: 195.0, spread: 6.28, fg: col.ship ?? "white" });
    world.emit("ship.shockwave",   ship_id, #{ count: 20, speed: 320.0, spread: 6.28 });
    world.emit("asteroids.impact_sparks", ship_id, #{ count: 14, speed: 55.0, spread: 6.28, fg: col.ship ?? "white" });
    effects.shake(600, 4.0, 3.0, 8.0);
    effects.trigger("whiteout", 300, #{ intensity: 0.85 });
}

fn ensure_ship(world) {
    let WORLD_W = world.world_width();
    let WORLD_H = world.world_height();
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
        world.angular_body_attach(ship_id, #{ accel: 5.5, max: 7.0, deadband: 0.10, auto_brake: true });
        world.linear_brake_attach(ship_id, #{ decel: 45.0, deadband: 2.0, auto_brake: true });
        world.thruster_ramp_attach(ship_id, #{
            thrust_delay_ms:       8.0,
            thrust_ramp_ms:        12.0,
            no_input_threshold_ms: 30.0,
            rot_factor_max_vel:    7.0,
            burst_speed_threshold: 15.0,
            burst_wave_interval_ms: 150.0,
            burst_wave_count:      3,
        });
    }
    ship_id
}

// ======================== INITIALIZATION ===================================

if !(local.initialized ?? false) {
    // Free-flight mode: bounds set so world_width()/world_height() return correct values,
    // but no entity wrapping occurs because enable_wrap() is never called per-entity.
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    local.score = 0;
    local.lives = 3;
    local.ship_hp = 3;
    local.wave = 1;
    local.game_over = false;
    input.load_profile("default");
    // spawn_wave(world, scene, 1); // disabled — free-flight mode
}

// ======================== MAIN LOOP ========================================

let score     = local.score ?? 0;
let lives     = local.lives ?? 3;
let ship_hp   = local.ship_hp ?? 3;
let wave      = local.wave ?? 1;
let game_over = local.game_over ?? false;
let respawning = local.respawning ?? false;
let invincible = local.invincible ?? false;

// ── Engine timer resolution ───────────────────────────────────────────────────
if world.timer_fired("respawn_done") {
    local.respawning = false;
    respawning = false;
    local.invincible = true;
    invincible = true;
    local.just_spawned = true;
    world.after_ms("invincible_done", 2000);
}
if world.timer_fired("invincible_done") {
    local.invincible = false;
    invincible = false;
}
if world.timer_fired("hit_invincible_done") {
    local.invincible = false;
    invincible = false;
}

// ── HUD game state sync ───────────────────────────────────────────────────────
game.set("/score", score);
game.set("/wave", wave);
scene.set("heart-1", "visible", lives >= 1);
scene.set("heart-2", "visible", lives >= 2);
scene.set("heart-3", "visible", lives >= 3);

// ── ESC key: return to main menu ──────────────────────────────────────────────
if !game_over && (key.code == "Escape" || key.code == "escape") {
    world.reset_dynamic_entities();
    game.jump("asteroids-mainmenu");
}

// ── Game-over state ─────────────────────────────────────────────────────────
if game_over {
    // Scene transition already queued from the collision handler; just wait.
    #{state: local}
} else {

// ── Ship controls ────────────────────────────────────────────────────────────
let ship_id = if !respawning { ensure_ship(world) } else { 0 };
local.ship_id = ship_id;

if ship_id > 0 {
    // Sync 3-D roll with ship heading so the model rotates to match movement direction.
    // roll = -heading (radians→degrees) because OBJ +roll is CCW, game heading is CW.
    let xf = world.transform(ship_id);
    scene.set("ship-" + ship_id, "obj.roll", -(xf["heading"]) * 57.29578);

    // ── Camera follows ship — keep ship centered on screen ───────────────────
    let ship_x = xf["x"] ?? 320.0;
    let ship_y = xf["y"] ?? 180.0;
    world.set_camera(ship_x - 320.0, ship_y - 180.0);

    // ── Ship polygon collider — 6-point hull rotated by heading each frame ────
    // Body-frame hull outline: derived from ship.obj vertex projections onto the
    // 28×28 canvas (nose=top, wings=bottom). Offsets are from entity_pos center.
    let BASE_HULL = [[0.0, -6.9], [2.65, 5.39], [7.4, 6.9], [0.0, 2.7], [-7.4, 6.9], [-2.65, 5.39]];
    let h = xf["heading"];
    let ch = cos(h); let sh = sin(h);
    let hull_pts = [];
    for p in BASE_HULL {
        hull_pts.push([ch * p[0] - sh * p[1], sh * p[0] + ch * p[1]]);
    }
    world.entity(ship_id).physics.set_collider_polygon(hull_pts, 1, 6);

    // ── Warp-in materialisation on first frame after respawn ──────────────────
    if local.just_spawned ?? false {
        local.just_spawned = false;
        world.emit("ship.warp_in", ship_id, #{ count: 26, speed: 120.0, spread: 6.28, fg: col.ship ?? "white" });
        world.emit("ship.warp_in", ship_id, #{ count: 10, speed:  60.0, spread: 6.28, fg: col.ship ?? "white" });
        world.emit("asteroids.impact_sparks", ship_id, #{ count: 10, speed: 70.0, spread: 3.14, fg: "#AADDFF" });
        effects.shake(300, 2.5, 1.8, 10.0);
        effects.trigger("whiteout", 200, #{ intensity: 0.5 });
    }

    // ── Laser sight — DISABLED (keep for future re-enable) ────────────────────
    // 8 dots spaced 17 px apart along the heading vector, dimming with distance.
    // let LASER_COLS = [
    //     "#1A3860", "#142E52", "#102546", "#0D1E3A",
    //     "#091630", "#070F24", "#050B1A", "#030812"
    // ];
    // for i in 0..8 {
    //     let dist = 18.0 + i.to_float() * 17.0;
    //     world.emit("laser.dot", ship_id, #{
    //         count: 1,
    //         local_x: 0.0,
    //         local_y: -dist,
    //         speed: 0.0,
    //         fg: LASER_COLS[i],
    //     });
    // }

    rcs::update_ship_controls(world, input, ship_id, col);

    // ── Shooting ─────────────────────────────────────────────────────────────
    if input.action_just_pressed("fire") && world.entity(ship_id).cooldown_ready("fire") {
        let bullet_id = world.spawn_from_heading(ship_id, "bullet", #{
            speed: 960.0, offset: 17.0, ttl_ms: 1000, inherit_velocity: true
        });
        world.entity(ship_id).cooldown_start("fire", 180);

        // ── Recoil — small backward kick on each shot (Newton's 3rd law) ─────
        let phys = world.physics(ship_id);
        if type_of(phys) == "map" {
            let h = xf["heading"];
            let recoil = 14.0;
            world.set_physics(ship_id,
                (phys["vx"] ?? 0.0) - sin(h) * recoil,
                (phys["vy"] ?? 0.0) + cos(h) * recoil,
                phys["ax"] ?? 0.0, phys["ay"] ?? 0.0,
                phys["drag"] ?? 0.0, phys["max_speed"] ?? 0.0);
        }
    }
    // ── Bullet trail — luminous glow particles streaming behind each bullet ──────
    for bullet_id in world.query_tag("bullet") {
        world.emit("bullet.trail", bullet_id, #{
            count: 4, color_ramp: col.ramp_bullet_trail
        });
    }
}

// ── Wave complete check — DISABLED in free-flight mode ───────────────────────
// if world.count_kind("asteroid") == 0 && world.count_tag("bullet") == 0 && !respawning {
//     wave = wave + 1;
//     local.wave = wave;
//     spawn_wave(world, scene, wave);
// }

// ── Collision handling ────────────────────────────────────────────────────────
// bullet hits asteroid — emit debris, score, split
for hit in collision.enters("bullet", "asteroid") {
    let bullet_id   = hit["bullet"];
    let asteroid_id = hit["asteroid"];
    if !world.exists(bullet_id) || !world.exists(asteroid_id) { continue; }
    let axf = world.transform(asteroid_id);
    let ax = if type_of(axf) == "map" { axf["x"] ?? 0.0 } else { 0.0 };
    let ay = if type_of(axf) == "map" { axf["y"] ?? 0.0 } else { 0.0 };
    let aphys = world.physics(asteroid_id);
    let avx = if type_of(aphys) == "map" { aphys["vx"] ?? 0.0 } else { 0.0 };
    let avy = if type_of(aphys) == "map" { aphys["vy"] ?? 0.0 } else { 0.0 };

    // Engine-level elastic impulse already applied before scripts run.
    // Read bullet velocity for shrapnel spawn direction.
    let bphys = world.physics(bullet_id);
    let bvx = if type_of(bphys) == "map" { bphys["vx"] ?? 0.0 } else { 0.0 };
    let bvy = if type_of(bphys) == "map" { bphys["vy"] ?? 0.0 } else { 0.0 };

    // Shrapnel: real physics entities — ricochet off asteroids, damage ship
    let shard_count = if world.tag_has(asteroid_id, "asteroid-large") { 5 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 4 }
                 else { 3 };
    let shard_speed = if world.tag_has(asteroid_id, "asteroid-large") { 220.0 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 280.0 }
                 else { 340.0 };
    spawn_shrapnel(world, ax, ay, bvx, bvy, avx, avy, shard_count, shard_speed);

    // Fragment halo: glowing dust cloud that fades to background over 2s (visual only)
    let frag_count = shard_count - 1;
    let frag_speed = shard_speed * 0.4;
    world.emit("asteroid.impact_fragments", asteroid_id, #{
        count: frag_count, speed: frag_speed, spread: 6.28, fg: col.asteroid ?? "white"
    });

    // Impact sparks and debris
    let spark_count = if world.tag_has(asteroid_id, "asteroid-large") { 12 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 8 }
                 else { 5 };
    world.emit("asteroids.impact_sparks", asteroid_id, #{
        count: spark_count, speed: 60.0, spread: 3.14, fg: col.asteroid ?? "white"
    });
    world.emit("asteroid.debris", asteroid_id, #{
        count: spark_count / 2, speed: 35.0, spread: 6.28, fg: col.asteroid ?? "white"
    });

    // Score + split
    if world.tag_has(asteroid_id, "asteroid-large") {
        score = score + 20;
        world.despawn(asteroid_id);
        spawn_medium_asteroids(world, scene, ax, ay, avx, avy);
    } else if world.tag_has(asteroid_id, "asteroid-medium") {
        score = score + 50;
        world.despawn(asteroid_id);
        spawn_small_asteroids(world, scene, ax, ay, avx, avy);
    } else if world.tag_has(asteroid_id, "asteroid-small") {
        score = score + 100;
        world.despawn(asteroid_id);
    }
    local.score = score;
    // Let bullet ricochet briefly instead of instant despawn — engine impulse already deflected it
    world.set_lifetime(bullet_id, 250);
}

// shrapnel hits asteroid — ricochet: reflect velocity × 0.5, nudge asteroid
for hit in collision.enters("shrapnel", "asteroid") {
    let s_id = hit["shrapnel"];
    let a_id = hit["asteroid"];
    if !world.exists(s_id) || !world.exists(a_id) { continue; }
    let sphys = world.physics(s_id);
    let svx = if type_of(sphys) == "map" { sphys["vx"] ?? 0.0 } else { 0.0 };
    let svy = if type_of(sphys) == "map" { sphys["vy"] ?? 0.0 } else { 0.0 };
    // Reflect and bleed 50% speed
    if type_of(sphys) == "map" {
        world.set_physics(s_id,
            -svx * 0.5, -svy * 0.5,
            sphys["ax"] ?? 0.0, sphys["ay"] ?? 0.0,
            sphys["drag"] ?? 0.0, sphys["max_speed"] ?? 0.0);
    }
    // 3% momentum nudge to the asteroid
    let aphys = world.physics(a_id);
    if type_of(aphys) == "map" {
        let push = 0.03;
        world.set_physics(a_id,
            (aphys["vx"] ?? 0.0) + svx * push,
            (aphys["vy"] ?? 0.0) + svy * push,
            aphys["ax"] ?? 0.0, aphys["ay"] ?? 0.0,
            aphys["drag"] ?? 0.0, aphys["max_speed"] ?? 0.0);
    }
    // Spark flash at ricochet
    world.emit("asteroids.impact_sparks", a_id, #{
        count: 3, speed: 40.0, spread: 2.0, fg: col.asteroid ?? "white"
    });
}

// shrapnel hits ship — 3-hit HP system: lose a life only on 3rd hit
if !invincible && ship_id > 0 {
    for hit in collision.enters("shrapnel", "ship") {
        let s_id      = hit["shrapnel"];
        let hit_ship  = hit["ship"];
        if hit_ship != ship_id || !world.exists(s_id) { continue; }
        world.despawn(s_id);

        ship_hp = ship_hp - 1;
        local.ship_hp = ship_hp;

        if ship_hp <= 0 {
            // Full death sequence
            do_ship_death_fx(world, effects, col, ship_id);
            world.despawn(ship_id);
            local.ship_id = 0;
            ship_id = 0;
            local.ship_hp = 3;
            ship_hp = 3;
            lives = lives - 1;
            local.lives = lives;
            if lives <= 0 {
                local.game_over = true;
                local.lives = 0;
                let final_score = score;
                let saved = game.get("/highscores/scores");
                let scores = if type_of(saved) == "array" { saved } else { [] };
                let rank = -1;
                for i in 0..scores.len() {
                    if final_score > scores[i].score { rank = i; break; }
                }
                if rank < 0 && scores.len() < 10 { rank = scores.len(); }
                if rank >= 0 {
                    let entry = #{ rank: rank + 1, score: final_score, name: "---", wave: wave };
                    scores.insert(rank, entry);
                    while scores.len() > 10 { scores.pop(); }
                    for i in 0..scores.len() { scores[i].rank = i + 1; }
                    game.set("/highscores/scores", scores);
                    game.set("/highscores/last_entry_rank", rank);
                } else {
                    game.set("/highscores/last_entry_rank", -1);
                }
                game.set("/score", final_score);
                for id in world.query_tag("asteroid") { world.despawn(id); }
                for id in world.query_tag("bullet")   { world.despawn(id); }
                for id in world.query_tag("shrapnel") { world.despawn(id); }
                game.jump("asteroids-highscores");
            } else {
                local.respawning = true;
                world.after_ms("respawn_done", 3000);
            }
        } else {
            // Hit flash — brief invincibility, small spark burst only
            world.emit("asteroids.impact_sparks", ship_id, #{
                count: 6, speed: 50.0, spread: 6.28, fg: col.ship ?? "white"
            });
            local.invincible = true;
            invincible = true;
            world.after_ms("hit_invincible_done", 800);
        }
        break;
    }
}

// ship hits asteroid — 3-hit HP system: lose a life only on 3rd hit
if !invincible && ship_id > 0 {
    for hit in collision.enters("ship", "asteroid") {
        let hit_ship_id = hit["ship"];
        if hit_ship_id != ship_id { continue; }

        ship_hp = ship_hp - 1;
        local.ship_hp = ship_hp;

        if ship_hp <= 0 {
            // Full death sequence
            do_ship_death_fx(world, effects, col, ship_id);
            world.despawn(ship_id);
            local.ship_id = 0;
            ship_id = 0;
            local.ship_hp = 3;
            ship_hp = 3;
            lives = lives - 1;
            local.lives = lives;
            if lives <= 0 {
                local.game_over = true;
                local.lives = 0;

                // Save score to high-scores table (must be inline; functions can't call game.get at preflight)
                let final_score = score;
                let saved = game.get("/highscores/scores");
                let scores = if type_of(saved) == "array" { saved } else { [] };
                let rank = -1;
                for i in 0..scores.len() {
                    if final_score > scores[i].score { rank = i; break; }
                }
                if rank < 0 && scores.len() < 10 { rank = scores.len(); }
                if rank >= 0 {
                    let entry = #{ rank: rank + 1, score: final_score, name: "---", wave: wave };
                    scores.insert(rank, entry);
                    while scores.len() > 10 { scores.pop(); }
                    for i in 0..scores.len() { scores[i].rank = i + 1; }
                    game.set("/highscores/scores", scores);
                    game.set("/highscores/last_entry_rank", rank);
                } else {
                    game.set("/highscores/last_entry_rank", -1);
                }
                game.set("/score", final_score);

                for id in world.query_tag("asteroid") { world.despawn(id); }
                for id in world.query_tag("bullet")   { world.despawn(id); }
                for id in world.query_tag("shrapnel") { world.despawn(id); }
                game.jump("asteroids-highscores");
            } else {
                local.respawning = true;
                world.after_ms("respawn_done", 3000);
            }
        } else {
            // Hit flash — brief invincibility, small spark burst only
            world.emit("asteroids.impact_sparks", ship_id, #{
                count: 8, speed: 60.0, spread: 6.28, fg: col.ship ?? "white"
            });
            local.invincible = true;
            invincible = true;
            world.after_ms("hit_invincible_done", 800);
        }
        break;
    }
}

#{state: local}
} // end game_over else block
