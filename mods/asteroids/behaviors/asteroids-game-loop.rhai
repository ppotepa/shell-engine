import "asteroids-shared" as h;

if time.stage == "on_enter" {
    let cfg = h::load_game_config(level);
    local["cfg"] = cfg;
    local["thrust_start"] = -1;

    world.reset_dynamic_entities();
    world.set_world_bounds(-320.0, -240.0, 320.0, 240.0);
    world.rand_seed(time.scene_elapsed_ms);

    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["lives"]);
    game.set("/ast/wave", 1);

    let ship_id = h::spawn_ship(world, cfg);
    local["ship_id"] = ship_id;
    game.set("/ast/ship_id", ship_id);

    world.spawn_group("asteroids.initial", "asteroid");
    input.load_profile("asteroids.default");
}

if time.stage == "on_idle" {
    let cfg = if local.contains("cfg") { local["cfg"] } else { h::load_game_config(level) };
    let ship_id = game.get_i("/ast/ship_id", 0);
    let dt = time.delta_ms(220, "/ast/last_ms");

    if key.pressed == true {
        let code = if key.code.type_of() == "string" { key.code } else { "" };
        if code == KEY_ESC {
            game.jump("asteroids.mainmenu");
            return ();
        }
        if code == "r" || code == "R" {
            world.reset_dynamic_entities();
            world.rand_seed(time.scene_elapsed_ms);
            game.set("/ast/score", 0);
            game.set("/ast/lives", cfg["lives"]);
            game.set("/ast/wave", 1);
            let reset_ship_id = h::spawn_ship(world, cfg);
            local["ship_id"] = reset_ship_id;
            game.set("/ast/ship_id", reset_ship_id);
            world.spawn_group("asteroids.initial", "asteroid");
            local["thrust_start"] = -1;
            ui.flash_message("RESET", 500);
            return ();
        }
    }

    let ship = world.entity(ship_id);
    if ship.exists() {
        let turn = if input.action_down("turn_left") { -1 }
            else if input.action_down("turn_right") { 1 }
            else { 0 };
        ship.set_turn(turn);
        ship.set_thrust(input.action_down("thrust"));

        if input.action_down("thrust") {
            let start = if local.contains("thrust_start") { local["thrust_start"] } else { -1 };
            if start < 0 {
                start = time.scene_elapsed_ms;
                local["thrust_start"] = start;
            }
            h::emit_thrust_smoke(world, ship_id, time.scene_elapsed_ms - start);
        } else {
            local["thrust_start"] = -1;
        }

        if input.action_down("fire") {
            h::fire_weapon(world, audio, ship_id, cfg);
        }
    }

    for ast_id in world.query_kind("asteroid") {
        let ast = world.entity(ast_id);
        if !ast.exists() { continue; }

        let anim = h::tick_heading_anim(ast, dt);
        let flash_ms = ast.get_i("/flash_ms", 0);
        let flash_total = ast.get_i("/flash_total_ms", 0);
        let split_pending = ast.flag("split_pending");

        if flash_ms > 0 {
            flash_ms -= dt;
            if flash_ms <= 0 && split_pending {
                let split = h::split_asteroid(world, ast_id);
                if split["handled"] {
                    audio.event("gameplay.asteroid.split", 1.0);
                    continue;
                }
            }
        }

        if !ast.exists() { continue; }
        ast.set_many(#{
            flash_ms: flash_ms,
            flash_total_ms: if flash_ms <= 0 { 0 } else { flash_total },
            split_pending: if flash_ms <= 0 { false } else { split_pending },
            rot_phase: anim["rot_phase"],
            rot_accum_ms: anim["rot_accum_ms"]
        });
    }

    let score = game.get_i("/ast/score", 0);
    let lives = game.get_i("/ast/lives", 0);

    for hit in world.collision_enters("ship", "asteroid") {
        let ast_id = hit["asteroid"];
        let ast = world.entity(ast_id);
        if ast.exists() && ast.flag("split_pending") { continue; }

        let cur_id = game.get_i("/ast/ship_id", 0);
        let cur_ship = world.entity(cur_id);
        if !cur_ship.exists() || cur_ship.status_has("invulnerable") { continue; }

        let handled = h::handle_ship_hit(world, cur_id, ast_id, cfg);
        if !handled { continue; }

        audio.event("gameplay.ship.hit", 1.0);
        ui.flash_message("HIT!", 450);
        let ship_t = cur_ship.transform();
        h::emit_disintegration(world, ship_t.x, ship_t.y);

        lives -= 1;
        if world.exists(cur_id) { world.despawn(cur_id); }

        if lives > 0 {
            let next_ship_id = h::spawn_ship(world, cfg);
            local["ship_id"] = next_ship_id;
            game.set("/ast/ship_id", next_ship_id);
        } else {
            local["ship_id"] = 0;
            game.set("/ast/ship_id", 0);
        }
    }

    for hit in world.collision_enters("bullet", "asteroid") {
        let ast_id = hit["asteroid"];
        let ast = world.entity(ast_id);
        if ast.exists() && ast.flag("split_pending") { continue; }

        let result = h::handle_bullet_hit(world, hit["bullet"], ast_id, cfg["crack_duration_ms"], 0.35);
        if result["handled"] {
            score += h::asteroid_score(result["asteroid_size"]);
            ui.flash_message("HIT", 250);
        }
    }

    game.set("/ast/score", score);
    game.set("/ast/lives", lives);

    if lives <= 0 {
        game.jump("asteroids.mainmenu");
        return ();
    }

    if !world.any_alive("asteroid") {
        let wave = game.get_i("/ast/wave", 0) + 1;
        game.set("/ast/wave", wave);
        ui.flash_message(`WAVE ${wave}`, 650);

        let count = clamp_i(cfg["wave_base"] + wave, cfg["wave_min"], cfg["wave_max"]);
        let sid = game.get_i("/ast/ship_id", 0);
        let s = world.entity(sid);
        let pos = if s.exists() { s.transform() } else { #{ x: 0, y: 0 } };
        world.spawn_wave("asteroids.dynamic", #{
            spawn_count: count,
            ship_x: pos.x,
            ship_y: pos.y,
            min_x: -320.0,
            max_x: 320.0,
            min_y: -240.0,
            max_y: 240.0
        });
    }
}
