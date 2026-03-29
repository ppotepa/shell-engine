import "asteroids-shared" as h;

// ── Utility functions ──────────────────────────────────────

fn next_seed(seed) {
    ((seed * 1103515245) + 12345) % 2147483647
}

fn abs_int(v) {
    let n = to_i(v);
    if n < 0 { -n } else { n }
}

fn speed_for_seed(seed) {
    let v = (seed % 3) + 1;
    if ((seed / 7) % 2) == 0 { v } else { -v }
}

fn respawn_x(seed, min_x, max_x, ship_x) {
    let x = if (seed % 2) == 0 { min_x + 10 } else { max_x - 10 };
    if abs_int(x - ship_x) < 90 {
        x = if x < 0 { max_x - 10 } else { min_x + 10 };
    }
    x
}

fn respawn_y(seed, min_y, max_y, ship_y) {
    let span = (max_y - min_y) - 20;
    let y = min_y + 10 + (seed % span);
    if abs_int(y - ship_y) < 70 {
        y = y + 76;
        if y > max_y - 10 { y = min_y + 10; }
    }
    y
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
    let ship_thrust = to_float(clamp_i(level_i(level, "/player/thrust_fp", 170), 60, 600)) / 1024.0;
    let ship_max_speed = to_float(clamp_i(level_i(level, "/player/max_speed_fp", 4608), 1024, 14000)) / 1024.0;
    let turn_step_ms = clamp_i(level_i(level, "/player/turn_step_ms", 40), 16, 150);
    let wave_base_spawn_count = clamp_i(level_i(level, "/waves/base_spawn_count", 4), 1, 20);
    let wave_min_spawn_count = clamp_i(level_i(level, "/waves/min_spawn_count", 5), 1, 30);
    let wave_max_spawn_count = clamp_i(level_i(level, "/waves/max_spawn_count", 10), 1, 40);
    if wave_min_spawn_count > wave_max_spawn_count { wave_min_spawn_count = wave_max_spawn_count; }
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

// ── Spawn helpers ──────────────────────────────────────────
// ALL entities use manual per-tick integration (x += vx per 16ms step).
// No PhysicsBody2D — engine physics multiplies by dt_sec which is 62x too slow.
// Collisions only need Transform2D + Collider2D.

fn spawn_ship(world) {
    let id = world.spawn_object("ship", #{
        tags: ["ship"],
        h: 0,
        invuln_ms: 3000
    });
    if id > 0 {
        world.set_visual(id, "ship");
        world.set(id, "/visual_id", "ship");
        world.set_transform(id, 0.0, 0.0, 0.0);
        world.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF);
    }
    id
}

fn spawn_asteroid(world, scene, x, y, vx, vy, shape, size) {
    let spin_seed = abs_int(
        to_i(x * 31) + to_i(y * 17) + to_i(vx * 13) + to_i(vy * 19) + (shape * 23) + (size * 29)
    );
    let id = world.spawn_object("asteroid", #{
        tags: ["asteroid"],
        active: true,
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
    if id > 0 {
        let vid = `asteroid-${id}`;
        scene.spawn_object("asteroid-template", vid);
        world.set_visual(id, vid);
        let cv0 = `asteroid-${id}-crack-0`;
        let cv1 = `asteroid-${id}-crack-1`;
        let cv2 = `asteroid-${id}-crack-2`;
        scene.spawn_object("asteroid-template", cv0);
        scene.spawn_object("asteroid-template", cv1);
        scene.spawn_object("asteroid-template", cv2);
        world.bind_visual(id, cv0);
        world.bind_visual(id, cv1);
        world.bind_visual(id, cv2);
        world.set(id, "/visual_id", vid);
        world.set(id, "/crack_visual_0", cv0);
        world.set(id, "/crack_visual_1", cv1);
        world.set(id, "/crack_visual_2", cv2);
        world.set_transform(id, to_float(x), to_float(y), h::heading32_to_rad(0));
        world.set_collider_circle(id, to_float(asteroid_radius(size)), 0xFFFF, 0xFFFF);
    }
    id
}

fn spawn_bullet(world, scene, x, y, vx, vy, ttl_ms) {
    let id = world.spawn_object("bullet", #{
        tags: ["bullet"],
        active: true,
        vx: to_float(vx),
        vy: to_float(vy),
        ttl_ms: ttl_ms
    });
    if id > 0 {
        let vid = `bullet-${id}`;
        scene.spawn_object("bullet-template", vid);
        world.set_visual(id, vid);
        world.set_transform(id, to_float(x), to_float(y), 0.0);
        world.set_collider_circle(id, 3.0, 0xFFFF, 0xFFFF);
        world.set_lifetime(id, ttl_ms);
    }
    id
}

