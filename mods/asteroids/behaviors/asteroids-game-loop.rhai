import "asteroids-shared" as h;

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

fn spawn_and_bind_visual(scene, world, template, entity_id, visual_id) {
    scene.spawn_object(template, visual_id);
    world.bind_visual(entity_id, visual_id);
}

fn spawn_ship_entity(world) {
    let id = world.spawn_object("ship", #{
        tags: ["ship"],
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        h: 0,
        invuln_ms: 3000
    });
    if id > 0 {
        let ship_ref = world.entity(id);
        ship_ref.set_position(0.0, 0.0);
        ship_ref.set_heading(h::heading32_to_rad(0));
        ship_ref.set_velocity(0.0, 0.0);
        ship_ref.set_acceleration(0.0, 0.0);
        world.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF);
    }
    id
}

fn spawn_asteroid_entity(world, scene, x, y, vx, vy, shape, size) {
    let spin_seed = abs_i((x * 31) + (y * 17) + (vx * 13) + (vy * 19) + (shape * 23) + (size * 29));
    let entity_id = world.spawn_object("asteroid", #{
        tags: ["asteroid"],
        active: true,
        x: to_float(x),
        y: to_float(y),
        vx: to_float(vx),
        vy: to_float(vy),
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
        spawn_and_bind_visual(scene, world, "asteroid-template", entity_id, visual_id);
        spawn_and_bind_visual(scene, world, "asteroid-template", entity_id, crack_visual_0);
        spawn_and_bind_visual(scene, world, "asteroid-template", entity_id, crack_visual_1);
        spawn_and_bind_visual(scene, world, "asteroid-template", entity_id, crack_visual_2);
        world.set(entity_id, "/visual_id", visual_id);
        world.set(entity_id, "/crack_visual_0", crack_visual_0);
        world.set(entity_id, "/crack_visual_1", crack_visual_1);
        world.set(entity_id, "/crack_visual_2", crack_visual_2);
        world.set_visual(entity_id, visual_id);
        let asteroid_ref = world.entity(entity_id);
        asteroid_ref.set_position(to_float(x), to_float(y));
        asteroid_ref.set_heading(h::heading32_to_rad(0));
        asteroid_ref.set_velocity(to_float(vx), to_float(vy));
        asteroid_ref.set_acceleration(0.0, 0.0);
        world.set_collider_circle(entity_id, to_float(asteroid_radius(size)), 0xFFFF, 0xFFFF);
    }
    entity_id
}

fn spawn_bullet_entity(world, scene, x, y, vx, vy, ttl_ms) {
    let entity_id = world.spawn_object("bullet", #{
        tags: ["bullet"],
        active: true,
        x: x,
        y: y,
        vx: vx,
        vy: vy,
        ttl_ms: ttl_ms
    });
    if entity_id > 0 {
        let visual_id = `bullet-${entity_id}`;
        scene.spawn_object("bullet-template", visual_id);
        world.set(entity_id, "/visual_id", visual_id);
        world.set_visual(entity_id, visual_id);
        let bullet_ref = world.entity(entity_id);
        bullet_ref.set_position(to_float(x), to_float(y));
        bullet_ref.set_heading(h::heading32_to_rad(0));
        bullet_ref.set_velocity(to_float(vx), to_float(vy));
        bullet_ref.set_acceleration(0.0, 0.0);
        world.set_collider_circle(entity_id, 3.0, 0xFFFF, 0xFFFF);
        world.set_lifetime(entity_id, ttl_ms);
    }
    entity_id
}

