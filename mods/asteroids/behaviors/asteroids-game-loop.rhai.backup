fn crack_duration_ms() {
    1000
}

fn fragment_heading_offset(idx) {
    if idx == 0 { return 0; }
    if idx == 1 { return 11; }
    21
}

fn level_i(level_api, path, fallback) {
    if level_api.has(path) {
        let value = level_api.get(path);
        if value.type_of() == "i64" {
            return value;
        }
    }
    fallback
}

fn next_seed(seed) {
    ((seed * 1103515245) + 12345) % 2147483647
}

fn heading32_to_rad(h) {
    to_float(h) * (6.283185307179586 / 32.0)
}

fn sync_transform(world, id, x_fp, y_fp, heading32) {
    let px = to_float(x_fp) / 1024.0;
    let py = to_float(y_fp) / 1024.0;
    world.set_transform(id, px, py, heading32_to_rad(heading32));
}

fn sync_transform_xy(world, id, x, y, heading32) {
    world.set_transform(id, to_float(x), to_float(y), heading32_to_rad(heading32));
}

fn sync_physics(world, id, vx_fp, vy_fp) {
    world.set_physics(id, to_float(vx_fp) / 1024.0, to_float(vy_fp) / 1024.0, 0.0, 0.0, 0.0, 0.0);
}

fn set_collider_for_asteroid(world, id, size) {
    world.set_collider_circle(id, to_float(asteroid_radius(size)), 0xFFFF, 0xFFFF);
}

fn set_collider_for_bullet(world, id) {
    world.set_collider_circle(id, 3.0, 0xFFFF, 0xFFFF);
}

fn set_collider_for_ship(world, id) {
    world.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF);
}

fn set_collider_for_smoke(world, id) {
    world.set_collider_circle(id, 2.0, 0xFFFF, 0xFFFF);
}

fn speed_for_seed(seed) {
    let v = (seed % 3) + 1;
    if ((seed / 7) % 2) == 0 { v } else { -v }
}

fn respawn_x(seed, min_x, max_x, ship_x) {
    let x = if (seed % 2) == 0 { min_x + 10 } else { max_x - 10 };
    if abs_i(x - ship_x) < 90 {
        x = if x < 0 { max_x - 10 } else { min_x + 10 };
    }
    x
}

fn respawn_y(seed, min_y, max_y, ship_y) {
    let span = (max_y - min_y) - 20;
    let y = min_y + 10 + (seed % span);
    if abs_i(y - ship_y) < 70 {
        y = y + 76;
        if y > max_y - 10 {
            y = min_y + 10;
        }
    }
    y
}

fn spawn_ship_entity(world) {
    let id = world.spawn_object("ship", #{
        tags: ["ship"],
        x_fp: 0,
        y_fp: 0,
        vx_fp: 0,
        vy_fp: 0,
        h: 0,
        invuln_ms: 3000
    });
    if id > 0 {
        sync_transform(world, id, 0, 0, 0);
        sync_physics(world, id, 0, 0);
        set_collider_for_ship(world, id);
    }
    id
}

fn spawn_session_entity(
    world,
    base_lives,
    shot_cooldown_ms,
    ship_thrust_fp,
    ship_max_speed_fp,
    turn_step_ms,
    bullet_ttl_ms,
    bullet_speed_fp,
    hit_flash_ms,
    wave_base_spawn_count,
    wave_min_spawn_count,
    wave_max_spawn_count,
    rng_seed,
    msg_ttl_ms,
    last_diag_ms,
    last_ms,
    sim_accum
) {
    world.spawn_object("session", #{
        tags: ["session"],
        base_lives: base_lives,
        lives: base_lives,
        wave: 1,
        score: 0,
        shot_cd_ms: 0,
        shot_cooldown_ms: shot_cooldown_ms,
        ship_thrust_fp: ship_thrust_fp,
        ship_max_speed_fp: ship_max_speed_fp,
        turn_step_ms: turn_step_ms,
        turn_accum_ms: 0,
        bullet_ttl_ms: bullet_ttl_ms,
        bullet_speed_fp: bullet_speed_fp,
        hit_flash_ms: hit_flash_ms,
        ship_h: 0,
        ship_x_fp: 0,
        ship_y_fp: 0,
        vx_fp: 0,
        vy_fp: 0,
        ship_invuln_ms: 3000,
        ship_respawn_ms: 0,
        smoke_cd_ms: 0,
        wave_base_spawn_count: wave_base_spawn_count,
        wave_min_spawn_count: wave_min_spawn_count,
        wave_max_spawn_count: wave_max_spawn_count,
        msg_ttl_ms: msg_ttl_ms,
        last_diag_ms: last_diag_ms,
        rng: rng_seed,
        last_ms: last_ms,
        sim_accum: sim_accum
    })
}

