// Asteroids shared helpers
// Imported by asteroids-game-loop.rhai and asteroids-render-sync.rhai

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

// ── Score lookup ──────────────────────────────────────────
fn asteroid_score(size) {
    if size >= 3 { return 20; }
    if size == 2 { return 50; }
    100
}

// ── Level config loader ───────────────────────────────────
// All values are in real units (no fixed-point conversion needed).
// Level YAML is the single source of truth for tunable gameplay values.
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