fn spawn_smoke_entity(world, scene, x, y, vx, vy, ttl_ms, radius) {
    let entity_id = world.spawn_object("smoke", #{
        tags: ["smoke"],
        active: true,
        x: x,
        y: y,
        vx: vx,
        vy: vy,
        ttl_ms: ttl_ms,
        max_ttl_ms: ttl_ms,
        radius: radius
    });
    if entity_id > 0 {
        let visual_id = `smoke-${entity_id}`;
        scene.spawn_object("smoke-template", visual_id);
        world.set(entity_id, "/visual_id", visual_id);
        world.set_visual(entity_id, visual_id);
        let smoke_ref = world.entity(entity_id);
        smoke_ref.set_position(to_float(x), to_float(y));
        smoke_ref.set_heading(h::heading32_to_rad(0));
        smoke_ref.set_velocity(to_float(vx), to_float(vy));
        smoke_ref.set_acceleration(0.0, 0.0);
        // Drag factor: 0.04 ≈ (1 - 0.96) per second, similar to (vel * 96) / 100
        world.set_physics(entity_id, to_float(vx), to_float(vy), 0.0, 0.0, 0.04, 0.0);
        world.set_collider_circle(entity_id, 2.0, 0xFFFF, 0xFFFF);
        world.set_lifetime(entity_id, ttl_ms);
    }
    entity_id
}

fn spawn_ship_disintegration(world, scene, x, y) {
    for idx in 0..12 {
        let ang = (idx * 3) % 32;
        let dx = to_float(sin32(ang)) * 0.879;
        let dy = to_float(-sin32(ang + 8)) * 0.879;
        spawn_smoke_entity(
            world,
            scene,
            x,
            y,
            dx,
            dy,
            700,
            3
        );
    }
}



fn load_game_config(level) {
    let base_lives = level_i(level, "/player/lives", 3);
    let shot_cooldown_ms = clamp_i(level_i(level, "/player/shot_cooldown_ms", 200), 200, 1000);
    let ship_thrust = to_float(clamp_i(level_i(level, "/player/thrust_fp", 170), 60, 600)) / 1024.0;
    let ship_max_speed = to_float(clamp_i(level_i(level, "/player/max_speed_fp", 4608), 1024, 14000)) / 1024.0;
    let turn_step_ms = clamp_i(level_i(level, "/player/turn_step_ms", 40), 16, 150);
    let wave_base_spawn_count = clamp_i(level_i(level, "/waves/base_spawn_count", 4), 1, 20);
    let wave_min_spawn_count = clamp_i(level_i(level, "/waves/min_spawn_count", 5), 1, 30);
    let wave_max_spawn_count = clamp_i(level_i(level, "/waves/max_spawn_count", 10), 1, 40);
    let wave_min_spawn_count = if wave_min_spawn_count > wave_max_spawn_count { wave_max_spawn_count } else { wave_min_spawn_count };
    let bullet_ttl_ms = clamp_i(level_i(level, "/combat/bullet_ttl_ms", 900), 100, 6000);
    let bullet_speed = to_float(clamp_i(level_i(level, "/combat/bullet_speed_fp", 4200), 1000, 18000)) / 1024.0;
    let hit_flash_ms = clamp_i(level_i(level, "/combat/hit_flash_ms", 150), 40, 800);
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
        bullet_speed: bullet_speed,
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
    local["cfg"] = cfg;
    let clear_ids = world.ids();
    for entity_id in clear_ids {
        world.entity(entity_id).despawn();
    }
    world.clear();
    local["rng"] = next_seed(1337 + init_now + 101);
    
    // Initialize game state via game API
    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["base_lives"]);
    game.set("/ast/wave", 1);
    game.set("/ast/shot_cd_ms", 0);
    game.set("/ast/ship_h", 0);
    game.set("/ast/ship_x_fp", 0);
    game.set("/ast/ship_y_fp", 0);
    game.set("/ast/vx_fp", 0);
    game.set("/ast/vy_fp", 0);
    game.set("/ast/ship_invuln_ms", 3000);
    game.set("/ast/ship_respawn_ms", 0);
    game.set("/ast/turn_accum_ms", 0);
    game.set("/ast/smoke_cd_ms", 0);
    game.set("/ast/shot_cooldown_ms", cfg["shot_cooldown_ms"]);
    game.set("/ast/rng", local["rng"]);
    game.set("/ast/msg_ttl_ms", 0);
    game.set("/ast/last_ms", init_now);
    game.set("/ast/sim_accum", 0);
    game.set("/ast/last_diag_ms", 0);
    
    local["ship_id"] = spawn_ship_entity(world);
    spawn_initial_asteroids(world, scene);
    audio.play_song("asteroids.game-loop");
    local["init"] = true;
    diag.info("asteroids session init visuals=runtime-instanced bullets=dynamic asteroids=dynamic");
}

