import "asteroids-shared" as h;

fn spawn_ship(world, cfg) {
    let id = world.spawn_object("ship", #{});
    if id > 0 {
        let ship = world.entity(id);
        world.set_visual(id, "ship");
        world.set_transform(id, 0.0, 0.0, 0.0);
        world.set_physics(id, 0.0, 0.0, 0.0, 0.0, 0.10, cfg["ship_max_speed"]);
        world.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF);
        ship.attach_ship_controller(#{
            turn_step_ms: cfg["turn_step_ms"],
            thrust_power: cfg["ship_thrust"],
            max_speed: cfg["ship_max_speed"],
            heading_bits: 32
        });
        world.enable_wrap_bounds(id);
        ship.status_add("invulnerable", 3000);
    }
    id
}

fn spawn_asteroid(world, x, y, vx, vy, shape, size) {
    let id = world.spawn_visual("asteroid", "asteroid-template", #{
        x: x, y: y, heading: 0.0, collider_radius: asteroid_radius(size)
    });
    if id > 0 {
        world.set_physics(id, vx * h::velocity_scale(), vy * h::velocity_scale(), 0.0, 0.0, 0.0, 0.0);
        world.enable_wrap_bounds(id);
        let rot_phase = world.rand_i(0, 31);
        let rot_speed = if world.rand_i(0, 1) == 0 { 1 } else { -1 };
        let rot_step = 72 + world.rand_i(0, 2) * 16;
        world.entity(id).set_many(#{
            shape: shape, size: size, flash_ms: 0, flash_total_ms: 0, split_pending: false,
            rot_phase: rot_phase, rot_speed: rot_speed, rot_step_ms: rot_step, rot_accum_ms: 0
        });
    }
    id
}

fn spawn_bullet(world, x, y, vx, vy, ttl_ms) {
    let id = world.spawn_visual("bullet", "bullet-template", #{
        x: x, y: y, heading: 0.0, collider_radius: 3.0, lifetime_ms: ttl_ms
    });
    if id > 0 {
        world.set_physics(id, vx * h::velocity_scale(), vy * h::velocity_scale(), 0.0, 0.0, 0.0, 0.0);
        world.enable_wrap_bounds(id);
    }
    id
}

fn spawn_smoke(world, x, y, vx, vy, ttl_ms, radius) {
    let id = world.spawn_visual("smoke", "smoke-template", #{
        x: x, y: y, heading: 0.0, lifetime_ms: ttl_ms
    });
    if id > 0 {
        world.set_physics(id, vx * h::velocity_scale(), vy * h::velocity_scale(), 0.0, 0.0, 0.04, 0.0);
        world.entity(id).set_many(#{ttl_ms: ttl_ms, max_ttl_ms: ttl_ms, radius: radius});
    }
    id
}

fn spawn_disintegration(world, x, y) {
    for idx in 0..12 {
        let uv = unit_vec32((idx * 3) % 32);
        spawn_smoke(world, x, y, uv.x * 0.879, uv.y * 0.879, 700, 3);
    }
}

fn spawn_initial_asteroids(world) {
    spawn_asteroid(world, -300.0, -210.0, 2, 0, 0, 2);
    spawn_asteroid(world,  300.0, -210.0, 0, 2, 1, 3);
    spawn_asteroid(world,  300.0,  210.0,-2, 0, 2, 2);
    spawn_asteroid(world, -300.0,  210.0, 0,-2, 3, 1);
    spawn_asteroid(world, -290.0,  -40.0, 2, 1, 0, 2);
    spawn_asteroid(world, -140.0, -230.0, 2, 1, 1, 1);
    spawn_asteroid(world,  140.0, -230.0,-2, 1, 2, 2);
    spawn_asteroid(world,  290.0,   30.0,-2,-1, 3, 3);
    spawn_asteroid(world,  120.0,  230.0,-2,-1, 1, 2);
    spawn_asteroid(world, -120.0,  230.0, 2,-1, 0, 1);
}

fn respawn_x(seed, min_x, max_x, ship_x) {
    let x = if (seed % 2) == 0 { min_x + 10 } else { max_x - 10 };
    if abs_i(x - to_i(ship_x)) < 90 {
        x = if x < 0 { max_x - 10 } else { min_x + 10 };
    }
    x
}

