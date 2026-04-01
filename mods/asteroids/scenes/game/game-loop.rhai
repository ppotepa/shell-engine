// asteroids/scenes/game/game-loop.rhai
// Rhai functions are hoisted — defined first, main logic follows.

fn spawn_ship(world) {
    let WORLD_W = 640.0;
    let WORLD_H = 360.0;
    let ship_id = world.spawn_prefab("ship", #{
        x: WORLD_W / 2.0,
        y: WORLD_H / 2.0,
        heading: 0.0
    });
    ship_id
}

fn asteroid_shape(size, seed) {
    let sides  = if size == 3 { 10 } else if size == 2 { 8 } else { 6 };
    let radius = if size == 3 { 22 } else if size == 2 { 11 } else { 6 };
    let jitter = if size == 3 { 4 } else if size == 2 { 3 } else { 2 };
    let base   = regular_polygon(sides, radius);
    let rough  = jitter_points(base, jitter, seed);
    rotate_points(rough, seed % 32)
}

fn asteroid_score(size) {
    if size == 3 { return 20; }
    if size == 2 { return 50; }
    100
}

fn spawn_asteroid(world, scene, x, y, vx, vy, size, seed) {
    let shape = asteroid_shape(size, seed);
    let id = world.spawn_visual("asteroid", "asteroid-large", #{
        x: x,
        y: y,
        vx: vx,
        vy: vy,
        collider_polygon: shape,
        collider_layer: 2,
        collider_mask: 5
    });
    if world.exists(id) {
        let asteroid = world.entity(id);
        let visual = asteroid.get_s("visual_id", "");
        asteroid.set("size", size);
        asteroid.set("score", asteroid_score(size));
        asteroid.set("seed", seed);
        asteroid.set("shape_points", shape);
        if visual != "" {
            scene.set(visual, "vector.points", shape);
        }
    }
    id
}

fn spawn_wave(world, scene, local, count) {
    let WORLD_W  = 640.0;
    let WORLD_H  = 360.0;
    let TWO_PI   = 6.283185307;
    let spd_base = 12.0 + (local.wave - 1) * 4.0;
    let anchor_x = WORLD_W / 2.0;
    let anchor_y = WORLD_H / 2.0;

    let ship_id = local.ship_id ?? 0;
    if ship_id > 0 && world.exists(ship_id) {
        let ship_xf = world.entity(ship_id).transform();
        anchor_x = ship_xf["x"];
        anchor_y = ship_xf["y"];
    }

    for i in 0..count {
        let angle = (i.to_float() / count.to_float()) * TWO_PI;
        let dist  = 160.0 + rand() * 80.0;
        let cx    = (anchor_x + cos(angle) * dist).max(30.0).min(WORLD_W - 30.0);
        let cy    = (anchor_y + sin(angle) * dist).max(30.0).min(WORLD_H - 30.0);
        let va    = rand() * TWO_PI;
        let spd   = spd_base + rand() * 8.0;
        let spin  = (rand() - 0.5) * 1.4;

        let vx = cos(va) * spd;
        let vy = sin(va) * spd;
        let id = spawn_asteroid(world, scene, cx, cy, vx, vy, 3, local.wave * 100 + i);
        if world.exists(id) {
            let e = world.entity(id);
            e.set("angular_velocity", spin);
            e.set_heading(rand() * TWO_PI);
            e.physics.set_velocity(vx, vy);
        }
    }
}

fn spawn_debris(world, ox, oy, count) {
    let TWO_PI = 6.283185307;
    for i in 0..count {
        let angle = rand() * TWO_PI;
        let spd   = 20.0 + rand() * 35.0;
        world.spawn_prefab("debris", #{
            x:      ox,
            y:      oy,
            vx:     cos(angle) * spd,
            vy:     sin(angle) * spd,
            ttl_ms: 800
        });
    }
}

