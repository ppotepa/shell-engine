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

fn asteroid_hp(size) {
    if size == 3 { return 5; }
    if size == 2 { return 3; }
    1
}

fn asteroid_template(size) {
    if size == 3 { "asteroid-large" } else if size == 2 { "asteroid-medium" } else { "asteroid-small" }
}

fn asteroid_size_radius(size) {
    if size == 3 { 22 } else if size == 2 { 11 } else { 6 }
}

// Build a child fragment shape from the parent's current (possibly cracked) shape.
// Scales the parent polygon to child radius, then applies a slight random rotation
// and jitter so each fragment looks distinct.
fn asteroid_child_shape(parent_shape, parent_size, child_size, seed) {
    let pr = asteroid_size_radius(parent_size);
    let cr = asteroid_size_radius(child_size);
    let scaled = scale_points(parent_shape, cr, pr);
    let jittered = jitter_points(scaled, 2, seed);
    rotate_points(jittered, seed % 32)
}

fn spawn_asteroid(world, scene, x, y, vx, vy, size, seed) {
    spawn_asteroid_shaped(world, scene, x, y, vx, vy, size, seed, asteroid_shape(size, seed))
}

fn spawn_asteroid_shaped(world, scene, x, y, vx, vy, size, seed, shape) {
    let template = asteroid_template(size);
    let id = world.spawn_visual("asteroid", template, #{
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
        world.disable_wrap(id);
        // visual_id lives in VisualBinding component, not extra_data — store it explicitly
        let visual_id = `asteroid-${id}`;
        asteroid.set("size", size);
        asteroid.set("hp", asteroid_hp(size));
        asteroid.set("score", asteroid_score(size));
        asteroid.set("seed", seed);
        asteroid.set("shape_points", shape);
        asteroid.set("visual_id", visual_id);
        // scene.set here would no-op: resolver snapshot was taken before this
        // frame's SceneSpawn ran, so the sprite isn't registered yet.
        // Set a flag and apply on the next frame (deferred init).
        asteroid.set("needs_shape_init", true);
    }
    id
}

fn spawn_wave(world, scene, local, count) {
    let WORLD_W  = 640.0;
    let WORLD_H  = 360.0;
    let TWO_PI   = 6.283185307;
    let spd_base = 55.0 + (local.wave - 1) * 8.0;
    let spawn_pad = 60.0;

    for i in 0..count {
        let side  = i % 4;
        let spd   = spd_base + rand() * 10.0;
        let spin  = (rand() - 0.5) * 1.4;
        let cx = 0.0;
        let cy = 0.0;
        let vx = 0.0;
        let vy = 0.0;

        let lane = ((i / 4).to_float() + 0.5) / ((count / 4 + 1).to_float());
        if side == 0 {
            cx = -spawn_pad;
            cy = 20.0 + lane * (WORLD_H - 40.0);
            vx = spd;
            vy = (rand() - 0.5) * spd * 0.3;
        } else if side == 1 {
            cx = WORLD_W + spawn_pad;
            cy = 20.0 + lane * (WORLD_H - 40.0);
            vx = -spd;
            vy = (rand() - 0.5) * spd * 0.3;
        } else if side == 2 {
            cx = 20.0 + lane * (WORLD_W - 40.0);
            cy = -spawn_pad;
            vx = (rand() - 0.5) * spd * 0.3;
            vy = spd;
        } else {
            cx = 20.0 + lane * (WORLD_W - 40.0);
            cy = WORLD_H + spawn_pad;
            vx = (rand() - 0.5) * spd * 0.3;
            vy = -spd;
        }

        let id = spawn_asteroid(world, scene, cx, cy, vx, vy, 3, local.wave * 100 + i);
        if world.exists(id) {
            let e = world.entity(id);
            e.set("angular_velocity", spin);
            e.set_heading(rand() * TWO_PI);
        }
    }
}

