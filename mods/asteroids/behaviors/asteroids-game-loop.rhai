import "asteroids-shared" as h;

fn next_seed(seed) {
    ((seed * 1103515245) + 12345) % 2147483647
}

fn level_i(level_api, path, fallback) {
    if level_api.has(path) {
        let value = level_api.get(path);
        if value.type_of() == "i64" { return value; }
    }
    fallback
}

fn load_game_config(level) {
    let base_lives = level_i(level, "/player/lives", 3);
    let shot_cooldown_ms = clamp_i(level_i(level, "/player/shot_cooldown_ms", 200), 200, 1000);
    let ship_thrust = to_float(clamp_i(level_i(level, "/player/thrust_fp", 170), 60, 600)) / 1024.0 * h::velocity_scale();
    let ship_max_speed = to_float(clamp_i(level_i(level, "/player/max_speed_fp", 4608), 1024, 14000)) / 1024.0 * h::velocity_scale();
    let turn_step_ms = clamp_i(level_i(level, "/player/turn_step_ms", 40), 16, 150);
    let wave_base_spawn_count = clamp_i(level_i(level, "/waves/base_spawn_count", 4), 1, 20);
    let wave_min_spawn_count = clamp_i(level_i(level, "/waves/min_spawn_count", 5), 1, 30);
    let wave_max_spawn_count = clamp_i(level_i(level, "/waves/max_spawn_count", 10), 1, 40);
    if wave_min_spawn_count > wave_max_spawn_count { wave_min_spawn_count = wave_max_spawn_count; }
    let bullet_ttl_ms = clamp_i(level_i(level, "/combat/bullet_ttl_ms", 900), 100, 6000);
    let bullet_speed = to_float(clamp_i(level_i(level, "/combat/bullet_speed_fp", 4200), 1000, 18000)) / 1024.0 * h::velocity_scale();
    #{
        base_lives: base_lives,
        shot_cooldown_ms: shot_cooldown_ms,
        ship_thrust: ship_thrust,
        ship_max_speed: ship_max_speed,
        turn_step_ms: turn_step_ms,
        wave_base_spawn_count: wave_base_spawn_count,
        wave_min_spawn_count: wave_min_spawn_count,
        wave_max_spawn_count: wave_max_spawn_count,
        bullet_ttl_ms: bullet_ttl_ms,
        bullet_speed: bullet_speed
    }
}