fn emit_thrust(world, ship_id, thrust_ms, now) {
    if !world.exists(ship_id) { return; }
    let p = rand() * 2.0 - 1.0;
    let _smoke = world.emit("asteroids.ship_thrust", ship_id, #{
        kind: "fx",
        template: "debris",
        owner_bound: false,
        thrust_ms: thrust_ms,
        speed: 20.0 + rand() * 10.0,
        spread: p * 0.35,
        ttl_ms: 220 + (rand() * 120.0).to_int(),
        radius: 1,
        fg: if (now / 40) % 2 == 0 { "gray" } else { "dark_gray" }
    });
}

fn emit_impact_burst(world, owner_id, count, fg) {
    let TWO_PI = 6.283185307;
    for i in 0..count {
        let spread = rand() * TWO_PI - (TWO_PI / 2.0);
        let _spark = world.emit("asteroids.impact_sparks", owner_id, #{
            kind: "fx",
            template: "debris",
            owner_bound: false,
            speed: 28.0 + rand() * 32.0,
            spread: spread,
            ttl_ms: 180 + (rand() * 180.0).to_int(),
            radius: 1,
            fg: fg
        });
    }
}

fn update_hud(scene, local) {
    scene.set("hud-score", "text.content", `${local.score}`);
    scene.set("hud-lives", "text.content", `${local.lives}`);
    scene.set("hud-wave",  "text.content", `${local.wave}`);
}

// ─── Constants ────────────────────────────────────────────────────────────────
let WORLD_W     = 640.0;
let WORLD_H     = 360.0;
let BULLET_TTL  = 3000;
let BULLET_SPD  = 180.0;
let MAX_BULLETS = 4;
let FIRE_CD_MS  = 250;
let INV_MS      = 2500;
let RESPAWN_MS  = 1200;
let WAVE_CLEAR_MS = 900;
let INIT_LIVES  = 3;
let INIT_WAVE_N = 4;
let TWO_PI      = 6.283185307;
let STATE_PLAYING    = "playing";
let STATE_DEAD_PAUSE = "dead_pause";
let STATE_WAVE_CLEAR = "wave_clear";
let STATE_GAME_OVER  = "game_over";

// ─── Guard: first run, local is () until initialized ──────────────────────────
if type_of(local) == "()" { local = #{}; }

// ─── One-time init ────────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    input.load_profile("default");
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    scene.set("hud-gameover", "visible", false);

    let ship_id = spawn_ship(world);
    local.ship_id     = ship_id;
    local.score       = 0;
    local.lives       = INIT_LIVES;
    local.wave        = 1;
    local.state       = STATE_PLAYING;
    local.state_until = 0;
    local.game_over   = false;
    local.initialized = true;
    local.thrust_ms   = 0;

    if world.exists(ship_id) {
        world.entity(ship_id).status_add("invulnerable", INV_MS);
    }

    spawn_wave(world, scene, local, INIT_WAVE_N);
    update_hud(scene, local);
}

let now = time.scene_elapsed_ms;

if type_of(local.state) == "()" {
    local.state = STATE_PLAYING;
}
if type_of(local.state_until) == "()" {
    local.state_until = 0;
}

// ─── Game over: wait for restart ─────────────────────────────────────────────
if local.state == STATE_GAME_OVER {
    if input.action_just_pressed("restart") || input.action_just_pressed("fire") {
        world.reset_dynamic_entities();
        local.initialized = false;
        local.game_over   = false;
        scene.set("hud-gameover", "visible", false);
    }
    return #{state: local};
}

// ─── Dead pause: wait, then respawn with invulnerability ──────────────────────
if local.state == STATE_DEAD_PAUSE {
    if now >= local.state_until {
        let ship_id = spawn_ship(world);
        local.ship_id = ship_id;
        if world.exists(ship_id) {
            let ship = world.entity(ship_id);
            ship.physics.set_velocity(0.0, 0.0);
            ship.status_add("invulnerable", INV_MS);
        }
        local.state = STATE_PLAYING;
    }
    return #{state: local};
}

let ship_id = local.ship_id ?? 0;
if ship_id <= 0 || !world.exists(ship_id) {
    return #{state: local};
}

let ship = world.entity(ship_id);

// ─── Ship controller input ────────────────────────────────────────────────────
let turn = (if input.action_down("turn_right") { 1 } else { 0 })
         - (if input.action_down("turn_left")  { 1 } else { 0 });