fn spawn_smoke(world, scene, x, y, vx, vy, ttl_ms, radius) {
    let id = world.spawn_object("smoke", #{
        tags: ["smoke"],
        active: true,
        vx: to_float(vx),
        vy: to_float(vy),
        ttl_ms: ttl_ms,
        max_ttl_ms: ttl_ms,
        radius: radius
    });
    if id > 0 {
        let vid = `smoke-${id}`;
        scene.spawn_object("smoke-template", vid);
        world.set_visual(id, vid);
        world.set_transform(id, to_float(x), to_float(y), 0.0);
        world.set_collider_circle(id, 2.0, 0xFFFF, 0xFFFF);
        world.set_lifetime(id, ttl_ms);
    }
    id
}

fn spawn_disintegration(world, scene, x, y) {
    for idx in 0..12 {
        let ang = (idx * 3) % 32;
        let dx = to_float(sin32(ang)) * 0.879;
        let dy = to_float(-sin32(ang + 8)) * 0.879;
        spawn_smoke(world, scene, x, y, dx, dy, 700, 3);
    }
}

fn spawn_initial_asteroids(world, scene) {
    spawn_asteroid(world, scene, -300, -210, 2, 0, 0, 2);
    spawn_asteroid(world, scene, 300, -210, 0, 2, 1, 3);
    spawn_asteroid(world, scene, 300, 210, -2, 0, 2, 2);
    spawn_asteroid(world, scene, -300, 210, 0, -2, 3, 1);
    spawn_asteroid(world, scene, -290, -40, 2, 1, 0, 2);
    spawn_asteroid(world, scene, -140, -230, 2, 1, 1, 1);
    spawn_asteroid(world, scene, 140, -230, -2, 1, 2, 2);
    spawn_asteroid(world, scene, 290, 30, -2, -1, 3, 3);
    spawn_asteroid(world, scene, 120, 230, -2, -1, 1, 2);
    spawn_asteroid(world, scene, -120, 230, 2, -1, 0, 1);
}

// ── Init / Reset ───────────────────────────────────────────

fn reset_game_state(game, level, local, now) {
    let cfg = load_game_config(level);
    local["cfg"] = cfg;
    let current_rng = if local.contains("rng") && local["rng"].type_of() == "i64" { local["rng"] } else { 1337 };
    local["rng"] = next_seed(current_rng + now + 101);

    game.set("/ast/score", 0);
    game.set("/ast/lives", cfg["base_lives"]);
    game.set("/ast/wave", 1);
    game.set("/ast/shot_cd_ms", 0);
    game.set("/ast/ship_h", 0);
    game.set("/ast/ship_x", 0.0);
    game.set("/ast/ship_y", 0.0);
    game.set("/ast/vx", 0.0);
    game.set("/ast/vy", 0.0);
    game.set("/ast/ship_invuln_ms", 3000);
    game.set("/ast/ship_respawn_ms", 0);
    game.set("/ast/turn_accum_ms", 0);
    game.set("/ast/smoke_cd_ms", 0);
    game.set("/ast/shot_cooldown_ms", cfg["shot_cooldown_ms"]);
    game.set("/ast/rng", local["rng"]);
    game.set("/ast/msg_ttl_ms", 0);
    game.set("/ast/last_ms", now);
    game.set("/ast/sim_accum", 0);
    game.set("/ast/last_diag_ms", 0);
    cfg
}

fn clear_all_entities(world) {
    let ids = world.ids();
    for id in ids { world.entity(id).despawn(); }
    world.clear();
}

if !local.contains("init") {
    let now = if time.contains("scene_elapsed_ms") { time.scene_elapsed_ms } else { 0 };
    clear_all_entities(world);
    reset_game_state(game, level, local, now);
    local["ship_id"] = spawn_ship(world);
    spawn_initial_asteroids(world, scene);
    audio.play_song("asteroids.game-loop");
    local["init"] = true;
    diag.info("asteroids session init");
}