let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
let min_x = -320;
let max_x = 320;
let min_y = -240;
let max_y = 240;
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
        local["cfg"] = cfg;
        local["rng"] = next_seed(current_rng + now + 101);
        let clear_ids = world.ids();
        for entity_id in clear_ids {
            world.entity(entity_id).despawn();
        }
        world.clear();
        
        // Reinitialize game state via game API
        game.set("/ast/score", 0);
        game.set("/ast/lives", cfg["base_lives"]);
        game.set("/ast/wave", 1);
        game.set("/ast/shot_cd_ms", 0);
        game.set("/ast/ship_h", 0);
        game.set("/ast/ship_x_fp", 0);
        game.set("/ast/ship_y_fp", 0);
        game.set("/ast/vx_fp", 0);
        game.set("/ast/vy_fp", 0);
        game.set("/ast/ship_invuln_ms", 3000);
        game.set("/ast/ship_respawn_ms", 0);
        game.set("/ast/turn_accum_ms", 0);
        game.set("/ast/smoke_cd_ms", 0);
        game.set("/ast/shot_cooldown_ms", cfg["shot_cooldown_ms"]);
        game.set("/ast/rng", local["rng"]);
        
        local["ship_id"] = spawn_ship_entity(world);
        spawn_initial_asteroids(world, scene);
        game.set("/ast/msg_ttl_ms", 500);
        scene.set("game-message", "text.content", reset_label);
        diag.warn(`reset requested mode=${reset_label}`);
    }
}