fn spawn_asteroid_entity(world, scene, x, y, dx, dy, shape, size) {
    let spin_seed = abs_i((x * 31) + (y * 17) + (dx * 13) + (dy * 19) + (shape * 23) + (size * 29));
    let entity_id = world.spawn_object("asteroid", #{
        tags: ["asteroid"],
        active: true,
        x: x,
        y: y,
        dx: dx,
        dy: dy,
        shape: shape,
        size: size,
        flash_ms: 0,
        flash_total_ms: 0,
        split_pending: false,
        collide_cd_ms: 0,
        rot_phase: spin_seed % 32,
        rot_speed: if (spin_seed % 2) == 0 { 1 } else { -1 },
        rot_step_ms: 72 + ((spin_seed % 3) * 16),
        rot_accum_ms: 0
    });
    if entity_id > 0 {
        let visual_id = `asteroid-${entity_id}`;
        let crack_visual_0 = `asteroid-${entity_id}-crack-0`;
        let crack_visual_1 = `asteroid-${entity_id}-crack-1`;
        let crack_visual_2 = `asteroid-${entity_id}-crack-2`;
        scene.spawn_object("asteroid-template", visual_id);
        scene.spawn_object("asteroid-template", crack_visual_0);
        scene.spawn_object("asteroid-template", crack_visual_1);
        scene.spawn_object("asteroid-template", crack_visual_2);
        world.set(entity_id, "/visual_id", visual_id);
        world.set(entity_id, "/crack_visual_0", crack_visual_0);
        world.set(entity_id, "/crack_visual_1", crack_visual_1);
        world.set(entity_id, "/crack_visual_2", crack_visual_2);
        world.set_visual(entity_id, visual_id);
        sync_transform_xy(world, entity_id, x, y, 0);
        sync_physics(world, entity_id, dx * 64, dy * 64);
        set_collider_for_asteroid(world, entity_id, size);
    }
    entity_id
}

fn spawn_bullet_entity(world, scene, x_fp, y_fp, dx_fp, dy_fp, ttl_ms) {
    let entity_id = world.spawn_object("bullet", #{
        tags: ["bullet"],
        active: true,
        x_fp: x_fp,
        y_fp: y_fp,
        dx_fp: dx_fp,
        dy_fp: dy_fp,
        ttl_ms: ttl_ms
    });
    if entity_id > 0 {
        let visual_id = `bullet-${entity_id}`;
        scene.spawn_object("bullet-template", visual_id);
        world.set(entity_id, "/visual_id", visual_id);
        world.set_visual(entity_id, visual_id);
        sync_transform(world, entity_id, x_fp, y_fp, 0);
        sync_physics(world, entity_id, dx_fp, dy_fp);
        set_collider_for_bullet(world, entity_id);
        world.set_lifetime(entity_id, ttl_ms);
    }
    entity_id
}

fn spawn_smoke_entity(world, scene, x_fp, y_fp, dx_fp, dy_fp, ttl_ms, radius) {
    let entity_id = world.spawn_object("smoke", #{
        tags: ["smoke"],
        active: true,
        x_fp: x_fp,
        y_fp: y_fp,
        dx_fp: dx_fp,
        dy_fp: dy_fp,
        ttl_ms: ttl_ms,
        max_ttl_ms: ttl_ms,
        radius: radius
    });
    if entity_id > 0 {
        let visual_id = `smoke-${entity_id}`;
        scene.spawn_object("smoke-template", visual_id);
        world.set(entity_id, "/visual_id", visual_id);
        world.set_visual(entity_id, visual_id);
        sync_transform(world, entity_id, x_fp, y_fp, 0);
        sync_physics(world, entity_id, dx_fp, dy_fp);
        set_collider_for_smoke(world, entity_id);
        world.set_lifetime(entity_id, ttl_ms);
    }
    entity_id
}

fn spawn_ship_disintegration(world, scene, x_fp, y_fp) {
    for idx in 0..12 {
        let ang = (idx * 3) % 32;
        let dx = (sin32(ang) * 900) / 1024;
        let dy = (-sin32(ang + 8) * 900) / 1024;
        spawn_smoke_entity(
            world,
            scene,
            x_fp,
            y_fp,
            dx,
            dy,
            700,
            3
        );
    }
}

fn maybe_despawn_visual(scene, visual_id) {
    if visual_id.type_of() == "string" {
        scene.despawn_object(visual_id);
    }
}

fn despawn_entity_visual(world, scene, entity_id) {
    maybe_despawn_visual(scene, world.get(entity_id, "/visual_id"));
    maybe_despawn_visual(scene, world.get(entity_id, "/crack_visual_0"));
    maybe_despawn_visual(scene, world.get(entity_id, "/crack_visual_1"));
    maybe_despawn_visual(scene, world.get(entity_id, "/crack_visual_2"));
}

