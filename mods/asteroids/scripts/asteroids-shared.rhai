const VELOCITY_SCALE = 60.0;

fn flash_fill_hex(flash_ms, flash_total_ms) {
    if flash_ms <= 0 { return "black"; }
    let total = if flash_total_ms <= 0 { 1 } else { flash_total_ms };
    let ratio = (flash_ms * 100) / total;
    if ratio > 66 { return "#808080"; }
    if ratio > 33 { return "#555555"; }
    "#2a2a2a"
}

fn asteroid_stroke_hex(flash_ms, flash_total_ms) {
    if flash_ms <= 0 { return "#cccccc"; }
    let total = if flash_total_ms <= 0 { 1 } else { flash_total_ms };
    let ratio = (flash_ms * 100) / total;
    if ratio > 66 { return "#ffffff"; }
    if ratio > 33 { return "#e2e2e2"; }
    "#d0d0d0"
}

fn smoke_colour_hex(ttl_ms, max_ttl_ms) {
    if ttl_ms <= 0 { return "#000000"; }
    let total = if max_ttl_ms <= 0 { 1 } else { max_ttl_ms };
    let ratio = (ttl_ms * 100) / total;
    if ratio > 80 { return "#ffffff"; }
    if ratio > 60 { return "#c0c0c0"; }
    if ratio > 40 { return "#808080"; }
    if ratio > 20 { return "#404040"; }
    "#1a1a1a"
}

fn smoke_points(radius, ttl_ms, max_ttl_ms) {
    let total = if max_ttl_ms <= 0 { 1 } else { max_ttl_ms };
    let r = (radius * ttl_ms) / total;
    if r < 1 { r = 1; }
    if r == 1 {
        return [[0, -1], [1, 0], [0, 1], [-1, 0]];
    }
    if r == 2 {
        return [[0, -2], [1, -1], [2, 0], [1, 1], [0, 2], [-1, 1], [-2, 0], [-1, -1]];
    }
    [[0, -3], [2, -2], [3, 0], [2, 2], [0, 3], [-2, 2], [-3, 0], [-2, -2]]
}

fn crack_visual_id(ast_data, idx) {
    let key = `crack_visual_${idx}`;
    if ast_data.contains(key) { ast_data[key] } else { () }
}

fn hide_crack_visuals(scene, ast_data) {
    for i in 0..3 {
        let frag_id = crack_visual_id(ast_data, i);
        if frag_id.type_of() == "string" {
            scene.set_visible(frag_id, false);
        }
    }
}

fn asteroid_score(size) {
    if size >= 3 { return 20; }
    if size == 2 { return 50; }
    100
}

fn asteroid_radius(size) {
    if size >= 3 { return 15; }
    if size == 2 { return 11; }
    if size == 1 { return 7; }
    4
}

fn asteroid_base_points(shape) {
    if shape == 1 {
        return [[6, -7], [8, -1], [7, 5], [2, 8], [-4, 7], [-8, 1], [-6, -5], [-1, -8]];
    }
    if shape == 2 {
        return [[5, -8], [8, -3], [8, 3], [3, 8], [-3, 7], [-8, 2], [-7, -4], [-2, -8]];
    }
    if shape == 3 {
        return [[4, -8], [8, -4], [7, 2], [4, 8], [-2, 7], [-8, 3], [-7, -3], [-3, -8]];
    }
    [[5, -8], [8, -2], [7, 4], [3, 8], [-4, 7], [-8, 2], [-7, -4], [-2, -8]]
}

fn asteroid_points(shape, size) {
    let scale = asteroid_radius(size);
    let out = [];
    for pt in asteroid_base_points(shape) {
        out.push([(pt[0] * scale) / 8, (pt[1] * scale) / 8]);
    }
    out
}

fn ship_points(heading) {
    rotate_points([[0, -7], [4, 5], [0, 2], [-4, 5]], heading)
}

fn heading_vec(heading) {
    #{
        x: sin32(heading) / 1024.0,
        y: -sin32(heading + 8) / 1024.0
    }
}

fn wrap_heading(heading) {
    let wrapped = heading % 32;
    if wrapped < 0 { return wrapped + 32; }
    wrapped
}

fn tick_heading_anim(ast, dt_ms) {
    let rot_phase = ast.get_i("/rot_phase", 0);
    let rot_accum_ms = ast.get_i("/rot_accum_ms", 0);
    let rot_step_ms = ast.get_i("/rot_step_ms", 0);
    let rot_speed = ast.get_i("/rot_speed", 1);

    if rot_step_ms <= 0 || dt_ms <= 0 {
        return #{ rot_phase: rot_phase, rot_accum_ms: rot_accum_ms };
    }

    rot_accum_ms += dt_ms;
    while rot_accum_ms >= rot_step_ms {
        rot_accum_ms -= rot_step_ms;
        rot_phase = wrap_heading(rot_phase + rot_speed);
    }

    #{ rot_phase: rot_phase, rot_accum_ms: rot_accum_ms }
}