let step_ms = 16;
let sim_accum = game.get("/ast/sim_accum");
let accum = sim_accum + dt;
let steps = accum / step_ms;
if steps > 8 {
    steps = 8;
    accum = steps * step_ms;
}
game.set("/ast/sim_accum", accum - (steps * step_ms));

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
    if !world.exists(ship_id) {
        continue;
    }

    let score = game.get("/ast/score");
    let lives = game.get("/ast/lives");
    let wave = game.get("/ast/wave");
    let shot_cd_ms = game.get("/ast/shot_cd_ms");
    let ship_h = game.get("/ast/ship_h");
    let ship_x_fp = game.get("/ast/ship_x_fp");
    let ship_y_fp = game.get("/ast/ship_y_fp");
    let vx_fp = game.get("/ast/vx_fp");
    let vy_fp = game.get("/ast/vy_fp");
    let ship_invuln_ms = game.get("/ast/ship_invuln_ms");
    let cfg = if local.contains("cfg") { local["cfg"] } else { load_game_config(level) };
    let session_turn_step_ms = cfg["turn_step_ms"];
    let turn_accum_ms = game.get("/ast/turn_accum_ms");
    let ship_respawn_ms = game.get("/ast/ship_respawn_ms");
    let smoke_emit_interval_ms = 48;
    let smoke_ttl_ms = 520;
    let smoke_drift_fp = 360;
    let smoke_max_count = 40;
    let smoke_radius = 3;
    let smoke_cd_ms = game.get("/ast/smoke_cd_ms");

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
            game.set("/ast/msg_ttl_ms", 450);
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
            let thrust_fp = (cfg["ship_thrust"] * 1024.0) as i64;
            let max_speed_fp = (cfg["ship_max_speed"] * 1024.0) as i64;
            vx_fp = clamp_i(vx_fp + ((fx * thrust_fp) / 1024), -max_speed_fp, max_speed_fp);
            vy_fp = clamp_i(vy_fp + ((fy * thrust_fp) / 1024), -max_speed_fp, max_speed_fp);
            if smoke_cd_ms <= 0 && world.count_kind("smoke") < smoke_max_count {
                spawn_smoke_entity(
                    world,
                    scene,
                    ship_x_fp - (fx * 6),
                    ship_y_fp - (fy * 6),
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
                let bullet_speed_fp = (cfg["bullet_speed"] * 1024.0) as i64;
                spawn_bullet_entity(
                    world,
                    scene,
                    ship_x_fp + (fx * 9),
                    ship_y_fp + (fy * 9),
                    vx_fp + ((fx * bullet_speed_fp) / 1024),
                    vy_fp + ((fy * bullet_speed_fp) / 1024),
                    cfg["bullet_ttl_ms"]
                );
                shot_cd_ms = game.get("/ast/shot_cooldown_ms");
                audio.event("gameplay.ship.shoot", 1.0);
            }
        }
    }

    ship_x_fp = wrap_fp(ship_x_fp + vx_fp, min_x, max_x, 1024);
    ship_y_fp = wrap_fp(ship_y_fp + vy_fp, min_y, max_y, 1024);
    let ship_x = ship_x_fp / 1024;
    let ship_y = ship_y_fp / 1024;
    world.set_transform(ship_id, to_float(ship_x_fp) / 1024.0, to_float(ship_y_fp) / 1024.0, h::heading32_to_rad(ship_h));
    world.set_physics(ship_id, to_float(vx_fp) / 1024.0, to_float(vy_fp) / 1024.0, 0.0, 0.0, 0.0, 0.0);
    let ship_ref = world.entity(ship_id);
    ship_ref.set("/h", ship_h);
    ship_ref.set("/invuln_ms", ship_invuln_ms);

    let asteroid_ids = world.query_kind("asteroid");
    for ast_id in asteroid_ids {
        let ast = world.get(ast_id, "/");
        if ast["active"] == false {
            continue;
        }

        ast["rot_accum_ms"] = ast["rot_accum_ms"] + step_ms;
        while ast["rot_accum_ms"] >= ast["rot_step_ms"] {
            ast["rot_phase"] = h::wrap_heading32(ast["rot_phase"] + ast["rot_speed"]);
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
                        world.entity(ast_id).despawn();
                        diag.info(`asteroid removed id=${ast_id}`);
                        continue;
                    } else {
                        let next_size = ast["size"] - 1;
                        let base_heading = ast["rot_phase"];
                        for frag_idx in 0..3 {
                            let rng = game.get("/ast/rng");
                            rng = next_seed(rng + now + (ast_id * 41) + to_i(ast["x"]) + to_i(ast["y"]) + frag_idx * 67);
                            game.set("/ast/rng", rng);
                            let seed = rng;
                            let split_heading = h::wrap_heading32(base_heading + h::fragment_heading_offset(frag_idx));
                            let split_sin = sin32(split_heading);
                            let split_cos = sin32(split_heading + 8);
                            let dir_x = sign_i(split_sin, if frag_idx == 1 { 1 } else { -1 });
                            let dir_y = sign_i(-split_cos, if frag_idx == 0 { -1 } else { 1 });
                            let spawn_offset = asteroid_radius(next_size) + 4;
                            let child_id = spawn_asteroid_entity(
                                world,
                                scene,
                                wrap(ast["x"] + (to_float(split_sin * spawn_offset) / 1024.0), min_x, max_x),
                                wrap(ast["y"] - (to_float(split_cos * spawn_offset) / 1024.0), min_y, max_y),
                                clamp_f(ast["vx"] + (to_float(dir_x * (next_size + 2)) / 1000.0), -5.0, 5.0),
                                clamp_f(ast["vy"] + (to_float(dir_y * (next_size + 2)) / 1000.0), -5.0, 5.0),
                                (seed / 17) % 4,
                                next_size
                            );
                            if child_id > 0 {
                                world.set(child_id, "/collide_cd_ms", 180);
                            }
                        }
                        world.entity(ast_id).despawn();
                        audio.event("gameplay.asteroid.split", 1.0);
                        diag.info(`asteroid split id=${ast_id} pieces=3 size=${next_size}`);
                        continue;
                    }
                }
            }
        }
        world.set_transform(ast_id, to_float(ast["x"]), to_float(ast["y"]), h::heading32_to_rad(ast["rot_phase"]));
        let ast_ref = world.entity(ast_id);
        ast_ref.set_many(#{
            x: ast["x"],
            y: ast["y"],
            vx: ast["vx"],
            vy: ast["vy"],
            flash_ms: ast["flash_ms"],
            flash_total_ms: ast["flash_total_ms"],
            split_pending: ast["split_pending"],
            rot_phase: ast["rot_phase"],
            rot_accum_ms: ast["rot_accum_ms"]
        });
        if ast.contains("collide_cd_ms") && ast["collide_cd_ms"] > 0 {
            ast_ref.set("/collide_cd_ms", ast["collide_cd_ms"]);
        }
    }

    // asteroid-asteroid bounce handled via collisions buffer later (was manual polygon check)


    if shot_cd_ms > 0 {
        shot_cd_ms = shot_cd_ms - step_ms;
    }
    if smoke_cd_ms > 0 {
        smoke_cd_ms = smoke_cd_ms - step_ms;
    }
    if ship_invuln_ms > 0 {
        ship_invuln_ms = ship_invuln_ms - step_ms;
    }

    // Collision handling: ship vs asteroid
    let ship_ast_hits = world.collisions_between("ship", "asteroid");
    for hit in ship_ast_hits {
        if ship_invuln_ms <= 0 && ship_respawn_ms <= 0 {
            let ship_id_hit = hit["ship"];
            let ast_id_hit = hit["asteroid"];
            let ship_x = game.get("/ast/ship_x_fp") / 1024.0;
            let ship_y = game.get("/ast/ship_y_fp") / 1024.0;
            spawn_ship_disintegration(world, scene, ship_x, ship_y);
            audio.event("gameplay.ship.hit", 1.0);
            let hit_ref = world.entity(ast_id_hit);
            if hit_ref.exists() {
                hit_ref.set("/flash_ms", h::crack_duration_ms());
                hit_ref.set("/flash_total_ms", h::crack_duration_ms());
                hit_ref.set("/split_pending", true);
                let old_vx = hit_ref.get_f("/vx", 0.0);
                let old_vy = hit_ref.get_f("/vy", 0.0);
                hit_ref.set("/vx", clamp_f(-old_vx, -4.0, 4.0));
                hit_ref.set("/vy", clamp_f(-old_vy, -4.0, 4.0));
                hit_ref.set("/collide_cd_ms", 180);
            }
            lives = lives - 1;
            game.set("/ast/vx_fp", 0);
            game.set("/ast/vy_fp", 0);
            ship_respawn_ms = 750;
            ship_invuln_ms = 0;
            game.set("/ast/msg_ttl_ms", 450);
            scene.set("game-message", "text.content", "HIT!");
        }
    }

    // Collision handling: bullet vs asteroid
    let bullet_ast_hits = world.collisions_between("bullet", "asteroid");
    for hit in bullet_ast_hits {
        let bullet_id = hit["bullet"];
        let ast_id = hit["asteroid"];
        if world.exists(bullet_id) {
            world.despawn_object(bullet_id);
        }
        let hit_ref = world.entity(ast_id);
        if hit_ref.exists() && hit_ref.get_b("/active", false) {
            score = score + asteroid_score(hit_ref.get_i("/size", 0));
            hit_ref.set("/flash_ms", h::crack_duration_ms());
            hit_ref.set("/flash_total_ms", h::crack_duration_ms());
            hit_ref.set("/split_pending", true);
            game.set("/ast/msg_ttl_ms", 250);
            scene.set("game-message", "text.content", "HIT");
            diag.info(`hit asteroid=${ast_id} size=${hit_ref.get_i("/size", 0)} score=${score}`);
        }
    }

    game.set("/ast/score", score);
    game.set("/ast/lives", lives);
    game.set("/ast/wave", wave);
    game.set("/ast/shot_cd_ms", shot_cd_ms);
    game.set("/ast/ship_h", ship_h);
    game.set("/ast/ship_x_fp", ship_x_fp);
    game.set("/ast/ship_y_fp", ship_y_fp);
    game.set("/ast/vx_fp", vx_fp);
    game.set("/ast/vy_fp", vy_fp);
    game.set("/ast/ship_invuln_ms", ship_invuln_ms);
    game.set("/ast/ship_respawn_ms", ship_respawn_ms);
    game.set("/ast/turn_accum_ms", turn_accum_ms);
    game.set("/ast/smoke_cd_ms", smoke_cd_ms);
    let ship_ref = world.entity(ship_id);
    ship_ref.set("/h", ship_h);
    ship_ref.set("/invuln_ms", ship_invuln_ms);
}

