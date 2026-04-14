// ── Sphere Terrain Scene ──────────────────────────────────────────────────────
// S       → toggle settings panel
// ESC     → back to menu
// Ctrl+F  → toggle orbit/free-look camera
// ↑ / ↓   → cycle selected parameter (when panel open)
// ← / →   → adjust selected parameter value

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.panel_visible = false;
    local.sel       = 0;
    local.ambient   = 0.15;
    local.rot_speed = 5.0;
    local.light_x   = -0.45;
    local.light_y   =  0.70;
    local.light_z   = -0.85;
}

let panel_visible = local.panel_visible ?? false;
let sel       = local.sel       ?? 0;
let ambient   = local.ambient   ?? 0.15;
let rot_speed = local.rot_speed ?? 5.0;
let light_x   = local.light_x  ?? -0.45;
let light_y   = local.light_y  ??  0.70;
let light_z   = local.light_z  ?? -0.85;

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

scene.set("sphere-settings-box", "visible", panel_visible);

// ── Panel controls ────────────────────────────────────────────────────────────
if panel_visible && !orbit_active {
    let dt = frame_ms.to_float() / 1000.0;
    let NUM_PARAMS = 5;

    if input.just_pressed("Up")   { sel = (sel + NUM_PARAMS - 1) % NUM_PARAMS; local.sel = sel; }
    if input.just_pressed("Down") { sel = (sel + 1) % NUM_PARAMS; local.sel = sel; }

    if sel == 0 {
        if input.down("Left")  { ambient = (ambient - 0.3 * dt).max(0.0); local.ambient = ambient; }
        if input.down("Right") { ambient = (ambient + 0.3 * dt).min(1.0); local.ambient = ambient; }
    } else if sel == 1 {
        if input.down("Left")  { rot_speed = (rot_speed - 10.0 * dt).max(0.0);  local.rot_speed = rot_speed; }
        if input.down("Right") { rot_speed = (rot_speed + 10.0 * dt).min(60.0); local.rot_speed = rot_speed; }
    } else if sel == 2 {
        if input.down("Left")  { light_x = (light_x - 0.5 * dt).max(-1.0); local.light_x = light_x; }
        if input.down("Right") { light_x = (light_x + 0.5 * dt).min(1.0);  local.light_x = light_x; }
    } else if sel == 3 {
        if input.down("Left")  { light_y = (light_y - 0.5 * dt).max(-1.0); local.light_y = light_y; }
        if input.down("Right") { light_y = (light_y + 0.5 * dt).min(1.0);  local.light_y = light_y; }
    } else {
        if input.down("Left")  { light_z = (light_z - 0.5 * dt).max(-1.0); local.light_z = light_z; }
        if input.down("Right") { light_z = (light_z + 0.5 * dt).min(1.0);  local.light_z = light_z; }
    }

    let TRACK_W = 170.0;
    let TRACK_X = 4.0;

    let a0 = if sel == 0 { ">" } else { " " };
    let a1 = if sel == 1 { ">" } else { " " };
    let a2 = if sel == 2 { ">" } else { " " };
    let a3 = if sel == 3 { ">" } else { " " };
    let a4 = if sel == 4 { ">" } else { " " };

    scene.set("sp-amb-label", "text.content", `${a0} AMBIENT    ${(ambient * 100.0).round() / 100.0}`);
    scene.set("sp-rot-label", "text.content", `${a1} ROT SPEED  ${(rot_speed * 10.0).round() / 10.0}`);
    scene.set("sp-lx-label",  "text.content", `${a2} LIGHT X    ${(light_x * 100.0).round() / 100.0}`);
    scene.set("sp-ly-label",  "text.content", `${a3} LIGHT Y    ${(light_y * 100.0).round() / 100.0}`);
    scene.set("sp-lz-label",  "text.content", `${a4} LIGHT Z    ${(light_z * 100.0).round() / 100.0}`);

    scene.set("sp-amb-handle", "position.x", (TRACK_X + ambient * TRACK_W).round());
    scene.set("sp-rot-handle", "position.x", (TRACK_X + rot_speed / 60.0 * TRACK_W).round());
    scene.set("sp-lx-handle",  "position.x", (TRACK_X + (light_x + 1.0) / 2.0 * TRACK_W).round());
    scene.set("sp-ly-handle",  "position.x", (TRACK_X + (light_y + 1.0) / 2.0 * TRACK_W).round());
    scene.set("sp-lz-handle",  "position.x", (TRACK_X + (light_z + 1.0) / 2.0 * TRACK_W).round());

    scene.set("sphere-mesh", "obj.ambient",        ambient);
    scene.set("sphere-mesh", "obj.rotation-speed", rot_speed);
    scene.set("sphere-mesh", "obj.light.x",        light_x);
    scene.set("sphere-mesh", "obj.light.y",        light_y);
    scene.set("sphere-mesh", "obj.light.z",        light_z);
}