// ── Frame setup ────────────────────────────────────────────

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

// ── Key events (reset / level select / ESC) ────────────────

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
        ship_id = local["ship_id"];
        spawn_initial_asteroids(world, scene);
        game.set("/ast/msg_ttl_ms", 500);
        scene.set("game-message", "text.content", reset_label);
        diag.warn(`reset mode=${reset_label}`);
    }
}

// ── Fixed timestep accumulator ─────────────────────────────

let step_ms = 16;
let sim_accum = game.get("/ast/sim_accum");
let accum = sim_accum + dt;
let steps = accum / step_ms;
if steps > 8 { steps = 8; accum = steps * step_ms; }
game.set("/ast/sim_accum", accum - (steps * step_ms));

// ── Input state ────────────────────────────────────────────

let key_code = if key.code.type_of() == "string" { key.code } else { "" };
let key_pressed = key.pressed == true;
let left_event  = key_pressed && (key_code == "Left" || key_code == "a" || key_code == "A");
let right_event = key_pressed && (key_code == "Right" || key_code == "d" || key_code == "D");
let thrust_event = key_pressed && (key_code == "Up" || key_code == "w" || key_code == "W");
let fire_event  = key_pressed && (key_code == " " || key_code == "f" || key_code == "F");

let left_down  = input.down("Left") || input.down("a") || left_event;
let right_down = input.down("Right") || input.down("d") || right_event;
let thrust_down = input.down("Up") || input.down("w") || thrust_event;
let fire_down  = input.down(" ") || input.down("f") || fire_event;

// ── Simulation steps ───────────────────────────────────────