let msg_ttl = game.get("/ast/msg_ttl_ms");
if msg_ttl > 0 {
    let new_ttl = msg_ttl - dt;
    if new_ttl <= 0 {
        scene.set("game-message", "text.content", "");
    }
    game.set("/ast/msg_ttl_ms", new_ttl);
}
let score = game.get("/ast/score");
let lives = game.get("/ast/lives");
let wave = game.get("/ast/wave");
let ship_x_fp = game.get("/ast/ship_x_fp");
let ship_y_fp = game.get("/ast/ship_y_fp");
let vx_fp = game.get("/ast/vx_fp");
let vy_fp = game.get("/ast/vy_fp");

let active_count = world.count_kind("asteroid");
let last_diag_ms = game.get("/ast/last_diag_ms");
if now - last_diag_ms >= 1000 {
    game.set("/ast/last_diag_ms", now);
    diag.info(
        `stats lives=${lives} wave=${wave} score=${score} active=${active_count} bullets=${world.count_kind("bullet")} ship=(${ship_x_fp / 1024},${ship_y_fp / 1024}) vel=(${vx_fp},${vy_fp})`
    );
}
if active_count <= 0 {
    wave = wave + 1;
    game.set("/ast/msg_ttl_ms", 650);
    game.set("/ast/wave", wave);
    scene.set("game-message", "text.content", `WAVE ${wave}`);
    diag.info(`wave start wave=${wave}`);
    let wave_base = level.get("/wave_base_spawn_count");
    if wave_base == () { wave_base = 4; }
    let wave_min = level.get("/wave_min_spawn_count");
    if wave_min == () { wave_min = 5; }
    let wave_max = level.get("/wave_max_spawn_count");
    if wave_max == () { wave_max = 10; }
    let spawn_count = clamp_i(wave_base + wave, wave_min, wave_max);
    for idx in 0..spawn_count {
        let rng = game.get("/ast/rng");
        rng = next_seed(rng + now + idx * 53 + wave);
        game.set("/ast/rng", rng);
        let seed = rng;
        spawn_asteroid_entity(
            world,
            scene,
            respawn_x(seed, min_x, max_x, ship_x_fp / 1024),
            respawn_y(seed / 3, min_y, max_y, ship_y_fp / 1024),
            speed_for_seed(seed + 5),
            speed_for_seed(seed + 13),
            (seed / 11) % 4,
            if idx < 2 { 3 } else if idx < 5 { 2 } else { 1 }
        );
    }
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