fn respawn_y(seed, min_y, max_y, ship_y) {
    let span = (max_y - min_y) - 20;
    let y = min_y + 10 + (seed % span);
    if abs_i(y - to_i(ship_y)) < 70 {
        y = y + 76;
        if y > max_y - 10 { y = min_y + 10; }
    }
    y
}

fn speed_for_seed(seed) {
    let v = (seed % 3) + 1;
    if ((seed / 7) % 2) == 0 { v } else { -v }
}

fn clear_all_entities(world) {
    for id in world.ids() { world.entity(id).despawn(); }
}

fn reset_game_state(world, game, level, local, now) {
    let cfg = h::load_game_config(level);
    local["cfg"] = cfg;
    world.rand_seed(now);
    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["base_lives"]);
    game.set("/ast/wave", 1);
    game.set("/ast/msg_ttl_ms", 0);
    game.set("/ast/last_ms", now);
    cfg
}

let WORLD_MIN_X = -320.0;
let WORLD_MAX_X =  320.0;
let WORLD_MIN_Y = -240.0;
let WORLD_MAX_Y =  240.0;

if !local.contains("init") {
    let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
    clear_all_entities(world);
    world.set_world_bounds(WORLD_MIN_X, WORLD_MAX_X, WORLD_MIN_Y, WORLD_MAX_Y);
    let cfg = reset_game_state(world, game, level, local, now);
    local["ship_id"] = spawn_ship(world, cfg);
    spawn_initial_asteroids(world);
    for ast_id in world.query_kind("asteroid") {
        h::spawn_crack_visuals(world, scene, ast_id);
    }
    // Bind input actions once — scripts use action_down() instead of raw key polling
    input.bind_action("turn_left",  [KEY_LEFT,  "a", "A"]);
    input.bind_action("turn_right", [KEY_RIGHT, "d", "D"]);
    input.bind_action("thrust",     [KEY_UP,    "w", "W"]);
    input.bind_action("fire",       [KEY_SPACE, "f", "F"]);
    audio.play_song("asteroids.game-loop");
    local["init"] = true;
    diag.info("asteroids session init");
}

let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
let min_x = WORLD_MIN_X;
let max_x = WORLD_MAX_X;
let min_y = WORLD_MIN_Y;
let max_y = WORLD_MAX_Y;
let ship_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
let last_ms = game.get("/ast/last_ms");
let dt_raw = now - last_ms;
let dt = if dt_raw < 0 { 0 } else if dt_raw > 220 { 220 } else { dt_raw };
game.set("/ast/last_ms", now);

if key.pressed == true {
    let code = if key.code.type_of() == "string" { key.code } else { "" };
    if code == KEY_ESC {
        audio.stop_song();
        game.jump("asteroids.mainmenu");
        return ();
    }
    let reset_requested = false;
    let reset_label = "RESET";
    if code == "r" || code == "R" {
        reset_requested = true;
    } else if code == "F5" {
        if level.select("asteroids.easy") { reset_requested = true; reset_label = "EASY"; }
    } else if code == "F6" {
        if level.select("asteroids.default") { reset_requested = true; reset_label = "DEFAULT"; }
    } else if code == "F7" {
        if level.select("asteroids.hard") { reset_requested = true; reset_label = "HARD"; }
    }
    if reset_requested {
        clear_all_entities(world);
        let cfg = reset_game_state(world, game, level, local, now);
        local["ship_id"] = spawn_ship(world, cfg);
        spawn_initial_asteroids(world);
        for ast_id in world.query_kind("asteroid") {
            h::spawn_crack_visuals(world, scene, ast_id);
        }
        h::show_message(scene, game, reset_label, 500);
        diag.warn(`reset mode=${reset_label}`);
    }
}

let left_down  = input.action_down("turn_left");
let right_down = input.action_down("turn_right");
let thrust_down = input.action_down("thrust");
let fire_down  = input.action_down("fire");

