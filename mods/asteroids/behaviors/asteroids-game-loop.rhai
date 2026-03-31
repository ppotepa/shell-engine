import "asteroids-shared" as h;

// ════════════════════════════════════════════════════════════════════════
// on_enter: Initialize scene
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_enter" {
    let cfg = h::load_game_config(level);
    local["cfg"] = cfg;

    world.reset_dynamic_entities();
    world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
    world.rand_seed(time.scene_elapsed_ms);

    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["lives"]);
    game.set("/ast/wave", 1);

    local["ship_id"] = world.spawn_prefab("ship", #{
        cfg: cfg, invulnerable_ms: cfg["invulnerability_ms"]
    });
    game.set("/ast/ship_id", local["ship_id"]);
    world.spawn_group("asteroids.initial", "asteroid");
    input.load_profile("asteroids.default");
    audio.play_song("asteroids.game-loop");
    diag.info("asteroids session init");
}

// ════════════════════════════════════════════════════════════════════════
// on_idle: Per-frame gameplay
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_idle" {
    let cfg = if local.contains("cfg") { local["cfg"] } else { h::load_game_config(level) };
    let ship_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
    let dt = time.delta_ms(220, "/ast/last_ms");

    // ── Escape / Reset ───────────────────────────────────────────────
    if key.pressed == true {
        let code = if key.code.type_of() == "string" { key.code } else { "" };
        if code == KEY_ESC {
            audio.stop_song();
            game.jump("asteroids.mainmenu");
            return ();
        }
        if code == "r" || code == "R" {
            let new_cfg = h::load_game_config(level);
            local["cfg"] = new_cfg;
            world.reset_dynamic_entities();
            world.rand_seed(time.scene_elapsed_ms);
            game.set("/ast/score", 0);
            game.set("/ast/lives", new_cfg["lives"]);
            game.set("/ast/wave", 1);
            local["ship_id"] = world.spawn_prefab("ship", #{
                cfg: new_cfg, invulnerable_ms: new_cfg["invulnerability_ms"]
            });
            game.set("/ast/ship_id", local["ship_id"]);
            world.spawn_group("asteroids.initial", "asteroid");
            ui.flash_message("RESET", 500);
            return ();
        }
    }

    // ── Ship controls ────────────────────────────────────────────────
    let ship = world.entity(ship_id);
    if ship.exists() {
        let turn = if input.action_down("turn_left") { -1 }
                   else if input.action_down("turn_right") { 1 }
                   else { 0 };
        ship.set_turn(turn);
        ship.set_thrust(input.action_down("thrust"));

        if input.action_down("thrust") {
            let start = if local.contains("thrust_start") { local["thrust_start"] } else { -1 };
            if start < 0 { start = time.scene_elapsed_ms; local["thrust_start"] = start; }
            fx.emit("asteroids.ship_thrust_smoke", #{
                ship_id: ship_id, thrust_ms: time.scene_elapsed_ms - start
            });
        } else {
            local["thrust_start"] = -1;
        }

        if input.action_down("fire") {
            world.try_fire_weapon("asteroids.ship", ship_id, #{
                bullet_ttl_ms: cfg["bullet_ttl_ms"],
                shot_cooldown_ms: cfg["shot_cooldown_ms"],
                bullet_speed: cfg["bullet_speed"]
            });
        }
    }

    // ── Asteroid tick (rotation + flash/split countdown) ─────────────
    for ast_id in world.query_kind("asteroid") {
        let ast = world.entity(ast_id);
        if !ast.exists() { continue; }

        let anim = world.tick_heading_anim(ast_id, dt);
        let flash_ms = ast.get_i("/flash_ms", 0);
        let flash_total = ast.get_i("/flash_total_ms", 0);
        let split_pending = ast.flag("split_pending");

        if flash_ms > 0 {
            flash_ms -= dt;
            if flash_ms <= 0 && split_pending {
                if world.handle_asteroid_split(ast_id, #{})["handled"] { continue; }
            }
        }

        if !ast.exists() { continue; }
        ast.set_many(#{
            flash_ms: flash_ms,
            flash_total_ms: if flash_ms <= 0 { 0 } else { flash_total },
            split_pending: split_pending,
            rot_phase: anim["rot_phase"],
            rot_accum_ms: anim["rot_accum_ms"]
        });
    }

    // ── Collisions: ship ↔ asteroid ──────────────────────────────────
    let score = game.get_i("/ast/score", 0);
    let lives = game.get_i("/ast/lives", 0);

    for hit in world.collision_enters("ship", "asteroid") {
        let ast_id = hit["asteroid"];
        let ast = world.entity(ast_id);
        if ast.exists() && ast.flag("split_pending") { continue; }

        // Re-fetch ship each iteration (may have respawned from prior hit)
        let cur_id = if local.contains("ship_id") { local["ship_id"] } else { 0 };
        let cur_ship = world.entity(cur_id);
        if !cur_ship.exists() || cur_ship.status_has("invulnerable") { continue; }

        let handled = world.handle_ship_hit(cur_id, ast_id, #{
            crack_duration_ms: cfg["crack_duration_ms"],
            asteroid_velocity_limit: 240.0,
            ship_invulnerable_ms: cfg["invulnerability_ms"],
            ui_text: "HIT!", ui_ttl_ms: 450
        });
        if !handled { continue; }
        lives -= 1;

        if world.exists(cur_id) { world.despawn(cur_id); }
        local["ship_id"] = world.spawn_prefab("ship", #{
            cfg: cfg, invulnerable_ms: cfg["invulnerability_ms"]
        });
        game.set("/ast/ship_id", local["ship_id"]);

        if world.exists(ast_id) {
            world.spawn_flash_cracks(ast_id, cfg["crack_duration_ms"]);
        }
    }

    // ── Collisions: bullet ↔ asteroid ────────────────────────────────
    for hit in world.collision_enters("bullet", "asteroid") {
        let ast_id = hit["asteroid"];
        let ast = world.entity(ast_id);
        if ast.exists() && ast.flag("split_pending") { continue; }

        let result = world.handle_bullet_hit(hit["bullet"], ast_id, #{
            crack_duration_ms: cfg["crack_duration_ms"],
            bullet_momentum_factor: 0.35,
            ui_text: "HIT", ui_ttl_ms: 250
        });
        if result["handled"] {
            score += h::asteroid_score(result["asteroid_size"]);
            if world.exists(ast_id) {
                world.spawn_flash_cracks(ast_id, cfg["crack_duration_ms"]);
            }
        }
    }

    game.set("/ast/score", score);
    game.set("/ast/lives", lives);

    // ── Wave progression ─────────────────────────────────────────────
    if !world.any_alive("asteroid") {
        let wave = game.get_i("/ast/wave", 0) + 1;
        game.set("/ast/wave", wave);
        ui.flash_message(`WAVE ${wave}`, 650);

        let count = clamp_i(cfg["wave_base"] + wave, cfg["wave_min"], cfg["wave_max"]);
        let sid = if local.contains("ship_id") { local["ship_id"] } else { 0 };
        let s = world.entity(sid);
        let pos = if s.exists() { s.transform() } else { #{x: 0, y: 0} };
        world.spawn_wave("asteroids.dynamic", #{
            spawn_count: count,
            ship_x: pos.x, ship_y: pos.y,
            min_x: -320.0, max_x: 320.0,
            min_y: -240.0, max_y: 240.0
        });
    }

    // ── Game over ────────────────────────────────────────────────────
    if lives <= 0 {
        let wave = game.get_i("/ast/wave", 0);
        persist.push("/asteroids/highscores", #{ score: score, wave: wave, at_ms: time.scene_elapsed_ms });
        game.jump("asteroids.highscores");
        return ();
    }
}

// ════════════════════════════════════════════════════════════════════════
// on_leave
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_leave" {
    audio.stop_song();
}
