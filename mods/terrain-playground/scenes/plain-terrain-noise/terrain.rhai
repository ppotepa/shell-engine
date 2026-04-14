// ── Plain Terrain (Noise) Scene ───────────────────────────────────────────────
// S       → toggle settings panel
// ESC     → back to menu
// ↑ / ↓   → cycle selected parameter (when panel open)
// ← / →   → adjust selected parameter value

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.panel_visible = false;
    local.sel   = 0;        // 0 = pitch, 1 = speed
    local.pitch = -38.0;
    local.speed = 5.0;
}

let panel_visible = local.panel_visible ?? false;
let sel   = local.sel   ?? 0;
let pitch = local.pitch ?? -38.0;
let speed = local.speed ?? 5.0;

// ── Navigation ────────────────────────────────────────────────────────────────
if input.just_pressed("Escape") || input.just_pressed("Backspace") {
    game.jump("terrain-menu");
}

// ── Toggle panel ──────────────────────────────────────────────────────────────
if input.just_pressed("s") {
    panel_visible = !panel_visible;
    local.panel_visible = panel_visible;
}

// Re-apply every frame (object states reset to visible:true before behavior runs).
scene.set("settings-panel-box", "visible", panel_visible);

// ── Panel controls ────────────────────────────────────────────────────────────
if panel_visible {
    let dt = frame_ms.to_float() / 1000.0;

    // ↑ / ↓  cycle selected row
    if input.just_pressed("Up")   { sel = if sel == 0 { 1 } else { 0 }; local.sel = sel; }
    if input.just_pressed("Down") { sel = if sel == 0 { 1 } else { 0 }; local.sel = sel; }

    // ← / →  adjust selected value (continuous while held, dt-scaled)
    let PITCH_RATE = 30.0;   // degrees / sec
    let SPEED_RATE =  6.0;   // units / sec

    if sel == 0 {
        if input.down("Left")  { pitch = (pitch - PITCH_RATE * dt).max(-80.0); local.pitch = pitch; }
        if input.down("Right") { pitch = (pitch + PITCH_RATE * dt).min(-5.0);  local.pitch = pitch; }
    } else {
        if input.down("Left")  { speed = (speed - SPEED_RATE * dt).max(0.0);  local.speed = speed; }
        if input.down("Right") { speed = (speed + SPEED_RATE * dt).min(15.0); local.speed = speed; }
    }

    // ── Labels with selection indicator ───────────────────────────────────────
    let p_pfx = if sel == 0 { ">" } else { " " };
    let s_pfx = if sel == 1 { ">" } else { " " };
    scene.set("pitch-label", "text.content", `${p_pfx} PITCH ${pitch.round()}°`);
    scene.set("speed-label", "text.content", `${s_pfx} SPEED ${(speed * 10.0).round() / 10.0}°/s`);

    // ── Sync handle positions (track width = 170 px) ───────────────────────
    let pitch_t = (pitch - (-80.0)) / 75.0;
    scene.set("pitch-handle", "position.x", (pitch_t * 170.0).round());

    let speed_t = speed / 15.0;
    scene.set("speed-handle", "position.x", (speed_t * 170.0).round());
}
