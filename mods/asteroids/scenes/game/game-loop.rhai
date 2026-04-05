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
        world.angular_body_attach(ship_id, #{ accel: 5.5, max: 7.0, deadband: 0.10, auto_brake: true });
        world.linear_brake_attach(ship_id, #{ decel: 45.0, deadband: 2.0, auto_brake: true });
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
    scene.set("hud-gameover", "visible", false);
    scene.set("hud-gameover-sub", "visible", false);
    spawn_wave(world, 1, PAL_ASTEROID);
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
        local.respawning = false;
        local.invincible = false;
        world.cancel_timer("respawn_done");
        world.cancel_timer("invincible_done");
        scene.set("hud-gameover", "visible", false);
        scene.set("hud-gameover-sub", "visible", false);
        // Despawn all asteroids and bullets
        for id in world.query_tag("asteroid") { world.despawn(id); }
        for id in world.query_tag("bullet") { world.despawn(id); }
        spawn_wave(world, 1, PAL_ASTEROID);
    }
    #{state: local}
} else {

// ── Ship controls ────────────────────────────────────────────────────────────
let ship_id = if !respawning { ensure_ship(world, PAL_SHIP) } else { 0 };
local.ship_id = ship_id;

if ship_id > 0 {
    let pal = #{
        thrust_init: PAL_P_THRUST_INIT,
        hot_init: PAL_P_HOT_INIT,
        hot_core: PAL_P_HOT_CORE,
        pulse: PAL_P_PULSE,
        correct: PAL_P_CORRECT,
    };
    rcs::update_ship_controls(world, input, ship_id, local, pal);

    // ── Shooting ─────────────────────────────────────────────────────────────
    if input.action_just_pressed("fire") && world.entity(ship_id).cooldown_ready("fire") {
        let bullet_id = world.spawn_from_heading(ship_id, "bullet", #{
            speed: 280.0, offset: 17.0, ttl_ms: 1400, inherit_velocity: true
        });
        if bullet_id > 0 {
            world.tag_add(bullet_id, "bullet");
            world.tag_add(bullet_id, "player-bullet");
            world.set(bullet_id, "style.fg", PAL_BULLET);
        }
        world.entity(ship_id).cooldown_start("fire", 180);
    }
}

// ── Wave complete check ───────────────────────────────────────────────────────
if world.count_kind("asteroid") == 0 && world.count_tag("bullet") == 0 && !respawning {
    wave = wave + 1;
    local.wave = wave;
    spawn_wave(world, wave, PAL_ASTEROID);
}

// ── Collision handling ────────────────────────────────────────────────────────
// bullet hits asteroid
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
if !invincible && ship_id > 0 {
    for hit in collision.enters("ship", "asteroid") {
        let hit_ship_id = hit["ship"];
        if hit_ship_id != ship_id { continue; }
        world.despawn(ship_id);
        local.ship_id = 0;
        ship_id = 0;
        lives = lives - 1;
        local.lives = lives;
        if lives <= 0 {
            local.game_over = true;
            local.lives = 0;
        } else {
            local.respawning = true;
            world.after_ms("respawn_done", 2000);
        }
        break;
    }
}

#{state: local}
} // end game_over else block