if world.exists(ship_id) {
    let ship_ref = world.entity(ship_id);
    let cfg = if local.contains("cfg") { local["cfg"] } else { h::load_game_config(level) };

    // Use TopDownShipController API
    let turn_dir = if left_down { -1 } else if right_down { 1 } else { 0 };
    ship_ref.set_turn(turn_dir);
    ship_ref.set_thrust(thrust_down);

    let t = ship_ref.transform();
    let tx = t.x;
    let ty = t.y;
    let phys = ship_ref.physics();
    let vx = phys.vx;
    let vy = phys.vy;
    let ship_h = ship_ref.heading();

    // Periodic ship state diagnostic (every ~2 seconds)
    let diag_tick = to_i(now / 2000);
    let last_diag = if local.contains("diag_tick") { local["diag_tick"] } else { -1 };
    if diag_tick != last_diag {
        local["diag_tick"] = diag_tick;
        let invuln_remaining = ship_ref.status_remaining("invulnerable");
        diag.info(`ship pos=(${tx},${ty}) vel=(${vx},${vy}) h=${ship_h} invuln=${invuln_remaining}`);
    }

    // Thrust generates smoke particles
    let hv = ship_ref.heading_vector();
    if thrust_down {
        if ship_ref.cooldown_ready("smoke") && world.count_kind("smoke") < 40 {
            spawn_smoke(world, tx - (hv.x * 6.0), ty - (hv.y * 6.0),
                       vx / h::velocity_scale() - (hv.x * 0.35), vy / h::velocity_scale() - (hv.y * 0.35), 520, 3);
            ship_ref.cooldown_start("smoke", 48);
        }
    }

    if fire_down && ship_ref.cooldown_ready("shot") && world.count_kind("bullet") < 8 {
        let bspeed = cfg["bullet_speed"];
        spawn_bullet(world,
            tx + (hv.x * 9.0), ty + (hv.y * 9.0),
            vx / h::velocity_scale() + (hv.x * bspeed / h::velocity_scale()),
            vy / h::velocity_scale() + (hv.y * bspeed / h::velocity_scale()),
            cfg["bullet_ttl_ms"]);
        ship_ref.cooldown_start("shot", cfg["shot_cooldown_ms"]);
        audio.event("gameplay.ship.shoot", 1.0);
    }
}

for ast_id in world.query_kind("asteroid") {
    let ast_ref = world.entity(ast_id);
    if !ast_ref.exists() { continue; }
    
    let rot_accum = ast_ref.get_i("/rot_accum_ms", 0);
    let rot_phase = ast_ref.get_i("/rot_phase", 0);
    let rot_speed = ast_ref.get_i("/rot_speed", 1);
    let rot_step = ast_ref.get_i("/rot_step_ms", 72);
    
    rot_accum += dt;
    while rot_accum >= rot_step {
        rot_phase = h::wrap_heading32(rot_phase + rot_speed);
        rot_accum -= rot_step;
    }

    let flash_ms = ast_ref.get_i("/flash_ms", 0);
    let flash_total = ast_ref.get_i("/flash_total_ms", 0);
    let split_pending = ast_ref.flag("split_pending");
    let ast_size = ast_ref.get_i("/size", 1);

    if flash_ms > 0 {
        flash_ms -= dt;
        if flash_ms <= 0 && split_pending {
            if ast_size <= 0 {
                ast_ref.despawn();
                continue;
            }
            let next_size = ast_size - 1;
            let base_heading = rot_phase;
            let t = ast_ref.transform();
            for frag_idx in 0..3 {
                let split_heading = h::wrap_heading32(base_heading + h::fragment_heading_offset(frag_idx));
                let split_sin = sin32(split_heading);
                let split_cos = sin32(split_heading + 8);
                let dir_x = sign_i(split_sin, if frag_idx == 1 { 1 } else { -1 });
                let dir_y = sign_i(-split_cos, if frag_idx == 0 { -1 } else { 1 });
                let spawn_offset = asteroid_radius(next_size) + 4;
                spawn_asteroid(world,
                    t.x + (to_float(split_sin * spawn_offset) / 1024.0),
                    t.y - (to_float(split_cos * spawn_offset) / 1024.0),
                    to_float(dir_x * (next_size + 2)),
                    to_float(dir_y * (next_size + 2)),
                    world.rand_i(0, 3),
                    next_size
                );
            }
            ast_ref.despawn();
            audio.event("gameplay.asteroid.split", 1.0);
            for child_id in world.query_kind("asteroid") {
                if !world.exists(child_id) { continue; }
                h::spawn_crack_visuals(world, scene, child_id);
            }
            continue;
        }
    }

    ast_ref.set_many(#{
        flash_ms: flash_ms, flash_total_ms: if flash_ms <= 0 { 0 } else { flash_total },
        split_pending: split_pending, rot_phase: rot_phase, rot_accum_ms: rot_accum
    });
}

// Bullets wrap automatically via engine (enable_wrap set at spawn)