fn load_game_config(level) {
    let base_lives = level_i(level, "/player/lives", 3);
    let shot_cooldown_ms = clamp_i(level_i(level, "/player/shot_cooldown_ms", 200), 200, 1000);
    let ship_thrust_fp = clamp_i(level_i(level, "/player/thrust_fp", 170), 60, 600);
    let ship_max_speed_fp = clamp_i(level_i(level, "/player/max_speed_fp", 4608), 1024, 14000);
    let turn_step_ms = clamp_i(level_i(level, "/player/turn_step_ms", 40), 16, 150);
    let wave_base_spawn_count = clamp_i(level_i(level, "/waves/base_spawn_count", 4), 1, 20);
    let wave_min_spawn_count = clamp_i(level_i(level, "/waves/min_spawn_count", 5), 1, 30);
    let wave_max_spawn_count = clamp_i(level_i(level, "/waves/max_spawn_count", 10), 1, 40);
    let wave_min_spawn_count = if wave_min_spawn_count > wave_max_spawn_count { wave_max_spawn_count } else { wave_min_spawn_count };
    let bullet_ttl_ms = clamp_i(level_i(level, "/combat/bullet_ttl_ms", 900), 100, 6000);
    let bullet_speed_fp = clamp_i(level_i(level, "/combat/bullet_speed_fp", 4200), 1000, 18000);
    let hit_flash_ms = clamp_i(level_i(level, "/combat/hit_flash_ms", 150), 40, 800);
    #{
        base_lives: base_lives,
        shot_cooldown_ms: shot_cooldown_ms,
        ship_thrust_fp: ship_thrust_fp,
        ship_max_speed_fp: ship_max_speed_fp,
        turn_step_ms: turn_step_ms,
        wave_base_spawn_count: wave_base_spawn_count,
        wave_min_spawn_count: wave_min_spawn_count,
        wave_max_spawn_count: wave_max_spawn_count,
        bullet_ttl_ms: bullet_ttl_ms,
        bullet_speed_fp: bullet_speed_fp,
        hit_flash_ms: hit_flash_ms
    }
}

fn spawn_initial_asteroids(world, scene) {
    // Keep initial asteroids near borders with mostly tangential movement
    // to avoid instant center collisions on scene start.
    spawn_asteroid_entity(world, scene, -300, -210, 2, 0, 0, 2);
    spawn_asteroid_entity(world, scene, 300, -210, 0, 2, 1, 3);
    spawn_asteroid_entity(world, scene, 300, 210, -2, 0, 2, 2);
    spawn_asteroid_entity(world, scene, -300, 210, 0, -2, 3, 1);
    spawn_asteroid_entity(world, scene, -290, -40, 2, 1, 0, 2);
    spawn_asteroid_entity(world, scene, -140, -230, 2, 1, 1, 1);
    spawn_asteroid_entity(world, scene, 140, -230, -2, 1, 2, 2);
    spawn_asteroid_entity(world, scene, 290, 30, -2, -1, 3, 3);
    spawn_asteroid_entity(world, scene, 120, 230, -2, -1, 1, 2);
    spawn_asteroid_entity(world, scene, -120, 230, 2, -1, 0, 1);
}

if local.contains("init") == false {
    let init_now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
    let cfg = load_game_config(level);
    let clear_ids = world.ids();
    for entity_id in clear_ids {
        despawn_entity_visual(world, scene, entity_id);
    }
    world.clear();
    local["rng"] = next_seed(1337 + init_now + 101);
    local["session_id"] = spawn_session_entity(
        world,
        cfg["base_lives"],
        cfg["shot_cooldown_ms"],
        cfg["ship_thrust_fp"],
        cfg["ship_max_speed_fp"],
        cfg["turn_step_ms"],
        cfg["bullet_ttl_ms"],
        cfg["bullet_speed_fp"],
        cfg["hit_flash_ms"],
        cfg["wave_base_spawn_count"],
        cfg["wave_min_spawn_count"],
        cfg["wave_max_spawn_count"],
        local["rng"],
        0,
        0,
        init_now,
        0
    );
    local["ship_id"] = spawn_ship_entity(world);
    spawn_initial_asteroids(world, scene);
    audio.play_song("asteroids.game-loop");
    local["init"] = true;
    diag.info("asteroids session init visuals=runtime-instanced bullets=dynamic asteroids=dynamic");
}

