// ── Fractal Noise Terrain Scene ───────────────────────────────────────────────
// S       → toggle settings panel
// ESC     → back to menu
// Ctrl+F  → toggle orbit mode (drag to rotate, +/- to zoom)
// ↑ / ↓   → cycle selected parameter (when panel open)
// ← / →   → adjust selected parameter value

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.panel_visible = false;
    local.sel        = 0;
    local.amplitude  = 2.5;
    local.frequency  = 3.5;
    local.octaves    = 3.0;
    local.roughness  = 0.65;
    local.lacunarity = 2.0;
    local.seed_x     = 0.0;
    local.seed_z     = 0.0;
    local.plateau    = 0.0;
    local.sea_level  = 0.0;
    local.scale_x    = 1.0;
    local.scale_z    = 1.0;
    local.ridge      = false;
}

let panel_visible = local.panel_visible ?? false;
let sel        = local.sel        ?? 0;
let amplitude  = local.amplitude  ?? 2.5;
let frequency  = local.frequency  ?? 3.5;
let octaves    = local.octaves    ?? 3.0;
let roughness  = local.roughness  ?? 0.65;
let lacunarity = local.lacunarity ?? 2.0;
let seed_x     = local.seed_x     ?? 0.0;
let seed_z     = local.seed_z     ?? 0.0;
let plateau    = local.plateau    ?? 0.0;
let sea_level  = local.sea_level  ?? 0.0;
let scale_x    = local.scale_x    ?? 1.0;
let scale_z    = local.scale_z    ?? 1.0;
let ridge      = local.ridge      ?? false;

// ── Navigation ────────────────────────────────────────────────────────────────
if input.just_pressed("Escape") || input.just_pressed("Backspace") {
    game.jump("terrain-menu");
}

// ── Toggle settings panel (S) — only when not in orbit mode ──────────────────
if !orbit_active && input.just_pressed("s") {
    panel_visible = !panel_visible;
    local.panel_visible = panel_visible;
}
if orbit_active { panel_visible = false; }

scene.set("settings-panel-box", "visible", panel_visible);
scene.set("orbit-hint", "visible", orbit_active);