let score = game.get_i("/ast/score", 0);
let lives = game.get_i("/ast/lives", 0);
let ship_ref = world.entity(ship_id);
let ship_invuln = if ship_ref.exists() { ship_ref.status_has("invulnerable") } else { false };

for hit in world.collision_enters("ship", "asteroid") {
    let ast_hit_id = hit["asteroid"];
    let hit_ref = world.entity(ast_hit_id);

    if hit_ref.exists() && hit_ref.flag("split_pending") { continue; }

    if !ship_ref.status_has("invulnerable") && world.exists(ship_id) {
        let t = ship_ref.transform();
        spawn_disintegration(world, t.x, t.y);
        audio.event("gameplay.ship.hit", 1.0);

        if hit_ref.exists() {
            let phys = hit_ref.physics();
            hit_ref.set_many(#{
                flash_ms: h::CRACK_DURATION_MS, flash_total_ms: h::CRACK_DURATION_MS, split_pending: true
            });
            hit_ref.set_velocity(clamp_f(-phys.vx, -4.0 * h::velocity_scale(), 4.0 * h::velocity_scale()),
                                clamp_f(-phys.vy, -4.0 * h::velocity_scale(), 4.0 * h::velocity_scale()));
        }
        lives -= 1;
        ship_ref.set_velocity(0.0, 0.0);
        ship_ref.status_add("invulnerable", 3000);
        h::show_message(scene, game, "HIT!", 450);
    }
}

for hit in world.collision_enters("bullet", "asteroid") {
    let bullet_id = hit["bullet"];
    let ast_hit_id = hit["asteroid"];
    let hit_ref = world.entity(ast_hit_id);

    // collision_enters fires once per new contact; split_pending avoids double-scoring
    if hit_ref.exists() && hit_ref.flag("split_pending") { continue; }

    if world.exists(bullet_id) {
        world.entity(bullet_id).despawn();
    }
    if hit_ref.exists() {
        score += asteroid_score(hit_ref.get_i("/size", 0));
        hit_ref.set_many(#{
            flash_ms: h::CRACK_DURATION_MS, flash_total_ms: h::CRACK_DURATION_MS, split_pending: true
        });
        h::show_message(scene, game, "HIT", 250);
        diag.info(`hit asteroid=${ast_hit_id}`);
    }
}

game.set("/ast/score", score);
game.set("/ast/lives", lives);

let msg_ttl = game.get_i("/ast/msg_ttl_ms", 0);
if msg_ttl > 0 {
    let new_ttl = msg_ttl - dt;
    if new_ttl <= 0 { scene.set("game-message", "text.content", ""); }
    game.set("/ast/msg_ttl_ms", new_ttl);
}

if !world.any_alive("asteroid") {
    let wave = game.get_i("/ast/wave", 0);
    wave += 1;
    game.set("/ast/wave", wave);
    h::show_message(scene, game, `WAVE ${wave}`, 650);
    diag.info(`wave start wave=${wave}`);

    let cfg = if local.contains("cfg") { local["cfg"] } else { h::load_game_config(level) };
    let spawn_count = clamp_i(cfg["wave_base_spawn_count"] + wave, cfg["wave_min_spawn_count"], cfg["wave_max_spawn_count"]);

    let ship_t = if ship_ref.exists() { ship_ref.transform() } else { #{x: 0, y: 0} };
    for idx in 0..spawn_count {
        let rx = world.rand_i(0, 2147483646);
        let ry = world.rand_i(0, 2147483646);
        spawn_asteroid(world,
            respawn_x(rx, min_x, max_x, ship_t.x),
            respawn_y(ry, min_y, max_y, ship_t.y),
            speed_for_seed(world.rand_i(0, 2147483646)),
            speed_for_seed(world.rand_i(0, 2147483646)),
            world.rand_i(0, 3),
            if idx < 2 { 3 } else if idx < 5 { 2 } else { 1 }
        );
    }
    for new_ast_id in world.query_kind("asteroid") {
        if !world.exists(new_ast_id) { continue; }
        h::spawn_crack_visuals(world, scene, new_ast_id);
    }
}

if lives <= 0 {
    diag.warn(`game over score=${score} wave=${game.get_i("/ast/wave", 0)}`);
    persist.push("/asteroids/highscores", #{ score: score, wave: game.get_i("/ast/wave", 0), at_ms: now });
    game.jump("asteroids.highscores");
    return ();
}