let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
let scale = 1024;
let min_x = -320;
let max_x = 320;
let min_y = -240;
let max_y = 240;
let ship_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
let session_id = if local.contains("session_id") { local["session_id"] } else { 0 };
let session_ref = world.entity(session_id);
let last_ms = if session_ref.exists() { session_ref.get_i("/last_ms", now) } else { now };
let dt_raw = now - last_ms;
let dt = if dt_raw < 0 { 0 } else if dt_raw > 220 { 220 } else { dt_raw };
if session_ref.exists() {
    session_ref.set("/last_ms", now);
}

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
        if level.select("asteroids.easy") {
            reset_requested = true;
            reset_label = "EASY";
            diag.info("level selected asteroids.easy");
        } else {
            diag.warn("level select failed asteroids.easy");
        }
    } else if code == "F6" {
        if level.select("asteroids.default") {
            reset_requested = true;
            reset_label = "DEFAULT";
            diag.info("level selected asteroids.default");
        } else {
            diag.warn("level select failed asteroids.default");
        }
    } else if code == "F7" {
        if level.select("asteroids.hard") {
            reset_requested = true;
            reset_label = "HARD";
            diag.info("level selected asteroids.hard");
        } else {
            diag.warn("level select failed asteroids.hard");
        }
    }

    if reset_requested {
        let current_rng =
            if local.contains("rng") && local["rng"].type_of() == "i64" {
                local["rng"]
            } else {
                1337
            };
        let cfg = load_game_config(level);
        local["rng"] = next_seed(current_rng + now + 101);
        let clear_ids = world.ids();
        for entity_id in clear_ids {
            despawn_entity_visual(world, scene, entity_id);
        }
        world.clear();
        local["session_id"] = spawn_session_entity(
            world,
            cfg["base_lives"],
            cfg["shot_cooldown_ms"],
            cfg["ship_thrust_fp"],
            cfg["ship_max_speed_fp"],
            cfg["turn_step_ms"],
            cfg["bullet_ttl_ms"],
            cfg["bullet_speed_fp"],
            cfg["hit_flash_ms"],
            cfg["wave_base_spawn_count"],
            cfg["wave_min_spawn_count"],
            cfg["wave_max_spawn_count"],
            local["rng"],
            0,
            0,
            now,
            0
        );
        local["ship_id"] = spawn_ship_entity(world);
        spawn_initial_asteroids(world, scene);
        let session = world.get(local["session_id"], "/");
        if session.type_of() == "map" {
            session["msg_ttl_ms"] = 500;
            world.set(local["session_id"], "/", session);
        }
        scene.set("game-message", "text.content", reset_label);
        diag.warn(`reset requested mode=${reset_label}`);
    }
}

session_ref = world.entity(session_id);

let step_ms = 16;
let sim_accum = if session_ref.exists() { session_ref.get_i("/sim_accum", 0) } else { 0 };
let accum = sim_accum + dt;
let steps = accum / step_ms;
if steps > 8 {
    steps = 8;
    accum = steps * step_ms;
}
if session_ref.exists() {
    session_ref.set("/sim_accum", accum - (steps * step_ms));
}

let key_code = if key.code.type_of() == "string" { key.code } else { "" };
let key_pressed = key.pressed == true;
let left_event = key_pressed && (key_code == "Left" || key_code == "a" || key_code == "A");
let right_event = key_pressed && (key_code == "Right" || key_code == "d" || key_code == "D");
let thrust_event = key_pressed && (key_code == "Up" || key_code == "w" || key_code == "W");
let fire_event = key_pressed && (key_code == " " || key_code == "f" || key_code == "F");

let left_down = input.down("Left") || input.down("a") || left_event;
let right_down = input.down("Right") || input.down("d") || right_event;
let thrust_down = input.down("Up") || input.down("w") || thrust_event;
let fire_down = input.down(" ") || input.down("f") || fire_event;