fn spawn_ship(world) {
    let id = world.spawn_object("ship", #{});
    if id > 0 {
        world.set_visual(id, "ship");
        world.set_transform(id, 0.0, 0.0, 0.0);
        world.set_physics(id, 0.0, 0.0, 0.0, 0.0, 0.02, h::velocity_scale() * 4.5);
        world.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF);
        world.entity(id).set_many(#{h: 0, invuln_ms: 3000});
    }
    id
}

fn spawn_asteroid(world, x, y, vx, vy, shape, size) {
    let spin_seed = abs_i(to_i(x * 31) + to_i(y * 17) + to_i(vx * 13) + to_i(vy * 19) + (shape * 23) + (size * 29));
    let id = world.spawn_visual("asteroid", "asteroid-template", #{
        x: x, y: y, heading: 0.0, collider_radius: asteroid_radius(size)
    });
    if id > 0 {
        world.set_physics(id, vx * h::velocity_scale(), vy * h::velocity_scale(), 0.0, 0.0, 0.0, 0.0);
        world.entity(id).set_many(#{
            shape: shape, size: size, flash_ms: 0, flash_total_ms: 0, split_pending: false,
            collide_cd_ms: 0, rot_phase: spin_seed % 32,
            rot_speed: if (spin_seed % 2) == 0 { 1 } else { -1 },
            rot_step_ms: 72 + ((spin_seed % 3) * 16), rot_accum_ms: 0
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
        let ang = (idx * 3) % 32;
        let dx = to_float(sin32(ang)) * 0.879;
        let dy = -to_float(sin32(ang + 8)) * 0.879;
        spawn_smoke(world, x, y, dx, dy, 700, 3);
    }
}

fn spawn_initial_asteroids(world) {
    spawn_asteroid(world, -300.0, -210.0, 2, 0, 0, 2);
    spawn_asteroid(world, 300.0, -210.0, 0, 2, 1, 3);
    spawn_asteroid(world, 300.0, 210.0, -2, 0, 2, 2);
    spawn_asteroid(world, -300.0, 210.0, 0, -2, 3, 1);
    spawn_asteroid(world, -290.0, -40.0, 2, 1, 0, 2);
    spawn_asteroid(world, -140.0, -230.0, 2, 1, 1, 1);
    spawn_asteroid(world, 140.0, -230.0, -2, 1, 2, 2);
    spawn_asteroid(world, 290.0, 30.0, -2, -1, 3, 3);
    spawn_asteroid(world, 120.0, 230.0, -2, -1, 1, 2);
    spawn_asteroid(world, -120.0, 230.0, 2, -1, 0, 1);
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

fn reset_game_state(game, level, local, now) {
    let cfg = load_game_config(level);
    local["cfg"] = cfg;
    let current_rng = if local.contains("rng") && local["rng"].type_of() == "i64" { local["rng"] } else { 1337 };
    local["rng"] = next_seed(current_rng + now + 101);
    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["base_lives"]);
    game.set("/ast/wave", 1);
    game.set("/ast/shot_cd_ms", 0);
    game.set("/ast/rng", local["rng"]);
    game.set("/ast/msg_ttl_ms", 0);
    game.set("/ast/last_ms", now);
    cfg
}

if !local.contains("init") {
    let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
    clear_all_entities(world);
    reset_game_state(game, level, local, now);
    local["ship_id"] = spawn_ship(world);
    spawn_initial_asteroids(world);
    for ast_id in world.query_kind("asteroid") {
        for i in 0..3 {
            let cv = `asteroid-${ast_id}-crack-${i}`;
            scene.spawn_object("asteroid-template", cv);
            world.bind_visual(ast_id, cv);
            world.set(ast_id, `/crack_visual_${i}`, cv);
        }
    }
    audio.play_song("asteroids.game-loop");
    local["init"] = true;
    diag.info("asteroids session init");
}

let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
let min_x = -320.0;
let max_x = 320.0;
let min_y = -240.0;
let max_y = 240.0;
let ship_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
let last_ms = game.get("/ast/last_ms");
let dt_raw = now - last_ms;
let dt = if dt_raw < 0 { 0 } else if dt_raw > 220 { 220 } else { dt_raw };
game.set("/ast/last_ms", now);

if key.pressed == true {
    let code = if key.code.type_of() == "string" { key.code } else { "" };
    if code == "Esc" {
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
        reset_game_state(game, level, local, now);
        local["ship_id"] = spawn_ship(world);
        spawn_initial_asteroids(world);
        for ast_id in world.query_kind("asteroid") {
            for i in 0..3 {
                let cv = `asteroid-${ast_id}-crack-${i}`;
                scene.spawn_object("asteroid-template", cv);
                world.bind_visual(ast_id, cv);
                world.set(ast_id, `/crack_visual_${i}`, cv);
            }
        }
        game.set("/ast/msg_ttl_ms", 500);
        scene.set("game-message", "text.content", reset_label);
        diag.warn(`reset mode=${reset_label}`);
    }
}

let key_code = if key.code.type_of() == "string" { key.code } else { "" };
let key_pressed = key.pressed == true;
let left_down = input.down("Left") || input.down("a") || (key_pressed && (key_code == "Left" || key_code == "a" || key_code == "A"));
let right_down = input.down("Right") || input.down("d") || (key_pressed && (key_code == "Right" || key_code == "d" || key_code == "D"));
let thrust_down = input.down("Up") || input.down("w") || (key_pressed && (key_code == "Up" || key_code == "w" || key_code == "W"));
let fire_down = input.down(" ") || input.down("f") || (key_pressed && (key_code == " " || key_code == "f" || key_code == "F"));

if world.exists(ship_id) {
    let ship_ref = world.entity(ship_id);
    let ship_h = ship_ref.get_i("/h", 0);
    let invuln_ms = ship_ref.get_i("/invuln_ms", 0);
    let shot_cd_ms = to_i(game.get("/ast/shot_cd_ms"));
    let cfg = if local.contains("cfg") { local["cfg"] } else { load_game_config(level) };
    let turn_accum = if local.contains("turn_accum") { local["turn_accum"] } else { 0 };
    let smoke_cd = if local.contains("smoke_cd") { local["smoke_cd"] } else { 0 };

    if left_down != right_down {
        turn_accum += dt;
        while turn_accum >= cfg["turn_step_ms"] {
            if left_down {
                ship_h = (ship_h + 31) % 32;
            } else {
                ship_h = (ship_h + 1) % 32;
            }
            turn_accum -= cfg["turn_step_ms"];
        }
    } else {
        turn_accum = 0;
    }
    local["turn_accum"] = turn_accum;

    let t = ship_ref.transform();
    let tx = t.x;
    let ty = t.y;
    let phys = ship_ref.physics();
    let vx = phys.vx;
    let vy = phys.vy;

    // Periodic ship state diagnostic (every ~2 seconds)
    let diag_tick = to_i(now / 2000);
    let last_diag = if local.contains("diag_tick") { local["diag_tick"] } else { -1 };
    if diag_tick != last_diag {
        local["diag_tick"] = diag_tick;
        diag.info(`ship pos=(${tx},${ty}) vel=(${vx},${vy}) h=${ship_h} invuln=${invuln_ms}`);
    }
    
    if thrust_down {
        let fx = to_float(sin32(ship_h)) / 1024.0;
        let fy = -to_float(sin32(ship_h + 8)) / 1024.0;
        // cfg["ship_thrust"] is in velocity-scale units; multiply again to get acceleration/sec²
        // Engine does v += a * dt_sec, so a = thrust * VELOCITY_SCALE gives same dv/frame as old code
        let thrust = cfg["ship_thrust"] * h::velocity_scale();
        ship_ref.set_acceleration(fx * thrust, fy * thrust);
        // Log thrust on first application
        if !local.contains("thrust_log") {
            local["thrust_log"] = true;
            diag.info(`thrust ACTIVE: thrust=${thrust} dir=(${fx},${fy}) cfg_thrust=${cfg["ship_thrust"]}`);
        }
        if smoke_cd <= 0 && world.count_kind("smoke") < 40 {
            spawn_smoke(world, tx - (fx * 6.0), ty - (fy * 6.0),
                       vx / h::velocity_scale() - (fx * 0.35), vy / h::velocity_scale() - (fy * 0.35), 520, 3);
            smoke_cd = 48;
        }
    } else {
        ship_ref.set_acceleration(0.0, 0.0);
        local.remove("thrust_log");
    }

    if fire_down && shot_cd_ms <= 0 && world.count_kind("bullet") < 8 {
        let fx = to_float(sin32(ship_h)) / 1024.0;
        let fy = -to_float(sin32(ship_h + 8)) / 1024.0;
        let bspeed = cfg["bullet_speed"];
        spawn_bullet(world, tx + (fx * 9.0), ty + (fy * 9.0),
                    vx / h::velocity_scale() + (fx * bspeed / h::velocity_scale()),
                    vy / h::velocity_scale() + (fy * bspeed / h::velocity_scale()),
                    cfg["bullet_ttl_ms"]);
        shot_cd_ms = cfg["shot_cooldown_ms"];
        audio.event("gameplay.ship.shoot", 1.0);
    }

    let new_x = wrap(tx, min_x, max_x);
    let new_y = wrap(ty, min_y, max_y);
    ship_ref.set_position(new_x, new_y);
    ship_ref.set_heading(h::heading32_to_rad(ship_h));
    ship_ref.set("/h", ship_h);

    if shot_cd_ms > 0 { shot_cd_ms -= dt; }
    if smoke_cd > 0 { smoke_cd -= dt; }
    if invuln_ms > 0 { invuln_ms -= dt; }

    game.set("/ast/shot_cd_ms", shot_cd_ms);
    local["smoke_cd"] = smoke_cd;
    ship_ref.set("/invuln_ms", invuln_ms);
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
    let split_pending = ast_ref.get_b("/split_pending", false);
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
                let rng = game.get("/ast/rng");
                rng = next_seed(rng + now + (ast_id * 41) + to_i(t.x) + to_i(t.y) + frag_idx * 67);
                game.set("/ast/rng", rng);
                let split_heading = h::wrap_heading32(base_heading + h::fragment_heading_offset(frag_idx));
                let split_sin = sin32(split_heading);
                let split_cos = sin32(split_heading + 8);
                let dir_x = sign_i(split_sin, if frag_idx == 1 { 1 } else { -1 });
                let dir_y = sign_i(-split_cos, if frag_idx == 0 { -1 } else { 1 });
                let spawn_offset = asteroid_radius(next_size) + 4;
                spawn_asteroid(world,
                    wrap(t.x + (to_float(split_sin * spawn_offset) / 1024.0), min_x, max_x),
                    wrap(t.y - (to_float(split_cos * spawn_offset) / 1024.0), min_y, max_y),
                    to_float(dir_x * (next_size + 2)),
                    to_float(dir_y * (next_size + 2)),
                    (rng / 17) % 4,
                    next_size
                );
            }
            ast_ref.despawn();
            audio.event("gameplay.asteroid.split", 1.0);
            for child_id in world.query_kind("asteroid") {
                if !world.exists(child_id) { continue; }
                // crack_visual_0 is a string key — use data().contains() not get_b()
                let cd = world.entity(child_id).data();
                if cd.contains("crack_visual_0") { continue; }
                for i in 0..3 {
                    let cv = `asteroid-${child_id}-crack-${i}`;
                    scene.spawn_object("asteroid-template", cv);
                    world.bind_visual(child_id, cv);
                    world.set(child_id, `/crack_visual_${i}`, cv);
                }
            }
            continue;
        }
    }

    let t = ast_ref.transform();
    let new_x = wrap(t.x, min_x, max_x);
    let new_y = wrap(t.y, min_y, max_y);
    ast_ref.set_position(new_x, new_y);

    let cd = ast_ref.get_i("/collide_cd_ms", 0);
    if cd > 0 { cd -= dt; }

    ast_ref.set_many(#{
        flash_ms: flash_ms, flash_total_ms: if flash_ms <= 0 { 0 } else { flash_total },
        split_pending: split_pending, rot_phase: rot_phase, rot_accum_ms: rot_accum, collide_cd_ms: cd
    });
}