fn load_game_config(level) {
    let lives = if level.has("/player/lives") { level.get("/player/lives") } else { 3 };
    let thrust_power = if level.has("/player/thrust_power") { to_float(level.get("/player/thrust_power")) } else { 120.0 };
    let max_speed = if level.has("/player/max_speed") { to_float(level.get("/player/max_speed")) } else { 270.0 };
    let turn_step_ms = if level.has("/player/turn_step_ms") { level.get("/player/turn_step_ms") } else { 25 };

    let shot_cooldown_ms = if level.has("/combat/shot_cooldown_ms") { level.get("/combat/shot_cooldown_ms") } else { 200 };
    let bullet_ttl_ms = if level.has("/combat/bullet_ttl_ms") { level.get("/combat/bullet_ttl_ms") } else { 900 };
    let bullet_speed = if level.has("/combat/bullet_speed") { to_float(level.get("/combat/bullet_speed")) } else { 246.0 };
    let crack_duration_ms = if level.has("/combat/crack_duration_ms") { level.get("/combat/crack_duration_ms") } else { 1000 };
    let invulnerability_ms = if level.has("/combat/invulnerability_ms") { level.get("/combat/invulnerability_ms") } else { 3000 };

    let wave_base = if level.has("/waves/base_spawn_count") { level.get("/waves/base_spawn_count") } else { 4 };
    let wave_min = if level.has("/waves/min_spawn_count") { level.get("/waves/min_spawn_count") } else { 5 };
    let wave_max = if level.has("/waves/max_spawn_count") { level.get("/waves/max_spawn_count") } else { 10 };
    if wave_min > wave_max { wave_min = wave_max; }

    #{
        lives: lives,
        thrust_power: thrust_power,
        max_speed: max_speed,
        turn_step_ms: turn_step_ms,
        shot_cooldown_ms: shot_cooldown_ms,
        bullet_ttl_ms: bullet_ttl_ms,
        bullet_speed: bullet_speed,
        crack_duration_ms: crack_duration_ms,
        invulnerability_ms: invulnerability_ms,
        wave_base: wave_base,
        wave_min: wave_min,
        wave_max: wave_max
    }
}

fn spawn_ship(world, cfg) {
    world.spawn_prefab("ship", #{ cfg: cfg, invulnerable_ms: cfg["invulnerability_ms"] })
}

fn fire_weapon(world, audio, ship_id, cfg) {
    if ship_id <= 0 { return 0; }
    let ship = world.entity(ship_id);
    if !ship.exists() || !ship.cooldown_ready("shot") { return 0; }
    if world.count_kind("bullet") >= 8 { return 0; }

    let t = ship.transform();
    let dir = ship.heading_vector();
    let vel = ship.physics.velocity();
    let ship_vx = if vel.len > 0 { vel[0] } else { 0.0 };
    let ship_vy = if vel.len > 1 { vel[1] } else { 0.0 };
    let bullet_speed = cfg["bullet_speed"];

    let bullet_id = world.spawn_prefab("bullet", #{
        x: t.x + (dir["x"] * 9.0),
        y: t.y + (dir["y"] * 9.0),
        vx: (ship_vx + (dir["x"] * bullet_speed)) / VELOCITY_SCALE,
        vy: (ship_vy + (dir["y"] * bullet_speed)) / VELOCITY_SCALE,
        ttl_ms: cfg["bullet_ttl_ms"]
    });
    if bullet_id <= 0 { return 0; }

    ship.cooldown_start("shot", cfg["shot_cooldown_ms"]);
    audio.event("gameplay.ship.shoot", 1.0);
    bullet_id
}

fn emit_thrust_smoke(world, ship_id, thrust_ms) {
    if ship_id <= 0 { return 0; }
    let ship = world.entity(ship_id);
    if !ship.exists() || !ship.cooldown_ready("smoke") { return 0; }
    if world.count_kind("smoke") >= 10 { return 0; }

    let base_cd = 48;
    let min_cd = 16;
    let ramp_t = clamp_i((thrust_ms * 100) / 2000, 0, 100);
    let cooldown_ms = base_cd - (((base_cd - min_cd) * ramp_t) / 100);

    let t = ship.transform();
    let dir = ship.heading_vector();
    let vel = ship.physics.velocity();
    let ship_vx = if vel.len > 0 { vel[0] } else { 0.0 };
    let ship_vy = if vel.len > 1 { vel[1] } else { 0.0 };

    ship.cooldown_start("smoke", cooldown_ms);
    world.spawn_prefab("smoke", #{
        x: t.x - (dir["x"] * 6.0),
        y: t.y - (dir["y"] * 6.0),
        vx: (ship_vx - (dir["x"] * 21.0)) / VELOCITY_SCALE,
        vy: (ship_vy - (dir["y"] * 21.0)) / VELOCITY_SCALE,
        ttl_ms: 380,
        max_ttl_ms: 380,
        radius: 3
    })
}