for _s in 0..steps {
    if !world.exists(ship_id) { continue; }

    let score = game.get("/ast/score");
    let lives = game.get("/ast/lives");
    let wave  = game.get("/ast/wave");
    let shot_cd_ms = game.get("/ast/shot_cd_ms");
    let ship_h = game.get("/ast/ship_h");
    let ship_x = game.get("/ast/ship_x");
    let ship_y = game.get("/ast/ship_y");
    let vx = game.get("/ast/vx");
    let vy = game.get("/ast/vy");
    let ship_invuln_ms = game.get("/ast/ship_invuln_ms");
    let ship_respawn_ms = game.get("/ast/ship_respawn_ms");
    let turn_accum_ms = game.get("/ast/turn_accum_ms");
    let smoke_cd_ms = game.get("/ast/smoke_cd_ms");
    let cfg = if local.contains("cfg") { local["cfg"] } else { load_game_config(level) };

    // ── Ship respawn timer ─────────────────────────────────
    if ship_respawn_ms > 0 {
        ship_respawn_ms -= step_ms;
        if ship_respawn_ms <= 0 {
            ship_respawn_ms = 0;
            ship_x = 0.0; ship_y = 0.0;
            vx = 0.0; vy = 0.0;
            ship_h = 0;
            ship_invuln_ms = 3000;
            game.set("/ast/msg_ttl_ms", 450);
            scene.set("game-message", "text.content", "REBOOT");
        }
    } else {
        // ── Ship turning ───────────────────────────────────
        if left_down != right_down {
            turn_accum_ms += step_ms;
            while turn_accum_ms >= cfg["turn_step_ms"] {
                if left_down {
                    ship_h = (ship_h + 31) % 32;
                } else {
                    ship_h = (ship_h + 1) % 32;
                }
                turn_accum_ms -= cfg["turn_step_ms"];
            }
        } else {
            turn_accum_ms = 0;
        }

        // ── Ship thrust ────────────────────────────────────
        if thrust_down {
            let fx = to_float(sin32(ship_h)) / 1024.0;
            let fy = -to_float(sin32(ship_h + 8)) / 1024.0;
            let thrust = cfg["ship_thrust"];
            let max_speed = cfg["ship_max_speed"];
            vx = clamp_f(vx + (fx * thrust), -max_speed, max_speed);
            vy = clamp_f(vy + (fy * thrust), -max_speed, max_speed);

            if smoke_cd_ms <= 0 && world.count_kind("smoke") < 40 {
                spawn_smoke(world, scene,
                    ship_x - (fx * 6.0), ship_y - (fy * 6.0),
                    vx - (fx * 0.35), vy - (fy * 0.35),
                    520, 3
                );
                smoke_cd_ms = 48;
            }
        }

        // ── Ship fire ──────────────────────────────────────
        if fire_down && shot_cd_ms <= 0 && world.count_kind("bullet") < 8 {
            let fx = to_float(sin32(ship_h)) / 1024.0;
            let fy = -to_float(sin32(ship_h + 8)) / 1024.0;
            let bspeed = cfg["bullet_speed"];
            spawn_bullet(world, scene,
                ship_x + (fx * 9.0), ship_y + (fy * 9.0),
                vx + (fx * bspeed), vy + (fy * bspeed),
                cfg["bullet_ttl_ms"]
            );
            shot_cd_ms = cfg["shot_cooldown_ms"];
            audio.event("gameplay.ship.shoot", 1.0);
        }
    }

    // ── Ship position (manual per-tick integration) ────────
    ship_x = wrap(ship_x + vx, min_x, max_x);
    ship_y = wrap(ship_y + vy, min_y, max_y);
    world.set_transform(ship_id, ship_x, ship_y, h::heading32_to_rad(ship_h));

    // ── Asteroid update (manual per-tick integration) ──────
    let asteroid_ids = world.query_kind("asteroid");
    for ast_id in asteroid_ids {
        let ast_ref = world.entity(ast_id);
        if !ast_ref.exists() { continue; }
        let d = ast_ref.data();
        if !d.contains("active") || d["active"] == false { continue; }

        // Read position from transform component
        let t = ast_ref.transform();
        let ax = if t.len() > 0 { t.x } else { 0.0 };
        let ay = if t.len() > 0 { t.y } else { 0.0 };
        // Read velocity from entity data (per-tick units)
        let avx = if d.contains("vx") { d["vx"] } else { 0.0 };
        let avy = if d.contains("vy") { d["vy"] } else { 0.0 };

        // Integrate position (per-tick, same as ship)
        ax = wrap(ax + avx, min_x, max_x);
        ay = wrap(ay + avy, min_y, max_y);

        // Rotation tick
        let rot_accum = if d.contains("rot_accum_ms") { d["rot_accum_ms"] } else { 0 };
        let rot_phase = if d.contains("rot_phase") { d["rot_phase"] } else { 0 };
        let rot_speed = if d.contains("rot_speed") { d["rot_speed"] } else { 1 };
        let rot_step  = if d.contains("rot_step_ms") { d["rot_step_ms"] } else { 72 };
        rot_accum += step_ms;
        while rot_accum >= rot_step {
            rot_phase = h::wrap_heading32(rot_phase + rot_speed);
            rot_accum -= rot_step;
        }

        // Flash / split timer
        let flash_ms = if d.contains("flash_ms") { d["flash_ms"] } else { 0 };
        let flash_total = if d.contains("flash_total_ms") { d["flash_total_ms"] } else { 0 };
        let split_pending = if d.contains("split_pending") { d["split_pending"] } else { false };
        let ast_size = if d.contains("size") { d["size"] } else { 1 };

        if flash_ms > 0 {
            flash_ms -= step_ms;
            if flash_ms <= 0 {
                flash_ms = 0;
                flash_total = 0;
                if split_pending {
                    if ast_size <= 0 {
                        ast_ref.despawn();
                        diag.info(`asteroid removed id=${ast_id}`);
                        continue;
                    }
                    let next_size = ast_size - 1;
                    let base_heading = rot_phase;
                    for frag_idx in 0..3 {
                        let rng = game.get("/ast/rng");
                        rng = next_seed(rng + now + (ast_id * 41) + to_i(ax) + to_i(ay) + frag_idx * 67);
                        game.set("/ast/rng", rng);
                        let split_heading = h::wrap_heading32(base_heading + h::fragment_heading_offset(frag_idx));
                        let split_sin = sin32(split_heading);
                        let split_cos = sin32(split_heading + 8);
                        let dir_x = sign_i(split_sin, if frag_idx == 1 { 1 } else { -1 });
                        let dir_y = sign_i(-split_cos, if frag_idx == 0 { -1 } else { 1 });
                        let spawn_offset = asteroid_radius(next_size) + 4;
                        let child_id = spawn_asteroid(world, scene,
                            wrap(ax + (to_float(split_sin * spawn_offset) / 1024.0), min_x, max_x),
                            wrap(ay - (to_float(split_cos * spawn_offset) / 1024.0), min_y, max_y),
                            clamp_f(avx + (to_float(dir_x * (next_size + 2)) / 1000.0), -5.0, 5.0),
                            clamp_f(avy + (to_float(dir_y * (next_size + 2)) / 1000.0), -5.0, 5.0),
                            (rng / 17) % 4,
                            next_size
                        );
                        if child_id > 0 {
                            world.set(child_id, "/collide_cd_ms", 180);
                        }
                    }
                    ast_ref.despawn();
                    audio.event("gameplay.asteroid.split", 1.0);
                    diag.info(`asteroid split id=${ast_id} size=${next_size}`);
                    continue;
                }
            }
        }

        // Write position back to transform
        ast_ref.set_position(ax, ay);
        ast_ref.set_heading(h::heading32_to_rad(rot_phase));

        // Decrement collision cooldown
        let cd = if d.contains("collide_cd_ms") { d["collide_cd_ms"] } else { 0 };
        if cd > 0 { cd -= step_ms; }

        // Write game-logic state
        ast_ref.set_many(#{
            vx: avx,
            vy: avy,
            flash_ms: flash_ms,
            flash_total_ms: flash_total,
            split_pending: split_pending,
            rot_phase: rot_phase,
            rot_accum_ms: rot_accum,
            collide_cd_ms: cd
        });
    }

    // ── Bullet update (manual per-tick integration) ────────
    for bid in world.query_kind("bullet") {
        let bref = world.entity(bid);
        if !bref.exists() { continue; }
        let bd = bref.data();
        let bvx = if bd.contains("vx") { bd["vx"] } else { 0.0 };
        let bvy = if bd.contains("vy") { bd["vy"] } else { 0.0 };
        let bt = bref.transform();
        let bx = if bt.len() > 0 { bt.x } else { 0.0 };
        let by = if bt.len() > 0 { bt.y } else { 0.0 };
        bref.set_position(wrap(bx + bvx, min_x, max_x), wrap(by + bvy, min_y, max_y));
    }

    // ── Smoke update (manual per-tick integration + drag) ──
    for sid in world.query_kind("smoke") {
        let sref = world.entity(sid);
        if !sref.exists() { continue; }
        let sd = sref.data();
        let svx = if sd.contains("vx") { sd["vx"] } else { 0.0 };
        let svy = if sd.contains("vy") { sd["vy"] } else { 0.0 };
        // Apply drag (4% velocity reduction per tick)
        svx = svx * 0.96;
        svy = svy * 0.96;
        let st = sref.transform();
        let sx = if st.len() > 0 { st.x } else { 0.0 };
        let sy = if st.len() > 0 { st.y } else { 0.0 };
        sref.set_position(sx + svx, sy + svy);
        // Decrement ttl for render-sync fade
        let sttl = if sd.contains("ttl_ms") { sd["ttl_ms"] } else { 0 };
        sttl -= step_ms;
        sref.set_many(#{ vx: svx, vy: svy, ttl_ms: sttl });
    }

    // ── Cooldown ticks ─────────────────────────────────────
    if shot_cd_ms > 0 { shot_cd_ms -= step_ms; }
    if smoke_cd_ms > 0 { smoke_cd_ms -= step_ms; }
    if ship_invuln_ms > 0 { ship_invuln_ms -= step_ms; }

    // ── Collision: ship vs asteroid ────────────────────────
    for hit in world.collisions_between("ship", "asteroid") {
        if ship_invuln_ms <= 0 && ship_respawn_ms <= 0 {
            let ast_hit_id = hit["asteroid"];
            spawn_disintegration(world, scene, ship_x, ship_y);
            audio.event("gameplay.ship.hit", 1.0);

            let hit_ref = world.entity(ast_hit_id);
            if hit_ref.exists() {
                hit_ref.set("/flash_ms", h::crack_duration_ms());
                hit_ref.set("/flash_total_ms", h::crack_duration_ms());
                hit_ref.set("/split_pending", true);
                let hd = hit_ref.data();
                let old_vx = if hd.contains("vx") { hd["vx"] } else { 0.0 };
                let old_vy = if hd.contains("vy") { hd["vy"] } else { 0.0 };
                hit_ref.set("/vx", clamp_f(-old_vx, -4.0, 4.0));
                hit_ref.set("/vy", clamp_f(-old_vy, -4.0, 4.0));
                hit_ref.set("/collide_cd_ms", 180);
            }
            lives -= 1;
            vx = 0.0; vy = 0.0;
            ship_respawn_ms = 750;
            ship_invuln_ms = 0;
            game.set("/ast/msg_ttl_ms", 450);
            scene.set("game-message", "text.content", "HIT!");
        }
    }

    // ── Collision: bullet vs asteroid ──────────────────────
    for hit in world.collisions_between("bullet", "asteroid") {
        let bullet_id = hit["bullet"];
        let ast_hit_id = hit["asteroid"];
        if world.exists(bullet_id) {
            world.despawn_object(bullet_id);
        }
        let hit_ref = world.entity(ast_hit_id);
        if hit_ref.exists() && hit_ref.get_b("/active", false) {
            score += asteroid_score(hit_ref.get_i("/size", 0));
            hit_ref.set("/flash_ms", h::crack_duration_ms());
            hit_ref.set("/flash_total_ms", h::crack_duration_ms());
            hit_ref.set("/split_pending", true);
            game.set("/ast/msg_ttl_ms", 250);
            scene.set("game-message", "text.content", "HIT");
            diag.info(`hit asteroid=${ast_hit_id} score=${score}`);
        }
    }

    // ── Write game state back ──────────────────────────────
    game.set("/ast/score", score);
    game.set("/ast/lives", lives);
    game.set("/ast/wave", wave);
    game.set("/ast/shot_cd_ms", shot_cd_ms);
    game.set("/ast/ship_h", ship_h);
    game.set("/ast/ship_x", ship_x);
    game.set("/ast/ship_y", ship_y);
    game.set("/ast/vx", vx);
    game.set("/ast/vy", vy);
    game.set("/ast/ship_invuln_ms", ship_invuln_ms);
    game.set("/ast/ship_respawn_ms", ship_respawn_ms);
    game.set("/ast/turn_accum_ms", turn_accum_ms);
    game.set("/ast/smoke_cd_ms", smoke_cd_ms);

    let ship_ref = world.entity(ship_id);
    if ship_ref.exists() {
        ship_ref.set("/h", ship_h);
        ship_ref.set("/invuln_ms", ship_invuln_ms);
    }
}