for bid in world.query_kind("bullet") {
    let bref = world.entity(bid);
    if !bref.exists() { continue; }
    let bt = bref.transform();
    bref.set_position(wrap(bt.x, min_x, max_x), wrap(bt.y, min_y, max_y));
}

let score = to_i(game.get("/ast/score"));
let lives = to_i(game.get("/ast/lives"));
let ship_ref = world.entity(ship_id);
let ship_invuln = if ship_ref.exists() { ship_ref.get_i("/invuln_ms", 0) } else { 0 };

for hit in world.collisions_between("ship", "asteroid") {
    let ast_hit_id = hit["asteroid"];
    let hit_ref = world.entity(ast_hit_id);
    
    // Skip if asteroid is already being split (avoid double-hits)
    if hit_ref.exists() && hit_ref.get_b("/split_pending", false) { continue; }
    
    if ship_invuln <= 0 && world.exists(ship_id) {
        let t = ship_ref.transform();
        spawn_disintegration(world, t.x, t.y);
        audio.event("gameplay.ship.hit", 1.0);

        if hit_ref.exists() {
            let phys = hit_ref.physics();
            hit_ref.set_many(#{
                flash_ms: h::crack_duration_ms(), flash_total_ms: h::crack_duration_ms(), split_pending: true
            });
            hit_ref.set_velocity(clamp_f(-phys.vx, -4.0 * h::velocity_scale(), 4.0 * h::velocity_scale()),
                                clamp_f(-phys.vy, -4.0 * h::velocity_scale(), 4.0 * h::velocity_scale()));
            hit_ref.set("/collide_cd_ms", 180);
        }
        lives -= 1;
        ship_ref.set_velocity(0.0, 0.0);
        ship_ref.set("/invuln_ms", 3000);
        game.set("/ast/msg_ttl_ms", 450);
        scene.set("game-message", "text.content", "HIT!");
    }
}

