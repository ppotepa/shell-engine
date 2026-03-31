import "asteroids-shared" as h;

// ════════════════════════════════════════════════════════════════════════
// Scene initialization and helpers
// ════════════════════════════════════════════════════════════════════════

let WORLD_MIN_X = -320.0;
let WORLD_MAX_X =  320.0;
let WORLD_MIN_Y = -240.0;
let WORLD_MAX_Y =  240.0;

fn spawn_ship(world, cfg) {
    world.spawn_prefab("ship", #{ cfg: cfg, invulnerable_ms: 3000 })
}

fn spawn_asteroid(world, x, y, vx, vy, shape, size) {
    world.spawn_prefab("asteroid", #{
        x: x, y: y, vx: vx, vy: vy, shape: shape, size: size
    })
}

fn spawn_initial_asteroids(world) {
    world.spawn_group("asteroids.initial", "asteroid");
}

fn reset_game_state(world, game, level, local, now) {
    let cfg = h::load_game_config(level);
    local["cfg"] = cfg;
    world.rand_seed(now);
    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["base_lives"]);
    game.set("/ast/wave", 1);
    game.set("/ast/last_ms", now);
    cfg
}

// ════════════════════════════════════════════════════════════════════════
// on_enter: Initialize scene on first load
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_enter" {
    let now = time.get_i("scene_elapsed_ms", 0);
    world.reset_dynamic_entities();
    world.set_world_bounds(WORLD_MIN_X, WORLD_MAX_X, WORLD_MIN_Y, WORLD_MAX_Y);
    let cfg = reset_game_state(world, game, level, local, now);
    local["ship_id"] = spawn_ship(world, cfg);
    spawn_initial_asteroids(world);
    input.load_profile("asteroids.default");
    audio.play_song("asteroids.game-loop");
    diag.info("asteroids session init");
}