// ── Panel controls ────────────────────────────────────────────────────────────
if panel_visible && !orbit_active {
    let dt = frame_ms.to_float() / 1000.0;
    let NUM_PARAMS = 12;

    if input.just_pressed("Up")   { sel = (sel + NUM_PARAMS - 1) % NUM_PARAMS; local.sel = sel; }
    if input.just_pressed("Down") { sel = (sel + 1) % NUM_PARAMS; local.sel = sel; }

    if sel == 0 {
        if input.down("Left")         { amplitude = (amplitude - 1.5 * dt).max(0.1); local.amplitude = amplitude; }
        if input.down("Right")        { amplitude = (amplitude + 1.5 * dt).min(5.0); local.amplitude = amplitude; }
    } else if sel == 1 {
        if input.down("Left")         { frequency = (frequency - 2.0 * dt).max(0.1); local.frequency = frequency; }
        if input.down("Right")        { frequency = (frequency + 2.0 * dt).min(8.0); local.frequency = frequency; }
    } else if sel == 2 {
        if input.just_pressed("Left") { octaves = (octaves - 1.0).max(1.0); local.octaves = octaves; }
        if input.just_pressed("Right"){ octaves = (octaves + 1.0).min(3.0); local.octaves = octaves; }
    } else if sel == 3 {
        if input.down("Left")         { roughness = (roughness - 0.5 * dt).max(0.0); local.roughness = roughness; }
        if input.down("Right")        { roughness = (roughness + 0.5 * dt).min(1.0); local.roughness = roughness; }
    } else if sel == 4 {
        if input.down("Left")         { lacunarity = (lacunarity - 1.0 * dt).max(1.0); local.lacunarity = lacunarity; }
        if input.down("Right")        { lacunarity = (lacunarity + 1.0 * dt).min(4.0); local.lacunarity = lacunarity; }
    } else if sel == 5 {
        if input.down("Left")         { seed_x = (seed_x - 2.0 * dt).max(-10.0); local.seed_x = seed_x; }
        if input.down("Right")        { seed_x = (seed_x + 2.0 * dt).min(10.0);  local.seed_x = seed_x; }
    } else if sel == 6 {
        if input.down("Left")         { seed_z = (seed_z - 2.0 * dt).max(-10.0); local.seed_z = seed_z; }
        if input.down("Right")        { seed_z = (seed_z + 2.0 * dt).min(10.0);  local.seed_z = seed_z; }
    } else if sel == 7 {
        if input.down("Left")         { plateau = (plateau - 0.5 * dt).max(0.0); local.plateau = plateau; }
        if input.down("Right")        { plateau = (plateau + 0.5 * dt).min(1.0); local.plateau = plateau; }
    } else if sel == 8 {
        if input.down("Left")         { sea_level = (sea_level - 0.5 * dt).max(0.0); local.sea_level = sea_level; }
        if input.down("Right")        { sea_level = (sea_level + 0.5 * dt).min(1.0); local.sea_level = sea_level; }
    } else if sel == 9 {
        if input.down("Left")         { scale_x = (scale_x - 1.0 * dt).max(0.25); local.scale_x = scale_x; }
        if input.down("Right")        { scale_x = (scale_x + 1.0 * dt).min(4.0);  local.scale_x = scale_x; }
    } else if sel == 10 {
        if input.down("Left")         { scale_z = (scale_z - 1.0 * dt).max(0.25); local.scale_z = scale_z; }
        if input.down("Right")        { scale_z = (scale_z + 1.0 * dt).min(4.0);  local.scale_z = scale_z; }
    } else {
        if input.just_pressed("Left") || input.just_pressed("Right") { ridge = !ridge; local.ridge = ridge; }
    }

    let TRACK_W = 170.0;
    let TRACK_X = 4.0;

    let a0 = if sel ==  0 { ">" } else { " " };
    let a1 = if sel ==  1 { ">" } else { " " };
    let a2 = if sel ==  2 { ">" } else { " " };
    let a3 = if sel ==  3 { ">" } else { " " };
    let a4 = if sel ==  4 { ">" } else { " " };
    let a5 = if sel ==  5 { ">" } else { " " };
    let a6 = if sel ==  6 { ">" } else { " " };
    let a7 = if sel ==  7 { ">" } else { " " };
    let a8 = if sel ==  8 { ">" } else { " " };
    let a9 = if sel ==  9 { ">" } else { " " };
    let a10 = if sel == 10 { ">" } else { " " };
    let a11 = if sel == 11 { ">" } else { " " };

    scene.set("amp-label",   "text.content", `${a0} AMPLITUDE  ${(amplitude * 10.0).round() / 10.0}`);
    scene.set("freq-label",  "text.content", `${a1} FREQUENCY  ${(frequency * 10.0).round() / 10.0}`);
    scene.set("oct-label",   "text.content", `${a2} OCTAVES    ${octaves.round()}`);
    scene.set("rough-label", "text.content", `${a3} ROUGHNESS  ${(roughness * 100.0).round()}%`);
    scene.set("lac-label",   "text.content", `${a4} LACUNARITY ${(lacunarity * 10.0).round() / 10.0}`);
    scene.set("seedx-label", "text.content", `${a5} SEED X     ${(seed_x * 10.0).round() / 10.0}`);
    scene.set("seedz-label", "text.content", `${a6} SEED Z     ${(seed_z * 10.0).round() / 10.0}`);
    scene.set("plat-label",  "text.content", `${a7} PLATEAU    ${(plateau * 100.0).round()}%`);
    scene.set("sea-label",   "text.content", `${a8} SEA LEVEL  ${(sea_level * 100.0).round()}%`);
    scene.set("scx-label",   "text.content", `${a9} SCALE X    ${(scale_x * 10.0).round() / 10.0}`);
    scene.set("scz-label",   "text.content", `${a10} SCALE Z    ${(scale_z * 10.0).round() / 10.0}`);
    let ridge_str = if ridge { "ON " } else { "OFF" };
    scene.set("ridge-label", "text.content", `${a11} RIDGE MODE [${ridge_str}]`);

    scene.set("amp-handle",   "position.x", (TRACK_X + (amplitude - 0.1) / 4.9 * TRACK_W).round());
    scene.set("freq-handle",  "position.x", (TRACK_X + (frequency - 0.1) / 7.9 * TRACK_W).round());
    scene.set("oct-handle",   "position.x", (TRACK_X + (octaves - 1.0) / 2.0 * TRACK_W).round());
    scene.set("rough-handle", "position.x", (TRACK_X + roughness * TRACK_W).round());
    scene.set("lac-handle",   "position.x", (TRACK_X + (lacunarity - 1.0) / 3.0 * TRACK_W).round());
    scene.set("seedx-handle", "position.x", (TRACK_X + (seed_x + 10.0) / 20.0 * TRACK_W).round());
    scene.set("seedz-handle", "position.x", (TRACK_X + (seed_z + 10.0) / 20.0 * TRACK_W).round());
    scene.set("plat-handle",  "position.x", (TRACK_X + plateau * TRACK_W).round());
    scene.set("sea-handle",   "position.x", (TRACK_X + sea_level * TRACK_W).round());
    scene.set("scx-handle",   "position.x", (TRACK_X + (scale_x - 0.25) / 3.75 * TRACK_W).round());
    scene.set("scz-handle",   "position.x", (TRACK_X + (scale_z - 0.25) / 3.75 * TRACK_W).round());
    scene.set("ridge-handle", "position.x", if ridge { TRACK_X + TRACK_W } else { TRACK_X });

    scene.set("terrain-mesh", "terrain.amplitude",  amplitude);
    scene.set("terrain-mesh", "terrain.frequency",  frequency);
    scene.set("terrain-mesh", "terrain.octaves",    octaves);
    scene.set("terrain-mesh", "terrain.roughness",  roughness);
    scene.set("terrain-mesh", "terrain.lacunarity", lacunarity);
    scene.set("terrain-mesh", "terrain.seed_x",     seed_x);
    scene.set("terrain-mesh", "terrain.seed_z",     seed_z);
    scene.set("terrain-mesh", "terrain.plateau",    plateau);
    scene.set("terrain-mesh", "terrain.sea_level",  sea_level);
    scene.set("terrain-mesh", "terrain.scale_x",    scale_x);
    scene.set("terrain-mesh", "terrain.scale_z",    scale_z);
    scene.set("terrain-mesh", "terrain.ridge",      ridge);
}