ship.set_turn(turn);
let thrust_on = input.action_down("thrust");
ship.set_thrust(thrust_on);
if thrust_on {
    local.thrust_ms = (local.thrust_ms ?? 0) + 16;
    emit_thrust(world, ship_id, local.thrust_ms, now);
} else {
    local.thrust_ms = 0;
}

// ─── Wave clear pause ──────────────────────────────────────────────────────────
if local.state == STATE_WAVE_CLEAR {
    if now >= local.state_until {
        local.wave += 1;
        spawn_wave(world, scene, local, INIT_WAVE_N + local.wave - 1);
        update_hud(scene, local);
        local.state = STATE_PLAYING;
    } else {
        return #{state: local};
    }
}

// ─── Fire ─────────────────────────────────────────────────────────────────────
if world.count_kind("bullet") < MAX_BULLETS
    && input.action_just_pressed("fire")
    && ship.cooldown_ready("fire") {
    let hv = ship.heading_vector();
    let xf = ship.transform();
    world.spawn_prefab("bullet", #{
        x:      xf["x"]  + hv["x"] * 14.0,
        y:      xf["y"]  + hv["y"] * 14.0,
        vx:     hv["x"] * BULLET_SPD,
        vy:     hv["y"] * BULLET_SPD,
        ttl_ms: BULLET_TTL
    });
    ship.cooldown_start("fire", FIRE_CD_MS);
    audio.cue("fire");
}

// ─── Collision: bullet × asteroid ─────────────────────────────────────────────
for hit in world.collision_enters("bullet", "asteroid") {
    let bullet_id   = hit["bullet"];
    let asteroid_id = hit["asteroid"];
    if !world.exists(bullet_id) || !world.exists(asteroid_id) { continue; }

        let ae   = world.entity(asteroid_id);
        let xf   = ae.transform();
        let size = ae.get_i("size", 3);
        let pts  = ae.get_i("score", 20);

    local.score += pts;
    emit_impact_burst(world, asteroid_id, 5, "yellow");
    spawn_debris(world, xf["x"], xf["y"], 3);
    audio.cue(if size == 3 { "explosion_large" } else { "explosion_small" });

    world.despawn(bullet_id);
    world.despawn(asteroid_id);

        if size > 1 {
            for j in 0..2 {
                let angle = rand() * TWO_PI;
                let spd   = 18.0 + rand() * 15.0;
                let child_size = size - 1;
                let vx = cos(angle) * spd;
                let vy = sin(angle) * spd;
                let nid = spawn_asteroid(
                    world,
                    scene,
                    xf["x"],
                    xf["y"],
                    vx,
                    vy,
                    child_size,
                    asteroid_id + j + size * 1000,
                );
                if world.exists(nid) {
                    let ne    = world.entity(nid);
                    let spin2 = (rand() - 0.5) * 1.6;
                    ne.set("angular_velocity", spin2);
                    ne.set_heading(rand() * TWO_PI);
                    ne.physics.set_velocity(vx, vy);
                }
            }
        }

    update_hud(scene, local);
}

// ─── Collision: ship × asteroid ───────────────────────────────────────────────
if !ship.status_has("invulnerable") {
    for hit in world.collision_enters("ship", "asteroid") {
        let _hit = hit;
        local.lives -= 1;
        let xf = ship.transform();
        emit_impact_burst(world, ship_id, 7, "white");
        spawn_debris(world, xf["x"], xf["y"], 4);
        audio.cue("explosion_large");

        if local.lives <= 0 {
            world.despawn(ship_id);
            local.ship_id = 0;
            local.state = STATE_GAME_OVER;
            local.game_over = true;
            scene.set("hud-gameover", "visible", true);
        } else {
            world.despawn(ship_id);
            local.ship_id = 0;
            local.state = STATE_DEAD_PAUSE;
            local.state_until = now + RESPAWN_MS;
        }
        update_hud(scene, local);
        break;
    }
}

// ─── Wave clear ───────────────────────────────────────────────────────────────
if world.count_kind("asteroid") == 0 {
    local.state = STATE_WAVE_CLEAR;
    local.state_until = now + WAVE_CLEAR_MS;
}

#{state: local}