fn emit_disintegration(world, x, y) {
    let ids = [];
    for i in 0..12 {
        if world.count_kind("smoke") >= 20 { break; }
        let dir = heading_vec((i * 32) / 12);
        let smoke_id = world.spawn_prefab("smoke", #{
            x: x,
            y: y,
            vx: dir["x"] * 0.5,
            vy: dir["y"] * 0.5,
            ttl_ms: 520,
            max_ttl_ms: 520,
            radius: 3
        });
        if smoke_id > 0 { ids.push(smoke_id); }
    }
    ids
}

fn mark_asteroid_hit(ast, crack_duration_ms) {
    let duration = if crack_duration_ms < 0 { 0 } else { crack_duration_ms };
    ast.set_many(#{
        flash_ms: duration,
        flash_total_ms: duration,
        split_pending: duration > 0
    });
}

fn handle_ship_hit(world, ship_id, asteroid_id, cfg) {
    if ship_id <= 0 || asteroid_id <= 0 { return false; }

    let ship = world.entity(ship_id);
    let asteroid = world.entity(asteroid_id);
    if !ship.exists() || !asteroid.exists() || ship.status_has("invulnerable") || asteroid.flag("split_pending") {
        return false;
    }

    ship.physics.set_velocity(0.0, 0.0);
    ship.status_add("invulnerable", cfg["invulnerability_ms"]);

    let asteroid_vel = asteroid.physics.velocity();
    let hit_vx = if asteroid_vel.len > 0 { asteroid_vel[0] } else { 0.0 };
    let hit_vy = if asteroid_vel.len > 1 { asteroid_vel[1] } else { 0.0 };
    asteroid.physics.set_velocity(-hit_vx * 60.0, -hit_vy * 60.0);
    mark_asteroid_hit(asteroid, cfg["crack_duration_ms"]);
    true
}

fn handle_bullet_hit(world, bullet_id, asteroid_id, crack_duration_ms, bullet_momentum_factor) {
    let result = #{ handled: false, asteroid_size: 0 };
    if asteroid_id <= 0 { return result; }

    let asteroid = world.entity(asteroid_id);
    if !asteroid.exists() || asteroid.flag("split_pending") {
        return result;
    }

    let asteroid_size = asteroid.get_i("/size", 1);
    if bullet_id > 0 {
        let bullet = world.entity(bullet_id);
        if bullet.exists() {
            let bullet_vel = bullet.physics.velocity();
            let bullet_vx = if bullet_vel.len > 0 { bullet_vel[0] } else { 0.0 };
            let bullet_vy = if bullet_vel.len > 1 { bullet_vel[1] } else { 0.0 };
            bullet.despawn();
            if bullet_momentum_factor > 0.0 {
                let divisor = if asteroid_size <= 0 { 1.0 } else { asteroid_size * 1.0 };
                asteroid.physics.add_velocity(
                    (bullet_vx * bullet_momentum_factor) / divisor,
                    (bullet_vy * bullet_momentum_factor) / divisor
                );
            }
        }
    }

    mark_asteroid_hit(asteroid, crack_duration_ms);
    result["handled"] = true;
    result["asteroid_size"] = asteroid_size;
    result
}

fn split_asteroid(world, asteroid_id) {
    let result = #{ handled: false, despawned: false, children: [] };
    if asteroid_id <= 0 { return result; }

    let asteroid = world.entity(asteroid_id);
    if !asteroid.exists() { return result; }

    let size = asteroid.get_i("/size", 1);
    let shape = asteroid.get_i("/shape", 0);
    if size <= 1 {
        asteroid.despawn();
        result["handled"] = true;
        result["despawned"] = true;
        return result;
    }

    let next_size = size - 1;
    let t = asteroid.transform();
    let vel = asteroid.physics.velocity();
    let base_vx = if vel.len > 0 { vel[0] } else { 0.0 };
    let base_vy = if vel.len > 1 { vel[1] } else { 0.0 };
    let base_heading = asteroid.get_i("/rot_phase", 0);
    let children = [];

    for i in 0..3 {
        let split_heading = wrap_heading(base_heading + (i * 11) - 11);
        let dir = heading_vec(split_heading);
        let spawn_offset = asteroid_radius(next_size) + 4;
        let child_id = world.spawn_prefab("asteroid", #{
            x: t.x + (dir["x"] * spawn_offset),
            y: t.y + (dir["y"] * spawn_offset),
            vx: (base_vx + (dir["x"] * (next_size + 2))) / VELOCITY_SCALE,
            vy: (base_vy + (dir["y"] * (next_size + 2))) / VELOCITY_SCALE,
            shape: wrap_heading(shape + i) % 4,
            size: next_size,
            rot_phase: split_heading,
            rot_accum_ms: 0
        });
        if child_id > 0 { children.push(child_id); }
    }

    asteroid.despawn();
    result["handled"] = true;
    result["despawned"] = true;
    result["children"] = children;
    result
}
