// Asteroids shared helpers
// Imported by asteroids-game-loop.rhai and asteroids-render-sync.rhai

// ── Constants ──────────────────────────────────────────────
fn crack_duration_ms() { 1000 }

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
            scene.set(frag_id, "vector.visible", false);
        }
    }
}
