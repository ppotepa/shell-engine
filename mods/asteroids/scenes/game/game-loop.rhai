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
    };

    // Apply palette-driven static styles (only needed on change)
    scene.set_bg(PAL_BG);
}

let col  = local.pal ?? #{};

// ======================== HELPER FUNCTIONS =================================

// Base vertex shapes (centered at origin, matching object YAMLs).
fn base_points_large() {
    [[-14,-22],[9,-20],[22,-9],[20,11],[9,22],[-11,22],[-22,9],[-20,-11]]
}
fn base_points_medium() {
    [[-7,-11],[5,-11],[11,-2],[9,9],[-4,11],[-11,2]]
}
fn base_points_small() {
    [[-3,-7],[5,-5],[7,2],[-2,7],[-7,2],[-5,-5]]
}

// Perturb each vertex by ±jitter fraction of its distance from origin.
fn randomize_shape(base, jitter) {
    let out = [];
    for pt in base {
        let px = to_float(pt[0]);
        let py = to_float(pt[1]);
        let dx = (rand() - 0.5) * 2.0 * jitter * absf(px).max(2.0);
        let dy = (rand() - 0.5) * 2.0 * jitter * absf(py).max(2.0);
        out.push([to_int(px + dx), to_int(py + dy)]);
    }
    out
}

// Apply unique polygon to a freshly-spawned asteroid visual.
fn apply_shape(scene, id, size) {
    let visual = "asteroid-" + id;
    let base = if size == 3 { base_points_large() }
          else if size == 2 { base_points_medium() }
          else { base_points_small() };
    let jitter = 0.25;
    let pts = randomize_shape(base, jitter);
    scene.set(visual, "vector.points", pts);
}

fn keep_asteroids_alive(world, scene) {
    if world.count_kind("asteroid") > 0 { return; }
    spawn_wave(world, scene, 1);
}

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
            apply_shape(scene, id, 3);
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
            apply_shape(scene, id, 2);
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
            apply_shape(scene, id, 1);
        }
    }
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
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    local.score = 0;
    local.lives = 3;
    local.wave = 1;
    local.game_over = false;
    input.load_profile("default");
    spawn_wave(world, scene, 1);
}

// ======================== MAIN LOOP ========================================

let score     = local.score ?? 0;
let lives     = local.lives ?? 3;
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
    world.after_ms("invincible_done", 2000);
}
if world.timer_fired("invincible_done") {
    local.invincible = false;
    invincible = false;
}

// ── HUD game state sync ───────────────────────────────────────────────────────
game.set("/score", score);
game.set("/lives", lives);
game.set("/wave", wave);

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
    rcs::update_ship_controls(world, input, ship_id, col);

    // ── Shooting ─────────────────────────────────────────────────────────────
    if input.action_just_pressed("fire") && world.entity(ship_id).cooldown_ready("fire") {
        let bullet_id = world.spawn_from_heading(ship_id, "bullet", #{
            speed: 280.0, offset: 17.0, ttl_ms: 1400, inherit_velocity: true
        });
        world.entity(ship_id).cooldown_start("fire", 180);
    }
}

// ── Wave complete check ───────────────────────────────────────────────────────
if world.count_kind("asteroid") == 0 && world.count_tag("bullet") == 0 && !respawning {
    wave = wave + 1;
    local.wave = wave;
    spawn_wave(world, scene, wave);
}

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

    // Bullet push: transfer 6% of bullet momentum to asteroid before spawning anything
    let bphys = world.physics(bullet_id);
    if type_of(bphys) == "map" && type_of(aphys) == "map" {
        let bvx = bphys["vx"] ?? 0.0;
        let bvy = bphys["vy"] ?? 0.0;
        let impulse = 0.06;
        world.set_physics(asteroid_id,
            avx + bvx * impulse, avy + bvy * impulse,
            aphys["ax"] ?? 0.0, aphys["ay"] ?? 0.0,
            aphys["drag"] ?? 0.0, aphys["max_speed"] ?? 0.0);
    }

    // Fragment splash: 3–5 large glowing pieces that fade to background over 2s
    let frag_count = if world.tag_has(asteroid_id, "asteroid-large") { 5 }
                else if world.tag_has(asteroid_id, "asteroid-medium") { 4 }
                else { 3 };
    let frag_speed = if world.tag_has(asteroid_id, "asteroid-large") { 45.0 }
                else if world.tag_has(asteroid_id, "asteroid-medium") { 55.0 }
                else { 70.0 };
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
    world.despawn(bullet_id);
}

// ship hits asteroid — explosion particles, lose life
if !invincible && ship_id > 0 {
    for hit in collision.enters("ship", "asteroid") {
        let hit_ship_id = hit["ship"];
        if hit_ship_id != ship_id { continue; }

        // Ship explosion particles (emit while ship still exists)
        world.emit("ship.explosion_debris", ship_id, #{
            count: 24, speed: 80.0, spread: 6.28, fg: col.ship ?? "white"
        });
        world.emit("asteroids.impact_sparks", ship_id, #{
            count: 16, speed: 50.0, spread: 6.28, fg: col.ship ?? "white"
        });

        world.despawn(ship_id);
        local.ship_id = 0;
        ship_id = 0;
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
            for id in world.query_tag("bullet") { world.despawn(id); }
            game.jump("asteroids-highscores");
        } else {
            local.respawning = true;
            world.after_ms("respawn_done", 2000);
        }
        break;
    }
}

#{state: local}
} // end game_over else block