// ════════════════════════════════════════════════════════════════════════
// on_idle: Per-frame gameplay logic
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_idle" {
    let now = time.get_i("scene_elapsed_ms", 0);
    let min_x = WORLD_MIN_X;
    let max_x = WORLD_MAX_X;
    let min_y = WORLD_MIN_Y;
    let max_y = WORLD_MAX_Y;
    let ship_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
    let dt = time.delta_ms(220, "/ast/last_ms");

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
            world.reset_dynamic_entities();
            let cfg = reset_game_state(world, game, level, local, now);
            local["ship_id"] = spawn_ship(world, cfg);
            spawn_initial_asteroids(world);
            ui.flash_message(reset_label, 500);
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

        let turn_dir = if left_down { -1 } else if right_down { 1 } else { 0 };
        ship_ref.set_turn(turn_dir);
        ship_ref.set_thrust(thrust_down);

        let t = ship_ref.transform();
        let tx = t.x;
        let ty = t.y;
        let vel = ship_ref.physics.velocity();
        let ship_h = ship_ref.heading();
        let vx = vel[0];
        let vy = vel[1];

        let diag_tick = to_i(now / 2000);
        let last_diag = if local.contains("diag_tick") { local["diag_tick"] } else { -1 };
        if diag_tick != last_diag {
            local["diag_tick"] = diag_tick;
            let invuln_remaining = ship_ref.status_remaining("invulnerable");
            diag.info(`ship pos=(${tx},${ty}) vel=(${vx},${vy}) h=${ship_h} turn=${turn_dir} L=${left_down} R=${right_down}`);
        }

        if thrust_down {
            fx.emit("asteroids.ship_thrust_smoke", #{ ship_id: ship_id });
        }

        if fire_down {
            world.try_fire_weapon("asteroids.ship", ship_id, cfg);
        }
    }

    for ast_id in world.query_kind("asteroid") {
        let ast_ref = world.entity(ast_id);
        if !ast_ref.exists() { continue; }

        let heading_anim = world.tick_heading_anim(ast_id, dt);
        let rot_phase = heading_anim["rot_phase"];
        let rot_accum = heading_anim["rot_accum_ms"];

        let flash_ms = ast_ref.get_i("/flash_ms", 0);
        let flash_total = ast_ref.get_i("/flash_total_ms", 0);
        let split_pending = ast_ref.flag("split_pending");
        let ast_size = ast_ref.get_i("/size", 1);

        if flash_ms > 0 {
            flash_ms -= dt;
            if flash_ms <= 0 && split_pending {
                let split_result = world.handle_asteroid_split(ast_id, #{});
                if split_result["despawned"] {
                    if split_result["handled"] {
                        continue;
                    }
                }
            }
        }

        ast_ref.set_many(#{
            flash_ms: flash_ms, flash_total_ms: if flash_ms <= 0 { 0 } else { flash_total },
            split_pending: split_pending, rot_phase: rot_phase, rot_accum_ms: rot_accum
        });
    }

    let score = game.get_i("/ast/score", 0);
    let lives = game.get_i("/ast/lives", 0);
    let ship_ref = world.entity(ship_id);
    let ship_invuln = if ship_ref.exists() { ship_ref.status_has("invulnerable") } else { false };

    for hit in world.collision_enters("ship", "asteroid") {
        let ast_hit_id = hit["asteroid"];
        let hit_ref = world.entity(ast_hit_id);

        if hit_ref.exists() && hit_ref.flag("split_pending") { continue; }

        if !ship_ref.status_has("invulnerable") && world.exists(ship_id) {
            let handled = world.handle_ship_hit(ship_id, ast_hit_id, #{
                crack_duration_ms: h::CRACK_DURATION_MS(),
                asteroid_velocity_limit: 4.0 * h::velocity_scale(),
                ship_invulnerable_ms: 3000,
                ui_text: "HIT!",
                ui_ttl_ms: 450
            });
            if !handled { continue; }
            lives -= 1;
            
            // Spawn temporary crack visuals for the flashing asteroid
            if world.exists(ast_hit_id) {
                let flash_duration = h::CRACK_DURATION_MS();
                world.spawn_flash_cracks(ast_hit_id, flash_duration);
            }
        }
    }

    for hit in world.collision_enters("bullet", "asteroid") {
        let bullet_id = hit["bullet"];
        let ast_hit_id = hit["asteroid"];
        let hit_ref = world.entity(ast_hit_id);

        if hit_ref.exists() && hit_ref.flag("split_pending") { continue; }

        let bullet_hit = world.handle_bullet_hit(bullet_id, ast_hit_id, #{
            crack_duration_ms: h::CRACK_DURATION_MS(),
            ui_text: "HIT",
            ui_ttl_ms: 250
        });
        if bullet_hit["handled"] {
            score += asteroid_score(bullet_hit["asteroid_size"]);
            diag.info(`hit asteroid=${ast_hit_id}`);
            
            // Spawn temporary crack visuals for the flashing asteroid
            if world.exists(ast_hit_id) {
                let flash_duration = h::CRACK_DURATION_MS();
                world.spawn_flash_cracks(ast_hit_id, flash_duration);
            }
        }
    }

    game.set("/ast/score", score);
    game.set("/ast/lives", lives);

    if !world.any_alive("asteroid") {
        let wave = game.get_i("/ast/wave", 0);
        wave += 1;
        game.set("/ast/wave", wave);
        ui.flash_message(`WAVE ${wave}`, 650);
         diag.info(`wave start wave=${wave}`);

         let cfg = if local.contains("cfg") { local["cfg"] } else { h::load_game_config(level) };
         let spawn_count = clamp_i(cfg["wave_base_spawn_count"] + wave, cfg["wave_min_spawn_count"], cfg["wave_max_spawn_count"]);

         let ship_t = if ship_ref.exists() { ship_ref.transform() } else { #{x: 0, y: 0} };
         let wave_ids = world.spawn_wave("asteroids.dynamic", #{
             spawn_count: spawn_count,
             ship_x: ship_t.x,
             ship_y: ship_t.y,
             min_x: min_x,
             max_x: max_x,
             min_y: min_y,
             max_y: max_y
         });
     }

    if lives <= 0 {
        diag.warn(`game over score=${score} wave=${game.get_i("/ast/wave", 0)}`);
        persist.push("/asteroids/highscores", #{ score: score, wave: game.get_i("/ast/wave", 0), at_ms: now });
        game.jump("asteroids.highscores");
        return ();
    }
}

// ════════════════════════════════════════════════════════════════════════
// on_leave: Cleanup on scene transition
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_leave" {
    audio.stop_song();
}