for hit in world.collisions_between("bullet", "asteroid") {
    let bullet_id = hit["bullet"];
    let ast_hit_id = hit["asteroid"];
    let hit_ref = world.entity(ast_hit_id);
    
    // Skip if asteroid is already being split (avoid double-hits)
    if hit_ref.exists() && hit_ref.get_b("/split_pending", false) { continue; }
    
    if world.exists(bullet_id) {
        world.entity(bullet_id).despawn();
    }
    if hit_ref.exists() {
        score += asteroid_score(hit_ref.get_i("/size", 0));
        hit_ref.set_many(#{
            flash_ms: h::crack_duration_ms(), flash_total_ms: h::crack_duration_ms(), split_pending: true
        });
        game.set("/ast/msg_ttl_ms", 250);
        scene.set("game-message", "text.content", "HIT");
        diag.info(`hit asteroid=${ast_hit_id}`);
    }
}

game.set("/ast/score", score);
game.set("/ast/lives", lives);

let msg_ttl = to_i(game.get("/ast/msg_ttl_ms"));
if msg_ttl > 0 {
    let new_ttl = msg_ttl - dt;
    if new_ttl <= 0 { scene.set("game-message", "text.content", ""); }
    game.set("/ast/msg_ttl_ms", new_ttl);
}