// ── Message TTL (outside sim loop, uses real dt) ───────────

let msg_ttl = game.get("/ast/msg_ttl_ms");
if msg_ttl > 0 {
    let new_ttl = msg_ttl - dt;
    if new_ttl <= 0 { scene.set("game-message", "text.content", ""); }
    game.set("/ast/msg_ttl_ms", new_ttl);
}

// ── Wave spawning ──────────────────────────────────────────

let score = game.get("/ast/score");
let lives = game.get("/ast/lives");
let wave  = game.get("/ast/wave");
let ship_x = game.get("/ast/ship_x");
let ship_y = game.get("/ast/ship_y");

let active_count = world.count_kind("asteroid");

let last_diag_ms = game.get("/ast/last_diag_ms");
if now - last_diag_ms >= 1000 {
    game.set("/ast/last_diag_ms", now);
    diag.info(`stats lives=${lives} wave=${wave} score=${score} active=${active_count} bullets=${world.count_kind("bullet")}`);
}

if active_count <= 0 {
    wave += 1;
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
        spawn_asteroid(world, scene,
            respawn_x(rng, min_x, max_x, to_i(ship_x)),
            respawn_y(rng / 3, min_y, max_y, to_i(ship_y)),
            speed_for_seed(rng + 5),
            speed_for_seed(rng + 13),
            (rng / 11) % 4,
            if idx < 2 { 3 } else if idx < 5 { 2 } else { 1 }
        );
    }
}

// ── Game over ──────────────────────────────────────────────

if lives <= 0 {
    diag.warn(`game over score=${score} wave=${wave}`);
    persist.push("/asteroids/highscores", #{ score: score, wave: wave, at_ms: now });
    game.jump("asteroids.highscores");
    return ();
}