fn spawn_debris(world, ox, oy, count) {
    let TWO_PI = 6.283185307;
    for i in 0..count {
        let angle = rand() * TWO_PI;
        let spd   = 50.0 + rand() * 100.0;
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
let BULLET_SPD  = 300.0;
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
    audio.event("fire");
}

// ─── Collision: bullet × asteroid (dent deformation + HP) ─────────────────────
for hit in world.collision_enters("bullet", "asteroid") {
    let bullet_id   = hit["bullet"];
    let asteroid_id = hit["asteroid"];
    if !world.exists(bullet_id) || !world.exists(asteroid_id) { continue; }

    let ae   = world.entity(asteroid_id);
    let xf   = ae.transform();
    let size = ae.get_i("size", 3);
    let pts  = ae.get_i("score", 20);

    // Impact point in asteroid-local coordinates
    // Counter-rotate by entity heading so dent aligns with the visually rotated polygon
    let bxf = world.entity(bullet_id).transform();
    let raw_x = bxf["x"] - xf["x"];
    let raw_y = bxf["y"] - xf["y"];
    let heading = xf["heading"];
    let impact_x = (raw_x * cos(heading) + raw_y * sin(heading)).to_int();
    let impact_y = (-raw_x * sin(heading) + raw_y * cos(heading)).to_int();

    world.despawn(bullet_id);

    // Decrement HP
    let hp = ae.get_i("hp", 1) - 1;
    ae.set("hp", hp);

    // FX at impact (scaled by remaining HP)
    let spark_count = if hp > 0 { 3 } else { 6 };
    emit_impact_burst(world, asteroid_id, spark_count, "yellow");
    spawn_debris(world, bxf["x"], bxf["y"], if hp > 0 { 1 } else { 3 });
    audio.event(if hp > 0 { "explosion_small" } else if size == 3 { "explosion_large" } else { "explosion_small" });
    effects.shake(if hp > 0 { 80 } else { 220 }, if hp > 0 { 0.3 } else { 1.0 }, if hp > 0 { 0.15 } else { 0.4 }, 9.0);

    if hp > 0 {
        // Still alive — carve a V-notch crack at the impact point.
        // Strength 50–75% ensures the crack tip is always visible at terminal resolution.
        let shape = ae.get("shape_points");
        if type_of(shape) != "()" {
            let depth = 50 + (rand() * 25.0).to_int();
            let new_shape = crack_polygon(shape, impact_x, impact_y, depth);
            ae.set("shape_points", new_shape);
            let visual_id = ae.get_s("visual_id", "");
            if visual_id != "" {
                scene.set(visual_id, "vector.points", new_shape);
            }
            ae.physics.set_collider_polygon(new_shape, 2, 5);
        }
    } else {
        // Destroyed — score, big FX, and split if large enough
        local.score += pts;
        spawn_debris(world, xf["x"], xf["y"], 3);
        effects.shake(220, 1.2, 0.5, 9.0);
        world.despawn(asteroid_id);

        if size > 1 {
            let parent_shape = ae.get("shape_points");
            for j in 0..2 {
                let angle = rand() * TWO_PI;
                let spd   = 70.0 + rand() * 45.0;
                let child_size = size - 1;
                let vx = cos(angle) * spd;
                let vy = sin(angle) * spd;
                let child_seed = asteroid_id + j + size * 1000;
                // Derive child shape from parent's current (cracked) outline,
                // scaled to child radius. Falls back to a fresh shape if missing.
                let child_shape = if type_of(parent_shape) != "()" {
                    asteroid_child_shape(parent_shape, size, child_size, child_seed)
                } else {
                    asteroid_shape(child_size, child_seed)
                };
                let nid = spawn_asteroid_shaped(
                    world,
                    scene,
                    xf["x"],
                    xf["y"],
                    vx,
                    vy,
                    child_size,
                    child_seed,
                    child_shape,
                );
                if world.exists(nid) {
                    let ne    = world.entity(nid);
                    let spin2 = (rand() - 0.5) * 1.6;
                    ne.set("angular_velocity", spin2);
                    ne.set_heading(rand() * TWO_PI);
                }
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
        audio.event("explosion_large");
        effects.shake(400, 2.0, 0.8, 7.0);

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

// ─── Deferred shape init (resolves same-frame resolver staleness) ─────────────
// Sprites spawned this frame aren't in the resolver snapshot yet, so any
// scene.set in spawn_asteroid would silently no-op. We store needs_shape_init=true
// at spawn and flush here, one frame later, when the resolver is fresh.
for id in world.query_kind("asteroid") {
    let e = world.entity(id);
    if e.get_bool("needs_shape_init", false) {
        let vid = e.get_s("visual_id", "");
        let pts = e.get("shape_points");
        if vid != "" && type_of(pts) != "()" {
            scene.set(vid, "vector.points", pts);
        }
        e.set("needs_shape_init", false);
    }
}

// ─── Out-of-bounds re-entry ───────────────────────────────────────────────────
let OOB_PAD = 250.0;
let REENTRY_PAD = 60.0;
let REENTRY_SPD_BASE = 55.0 + ((local.wave ?? 1) - 1) * 8.0;
for id in world.query_kind("asteroid") {
    let xf = world.entity(id).transform();
    if xf["x"] < -OOB_PAD || xf["x"] > WORLD_W + OOB_PAD
    || xf["y"] < -OOB_PAD || xf["y"] > WORLD_H + OOB_PAD {
        let e = world.entity(id);
        let side = (rand() * 4.0).to_int();
        let spd  = REENTRY_SPD_BASE + rand() * 15.0;
        let nx = 0.0;
        let ny = 0.0;
        let nvx = 0.0;
        let nvy = 0.0;
        if side == 0 {
            nx = -REENTRY_PAD;
            ny = 20.0 + rand() * (WORLD_H - 40.0);
            nvx = spd;
            nvy = (rand() - 0.5) * spd * 0.3;
        } else if side == 1 {
            nx = WORLD_W + REENTRY_PAD;
            ny = 20.0 + rand() * (WORLD_H - 40.0);
            nvx = -spd;
            nvy = (rand() - 0.5) * spd * 0.3;
        } else if side == 2 {
            nx = 20.0 + rand() * (WORLD_W - 40.0);
            ny = -REENTRY_PAD;
            nvx = (rand() - 0.5) * spd * 0.3;
            nvy = spd;
        } else {
            nx = 20.0 + rand() * (WORLD_W - 40.0);
            ny = WORLD_H + REENTRY_PAD;
            nvx = (rand() - 0.5) * spd * 0.3;
            nvy = -spd;
        }
        e.set_position(nx, ny);
        e.physics.set_velocity(nvx, nvy);
    }
}

// ─── Wave clear ───────────────────────────────────────────────────────────────
if world.count_kind("asteroid") == 0 {
    local.state = STATE_WAVE_CLEAR;
    local.state_until = now + WAVE_CLEAR_MS;
}

#{state: local}