let active_count = world.count_kind("asteroid");

if active_count <= 0 {
    let wave = to_i(game.get("/ast/wave"));
    wave += 1;
    game.set("/ast/msg_ttl_ms", 650);
    game.set("/ast/wave", wave);
    scene.set("game-message", "text.content", `WAVE ${wave}`);
    diag.info(`wave start wave=${wave}`);

    let cfg = if local.contains("cfg") { local["cfg"] } else { load_game_config(level) };
    let spawn_count = clamp_i(cfg["wave_base_spawn_count"] + wave, cfg["wave_min_spawn_count"], cfg["wave_max_spawn_count"]);

    let ship_t = if ship_ref.exists() { ship_ref.transform() } else { #{x: 0, y: 0} };
    for idx in 0..spawn_count {
        let rng = game.get("/ast/rng");
        rng = next_seed(rng + now + idx * 53 + wave);
        game.set("/ast/rng", rng);
        spawn_asteroid(world,
            respawn_x(rng, min_x, max_x, ship_t.x),
            respawn_y(rng / 3, min_y, max_y, ship_t.y),
            speed_for_seed(rng + 5),
            speed_for_seed(rng + 13),
            (rng / 11) % 4,
            if idx < 2 { 3 } else if idx < 5 { 2 } else { 1 }
        );
    }
    for new_ast_id in world.query_kind("asteroid") {
        if !world.exists(new_ast_id) { continue; }
        for i in 0..3 {
            let cv = `asteroid-${new_ast_id}-crack-${i}`;
            if !scene.get(cv).exists() {
                scene.spawn_object("asteroid-template", cv);
                world.bind_visual(new_ast_id, cv);
                world.set(new_ast_id, `/crack_visual_${i}`, cv);
            }
        }
    }
}

if lives <= 0 {
    diag.warn(`game over score=${score} wave=${to_i(game.get("/ast/wave"))}`);
    persist.push("/asteroids/highscores", #{ score: score, wave: to_i(game.get("/ast/wave")), at_ms: now });
    game.jump("asteroids.highscores");
    return ();
}