for _s in 0..steps {
    let session = world.get(session_id, "/");
    let session_ref = world.entity(session_id);
    if session.type_of() != "map" || !session_ref.exists() {
        continue;
    }

    let ship = world.get(ship_id, "/");
    if ship.type_of() != "map" {
        continue;
    }

    let score = session_ref.get_i("/score", 0);
    let lives = session_ref.get_i("/lives", 0);
    let wave = session_ref.get_i("/wave", 1);
    let shot_cd_ms = session_ref.get_i("/shot_cd_ms", 0);
    let ship_h = session_ref.get_i("/ship_h", 0);
    let ship_x_fp = session_ref.get_i("/ship_x_fp", 0);
    let ship_y_fp = session_ref.get_i("/ship_y_fp", 0);
    let vx_fp = session_ref.get_i("/vx_fp", 0);
    let vy_fp = session_ref.get_i("/vy_fp", 0);
    let ship_invuln_ms = session_ref.get_i("/ship_invuln_ms", 0);
    let session_ship_thrust_fp = session_ref.get_i("/ship_thrust_fp", 170);
    let session_max_speed_fp = session_ref.get_i("/ship_max_speed_fp", 4608);
    let session_turn_step_ms = session_ref.get_i("/turn_step_ms", 40);
    let turn_accum_ms = session_ref.get_i("/turn_accum_ms", 0);
    let session_bullet_ttl_ms = session_ref.get_i("/bullet_ttl_ms", 900);
    let session_bullet_speed_fp = session_ref.get_i("/bullet_speed_fp", 4200);
    let session_hit_flash_ms = session_ref.get_i("/hit_flash_ms", 150);
    let ship_respawn_ms = session_ref.get_i("/ship_respawn_ms", 0);
    let smoke_emit_interval_ms = 48;
    let smoke_ttl_ms = 520;
    let smoke_drift_fp = 360;
    let smoke_max_count = 40;
    let smoke_radius = 3;
    let smoke_cd_ms = session_ref.get_i("/smoke_cd_ms", 0);

    if ship_respawn_ms > 0 {
        ship_respawn_ms = ship_respawn_ms - step_ms;
        if ship_respawn_ms <= 0 {
            ship_respawn_ms = 0;
            ship_x_fp = 0;
            ship_y_fp = 0;
            vx_fp = 0;
            vy_fp = 0;
            ship_h = 0;
            ship_invuln_ms = 3000;
            session["msg_ttl_ms"] = 450;
            scene.set("game-message", "text.content", "REBOOT");
        }
    } else {
        if left_down != right_down {
            turn_accum_ms = turn_accum_ms + step_ms;
            while turn_accum_ms >= session_turn_step_ms {
                if left_down {
                    ship_h = (ship_h + 31) % 32;
                } else {
                    ship_h = (ship_h + 1) % 32;
                }
                turn_accum_ms = turn_accum_ms - session_turn_step_ms;
            }
        } else {
            turn_accum_ms = 0;
        }

        if thrust_down {
            let fx = sin32(ship_h);
            let fy = -sin32(ship_h + 8);
            vx_fp = clamp_i(vx_fp + ((fx * session_ship_thrust_fp) / 1024), -session_max_speed_fp, session_max_speed_fp);
            vy_fp = clamp_i(vy_fp + ((fy * session_ship_thrust_fp) / 1024), -session_max_speed_fp, session_max_speed_fp);
            if smoke_cd_ms <= 0 && world.count_kind("smoke") < smoke_max_count {
                spawn_smoke_entity(
                    world,
                    scene,
                    ship_x_fp - (((fx * 6) / 1024) * scale),
                    ship_y_fp - (((fy * 6) / 1024) * scale),
                    vx_fp - ((fx * smoke_drift_fp) / 1024),
                    vy_fp - ((fy * smoke_drift_fp) / 1024),
                    smoke_ttl_ms,
                    smoke_radius
                );
                smoke_cd_ms = smoke_emit_interval_ms;
            }
        }

        if fire_down && (shot_cd_ms <= 0) {
            let fx = sin32(ship_h);
            let fy = -sin32(ship_h + 8);
            if world.count_kind("bullet") < 8 {
                spawn_bullet_entity(
                    world,
                    scene,
                    ship_x_fp + ((fx * 9) / 1024) * scale,
                    ship_y_fp + ((fy * 9) / 1024) * scale,
                    vx_fp + ((fx * session_bullet_speed_fp) / 1024),
                    vy_fp + ((fy * session_bullet_speed_fp) / 1024),
                    session_bullet_ttl_ms
                );
                shot_cd_ms = session["shot_cooldown_ms"];
                audio.event("gameplay.ship.shoot", 1.0);
            }
        }
    }

    ship_x_fp = wrap_fp(ship_x_fp + vx_fp, min_x, max_x, scale);
    ship_y_fp = wrap_fp(ship_y_fp + vy_fp, min_y, max_y, scale);
    let ship_x = ship_x_fp / scale;
    let ship_y = ship_y_fp / scale;
    sync_transform(world, ship_id, ship_x_fp, ship_y_fp, ship_h);
    sync_physics(world, ship_id, vx_fp, vy_fp);
    world.set(ship_id, "/", #{
        h: ship_h,
        invuln_ms: ship_invuln_ms
    });

    let asteroid_ids = world.query_kind("asteroid");
    for ast_id in asteroid_ids {
        let ast = world.get(ast_id, "/");
        if ast["active"] == false {
            continue;
        }

        ast["rot_accum_ms"] = ast["rot_accum_ms"] + step_ms;
        while ast["rot_accum_ms"] >= ast["rot_step_ms"] {
            ast["rot_phase"] = wrap_heading32(ast["rot_phase"] + ast["rot_speed"]);
            ast["rot_accum_ms"] = ast["rot_accum_ms"] - ast["rot_step_ms"];
        }

        if ast["flash_ms"] > 0 {
            ast["flash_ms"] = ast["flash_ms"] - step_ms;
            if ast["flash_ms"] <= 0 {
                ast["flash_ms"] = 0;
                ast["flash_total_ms"] = 0;
                if ast["split_pending"] {
                    ast["split_pending"] = false;
                    if ast["size"] <= 0 {
                        despawn_entity_visual(world, scene, ast_id);
                        world.despawn_object(ast_id);
                        diag.info(`asteroid removed id=${ast_id}`);
                        continue;
                    } else {
                        let next_size = ast["size"] - 1;
                        let base_heading = ast["rot_phase"];
                        for frag_idx in 0..3 {
                            session["rng"] =
                                next_seed(session["rng"] + now + (ast_id * 41) + ast["x"] + ast["y"] + frag_idx * 67);
                            let seed = session["rng"];
                            let split_heading = wrap_heading32(base_heading + fragment_heading_offset(frag_idx));
                            let split_sin = sin32(split_heading);
                            let split_cos = sin32(split_heading + 8);
                            let dir_x = sign_i(split_sin, if frag_idx == 1 { 1 } else { -1 });
                            let dir_y = sign_i(-split_cos, if frag_idx == 0 { -1 } else { 1 });
                            let spawn_offset = asteroid_radius(next_size) + 4;
                            let child_id = spawn_asteroid_entity(
                                world,
                                scene,
                                wrap(ast["x"] + ((split_sin * spawn_offset) / 1024), min_x, max_x),
                                wrap(ast["y"] - ((split_cos * spawn_offset) / 1024), min_y, max_y),
                                clamp_i(ast["dx"] + (dir_x * (next_size + 2)), -5, 5),
                                clamp_i(ast["dy"] + (dir_y * (next_size + 2)), -5, 5),
                                (seed / 17) % 4,
                                next_size
                            );
                            if child_id > 0 {
                                world.set(child_id, "/collide_cd_ms", 180);
                            }
                        }
                        despawn_entity_visual(world, scene, ast_id);
                        world.despawn_object(ast_id);
                        audio.event("gameplay.asteroid.split", 1.0);
                        diag.info(`asteroid split id=${ast_id} pieces=3 size=${next_size}`);
                        continue;
                    }
                }
            }
        } else {
            ast["x"] = wrap(ast["x"] + ast["dx"], min_x, max_x);
            ast["y"] = wrap(ast["y"] + ast["dy"], min_y, max_y);
        }
        sync_transform_xy(world, ast_id, ast["x"], ast["y"], ast["rot_phase"]);
        sync_physics(world, ast_id, ast["dx"] * 64, ast["dy"] * 64);
        world.set(ast_id, "/", #{
            active: ast["active"],
            x: ast["x"],
            y: ast["y"],
            dx: ast["dx"],
            dy: ast["dy"],
            shape: ast["shape"],
            size: ast["size"],
            visual_id: if ast.contains("visual_id") { ast["visual_id"] } else { () },
            crack_visual_0: if ast.contains("crack_visual_0") { ast["crack_visual_0"] } else { () },
            crack_visual_1: if ast.contains("crack_visual_1") { ast["crack_visual_1"] } else { () },
            crack_visual_2: if ast.contains("crack_visual_2") { ast["crack_visual_2"] } else { () },
            flash_ms: ast["flash_ms"],
            flash_total_ms: ast["flash_total_ms"],
            split_pending: ast["split_pending"],
            collide_cd_ms: if ast.contains("collide_cd_ms") { ast["collide_cd_ms"] } else { 0 },
            rot_phase: ast["rot_phase"],
            rot_speed: ast["rot_speed"],
            rot_step_ms: ast["rot_step_ms"],
            rot_accum_ms: ast["rot_accum_ms"]
        });
    }

    // asteroid-asteroid bounce handled via collisions buffer later (was manual polygon check)

    let bullet_ids = world.query_kind("bullet");
    for bullet_id in bullet_ids {
        let bullet = world.get(bullet_id, "/");
        if bullet["active"] == false {
            continue;
        }

        let bullet_prev_x = bullet["x_fp"] / scale;
        let bullet_prev_y = bullet["y_fp"] / scale;
        let bx = wrap_fp(bullet["x_fp"] + bullet["dx_fp"], min_x, max_x, scale);
        let by = wrap_fp(bullet["y_fp"] + bullet["dy_fp"], min_y, max_y, scale);
        let bullet_x = bx / scale;
        let bullet_y = by / scale;
        let wrapped_this_step = abs_i(bullet_x - bullet_prev_x) > 200 || abs_i(bullet_y - bullet_prev_y) > 160;
        let ttl = bullet["ttl_ms"] - step_ms;
        sync_transform(world, bullet_id, bx, by, 0);
        sync_physics(world, bullet_id, bullet["dx_fp"], bullet["dy_fp"]);
        world.set_lifetime(bullet_id, ttl);
        if ttl <= 0 {
            despawn_entity_visual(world, scene, bullet_id);
            world.despawn_object(bullet_id);
            continue;
        }

        // bullet collision handled via collisions buffer
        if world.exists(bullet_id) {
            world.set(bullet_id, "/", #{
                active: true,
                ttl_ms: ttl,
                x_fp: bx,
                y_fp: by,
                dx_fp: bullet["dx_fp"],
                dy_fp: bullet["dy_fp"],
                visual_id: if bullet.contains("visual_id") { bullet["visual_id"] } else { () }
            });
        }
    }

    let smoke_ids = world.query_kind("smoke");
    for smoke_id in smoke_ids {
        let smoke = world.get(smoke_id, "/");
        if smoke["active"] == false {
            continue;
        }
        let sx = smoke["x_fp"] + smoke["dx_fp"];
        let sy = smoke["y_fp"] + smoke["dy_fp"];
        let sdx = (smoke["dx_fp"] * 96) / 100;
        let sdy = (smoke["dy_fp"] * 96) / 100;
        let ttl = smoke["ttl_ms"] - step_ms;
        sync_transform(world, smoke_id, sx, sy, 0);
        sync_physics(world, smoke_id, sdx, sdy);
        world.set_lifetime(smoke_id, ttl);
        if ttl <= 0 {
            despawn_entity_visual(world, scene, smoke_id);
            world.despawn_object(smoke_id);
            continue;
        }
        world.set(smoke_id, "/", #{
            active: true,
            x_fp: sx,
            y_fp: sy,
            dx_fp: sdx,
            dy_fp: sdy,
            ttl_ms: ttl,
            max_ttl_ms: if smoke.contains("max_ttl_ms") { smoke["max_ttl_ms"] } else { ttl },
            radius: if smoke.contains("radius") { smoke["radius"] } else { 2 },
            visual_id: if smoke.contains("visual_id") { smoke["visual_id"] } else { () }
        });
    }

    if shot_cd_ms > 0 {
        shot_cd_ms = shot_cd_ms - step_ms;
    }
    if smoke_cd_ms > 0 {
        smoke_cd_ms = smoke_cd_ms - step_ms;
    }
    if ship_invuln_ms > 0 {
        ship_invuln_ms = ship_invuln_ms - step_ms;
    }

    // Collision handling via engine collision buffer
    let hits = world.collisions();
    for hit in hits {
        let a_id = if hit.contains("a") { hit["a"] } else { 0 };
        let b_id = if hit.contains("b") { hit["b"] } else { 0 };
        if a_id <= 0 || b_id <= 0 {
            continue;
        }
        let a_kind = world.kind(a_id);
        let b_kind = world.kind(b_id);

        let ship_vs_ast =
            (a_kind == "ship" && b_kind == "asteroid") || (a_kind == "asteroid" && b_kind == "ship");
        let bullet_vs_ast =
            (a_kind == "bullet" && b_kind == "asteroid") || (a_kind == "asteroid" && b_kind == "bullet");

        if ship_vs_ast && ship_invuln_ms <= 0 && ship_respawn_ms <= 0 {
            let ship_id_hit = if a_kind == "ship" { a_id } else { b_id };
            let ast_id_hit = if a_kind == "asteroid" { a_id } else { b_id };
            spawn_ship_disintegration(world, scene, ship_x_fp, ship_y_fp);
            audio.event("gameplay.ship.hit", 1.0);
            let mut_ast = world.get(ast_id_hit, "/");
            if mut_ast.type_of() == "map" {
                mut_ast["flash_ms"] = crack_duration_ms();
                mut_ast["flash_total_ms"] = crack_duration_ms();
                mut_ast["split_pending"] = true;
                mut_ast["dx"] = clamp_i(-mut_ast["dx"], -4, 4);
                mut_ast["dy"] = clamp_i(-mut_ast["dy"], -4, 4);
                mut_ast["collide_cd_ms"] = 180;
                world.set(ast_id_hit, "/", mut_ast);
            }
            lives = lives - 1;
            vx_fp = 0;
            vy_fp = 0;
            ship_respawn_ms = 750;
            ship_invuln_ms = 0;
            session["msg_ttl_ms"] = 450;
            scene.set("game-message", "text.content", "HIT!");
        } else if bullet_vs_ast {
            let bullet_id = if a_kind == "bullet" { a_id } else { b_id };
            let ast_id = if a_kind == "asteroid" { a_id } else { b_id };
            if world.exists(bullet_id) {
                despawn_entity_visual(world, scene, bullet_id);
                world.despawn_object(bullet_id);
            }
            let ast = world.get(ast_id, "/");
            if ast.type_of() == "map" && ast["active"] {
                score = score + asteroid_score(ast["size"]);
                let mut_ast = ast;
                mut_ast["flash_ms"] = crack_duration_ms();
                mut_ast["flash_total_ms"] = crack_duration_ms();
                mut_ast["split_pending"] = true;
                world.set(ast_id, "/", mut_ast);
                session["msg_ttl_ms"] = 250;
                scene.set("game-message", "text.content", "HIT");
                diag.info(`hit asteroid=${ast_id} size=${ast["size"]} score=${score}`);
            }
        }
    }

    // ship collision handled via collisions buffer

    session["score"] = score;
    session["lives"] = lives;
    session["wave"] = wave;
    session["shot_cd_ms"] = shot_cd_ms;
    ship["h"] = ship_h;
    ship["x_fp"] = ship_x_fp;
    ship["y_fp"] = ship_y_fp;
    ship["vx_fp"] = vx_fp;
    ship["vy_fp"] = vy_fp;
    ship["invuln_ms"] = ship_invuln_ms;
    world.set(ship_id, "/", ship);
    session["ship_h"] = ship_h;
    session["ship_x_fp"] = ship_x_fp;
    session["ship_y_fp"] = ship_y_fp;
    session["vx_fp"] = vx_fp;
    session["vy_fp"] = vy_fp;
    session["ship_invuln_ms"] = ship_invuln_ms;
    session["ship_respawn_ms"] = ship_respawn_ms;
    session["turn_accum_ms"] = turn_accum_ms;
    session["shot_cd_ms"] = shot_cd_ms;
    session["smoke_cd_ms"] = smoke_cd_ms;
    world.set(session_id, "/", session);
}

