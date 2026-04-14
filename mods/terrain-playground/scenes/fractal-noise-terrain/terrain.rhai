// ── Fractal Noise Terrain Scene ───────────────────────────────────────────────
// S           → toggle settings panel
// ESC         → back to menu
// Ctrl+F      → toggle orbit mode (drag to rotate, +/- to zoom)
// ↑ / ↓       → cycle selected parameter (when panel open)
// ← / →       → adjust selected parameter value

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized    = true;
    local.panel_visible  = false;
    local.sel            = 0;
    local.amplitude      = 2.5;
    local.frequency      = 3.5;
    local.octaves        = 3.0;
    local.roughness      = 0.65;
}

let panel_visible = local.panel_visible ?? false;
let sel           = local.sel           ?? 0;
let amplitude     = local.amplitude     ?? 2.5;
let frequency     = local.frequency     ?? 3.5;
let octaves       = local.octaves       ?? 3.0;
let roughness     = local.roughness     ?? 0.65;

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

    if input.just_pressed("Up")   { sel = (sel + 3) % 4; local.sel = sel; }
    if input.just_pressed("Down") { sel = (sel + 1) % 4; local.sel = sel; }

    let AMP_RATE   = 1.5;
    let FREQ_RATE  = 2.0;
    let OCT_RATE   = 2.0;
    let ROUGH_RATE = 0.8;

    if sel == 0 {
        if input.down("Left")  { amplitude = (amplitude - AMP_RATE * dt).max(0.1);  local.amplitude = amplitude; }
        if input.down("Right") { amplitude = (amplitude + AMP_RATE * dt).min(5.0);  local.amplitude = amplitude; }
    } else if sel == 1 {
        if input.down("Left")  { frequency = (frequency - FREQ_RATE * dt).max(0.1); local.frequency = frequency; }
        if input.down("Right") { frequency = (frequency + FREQ_RATE * dt).min(8.0); local.frequency = frequency; }
    } else if sel == 2 {
        if input.down("Left")  {
            let next = (octaves - OCT_RATE * dt).max(1.0);
            octaves = if next < 1.5 { 1.0 } else if next < 2.5 { 2.0 } else { 3.0 };
            local.octaves = octaves;
        }
        if input.down("Right") {
            let next = (octaves + OCT_RATE * dt).min(3.0);
            octaves = if next > 2.5 { 3.0 } else if next > 1.5 { 2.0 } else { 1.0 };
            local.octaves = octaves;
        }
    } else {
        if input.down("Left")  { roughness = (roughness - ROUGH_RATE * dt).max(0.0); local.roughness = roughness; }
        if input.down("Right") { roughness = (roughness + ROUGH_RATE * dt).min(1.0); local.roughness = roughness; }
    }

    let TRACK_W = 170.0;
    let TRACK_X =   4.0;

    let a_pfx = if sel == 0 { ">" } else { " " };
    let f_pfx = if sel == 1 { ">" } else { " " };
    let o_pfx = if sel == 2 { ">" } else { " " };
    let r_pfx = if sel == 3 { ">" } else { " " };

    let amp_val   = (amplitude * 10.0).round() / 10.0;
    let freq_val  = (frequency * 10.0).round() / 10.0;
    let oct_val   = octaves.round();
    let rough_val = (roughness * 100.0).round();

    scene.set("amp-label",   "text.content", `${a_pfx} AMPLITUDE  ${amp_val}`);
    scene.set("freq-label",  "text.content", `${f_pfx} FREQUENCY  ${freq_val}`);
    scene.set("oct-label",   "text.content", `${o_pfx} OCTAVES    ${oct_val}`);
    scene.set("rough-label", "text.content", `${r_pfx} ROUGHNESS  ${rough_val}%`);

    let amp_t   = (amplitude - 0.1) / 4.9;
    scene.set("amp-handle",   "position.x", (TRACK_X + amp_t   * TRACK_W).round());
    let freq_t  = (frequency - 0.1) / 7.9;
    scene.set("freq-handle",  "position.x", (TRACK_X + freq_t  * TRACK_W).round());
    let oct_t   = (octaves - 1.0) / 2.0;
    scene.set("oct-handle",   "position.x", (TRACK_X + oct_t   * TRACK_W).round());
    let rough_t = roughness;
    scene.set("rough-handle", "position.x", (TRACK_X + rough_t * TRACK_W).round());

    // Push terrain params to engine — orbit camera sets obj.yaw/pitch/camera-distance.
    scene.set("terrain-mesh", "terrain.amplitude", amplitude);
    scene.set("terrain-mesh", "terrain.frequency",  frequency);
    scene.set("terrain-mesh", "terrain.octaves",    octaves);
    scene.set("terrain-mesh", "terrain.roughness",  roughness);
}
