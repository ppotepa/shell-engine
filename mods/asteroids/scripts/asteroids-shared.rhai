// Asteroids shared helpers
// Imported by asteroids-game-loop.rhai and asteroids-render-sync.rhai

// ── Constants ──────────────────────────────────────────────
fn CRACK_DURATION_MS() { 1000 }
fn velocity_scale() { 60.0 }

// ── Heading arithmetic (0-31 on 32-step circle) ───────────
fn wrap_heading32(v) {
    let out = v % 32;
    if out < 0 { out + 32 } else { out }
}

fn heading32_to_rad(h) {
    to_float(h) * (6.283185307179586 / 32.0)
}

fn fragment_heading_offset(idx) {
    if idx == 0 { return 0; }
    if idx == 1 { return 11; }
    21
}

// ── Visual colour helpers ─────────────────────────────────
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
    if ttl_ms <= 0 { return "#101010"; }
    let total = if max_ttl_ms <= 0 { 1 } else { max_ttl_ms };
    let ratio = (ttl_ms * 100) / total;
    if ratio > 75 { return "#8f8f8f"; }
    if ratio > 50 { return "#6f6f6f"; }
    if ratio > 25 { return "#4a4a4a"; }
    "#262626"
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

// ── Crack visual management ───────────────────────────────
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

// ── Level config helpers ──────────────────────────────────
fn level_i(level_api, path, fallback) {
    if level_api.has(path) {
        let value = level_api.get(path);
        if value.type_of() == "i64" { return value; }
    }
    fallback
}

fn next_seed(seed) {
    ((seed * 1103515245) + 12345) % 2147483647
}

fn load_game_config(level) {
    let base_lives = level_i(level, "/player/lives", 3);
    let shot_cooldown_ms = clamp_i(level_i(level, "/player/shot_cooldown_ms", 200), 200, 1000);
    let ship_thrust = to_float(clamp_i(level_i(level, "/player/thrust_fp", 2048), 60, 10000)) / 1024.0 * velocity_scale();
    let ship_max_speed = to_float(clamp_i(level_i(level, "/player/max_speed_fp", 4608), 1024, 14000)) / 1024.0 * velocity_scale();
    let turn_step_ms = clamp_i(level_i(level, "/player/turn_step_ms", 40), 16, 150);
    let wave_base_spawn_count = clamp_i(level_i(level, "/waves/base_spawn_count", 4), 1, 20);
    let wave_min_spawn_count = clamp_i(level_i(level, "/waves/min_spawn_count", 5), 1, 30);
    let wave_max_spawn_count = clamp_i(level_i(level, "/waves/max_spawn_count", 10), 1, 40);
    if wave_min_spawn_count > wave_max_spawn_count { wave_min_spawn_count = wave_max_spawn_count; }
    let bullet_ttl_ms = clamp_i(level_i(level, "/combat/bullet_ttl_ms", 900), 100, 6000);
    let bullet_speed = to_float(clamp_i(level_i(level, "/combat/bullet_speed_fp", 4200), 1000, 18000)) / 1024.0 * velocity_scale();
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

fn show_message(scene, game, text, ttl_ms) {
    game.set("/ast/msg_ttl_ms", ttl_ms);
    scene.set("game-message", "text.content", text);
}

fn spawn_crack_visuals(world, scene, ast_id) {
    // Guard: skip if crack visuals already spawned for this asteroid
    // (prevents re-activating crack visuals for existing asteroids when others split)
    let e = world.entity(ast_id);
    if e.get_b("/cracks_spawned", false) { return; }
    for i in 0..3 {
        let cv = `asteroid-${ast_id}-crack-${i}`;
        scene.spawn_object("asteroid-template", cv);
        world.bind_visual(ast_id, cv);
        world.set(ast_id, `/crack_visual_${i}`, cv);
    }
    world.set(ast_id, "/cracks_spawned", true);
}