let session = if world.exists(session_id) { world.get(session_id, "/") } else { #{} };
if session.type_of() == "map" && session.contains("msg_ttl_ms") && session["msg_ttl_ms"] > 0 {
    session["msg_ttl_ms"] = session["msg_ttl_ms"] - dt;
    if session["msg_ttl_ms"] <= 0 {
        scene.set("game-message", "text.content", "");
    }
}
let score = if session.type_of() == "map" && session.contains("score") { session["score"] } else { 0 };
let lives = if session.type_of() == "map" && session.contains("lives") { session["lives"] } else { 0 };
let wave = if session.type_of() == "map" && session.contains("wave") { session["wave"] } else { 1 };
let ship_x_fp = if session.type_of() == "map" && session.contains("ship_x_fp") { session["ship_x_fp"] } else { 0 };
let ship_y_fp = if session.type_of() == "map" && session.contains("ship_y_fp") { session["ship_y_fp"] } else { 0 };
let vx_fp = if session.type_of() == "map" && session.contains("vx_fp") { session["vx_fp"] } else { 0 };
let vy_fp = if session.type_of() == "map" && session.contains("vy_fp") { session["vy_fp"] } else { 0 };

let active_count = world.count_kind("asteroid");
let last_diag_ms =
    if session.type_of() == "map" && session.contains("last_diag_ms") && session["last_diag_ms"].type_of() == "i64" {
        session["last_diag_ms"]
    } else {
        0
    };
if now - last_diag_ms >= 1000 {
    if session.type_of() == "map" {
        session["last_diag_ms"] = now;
    }
    diag.info(
        `stats lives=${lives} wave=${wave} score=${score} active=${active_count} bullets=${world.count_kind("bullet")} ship=(${ship_x_fp / scale},${ship_y_fp / scale}) vel=(${vx_fp},${vy_fp})`
    );
}
if active_count <= 0 {
    wave = wave + 1;
    if session.type_of() == "map" {
        session["msg_ttl_ms"] = 650;
        session["wave"] = wave;
    }
    scene.set("game-message", "text.content", `WAVE ${wave}`);
    diag.info(`wave start wave=${wave}`);
    let wave_base = if session.type_of() == "map" && session.contains("wave_base_spawn_count") { session["wave_base_spawn_count"] } else { 4 };
    let wave_min = if session.type_of() == "map" && session.contains("wave_min_spawn_count") { session["wave_min_spawn_count"] } else { 5 };
    let wave_max = if session.type_of() == "map" && session.contains("wave_max_spawn_count") { session["wave_max_spawn_count"] } else { 10 };
    let spawn_count = clamp_i(wave_base + wave, wave_min, wave_max);
    for idx in 0..spawn_count {
        if session.type_of() == "map" {
            session["rng"] = next_seed(session["rng"] + now + idx * 53 + wave);
        }
        let seed = if session.type_of() == "map" { session["rng"] } else { next_seed(now + idx * 53 + wave) };
        spawn_asteroid_entity(
            world,
            scene,
            respawn_x(seed, min_x, max_x, ship_x_fp / scale),
            respawn_y(seed / 3, min_y, max_y, ship_y_fp / scale),
            speed_for_seed(seed + 5),
            speed_for_seed(seed + 13),
            (seed / 11) % 4,
            if idx < 2 { 3 } else if idx < 5 { 2 } else { 1 }
        );
    }
}
if session.type_of() == "map" {
    world.set(session_id, "/", session);
}

if lives <= 0 {
    let entry = #{
        score: score,
        wave: wave,
        at_ms: now
    };
    diag.warn(`game over score=${score} wave=${wave}`);
    persist.push("/asteroids/highscores", entry);
    game.jump("asteroids.highscores");
    return ();
}
