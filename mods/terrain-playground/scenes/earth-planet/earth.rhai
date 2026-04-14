// ── Earth-Like Planet Scene ───────────────────────────────────────────────────
// S       → toggle world params panel
// ESC     → back to menu
// Ctrl+F  → toggle orbit/free-look camera
// ↑ / ↓   → cycle selected parameter (when panel open)
// ← / →   → adjust selected parameter value
//
// Uses world:// URI — full biome/climate pipeline (engine-terrain):
//   elevation noise → mountain ridges → climate → biome classification → face colors
// Changing any param regenerates the planet mesh (cache miss → new heightmap).

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized           = true;
    local.panel_visible         = false;
    local.sel                   = 0;
    local.seed                  = 0.0;
    local.ocean_fraction        = 0.55;
    local.continent_scale       = 2.5;
    local.continent_warp        = 0.65;
    local.mountain_strength     = 0.45;
    local.displacement_scale    = 0.22;
}

let panel_visible       = local.panel_visible       ?? false;
let sel                 = local.sel                 ?? 0;
let seed                = local.seed                ?? 0.0;
let ocean_fraction      = local.ocean_fraction      ?? 0.55;
let continent_scale     = local.continent_scale     ?? 2.5;
let continent_warp      = local.continent_warp      ?? 0.65;
let mountain_strength   = local.mountain_strength   ?? 0.45;
let displacement_scale  = local.displacement_scale  ?? 0.22;

// ── Navigation ────────────────────────────────────────────────────────────────
if input.just_pressed("Escape") || input.just_pressed("Backspace") {
    game.jump("terrain-menu");
}

// ── Toggle settings panel (S) ─────────────────────────────────────────────────
if !orbit_active && input.just_pressed("s") {
    panel_visible = !panel_visible;
    local.panel_visible = panel_visible;
}
if orbit_active { panel_visible = false; }

scene.set("earth-settings-box", "visible", panel_visible);

// ── Panel controls ────────────────────────────────────────────────────────────
if panel_visible && !orbit_active {
    let dt = frame_ms.to_float() / 1000.0;
    let NUM_PARAMS = 6;

    if input.just_pressed("Up")   { sel = (sel + NUM_PARAMS - 1) % NUM_PARAMS; local.sel = sel; }
    if input.just_pressed("Down") { sel = (sel + 1) % NUM_PARAMS; local.sel = sel; }

    if sel == 0 {
        // Seed: integer steps
        if input.just_pressed("Left")  { seed = (seed - 1.0).max(0.0);  local.seed = seed; }
        if input.just_pressed("Right") { seed = (seed + 1.0).min(99.0); local.seed = seed; }
    } else if sel == 1 {
        if input.down("Left")  { ocean_fraction = (ocean_fraction - 0.3 * dt).max(0.1); local.ocean_fraction = ocean_fraction; }
        if input.down("Right") { ocean_fraction = (ocean_fraction + 0.3 * dt).min(0.9); local.ocean_fraction = ocean_fraction; }
    } else if sel == 2 {
        if input.down("Left")  { continent_scale = (continent_scale - 1.5 * dt).max(0.5); local.continent_scale = continent_scale; }
        if input.down("Right") { continent_scale = (continent_scale + 1.5 * dt).min(10.0); local.continent_scale = continent_scale; }
    } else if sel == 3 {
        if input.down("Left")  { continent_warp = (continent_warp - 0.8 * dt).max(0.0); local.continent_warp = continent_warp; }
        if input.down("Right") { continent_warp = (continent_warp + 0.8 * dt).min(2.0); local.continent_warp = continent_warp; }
    } else if sel == 4 {
        if input.down("Left")  { mountain_strength = (mountain_strength - 0.5 * dt).max(0.0); local.mountain_strength = mountain_strength; }
        if input.down("Right") { mountain_strength = (mountain_strength + 0.5 * dt).min(1.0); local.mountain_strength = mountain_strength; }
    } else {
        if input.down("Left")  { displacement_scale = (displacement_scale - 0.2 * dt).max(0.0); local.displacement_scale = displacement_scale; }
        if input.down("Right") { displacement_scale = (displacement_scale + 0.2 * dt).min(0.6); local.displacement_scale = displacement_scale; }
    }

    let TRACK_W = 170.0;
    let TRACK_X = 4.0;

    let a0 = if sel == 0 { ">" } else { " " };
    let a1 = if sel == 1 { ">" } else { " " };
    let a2 = if sel == 2 { ">" } else { " " };
    let a3 = if sel == 3 { ">" } else { " " };
    let a4 = if sel == 4 { ">" } else { " " };
    let a5 = if sel == 5 { ">" } else { " " };

    scene.set("ep-seed-label",    "text.content", `${a0} SEED           ${seed.round()}`);
    scene.set("ep-ocean-label",   "text.content", `${a1} OCEAN          ${(ocean_fraction * 100.0).round()}%`);
    scene.set("ep-cscale-label",  "text.content", `${a2} CONTINENT SZ   ${(continent_scale * 10.0).round() / 10.0}`);
    scene.set("ep-cwarp-label",   "text.content", `${a3} COAST CHAOS    ${(continent_warp * 10.0).round() / 10.0}`);
    scene.set("ep-mstr-label",    "text.content", `${a4} MTN STRENGTH   ${(mountain_strength * 100.0).round()}%`);
    scene.set("ep-disp-label",    "text.content", `${a5} DISPLACEMENT   ${(displacement_scale * 100.0).round()}%`);

    scene.set("ep-seed-handle",   "position.x", (TRACK_X + seed / 99.0 * TRACK_W).round());
    scene.set("ep-ocean-handle",  "position.x", (TRACK_X + (ocean_fraction - 0.1) / 0.8 * TRACK_W).round());
    scene.set("ep-cscale-handle", "position.x", (TRACK_X + (continent_scale - 0.5) / 9.5 * TRACK_W).round());
    scene.set("ep-cwarp-handle",  "position.x", (TRACK_X + continent_warp / 2.0 * TRACK_W).round());
    scene.set("ep-mstr-handle",   "position.x", (TRACK_X + mountain_strength * TRACK_W).round());
    scene.set("ep-disp-handle",   "position.x", (TRACK_X + displacement_scale / 0.6 * TRACK_W).round());

    // Push world params — URI key changes → planet regenerated on next frame
    scene.set("earth-mesh", "world.seed",               seed);
    scene.set("earth-mesh", "world.ocean_fraction",     ocean_fraction);
    scene.set("earth-mesh", "world.continent_scale",    continent_scale);
    scene.set("earth-mesh", "world.continent_warp",     continent_warp);
    scene.set("earth-mesh", "world.mountain_strength",  mountain_strength);
    scene.set("earth-mesh", "world.displacement_scale", displacement_scale);
}
