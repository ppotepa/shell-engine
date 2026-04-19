// ── Planet Generator ─────────────────────────────────────────────────────────
// 1/2/3/4        → switch model (Planet / Sphere / Cube / Suzanne)
// Q/W/E/Y/U       → switch tab (Continents / Mountains / Climate / Visual / Atmosphere)
// Mouse click     → button + tab/model selection where available
// F1-F7           → load preset (Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago)
// Mouse drag → orbit camera around planet (always active; drag outside HUD)
// +/-      → zoom orbit camera in/out
// Ctrl+F   → toggle free-look fly camera (WASD/QE to fly, mouse to look)
// R        → toggle planet auto-rotation on/off
// Delete   → reset to Earth defaults
//
// 8 multiplexed slider slots: content remapped per active tab.
// All gui widgets use min=0, max=1 (normalized). Rhai maps to/from param domains.

// ── Preset definitions ────────────────────────────────────────────────────────
fn preset_earth() {
    #{
        seed: 0.0, ocean: 0.55, cscale: 2.5, cwarp: 0.65, coct: 5.0,
        mscale: 6.0, mstr: 0.45, mroct: 5.0,
        moisture: 3.0, ice: 1.0, lapse: 0.6, rain: 0.4,
        disp: 0.22, coloring: 0.0, rotspeed: 3.0,
        sunaz: 210.0, sunel: 40.0, ambient: 0.12,
        atmo_color: 2.0, atmo_density: 0.48, atmo_height: 0.11, atmo_forward_scatter: 0.78, atmo_absorption: 0.12, atmo_rayleigh_falloff: 0.22, atmo_haze_amount: 0.22, atmo_haze_falloff: 0.46
    }
}
fn preset_mars() {
    #{
        seed: 42.0, ocean: 0.05, cscale: 3.5, cwarp: 0.3, coct: 4.0,
        mscale: 8.0, mstr: 0.65, mroct: 6.0,
        moisture: 1.0, ice: 0.1, lapse: 0.3, rain: 0.05,
        disp: 0.38, coloring: 1.0, rotspeed: 3.0,
        sunaz: 190.0, sunel: 30.0, ambient: 0.08,
        atmo_color: 0.0, atmo_density: 0.20, atmo_height: 0.08, atmo_forward_scatter: 0.18, atmo_absorption: 0.06, atmo_rayleigh_falloff: 0.17, atmo_haze_amount: 0.14, atmo_haze_falloff: 0.38
    }
}
fn preset_ocean() {
    #{
        seed: 7.0, ocean: 0.88, cscale: 1.5, cwarp: 1.2, coct: 3.0,
        mscale: 4.0, mstr: 0.15, mroct: 3.0,
        moisture: 5.0, ice: 0.8, lapse: 0.7, rain: 0.6,
        disp: 0.08, coloring: 0.0, rotspeed: 2.0,
        sunaz: 220.0, sunel: 50.0, ambient: 0.15,
        atmo_color: 2.0, atmo_density: 0.65, atmo_height: 0.20, atmo_forward_scatter: 0.72, atmo_absorption: 0.28, atmo_rayleigh_falloff: 0.25, atmo_haze_amount: 0.45, atmo_haze_falloff: 0.56
    }
}
fn preset_desert() {
    #{
        seed: 13.0, ocean: 0.15, cscale: 4.0, cwarp: 0.2, coct: 3.0,
        mscale: 5.0, mstr: 0.3, mroct: 4.0,
        moisture: 1.2, ice: 0.0, lapse: 0.4, rain: 0.05,
        disp: 0.3, coloring: 0.0, rotspeed: 4.0,
        sunaz: 200.0, sunel: 25.0, ambient: 0.18,
        atmo_color: 0.0, atmo_density: 0.42, atmo_height: 0.11, atmo_forward_scatter: 0.36, atmo_absorption: 0.14, atmo_rayleigh_falloff: 0.21, atmo_haze_amount: 0.28, atmo_haze_falloff: 0.43
    }
}
fn preset_ice() {
    #{
        seed: 99.0, ocean: 0.45, cscale: 2.0, cwarp: 0.5, coct: 4.0,
        mscale: 5.0, mstr: 0.35, mroct: 4.0,
        moisture: 2.5, ice: 3.0, lapse: 1.0, rain: 0.3,
        disp: 0.18, coloring: 0.0, rotspeed: 1.5,
        sunaz: 230.0, sunel: 15.0, ambient: 0.25,
        atmo_color: 4.0, atmo_density: 0.50, atmo_height: 0.18, atmo_forward_scatter: 0.46, atmo_absorption: 0.24, atmo_rayleigh_falloff: 0.26, atmo_haze_amount: 0.42, atmo_haze_falloff: 0.59
    }
}
fn preset_volcanic() {
    #{
        seed: 66.0, ocean: 0.3, cscale: 3.0, cwarp: 1.5, coct: 6.0,
        mscale: 10.0, mstr: 0.85, mroct: 7.0,
        moisture: 1.5, ice: 0.0, lapse: 0.5, rain: 0.15,
        disp: 0.55, coloring: 0.0, rotspeed: 5.0,
        sunaz: 180.0, sunel: 35.0, ambient: 0.1,
        atmo_color: 1.0, atmo_density: 0.35, atmo_height: 0.10, atmo_forward_scatter: 0.32, atmo_absorption: 0.12, atmo_rayleigh_falloff: 0.18, atmo_haze_amount: 0.22, atmo_haze_falloff: 0.37
    }
}
fn preset_archipelago() {
    #{
        seed: 27.0, ocean: 0.75, cscale: 1.0, cwarp: 1.8, coct: 7.0,
        mscale: 3.0, mstr: 0.2, mroct: 3.0,
        moisture: 4.0, ice: 0.5, lapse: 0.65, rain: 0.5,
        disp: 0.12, coloring: 0.0, rotspeed: 2.5,
        sunaz: 215.0, sunel: 45.0, ambient: 0.14,
        atmo_color: 2.0, atmo_density: 0.58, atmo_height: 0.17, atmo_forward_scatter: 0.60, atmo_absorption: 0.20, atmo_rayleigh_falloff: 0.25, atmo_haze_amount: 0.40, atmo_haze_falloff: 0.56
    }
}

// ── Normalize/denormalize helpers (pure functions, no external state) ─────────
fn model_source(mode) {
    if mode == 1 {
        "world://32"
    } else if mode == 2 {
        "/assets/3d/sphere.obj"
    } else if mode == 3 {
        "/assets/3d/cube.obj"
    } else if mode == 4 {
        "/assets/3d/suzanne.obj"
    } else {
        "world://32"
    }
}

fn seed_to_01(v)      { (v / 9999.0).max(0.0).min(1.0) }
fn seed_from_01(v)    { v * 9999.0 }
fn ocean_to_01(v)     { ((v - 0.01) / 0.98).max(0.0).min(1.0) }
fn ocean_from_01(v)   { 0.01 + v * 0.98 }
fn cscale_to_01(v)    { ((v - 0.5) / 9.5).max(0.0).min(1.0) }
fn cscale_from_01(v)  { 0.5 + v * 9.5 }
fn cwarp_to_01(v)     { (v / 2.0).max(0.0).min(1.0) }
fn cwarp_from_01(v)   { v * 2.0 }
fn coct_to_01(v)      { ((v - 1.0) / 7.0).max(0.0).min(1.0) }
fn coct_from_01(v)    { (v * 7.0 + 1.0).round().max(1.0).min(8.0) }
fn mscale_to_01(v)    { ((v - 1.0) / 14.0).max(0.0).min(1.0) }
fn mscale_from_01(v)  { 1.0 + v * 14.0 }
fn mstr_to_01(v)      { v.max(0.0).min(1.0) }
fn mstr_from_01(v)    { v }
fn mroct_to_01(v)     { ((v - 1.0) / 7.0).max(0.0).min(1.0) }
fn mroct_from_01(v)   { (v * 7.0 + 1.0).round().max(1.0).min(8.0) }
fn moisture_to_01(v)  { ((v - 0.5) / 7.5).max(0.0).min(1.0) }
fn moisture_from_01(v){ 0.5 + v * 7.5 }
fn ice_to_01(v)       { (v / 3.0).max(0.0).min(1.0) }
fn ice_from_01(v)     { v * 3.0 }
fn lapse_to_01(v)     { (v / 1.5).max(0.0).min(1.0) }
fn lapse_from_01(v)   { v * 1.5 }
fn rain_to_01(v)      { v.max(0.0).min(1.0) }
fn rain_from_01(v)    { v }
fn subdiv_to_01(v)    { if v <= 48.0 { 0.0 } else if v <= 96.0 { 0.25 } else if v <= 192.0 { 0.5 } else if v <= 384.0 { 0.75 } else { 1.0 } }
fn subdiv_from_01(v)  { if v < 0.125 { 32.0 } else if v < 0.375 { 64.0 } else if v < 0.625 { 128.0 } else if v < 0.875 { 256.0 } else { 512.0 } }
fn disp_to_01(v)      { (v / 0.6).max(0.0).min(1.0) }
fn disp_from_01(v)    { v * 0.6 }
fn coloring_to_01(v)  { (v / 2.0).max(0.0).min(1.0) }
fn coloring_from_01(v){ (v * 2.0).round().max(0.0).min(2.0) }
fn rotspeed_to_01(v)  { (v / 10.0).max(0.0).min(1.0) }
fn rotspeed_from_01(v){ v * 10.0 }
fn sunaz_to_01(v)     { (v / 360.0).max(0.0).min(1.0) }
fn sunaz_from_01(v)   { v * 360.0 }
fn sunel_to_01(v)     { ((v + 10.0) / 95.0).max(0.0).min(1.0) }
fn sunel_from_01(v)   { v * 95.0 - 10.0 }
fn ambient_to_01(v)   { (v / 0.5).max(0.0).min(1.0) }
fn ambient_from_01(v) { v * 0.5 }
fn atmo_color_to_01(v)      { (v / 5.0).max(0.0).min(1.0) }
fn atmo_color_from_01(v)    { (v * 5.0).round().max(0.0).min(5.0) }
fn atmo_color_name(v)       { if v < 0.5 { "AMBER" } else if v < 1.5 { "ROSE" } else if v < 2.5 { "CYAN" } else if v < 3.5 { "LIME" } else if v < 4.5 { "AZURE" } else { "VIOLET" } }
fn atmo_color_hex(v)        { if v < 0.5 { "#d9b36a" } else if v < 1.5 { "#cf7a8c" } else if v < 2.5 { "#7cc8ff" } else if v < 3.5 { "#9bcf86" } else if v < 4.5 { "#9ab8ff" } else { "#b79dff" } }
fn atmo_haze_hex(v)         { if v < 0.5 { "#f6eddc" } else if v < 1.5 { "#f6ebef" } else if v < 2.5 { "#edf7ff" } else if v < 3.5 { "#eef7ea" } else if v < 4.5 { "#eef4ff" } else { "#f2eeff" } }
fn atmo_absorption_hex(v)   { if v < 0.5 { "#ffbf7b" } else if v < 1.5 { "#ffc0ad" } else if v < 2.5 { "#ffd59d" } else if v < 3.5 { "#ffd08f" } else if v < 4.5 { "#ffd9b4" } else { "#ffd0dc" } }
fn atmo_density_to_01(v)   { v.max(0.0).min(1.0) }
fn atmo_density_from_01(v) { v.max(0.0).min(1.0) }
fn atmo_height_to_01(v) { v.max(0.0).min(1.0) }
fn atmo_height_from_01(v) { v.max(0.0).min(1.0) }
fn atmo_forward_to_01(v) { v.max(0.0).min(1.0) }
fn atmo_forward_from_01(v) { v.max(0.0).min(1.0) }
fn atmo_absorption_to_01(v)       { v.max(0.0).min(1.0) }
fn atmo_absorption_from_01(v)     { v.max(0.0).min(1.0) }
fn atmo_ray_falloff_to_01(v)        { ((v - 0.05) / 0.85).max(0.0).min(1.0) }
fn atmo_ray_falloff_from_01(v)      { 0.05 + v * 0.85 }
fn atmo_haze_amount_to_01(v)       { v.max(0.0).min(1.0) }
fn atmo_haze_amount_from_01(v)     { v.max(0.0).min(1.0) }
fn atmo_haze_falloff_to_01(v) { ((v - 0.05) / 0.90).max(0.0).min(1.0) }
fn atmo_haze_falloff_from_01(v) { 0.05 + v * 0.90 }
fn near(a, b, eps) { (a - b).abs() <= eps }

// ── Flight-mode helpers ───────────────────────────────────────────────────────
fn sim_radius_km(real_radius_km, divisor) {
    let d = if divisor <= 0.0001 { 30.0 } else { divisor };
    real_radius_km / d
}

fn gravity_mu_km3_s2(surface_g_mps2, radius_km) {
    let r = radius_km.max(0.001);
    (surface_g_mps2.max(0.0) / 1000.0) * r * r
}

fn normalize_flight_profile_name(profile_name) {
    let name = profile_name ?? "arcade";
    if name == "sim-lite" || name == "sim_lite" {
        "sim-lite"
    } else {
        "arcade"
    }
}

fn flight_profile_state_path() {
    "/mods/planet-generator/flight/profile"
}

fn apply_flight_profile(local, profile_name) {
    let profile = normalize_flight_profile_name(profile_name);
    local["flight_profile"] = profile;
    if profile == "sim-lite" {
        local["flight_input_blend"] = 4.8;
        local["flight_tangent_accel"] = 2.1;
        local["flight_radial_accel"] = 1.5;
        local["flight_yaw_rate"] = 1.1;
        local["flight_tangent_damping"] = 0.14;
        local["flight_radial_damping"] = 0.18;
        local["flight_max_tangent_speed"] = 14.0;
        local["flight_max_radial_speed"] = 6.0;
        local["flight_boost_mult"] = 1.35;
        local["flight_brake_mult"] = 1.45;
    } else {
        local["flight_input_blend"] = 6.6;
        local["flight_tangent_accel"] = 4.0;
        local["flight_radial_accel"] = 3.0;
        local["flight_yaw_rate"] = 2.1;
        local["flight_tangent_damping"] = 0.26;
        local["flight_radial_damping"] = 0.34;
        local["flight_max_tangent_speed"] = 22.0;
        local["flight_max_radial_speed"] = 10.0;
        local["flight_boost_mult"] = 1.85;
        local["flight_brake_mult"] = 1.85;
    }
    local
}

fn toggle_flight_profile(local) {
    let current = normalize_flight_profile_name(local["flight_profile"] ?? "arcade");
    let next = if current == "arcade" { "sim-lite" } else { "arcade" };
    apply_flight_profile(local, next)
}

fn persist_flight_profile(local, game) {
    let profile = normalize_flight_profile_name(local["flight_profile"] ?? "arcade");
    game.set(flight_profile_state_path(), profile);
    local
}

fn build_generated_body_map(
    real_radius_km,
    scale_divisor,
    surface_gravity_mps2,
    atmo_top_km,
    atmo_dense_start_km,
    atmo_drag_max
) {
    let sim_radius = sim_radius_km(real_radius_km, scale_divisor);
    let mu_km3 = gravity_mu_km3_s2(surface_gravity_mps2, sim_radius);
    #{
        id: "generated-planet",
        center_x: 0.0,
        center_y: 0.0,
        radius_px: sim_radius,
        surface_radius: sim_radius,
        radius_km: sim_radius,
        gravity_mu_km3_s2: mu_km3,
        atmosphere_top_km: atmo_top_km.max(0.0),
        atmosphere_dense_start_km: atmo_dense_start_km.max(0.0),
        atmosphere_drag_max: atmo_drag_max.max(0.0),
        cloud_bottom_km: 0.0,
        cloud_top_km: (atmo_top_km.max(0.0) * 0.35)
    }
}

fn sync_generated_body(local, world) {
    let real_radius_km = local["planet_radius_real_km"] ?? 6371.0;
    let scale_divisor = local["planet_radius_scale_divisor"] ?? 30.0;
    let surface_gravity_mps2 = local["surface_gravity_mps2"] ?? 9.81;
    let atmo_top_km = local["atmo_top_km"] ?? (80.0 / 30.0);
    let atmo_dense_start_km = local["atmo_dense_start_km"] ?? (12.0 / 30.0);
    let atmo_drag_max = local["atmo_drag_max"] ?? 2.0;
    let body = build_generated_body_map(
        real_radius_km,
        scale_divisor,
        surface_gravity_mps2,
        atmo_top_km,
        atmo_dense_start_km,
        atmo_drag_max
    );

    let patched = world.body_patch("generated-planet", body);
    #{
        body: body,
        patched: patched
    }
}

fn ensure_ship(world, spawn_x, spawn_y) {
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    world.spawn_prefab("ship", #{ x: spawn_x, y: spawn_y, vx: 0.0, vy: 0.0 })
}

fn enter_flight_mode(local, world) {
    local = apply_flight_profile(local, local["flight_profile"] ?? "arcade");
    let body_sync = sync_generated_body(local, world);
    let body = body_sync["body"] ?? #{};
    let cx = body["center_x"] ?? 0.0;
    let cy = body["center_y"] ?? 0.0;
    let surface_radius_wu = body["surface_radius"] ?? 1.0;
    let spawn_altitude_km = local["spawn_altitude_km"] ?? 1.5;
    let spawn_r = surface_radius_wu + spawn_altitude_km.max(0.05);
    let ship_id = ensure_ship(world, cx + spawn_r, cy);
    if ship_id > 0 {
        world.set_transform(ship_id, cx + spawn_r, cy, 0.0);
        world.set_physics(ship_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        world.gravity_attach(ship_id, #{ mode: "point", body: "generated-planet", gravity_scale: 1.0 });
        world.atmosphere_attach(ship_id, #{ body: "generated-planet" });
    }
    local["mode"] = "flight";
    local["ship_id"] = ship_id;
    local["flight_initialized"] = true;
    local["flight_heading"] = 0.0;
    local["flight_tangent_speed"] = 0.0;
    local["flight_radial_speed"] = 0.0;
    local
}

fn exit_flight_mode(local, world) {
    let ship_id = local["ship_id"] ?? 0;
    if ship_id > 0 {
        world.set_physics(ship_id, 0.0, 0.0, 0.0, 0.0, 0.08, 1000.0);
    }
    local["mode"] = "generator";
    local["flight_initialized"] = false;
    local["ship_id"] = 0;
    local["flight_tangent_speed"] = 0.0;
    local["flight_radial_speed"] = 0.0;
    local["flight_heading"] = 0.0;
    local["flight_thrust_input"] = 0.0;
    local["flight_radial_input"] = 0.0;
    local["flight_yaw_input"] = 0.0;
    local["flight_altitude_km"] = 0.0;
    local["flight_speed_kms"] = 0.0;
    local["flight_vrad_kms"] = 0.0;
    apply_flight_profile(local, local["flight_profile"] ?? "arcade")
}

// ── Persistent state──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.mode = "generator";
    local.ship_id = 0;
    local.flight_initialized = false;
    local.planet_radius_real_km = 6371.0;
    local.planet_radius_scale_divisor = 30.0;
    local.planet_radius_sim_km = sim_radius_km(local.planet_radius_real_km, local.planet_radius_scale_divisor);
    local.surface_gravity_mps2 = 9.81;
    local.spawn_altitude_km = 1.5;
    local.atmo_top_km = 80.0 / 30.0;
    local.atmo_dense_start_km = 12.0 / 30.0;
    local.atmo_drag_max = 2.0;
    local.flight_profile = normalize_flight_profile_name(
        game.get_s(flight_profile_state_path(), "arcade")
    );
    local.flight_assist_alt_hold = false;
    local.flight_assist_heading_hold = false;
    local.flight_assist_alt_damping = 1.8;
    local.flight_assist_heading_damping = 2.1;
    local.tab = 0;
    local.model_mode = 1;
    let p = preset_earth();
    local.seed = p.seed; local.ocean = p.ocean; local.cscale = p.cscale; local.cwarp = p.cwarp;
    local.coct = p.coct; local.mscale = p.mscale; local.mstr = p.mstr; local.mroct = p.mroct;
    local.moisture = p.moisture; local.ice = p.ice; local.lapse = p.lapse; local.rain = p.rain;
    local.disp = p.disp; local.coloring = p.coloring; local.rotspeed = p.rotspeed;
    local.sunaz = p.sunaz; local.sunel = p.sunel; local.ambient = p.ambient;
    local.atmo_color = p.atmo_color; local.atmo_density = p.atmo_density; local.atmo_height = p.atmo_height; local.atmo_forward_scatter = p.atmo_forward_scatter; local.atmo_absorption = p.atmo_absorption; local.atmo_rayleigh_falloff = p.atmo_rayleigh_falloff;
    local.atmo_haze_amount = p.atmo_haze_amount; local.atmo_haze_falloff = p.atmo_haze_falloff;
    local.subdiv = 32.0;
    local.rotation_paused = false;
    local.preset_choice = 0;
    local.tab_sync_needed = true;
    local = apply_flight_profile(local, local.flight_profile);
    local = persist_flight_profile(local, game);
}

let tab  = local.tab  ?? 0;
let seed = local.seed ?? 0.0;
let ocean    = local.ocean    ?? 0.55;
let cscale   = local.cscale   ?? 2.5;
let cwarp    = local.cwarp    ?? 0.65;
let coct     = local.coct     ?? 5.0;
let mscale   = local.mscale   ?? 6.0;
let mstr     = local.mstr     ?? 0.45;
let mroct    = local.mroct    ?? 5.0;
let moisture = local.moisture ?? 3.0;
let ice      = local.ice      ?? 1.0;
let lapse    = local.lapse    ?? 0.6;
let rain     = local.rain     ?? 0.4;
let disp     = local.disp     ?? 0.22;
let subdiv   = local.subdiv   ?? 32.0;
let coloring = local.coloring ?? 0.0;
let rotspeed = local.rotspeed ?? 3.0;
let sunaz    = local.sunaz    ?? 210.0;
let sunel    = local.sunel    ?? 40.0;
let ambient  = local.ambient  ?? 0.12;
let atmo_color = local.atmo_color ?? 2.0;
let atmo_density = local.atmo_density ?? 0.48;
let atmo_height = local.atmo_height ?? 0.11;
let atmo_forward_scatter = local.atmo_forward_scatter ?? 0.78;
let atmo_absorption = local.atmo_absorption ?? 0.12;
let atmo_rayleigh_falloff = local.atmo_rayleigh_falloff ?? 0.22;
let atmo_haze_amount = local.atmo_haze_amount ?? 0.22;
let atmo_haze_falloff = local.atmo_haze_falloff ?? 0.46;
let rotation_paused   = local.rotation_paused   ?? false;
let model_mode = local.model_mode ?? 1;
let preset_choice = local.preset_choice ?? 0;
let mode = local.mode ?? "generator";
let is_flight_mode = mode == "flight";

// ── Tab switching (keyboard: Q/W/E/Y/U — mouse: segmented control) ───────────
if !is_flight_mode && input.just_pressed("q") { tab = 0; local.tab = 0; local.tab_sync_needed = true; gui.set_widget_value("planet-parameter-tabs", 0.0); }
if !is_flight_mode && input.just_pressed("w") { tab = 1; local.tab = 1; local.tab_sync_needed = true; gui.set_widget_value("planet-parameter-tabs", 1.0); }
if !is_flight_mode && input.just_pressed("e") { tab = 2; local.tab = 2; local.tab_sync_needed = true; gui.set_widget_value("planet-parameter-tabs", 2.0); }
if !is_flight_mode && input.just_pressed("y") { tab = 3; local.tab = 3; local.tab_sync_needed = true; gui.set_widget_value("planet-parameter-tabs", 3.0); }
if !is_flight_mode && input.just_pressed("u") { tab = 4; local.tab = 4; local.tab_sync_needed = true; gui.set_widget_value("planet-parameter-tabs", 4.0); }
if !is_flight_mode {
    let gui_tab = gui.selected_index("planet-parameter-tabs");
    if gui_tab >= 0 && gui_tab != tab {
        tab = gui_tab;
        local.tab = gui_tab;
        local.tab_sync_needed = true;
    }
}

// ── Model switching (keyboard: 1/2/3/4 — mouse: segmented control) ──────────
let model_changed = false;
if !is_flight_mode && input.just_pressed("1") { model_mode = 1; gui.set_widget_value("planet-model-selector", 0.0); model_changed = true; }
if !is_flight_mode && input.just_pressed("2") { model_mode = 2; gui.set_widget_value("planet-model-selector", 1.0); model_changed = true; }
if !is_flight_mode && input.just_pressed("3") { model_mode = 3; gui.set_widget_value("planet-model-selector", 2.0); model_changed = true; }
if !is_flight_mode && input.just_pressed("4") { model_mode = 4; gui.set_widget_value("planet-model-selector", 3.0); model_changed = true; }
if !is_flight_mode {
    let gui_model = gui.selected_index("planet-model-selector");
    if gui_model >= 0 {
        let next_model = gui_model + 1;
        if next_model != model_mode {
            model_mode = next_model;
            model_changed = true;
        }
    }
}
if model_changed {
    local.model_mode = model_mode;
    local.push_dirty = true;
    local.push_timer = 150.0;
}

// Highlight active tab: bright green = active, hover green = hovered, dim = inactive
let tab_fg_0 = if tab == 0 { "#aaffcc" } else { "#2a9440" };
let tab_fg_1 = if tab == 1 { "#aaffcc" } else { "#2a9440" };
let tab_fg_2 = if tab == 2 { "#aaffcc" } else { "#2a9440" };
let tab_fg_3 = if tab == 3 { "#aaffcc" } else { "#2a9440" };
let tab_fg_4 = if tab == 4 { "#aaffcc" } else { "#2a9440" };
scene.set("tab-label-continents", "text.fg", tab_fg_0);
scene.set("tab-label-mountains",  "text.fg", tab_fg_1);
scene.set("tab-label-climate",    "text.fg", tab_fg_2);
scene.set("tab-label-visual",     "text.fg", tab_fg_3);
scene.set("tab-label-atmo",       "text.fg", tab_fg_4);
let model_fg_1 = if model_mode == 1 { "#aaffcc" } else { "#2a9440" };
let model_fg_2 = if model_mode == 2 { "#aaffcc" } else { "#2a9440" };
let model_fg_3 = if model_mode == 3 { "#aaffcc" } else { "#2a9440" };
let model_fg_4 = if model_mode == 4 { "#aaffcc" } else { "#2a9440" };
scene.set("model-label-planet",  "text.fg", model_fg_1);
scene.set("model-label-sphere",  "text.fg", model_fg_2);
scene.set("model-label-cube",    "text.fg", model_fg_3);
scene.set("model-label-suzanne", "text.fg", model_fg_4);

// Action hover colors
scene.set("planet-randomize-button-label", "text.fg", if gui.widget_hovered("planet-randomize-button") { "#ffe060" } else { "#ffc030" });
scene.set("planet-flyaround-button-label", "text.fg",
    if is_flight_mode {
        if gui.widget_hovered("planet-flyaround-button") { "#9deef0" } else { "#7ad7da" }
    } else {
        if gui.widget_hovered("planet-flyaround-button") { "#76d8d4" } else { "#5bc0be" }
    }
);
scene.set("planet-reset-button-label", "text.fg", if gui.widget_hovered("planet-reset-button") { "#80b0ff" } else { "#4080ff" });
let preset_hover = if gui.widget_hovered("planet-preset-dropdown") || gui.widget_open("planet-preset-dropdown") { "#8cc08c" } else { "#507050" };
scene.set("preset-prefix-label", "text.fg", preset_hover);
scene.set("preset-arrow-label", "text.fg", preset_hover);
scene.set("preset-current-label", "text.fg", if gui.widget_open("planet-preset-dropdown") { "#aaffcc" } else { "#8cc08c" });

// ── Rotation toggle (R) ───────────────────────────────────────────────────────
if !is_flight_mode && input.just_pressed("r") {
    local.rotation_paused = !(local.rotation_paused ?? false);
}

// ── Flight mode toggle (F10 / GUI button) ───────────────────────────────────
let fly_button_clicked =
    gui.button_clicked("planet-flyaround-button")
    || gui.button_clicked("planet-flyaround-button-label");
if input.just_pressed("F10") || fly_button_clicked {
    if is_flight_mode {
        local = exit_flight_mode(local, world);
    } else {
        local = enter_flight_mode(local, world);
    }
}
if (local.mode ?? "generator") == "flight" && input.just_pressed("Escape") {
    local = exit_flight_mode(local, world);
}

let active_mode = local.mode ?? "generator";
let is_flight_active = active_mode == "flight";
let is_generator_active = !is_flight_active;
if is_flight_active && input.just_pressed("F9") {
    local = toggle_flight_profile(local);
    local = persist_flight_profile(local, game);
}
if is_flight_active && input.just_pressed("h") {
    local.flight_assist_alt_hold = !(local.flight_assist_alt_hold ?? false);
}
if is_flight_active && input.just_pressed("j") {
    local.flight_assist_heading_hold = !(local.flight_assist_heading_hold ?? false);
}

// ── Preset keys (F1-F7) ───────────────────────────────────────────────────────
let preset_applied = false;

if is_generator_active && input.just_pressed("F1") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 0;
    gui.set_widget_value("planet-preset-dropdown", 0.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F2") {
    let p = preset_mars();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 1;
    gui.set_widget_value("planet-preset-dropdown", 1.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F3") {
    let p = preset_ocean();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 2;
    gui.set_widget_value("planet-preset-dropdown", 2.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F4") {
    let p = preset_desert();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 3;
    gui.set_widget_value("planet-preset-dropdown", 3.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F5") {
    let p = preset_ice();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 4;
    gui.set_widget_value("planet-preset-dropdown", 4.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F6") {
    let p = preset_volcanic();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 5;
    gui.set_widget_value("planet-preset-dropdown", 5.0);
    preset_applied = true;
}
if is_generator_active && input.just_pressed("F7") {
    let p = preset_archipelago();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.preset_choice = 6;
    gui.set_widget_value("planet-preset-dropdown", 6.0);
    preset_applied = true;
}
let gui_preset = if is_flight_active { -1 } else { gui.selected_index("planet-preset-dropdown") };
if gui_preset >= 0 && gui_preset != preset_choice {
    local.preset_choice = gui_preset;
    if gui_preset == 0 {
        let p = preset_earth();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 1 {
        let p = preset_mars();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 2 {
        let p = preset_ocean();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 3 {
        let p = preset_desert();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 4 {
        let p = preset_ice();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 5 {
        let p = preset_volcanic();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    } else if gui_preset == 6 {
        let p = preset_archipelago();
        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
        local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
        local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
        local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
        preset_applied = true;
    }
}

// ── Randomize (GUI button only) ───────────────────────────────────────────────
if is_generator_active && gui.button_clicked("planet-randomize-button") {
    local.seed     = (seed * 1234.5 + 6789.0) % 9999.0;
    local.ocean    = 0.2 + ((local.seed * 0.618) % 0.7);
    local.cscale   = 1.0 + ((local.seed * 0.382) % 9.0);
    local.cwarp    = (local.seed * 0.271) % 2.0;
    local.coct     = 3.0 + ((local.seed * 0.157) % 5.0).round();
    local.mscale   = 2.0 + ((local.seed * 0.493) % 10.0);
    local.mstr     = (local.seed * 0.311) % 1.0;
    local.mroct    = 3.0 + ((local.seed * 0.229) % 5.0).round();
    local.moisture = 1.0 + ((local.seed * 0.441) % 5.0);
    local.ice      = (local.seed * 0.173) % 2.0;
    local.lapse    = 0.2 + ((local.seed * 0.337) % 0.8);
    local.rain     = (local.seed * 0.251) % 0.8;
    local.disp     = (local.seed * 0.197) % 0.6;
    local.rotspeed = 1.0 + ((local.seed * 0.283) % 6.0);
    local.sunaz    = (local.seed * 0.431) % 360.0;
    local.sunel    = 10.0 + ((local.seed * 0.317) % 70.0);
    local.ambient  = 0.05 + ((local.seed * 0.211) % 0.25);
    local.atmo_color = ((local.seed * 0.173) % 6.0).floor();
    local.atmo_density = (local.seed * 0.137) % 0.75;
    local.atmo_height = 0.06 + ((local.seed * 0.071) % 0.26);
    local.atmo_forward_scatter = (local.seed * 0.167) % 0.9;
    local.atmo_absorption = (local.seed * 0.119) % 0.55;
    local.atmo_rayleigh_falloff = 0.08 + ((local.seed * 0.191) % 0.52);
    local.atmo_haze_amount = (local.seed * 0.227) % 0.75;
    local.atmo_haze_falloff = 0.08 + ((local.seed * 0.149) % 0.68);
    preset_applied = true;
}

// ── Reset to Earth defaults (Delete) ─────────────────────────────────────────
if is_generator_active && (input.just_pressed("Delete") || gui.button_clicked("planet-reset-button")) {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_density=p.atmo_density; local.atmo_height=p.atmo_height; local.atmo_forward_scatter=p.atmo_forward_scatter; local.atmo_absorption=p.atmo_absorption; local.atmo_rayleigh_falloff=p.atmo_rayleigh_falloff; local.atmo_haze_amount=p.atmo_haze_amount; local.atmo_haze_falloff=p.atmo_haze_falloff;
    local.tab = 0; tab = 0;
    preset_applied = true;
}

// Re-read local after possible preset changes
seed     = local.seed     ?? seed;
ocean    = local.ocean    ?? ocean;
cscale   = local.cscale   ?? cscale;
cwarp    = local.cwarp    ?? cwarp;
coct     = local.coct     ?? coct;
mscale   = local.mscale   ?? mscale;
mstr     = local.mstr     ?? mstr;
mroct    = local.mroct    ?? mroct;
moisture = local.moisture ?? moisture;
ice      = local.ice      ?? ice;
lapse    = local.lapse    ?? lapse;
rain     = local.rain     ?? rain;
disp     = local.disp     ?? disp;
subdiv   = local.subdiv   ?? subdiv;
coloring = local.coloring ?? coloring;
rotspeed = local.rotspeed ?? rotspeed;
sunaz    = local.sunaz    ?? sunaz;
sunel    = local.sunel    ?? sunel;
ambient  = local.ambient  ?? ambient;
atmo_color = local.atmo_color ?? atmo_color;
atmo_density = local.atmo_density ?? atmo_density;
atmo_height = local.atmo_height ?? atmo_height;
atmo_forward_scatter = local.atmo_forward_scatter ?? atmo_forward_scatter;
atmo_absorption = local.atmo_absorption ?? atmo_absorption;
atmo_rayleigh_falloff = local.atmo_rayleigh_falloff ?? atmo_rayleigh_falloff;
atmo_haze_amount = local.atmo_haze_amount ?? atmo_haze_amount;
atmo_haze_falloff = local.atmo_haze_falloff ?? atmo_haze_falloff;

let planet_radius_real_km = local.planet_radius_real_km ?? 6371.0;
let planet_radius_scale_divisor = local.planet_radius_scale_divisor ?? 30.0;
let planet_radius_sim_km = sim_radius_km(planet_radius_real_km, planet_radius_scale_divisor);
let surface_gravity_mps2 = local.surface_gravity_mps2 ?? 9.81;
let spawn_altitude_km = local.spawn_altitude_km ?? 1.5;
let atmo_top_km = local.atmo_top_km ?? (80.0 / 30.0);
let atmo_dense_start_km = local.atmo_dense_start_km ?? (12.0 / 30.0);
let atmo_drag_max = local.atmo_drag_max ?? 2.0;
local.planet_radius_sim_km = planet_radius_sim_km;

// ── Sync slider widgets→ local state (tab switch or preset) ──────────────────
// When tab switches or preset fires, push current local values to gui sliders.
// This updates the visual handle positions and gui internal state.
let need_sync = preset_applied || (local.tab_sync_needed ?? false);
if need_sync {
    if tab == 0 {
        gui.set_widget_value("param-0", seed_to_01(seed));
        gui.set_widget_value("param-1", ocean_to_01(ocean));
        gui.set_widget_value("param-2", cscale_to_01(cscale));
        gui.set_widget_value("param-3", cwarp_to_01(cwarp));
        gui.set_widget_value("param-4", coct_to_01(coct));
    } else if tab == 1 {
        gui.set_widget_value("param-0", mscale_to_01(mscale));
        gui.set_widget_value("param-1", mstr_to_01(mstr));
        gui.set_widget_value("param-2", mroct_to_01(mroct));
    } else if tab == 2 {
        gui.set_widget_value("param-0", moisture_to_01(moisture));
        gui.set_widget_value("param-1", ice_to_01(ice));
        gui.set_widget_value("param-2", lapse_to_01(lapse));
        gui.set_widget_value("param-3", rain_to_01(rain));
    } else if tab == 3 {
        gui.set_widget_value("param-0", subdiv_to_01(subdiv));
        gui.set_widget_value("param-1", disp_to_01(disp));
        gui.set_widget_value("param-2", coloring_to_01(coloring));
        gui.set_widget_value("param-3", rotspeed_to_01(rotspeed));
        gui.set_widget_value("param-4", sunaz_to_01(sunaz));
        gui.set_widget_value("param-5", sunel_to_01(sunel));
        gui.set_widget_value("param-6", ambient_to_01(ambient));
    } else {
        gui.set_widget_value("param-0", atmo_color_to_01(atmo_color));
        gui.set_widget_value("param-1", atmo_density_to_01(atmo_density));
        gui.set_widget_value("param-2", atmo_height_to_01(atmo_height));
        gui.set_widget_value("param-3", atmo_forward_to_01(atmo_forward_scatter));
        gui.set_widget_value("param-4", atmo_absorption_to_01(atmo_absorption));
        gui.set_widget_value("param-5", atmo_ray_falloff_to_01(atmo_rayleigh_falloff));
        gui.set_widget_value("param-6", atmo_haze_amount_to_01(atmo_haze_amount));
        gui.set_widget_value("param-7", atmo_haze_falloff_to_01(atmo_haze_falloff));
    }
    local.tab_sync_needed = false;
}

// ── Read slider values → local state (normal mouse drag frames) ───────────────
// gui.slider_value() returns the value from this frame's gui state (after mouse events).
// Skip on sync frames (preset/tab-switch) — local.* is already correct.
if !need_sync && is_generator_active {
    if tab == 0 {
        seed     = seed_from_01(gui.slider_value("param-0"));
        ocean    = ocean_from_01(gui.slider_value("param-1"));
        cscale   = cscale_from_01(gui.slider_value("param-2"));
        cwarp    = cwarp_from_01(gui.slider_value("param-3"));
        let raw_coct = gui.slider_value("param-4");
        coct     = coct_from_01(raw_coct);
        let snap_coct = coct_to_01(coct);
        if (raw_coct - snap_coct).abs() > 0.005 { gui.set_widget_value("param-4", snap_coct); }
        local.seed = seed; local.ocean = ocean; local.cscale = cscale;
        local.cwarp = cwarp; local.coct = coct;
    } else if tab == 1 {
        mscale   = mscale_from_01(gui.slider_value("param-0"));
        mstr     = mstr_from_01(gui.slider_value("param-1"));
        let raw_mroct = gui.slider_value("param-2");
        mroct    = mroct_from_01(raw_mroct);
        let snap_mroct = mroct_to_01(mroct);
        if (raw_mroct - snap_mroct).abs() > 0.005 { gui.set_widget_value("param-2", snap_mroct); }
        local.mscale = mscale; local.mstr = mstr; local.mroct = mroct;
    } else if tab == 2 {
        moisture = moisture_from_01(gui.slider_value("param-0"));
        ice      = ice_from_01(gui.slider_value("param-1"));
        lapse    = lapse_from_01(gui.slider_value("param-2"));
        rain     = rain_from_01(gui.slider_value("param-3"));
        local.moisture = moisture; local.ice = ice; local.lapse = lapse; local.rain = rain;
    } else if tab == 3 {
        let raw_subdiv = gui.slider_value("param-0");
        subdiv   = subdiv_from_01(raw_subdiv);
        let snap_subdiv = subdiv_to_01(subdiv);
        if (raw_subdiv - snap_subdiv).abs() > 0.005 { gui.set_widget_value("param-0", snap_subdiv); }
        disp     = disp_from_01(gui.slider_value("param-1"));
        let raw_coloring = gui.slider_value("param-2");
        coloring = coloring_from_01(raw_coloring);
        let snap_coloring = coloring_to_01(coloring);
        if (raw_coloring - snap_coloring).abs() > 0.005 { gui.set_widget_value("param-2", snap_coloring); }
        rotspeed = rotspeed_from_01(gui.slider_value("param-3"));
        sunaz    = sunaz_from_01(gui.slider_value("param-4"));
        sunel    = sunel_from_01(gui.slider_value("param-5"));
        ambient  = ambient_from_01(gui.slider_value("param-6"));
        local.subdiv = subdiv; local.disp = disp; local.coloring = coloring;
        local.rotspeed = rotspeed; local.sunaz = sunaz; local.sunel = sunel; local.ambient = ambient;
    } else {
        let raw_atmo_color = gui.slider_value("param-0");
        atmo_color = atmo_color_from_01(raw_atmo_color);
        let snap_atmo_color = atmo_color_to_01(atmo_color);
        if (raw_atmo_color - snap_atmo_color).abs() > 0.005 { gui.set_widget_value("param-0", snap_atmo_color); }
        atmo_density = atmo_density_from_01(gui.slider_value("param-1"));
        atmo_height = atmo_height_from_01(gui.slider_value("param-2"));
        atmo_forward_scatter = atmo_forward_from_01(gui.slider_value("param-3"));
        atmo_absorption = atmo_absorption_from_01(gui.slider_value("param-4"));
        atmo_rayleigh_falloff = atmo_ray_falloff_from_01(gui.slider_value("param-5"));
        atmo_haze_amount = atmo_haze_amount_from_01(gui.slider_value("param-6"));
        atmo_haze_falloff = atmo_haze_falloff_from_01(gui.slider_value("param-7"));
        local.atmo_color = atmo_color; local.atmo_density = atmo_density; local.atmo_height = atmo_height; local.atmo_forward_scatter = atmo_forward_scatter; local.atmo_absorption = atmo_absorption; local.atmo_rayleigh_falloff = atmo_rayleigh_falloff;
        local.atmo_haze_amount = atmo_haze_amount; local.atmo_haze_falloff = atmo_haze_falloff;
    }
}

// ── Slot visibility (hide unused slots per tab) ───────────────────────────────
let NUM_PARAMS = if tab == 0 { 5 } else if tab == 1 { 3 } else if tab == 2 { 4 } else if tab == 3 { 7 } else { 8 };
for i in 0..8 {
    let vis = i < NUM_PARAMS;
    scene.set(`param-label-${i}`,  "visible", vis);
    scene.set(`param-value-${i}`,  "visible", vis);
    scene.set(`param-track-${i}`,  "visible", vis);
    scene.set(`param-handle-${i}`, "visible", vis);
}

// ── Update labels & title per active tab ──────────────────────────────────────
if tab == 0 {
    scene.set("param-title", "text.content", "CONTINENTS");
    scene.set("param-summary", "text.content", "Shape the macro landmass before erosion and climate.");
    scene.set("param-label-0", "text.content", "SEED");
    scene.set("param-value-0", "text.content", `${seed.round()}`);
    scene.set("param-label-1", "text.content", "OCEAN");
    scene.set("param-value-1", "text.content", `${(ocean * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", "CONTINENT SZ");
    scene.set("param-value-2", "text.content", `${(cscale * 10.0).round() / 10.0}`);
    scene.set("param-label-3", "text.content", "COAST CHAOS");
    scene.set("param-value-3", "text.content", `${(cwarp * 10.0).round() / 10.0}`);
    scene.set("param-label-4", "text.content", "OCTAVES");
    scene.set("param-value-4", "text.content", `${coct.round()}`);
} else if tab == 1 {
    scene.set("param-title", "text.content", "MOUNTAINS");
    scene.set("param-summary", "text.content", "Control ridge spacing, height and high-frequency breakup.");
    scene.set("param-label-0", "text.content", "MTN SPACING");
    scene.set("param-value-0", "text.content", `${(mscale * 10.0).round() / 10.0}`);
    scene.set("param-label-1", "text.content", "MTN HEIGHT");
    scene.set("param-value-1", "text.content", `${(mstr * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", "RIDGE DETAIL");
    scene.set("param-value-2", "text.content", `${mroct.round()}`);
} else if tab == 2 {
    scene.set("param-title", "text.content", "CLIMATE");
    scene.set("param-summary", "text.content", "Tune moisture transport, cold zones and mountain rain loss.");
    scene.set("param-label-0", "text.content", "MOISTURE SZ");
    scene.set("param-value-0", "text.content", `${(moisture * 10.0).round() / 10.0}`);
    scene.set("param-label-1", "text.content", "ICE CAPS");
    scene.set("param-value-1", "text.content", `${(ice * 10.0).round() / 10.0}`);
    scene.set("param-label-2", "text.content", "ALT COOLING");
    scene.set("param-value-2", "text.content", `${(lapse * 10.0).round() / 10.0}`);
    scene.set("param-label-3", "text.content", "RAIN SHADOW");
    scene.set("param-value-3", "text.content", `${(rain * 10.0).round() / 10.0}`);
} else if tab == 3 {
    let coloring_name = if coloring < 0.5 { "BIOME" } else if coloring < 1.5 { "ELEV" } else { "MOIST" };
    scene.set("param-title", "text.content", "VISUAL");
    scene.set("param-summary", "text.content", "Swap render density, palette source and direct lighting.");
    scene.set("param-label-0", "text.content", "RESOLUTION");
    scene.set("param-value-0", "text.content", `${subdiv.round()}`);
    scene.set("param-label-1", "text.content", "DISPLACEMENT");
    scene.set("param-value-1", "text.content", `${(disp * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", "COLORING");
    scene.set("param-value-2", "text.content", `[${coloring_name}]`);
    scene.set("param-label-3", "text.content", "ROTATION");
    scene.set("param-value-3", "text.content", `${(rotspeed * 10.0).round() / 10.0}`);
    scene.set("param-label-4", "text.content", "SUN AZIMUTH");
    scene.set("param-value-4", "text.content", `${sunaz.round()}`);
    scene.set("param-label-5", "text.content", "SUN ELEVATION");
    scene.set("param-value-5", "text.content", `${sunel.round()}`);
    scene.set("param-label-6", "text.content", "AMBIENT");
    scene.set("param-value-6", "text.content", `${(ambient * 100.0).round()}%`);
} else {
    let atmo_color_label = atmo_color_name(atmo_color);
    scene.set("param-title", "text.content", "ATMOSPHERE");
    scene.set("param-summary", "text.content", "Drive halo thickness, limb brightness and haze layering.");
    scene.set("param-label-0", "text.content", "COLOR");
    scene.set("param-value-0", "text.content", `[${atmo_color_label}]`);
    scene.set("param-label-1", "text.content", "ATMO DENSITY");
    scene.set("param-value-1", "text.content", `${(atmo_density * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", "ATMO HEIGHT");
    scene.set("param-value-2", "text.content", `${(atmo_height * 100.0).round()}%`);
    scene.set("param-label-3", "text.content", "FWD SCATTER");
    scene.set("param-value-3", "text.content", `${(atmo_forward_scatter * 100.0).round()}%`);
    scene.set("param-label-4", "text.content", "ABSORPTION");
    scene.set("param-value-4", "text.content", `${(atmo_absorption * 100.0).round()}%`);
    scene.set("param-label-5", "text.content", "RAY FALLOFF");
    scene.set("param-value-5", "text.content", `${(atmo_rayleigh_falloff * 100.0).round()}%`);
    scene.set("param-label-6", "text.content", "HAZE AMOUNT");
    scene.set("param-value-6", "text.content", `${(atmo_haze_amount * 100.0).round()}%`);
    scene.set("param-label-7", "text.content", "HAZE FALLOFF");
    scene.set("param-value-7", "text.content", `${(atmo_haze_falloff * 100.0).round()}%`);
}

// ── Stats strip ───────────────────────────────────────────────────────────────
scene.set("stats-seed", "text.content", `SEED: ${seed.round()}`);
let ps = planet_last_stats();
let stats_mode = if is_flight_active { "FLIGHT" } else { "GENERATOR" };
let stats_flight_profile = normalize_flight_profile_name(local.flight_profile ?? "arcade");
let stats_flight_profile_label = if stats_flight_profile == "sim-lite" { "SIM-LITE" } else { "ARCADE" };
let stats_alt_hold = local.flight_assist_alt_hold ?? false;
let stats_hdg_hold = local.flight_assist_heading_hold ?? false;
let stats_alt_hold_label = if stats_alt_hold { "ALT-HOLD" } else { "ALT-MAN" };
let stats_hdg_hold_label = if stats_hdg_hold { "HDG-HOLD" } else { "HDG-MAN" };
let stats_radius_real = local.planet_radius_real_km ?? 6371.0;
let stats_radius_sim = sim_radius_km(stats_radius_real, local.planet_radius_scale_divisor ?? 30.0);
let stats_surface_g = local.surface_gravity_mps2 ?? 9.81;
let stats_surface_g_fmt = (stats_surface_g * 100.0).round() / 100.0;
let stats_alt = (local.flight_altitude_km ?? 0.0).max(0.0);
let stats_spd = (local.flight_speed_kms ?? 0.0).abs();
let stats_vrad = local.flight_vrad_kms ?? 0.0;
scene.set("stats-flight-mode", "text.content", `MODE: ${stats_mode} ${stats_flight_profile_label} ${stats_alt_hold_label} ${stats_hdg_hold_label}`);
scene.set("stats-flight-profile", "text.content", `PROFILE: ${stats_flight_profile_label}  H[${stats_alt_hold_label}]  J[${stats_hdg_hold_label}]`);
scene.set("stats-radius-real", "text.content", `R REAL: ${stats_radius_real.round()} km`);
scene.set("stats-radius-sim", "text.content", `R SIM: ${(stats_radius_sim * 100.0).round() / 100.0} wu`);
scene.set("stats-gravity", "text.content", `G: ${stats_surface_g_fmt} m/s2`);
scene.set("stats-alt", "text.content", `ALT: ${stats_alt.round()} km`);
scene.set("stats-spd", "text.content", `SPD: ${(stats_spd * 100.0).round() / 100.0} km/s`);
scene.set("stats-vrad", "text.content", `VRAD: ${(stats_vrad * 100.0).round() / 100.0} km/s`);
if ps.len() > 0 {
    let ocean_pct  = (ps["ocean"]  * 100.0).round();
    let forest_pct = (ps["forest"] * 100.0).round();
    let desert_pct = (ps["desert"] * 100.0).round();
    let cold_pct   = (ps["cold"]   * 100.0).round();
    scene.set("stats-biomes", "text.content",
        `MODE ${stats_mode}  OCEAN ${ocean_pct}%  FOREST ${forest_pct}%  DESERT ${desert_pct}%  SNOW ${cold_pct}%  R ${stats_radius_real.round()}km/${stats_radius_sim.round()}wu  G ${stats_surface_g_fmt}`);
} else {
    scene.set("stats-biomes", "text.content",
        `MODE ${stats_mode}  OCEAN --  FOREST --  DESERT --  SNOW --  R ${stats_radius_real.round()}km/${stats_radius_sim.round()}wu  G ${stats_surface_g_fmt}`);
}

// ── Push world params to planet mesh (throttled to ≤ THROTTLE_MS between pushes) ──
// Push immediately on first frame, then only when dirty and enough time has passed.
let THROTTLE_MS = 150.0;

let prev_seed    = local.prev_seed    ?? seed;
let prev_ocean   = local.prev_ocean   ?? ocean;
let prev_cscale  = local.prev_cscale  ?? cscale;
let prev_cwarp   = local.prev_cwarp   ?? cwarp;
let prev_coct    = local.prev_coct    ?? coct;
let prev_mscale  = local.prev_mscale  ?? mscale;
let prev_mstr    = local.prev_mstr    ?? mstr;
let prev_mroct   = local.prev_mroct   ?? mroct;
let prev_moisture= local.prev_moisture?? moisture;
let prev_ice     = local.prev_ice     ?? ice;
let prev_lapse   = local.prev_lapse   ?? lapse;
let prev_rain    = local.prev_rain    ?? rain;
let prev_disp    = local.prev_disp    ?? disp;
let prev_subdiv  = local.prev_subdiv  ?? subdiv;
let prev_coloring= local.prev_coloring?? coloring;

let world_changed = model_mode == 1 && ((seed != prev_seed) || (ocean != prev_ocean) || (cscale != prev_cscale)
    || (cwarp != prev_cwarp) || (coct != prev_coct) || (mscale != prev_mscale)
    || (mstr != prev_mstr) || (mroct != prev_mroct) || (moisture != prev_moisture)
    || (ice != prev_ice) || (lapse != prev_lapse) || (rain != prev_rain)
    || (disp != prev_disp) || (subdiv != prev_subdiv) || (coloring != prev_coloring));

local.prev_seed = seed; local.prev_ocean = ocean; local.prev_cscale = cscale;
local.prev_cwarp = cwarp; local.prev_coct = coct; local.prev_mscale = mscale;
local.prev_mstr = mstr; local.prev_mroct = mroct; local.prev_moisture = moisture;
local.prev_ice = ice; local.prev_lapse = lapse; local.prev_rain = rain;
local.prev_disp = disp; local.prev_subdiv = subdiv; local.prev_coloring = coloring;

if !(local.push_ready ?? false) {
    local.push_ready = true;
    local.push_timer = THROTTLE_MS;  // Start ready to push on frame 1
    local.push_dirty = true;
}

local.push_timer = (local.push_timer ?? THROTTLE_MS) + frame_ms.to_float();

if world_changed {
    local.push_dirty = true;
}

let should_push = (local.push_dirty ?? false) && local.push_timer >= THROTTLE_MS;

if should_push {
    local.push_dirty = false;
    local.push_timer = 0.0;
    let coloring_str = if coloring < 0.5 { "biome" } else if coloring < 1.5 { "elevation" } else { "moisture" };
    scene.set("planet-mesh", "obj.source", model_source(model_mode));
    if model_mode == 1 {
        scene.set("planet-mesh", "world.seed",               seed);
        scene.set("planet-mesh", "world.ocean_fraction",     ocean);
        scene.set("planet-mesh", "world.continent_scale",    cscale);
        scene.set("planet-mesh", "world.continent_warp",     cwarp);
        scene.set("planet-mesh", "world.continent_octaves",  coct);
        scene.set("planet-mesh", "world.mountain_scale",         mscale);
        scene.set("planet-mesh", "world.mountain_strength",      mstr);
        scene.set("planet-mesh", "world.mountain_ridge_octaves", mroct);
        scene.set("planet-mesh", "world.moisture_scale",         moisture);
        scene.set("planet-mesh", "world.ice_cap_strength",       ice);
        scene.set("planet-mesh", "world.lapse_rate",             lapse);
        scene.set("planet-mesh", "world.rain_shadow",            rain);
        scene.set("planet-mesh", "world.displacement_scale",     disp);
        scene.set("planet-mesh", "world.subdivisions",           subdiv);
        scene.set("planet-mesh", "world.shape",                  "sphere");
        scene.set("planet-mesh", "world.base",                   "cube");
        scene.set("planet-mesh", "world.coloring",               coloring_str);
        scene.set("planet-mesh", "obj.surface_mode", "material");
    } else {
        scene.set("planet-mesh", "obj.surface_mode", "material");
    }
}

// Visual-only params always update immediately (no mesh rebuild)
let effective_rotspeed = if local.rotation_paused ?? false { 0.0 } else { rotspeed };
let visual_changed =
    !(local.prev_visual_pushed ?? false)
    || !near(local.prev_effective_rotspeed ?? effective_rotspeed, effective_rotspeed, 0.0001)
    || !near(local.prev_ambient ?? ambient, ambient, 0.0001);
if visual_changed {
    scene.set("planet-mesh", "obj.rotation-speed", effective_rotspeed);
    scene.set("planet-mesh", "obj.ambient",         ambient);
    local.prev_effective_rotspeed = effective_rotspeed;
    local.prev_ambient = ambient;
    local.prev_visual_pushed = true;
}

// Atmosphere profile (direct sliders -> profile fields)
let atmo_height_rel = (0.02 + atmo_height * 0.88).max(0.02).min(0.90);
let atmo_density_profile = atmo_density.max(0.0).min(1.0);
let atmo_forward = atmo_forward_scatter.max(0.0).min(1.0);
let atmo_limb = (0.95 + atmo_density * 1.7 + atmo_haze_amount * 1.15 + atmo_height * 0.55).max(0.2).min(4.0);
let atmo_term_soft = (0.75 + atmo_haze_amount * 1.55 + atmo_density * 0.35).max(0.05).min(4.0);
let atmo_absorb = atmo_absorption.max(0.0).min(1.0);
let atmo_ray_amount = (0.08 + atmo_density * 0.72).max(0.0).min(1.0);
let atmo_absorb_height = (0.70 + atmo_height * 0.10).max(0.50).min(0.92);
let atmo_absorb_width = (0.10 + atmo_haze_amount * 0.18 + atmo_absorb * 0.10).max(0.06).min(0.32);
let ray_color = atmo_color_hex(atmo_color);
let haze_color = atmo_haze_hex(atmo_color);
let absorb_color = atmo_absorption_hex(atmo_color);
let atmo_changed =
    !(local.prev_atmo_pushed ?? false)
    || !near(local.prev_atmo_height_rel ?? atmo_height_rel, atmo_height_rel, 0.0001)
    || !near(local.prev_atmo_density ?? atmo_density_profile, atmo_density_profile, 0.0001)
    || !near(local.prev_atmo_ray_amount ?? atmo_ray_amount, atmo_ray_amount, 0.0001)
    || (local.prev_atmo_ray_color ?? ray_color) != ray_color
    || !near(local.prev_atmo_ray_falloff ?? atmo_rayleigh_falloff, atmo_rayleigh_falloff, 0.0001)
    || !near(local.prev_atmo_haze_amount ?? atmo_haze_amount, atmo_haze_amount, 0.0001)
    || (local.prev_atmo_haze_color ?? haze_color) != haze_color
    || !near(local.prev_atmo_haze_falloff ?? atmo_haze_falloff, atmo_haze_falloff, 0.0001)
    || !near(local.prev_atmo_absorb ?? atmo_absorb, atmo_absorb, 0.0001)
    || (local.prev_atmo_absorb_color ?? absorb_color) != absorb_color
    || !near(local.prev_atmo_absorb_height ?? atmo_absorb_height, atmo_absorb_height, 0.0001)
    || !near(local.prev_atmo_absorb_width ?? atmo_absorb_width, atmo_absorb_width, 0.0001)
    || !near(local.prev_atmo_forward ?? atmo_forward, atmo_forward, 0.0001)
    || !near(local.prev_atmo_limb ?? atmo_limb, atmo_limb, 0.0001)
    || !near(local.prev_atmo_term_soft ?? atmo_term_soft, atmo_term_soft, 0.0001);
if atmo_changed {
    scene.set("planet-mesh", "obj.atmo.height",              atmo_height_rel);
    scene.set("planet-mesh", "obj.atmo.density",             atmo_density_profile);
    scene.set("planet-mesh", "obj.atmo.rayleigh_amount",     atmo_ray_amount);
    scene.set("planet-mesh", "obj.atmo.rayleigh_color",      ray_color);
    scene.set("planet-mesh", "obj.atmo.rayleigh_falloff",    atmo_rayleigh_falloff);
    scene.set("planet-mesh", "obj.atmo.haze_amount",         atmo_haze_amount);
    scene.set("planet-mesh", "obj.atmo.haze_color",          haze_color);
    scene.set("planet-mesh", "obj.atmo.haze_falloff",        atmo_haze_falloff);
    scene.set("planet-mesh", "obj.atmo.absorption_amount",   atmo_absorb);
    scene.set("planet-mesh", "obj.atmo.absorption_color",    absorb_color);
    scene.set("planet-mesh", "obj.atmo.absorption_height",   atmo_absorb_height);
    scene.set("planet-mesh", "obj.atmo.absorption_width",    atmo_absorb_width);
    scene.set("planet-mesh", "obj.atmo.forward_scatter",     atmo_forward);
    scene.set("planet-mesh", "obj.atmo.limb_boost",          atmo_limb);
    scene.set("planet-mesh", "obj.atmo.terminator_softness", atmo_term_soft);
    scene.set("planet-mesh", "obj.atmo.night_glow",          0.0);

    local.prev_atmo_height_rel = atmo_height_rel;
    local.prev_atmo_density = atmo_density_profile;
    local.prev_atmo_ray_amount = atmo_ray_amount;
    local.prev_atmo_ray_color = ray_color;
    local.prev_atmo_ray_falloff = atmo_rayleigh_falloff;
    local.prev_atmo_haze_amount = atmo_haze_amount;
    local.prev_atmo_haze_color = haze_color;
    local.prev_atmo_haze_falloff = atmo_haze_falloff;
    local.prev_atmo_absorb = atmo_absorb;
    local.prev_atmo_absorb_color = absorb_color;
    local.prev_atmo_absorb_height = atmo_absorb_height;
    local.prev_atmo_absorb_width = atmo_absorb_width;
    local.prev_atmo_forward = atmo_forward;
    local.prev_atmo_limb = atmo_limb;
    local.prev_atmo_term_soft = atmo_term_soft;
    local.prev_atmo_pushed = true;
}

let az_rad= sunaz * 0.01745329;
let el_rad = sunel * 0.01745329;
let light_x = -(az_rad.cos() * el_rad.cos());
let light_y = el_rad.sin();
let light_z = -(az_rad.sin() * el_rad.cos());
let light_changed =
    !(local.prev_light_pushed ?? false)
    || !near(local.prev_light_x ?? light_x, light_x, 0.0001)
    || !near(local.prev_light_y ?? light_y, light_y, 0.0001)
    || !near(local.prev_light_z ?? light_z, light_z, 0.0001);
if light_changed {
    scene.set("planet-mesh", "obj.light.x", light_x);
    scene.set("planet-mesh", "obj.light.y", light_y);
    scene.set("planet-mesh", "obj.light.z", light_z);
    local.prev_light_x = light_x;
    local.prev_light_y = light_y;
    local.prev_light_z = light_z;
    local.prev_light_pushed = true;
}

// ── Flight runtime (same scene, alternate mode) ─────────────────────────────
if is_flight_active {
    // Keep generated body sync in-flight so gravity/atmo contracts stay aligned.
    let body_sync = sync_generated_body(local, world);
    let body = body_sync["body"] ?? #{};
    let body_patch_ok = body_sync["patched"] ?? false;
    let cx = body["center_x"] ?? 0.0;
    let cy = body["center_y"] ?? 0.0;
    let surface_radius = (body["surface_radius"] ?? planet_radius_sim_km).max(0.1);
    let dt = (frame_ms.to_float() / 1000.0).max(0.0001);
    let min_alt = (surface_radius * 0.002).max(0.05);

    let ship_id = if (local.ship_id ?? 0) > 0 { local.ship_id ?? 0 } else { ensure_ship(world, cx + surface_radius + spawn_altitude_km, cy) };
    local.ship_id = ship_id;
    if ship_id > 0 {
        let xf = world.transform(ship_id) ?? #{};
        let mut_x = xf["x"] ?? (cx + surface_radius + spawn_altitude_km);
        let mut_y = xf["y"] ?? cy;

        let dx = mut_x - cx;
        let dy = mut_y - cy;
        let r = (sqrt(dx * dx + dy * dy)).max(surface_radius + min_alt);
        let nx = dx / r;
        let ny = dy / r;
        let tx = -ny;
        let ty = nx;

        let thrust_target =
            if input.down("Up") || input.down("w") { 1.0 }
            else if input.down("Down") || input.down("s") { -1.0 }
            else { 0.0 };
        let radial_target =
            if input.down("e") { 1.0 }
            else if input.down("q") { -1.0 }
            else { 0.0 };
        let yaw_target =
            if input.down("Left") || input.down("a") { 1.0 }
            else if input.down("Right") || input.down("d") { -1.0 }
            else { 0.0 };
        let input_blend = (dt * (local.flight_input_blend ?? 6.0)).min(1.0);
        let thrust_input = (local.flight_thrust_input ?? 0.0) + (thrust_target - (local.flight_thrust_input ?? 0.0)) * input_blend;
        let radial_input = (local.flight_radial_input ?? 0.0) + (radial_target - (local.flight_radial_input ?? 0.0)) * input_blend;
        let yaw_input = (local.flight_yaw_input ?? 0.0) + (yaw_target - (local.flight_yaw_input ?? 0.0)) * input_blend;
        let has_explicit_radial = radial_target.abs() > 0.001;
        let has_explicit_yaw = yaw_target.abs() > 0.001;

        let boost_mult = local.flight_boost_mult ?? 1.6;
        let brake_mult = local.flight_brake_mult ?? 1.7;
        let thrust_scale =
            if input.down("x") { boost_mult }
            else if input.down("z") { 1.0 / brake_mult.max(1.01) }
            else { 1.0 };
        let accel_tangent = thrust_input * (local.flight_tangent_accel ?? 3.5) * thrust_scale;
        let accel_radial = radial_input * (local.flight_radial_accel ?? 2.6);
        let max_tangent = (local.flight_max_tangent_speed ?? 18.0).max(0.1);
        let max_radial = (local.flight_max_radial_speed ?? 8.0).max(0.1);
        let tangent_speed = ((local.flight_tangent_speed ?? 0.0) + accel_tangent * dt).max(-max_tangent).min(max_tangent);
        let radial_speed = ((local.flight_radial_speed ?? 0.0) + accel_radial * dt).max(-max_radial).min(max_radial);
        let mut_radial_speed = radial_speed;
        if (local.flight_assist_alt_hold ?? false) && !has_explicit_radial {
            let alt_hold_damp = (local.flight_assist_alt_damping ?? 1.8).max(0.0);
            mut_radial_speed = mut_radial_speed * (1.0 / (1.0 + alt_hold_damp * dt));
        }
        let mut_yaw_input = yaw_input;
        if (local.flight_assist_heading_hold ?? false) && !has_explicit_yaw {
            let hdg_hold_damp = (local.flight_assist_heading_damping ?? 2.1).max(0.0);
            mut_yaw_input = mut_yaw_input * (1.0 / (1.0 + hdg_hold_damp * dt));
        }
        let heading = (local.flight_heading ?? 0.0) + mut_yaw_input * dt * (local.flight_yaw_rate ?? 1.8);

        let tangent_speed_damped = tangent_speed * (1.0 / (1.0 + (local.flight_tangent_damping ?? 0.22) * dt));
        let radial_speed_damped = mut_radial_speed * (1.0 / (1.0 + (local.flight_radial_damping ?? 0.28) * dt));

        let next_r = (r + radial_speed_damped * dt).max(surface_radius + min_alt);
        let arc = tangent_speed_damped * dt;
        let delta = arc / next_r.max(0.0001);
        let c = delta.cos();
        let s = delta.sin();
        let next_nx = nx * c + tx * s;
        let next_ny = ny * c + ty * s;
        let next_tx = -next_ny;
        let next_ty = next_nx;

        let next_x = cx + next_nx * next_r;
        let next_y = cy + next_ny * next_r;
        world.set_transform(ship_id, next_x, next_y, heading);
        world.set_physics(
            ship_id,
            (next_x - mut_x) / dt,
            (next_y - mut_y) / dt,
            0.0,
            0.0,
            0.02,
            250.0
        );

        // Drive planet renderer camera manually in flight mode.
        let scene_scale = 1.0 / surface_radius.max(0.0001);
        let scene_r = next_r * scene_scale;
        let cam_wx = next_nx * scene_r;
        let cam_wz = next_ny * scene_r;
        let fwd_x = (next_tx * heading.cos() + next_nx * heading.sin()).max(-1.0).min(1.0);
        let fwd_z = (next_ty * heading.cos() + next_ny * heading.sin()).max(-1.0).min(1.0);
        let up_x = next_nx;
        let up_z = next_ny;
        let right_x = fwd_z;
        let right_z = -fwd_x;

        scene.set("planet-mesh", "obj.cam.wx", cam_wx);
        scene.set("planet-mesh", "obj.cam.wy", 0.0);
        scene.set("planet-mesh", "obj.cam.wz", cam_wz);
        scene.set("planet-mesh", "obj.view.fx", fwd_x);
        scene.set("planet-mesh", "obj.view.fy", 0.0);
        scene.set("planet-mesh", "obj.view.fz", fwd_z);
        scene.set("planet-mesh", "obj.view.ux", up_x);
        scene.set("planet-mesh", "obj.view.uy", 0.0);
        scene.set("planet-mesh", "obj.view.uz", up_z);
        scene.set("planet-mesh", "obj.view.rx", right_x);
        scene.set("planet-mesh", "obj.view.ry", 1.0);
        scene.set("planet-mesh", "obj.view.rz", right_z);
        world.set_camera_3d_look_at(
            cam_wx - fwd_x * 0.20 + up_x * 0.03,
            0.0,
            cam_wz - fwd_z * 0.20 + up_z * 0.03,
            cam_wx + fwd_x * 0.25,
            0.0,
            cam_wz + fwd_z * 0.25
        );
        world.set_camera_3d_up(up_x, 0.0, up_z);

        local.flight_heading = heading;
        local.flight_tangent_speed = tangent_speed_damped;
        local.flight_radial_speed = radial_speed_damped;
        local.flight_thrust_input = thrust_input;
        local.flight_radial_input = radial_input;
        local.flight_yaw_input = mut_yaw_input;

        let altitude = (next_r - surface_radius).max(0.0);
        local.flight_altitude_km = altitude;
        local.flight_speed_kms = sqrt(
            tangent_speed_damped * tangent_speed_damped
            + radial_speed_damped * radial_speed_damped
        );
        local.flight_vrad_kms = radial_speed_damped;
        let alt_fmt = (altitude * 100.0).round() / 100.0;
        let spd_fmt = ((local.flight_speed_kms ?? 0.0) * 100.0).round() / 100.0;
        let vrad_fmt = (radial_speed_damped * 100.0).round() / 100.0;
        let vrad_txt = if vrad_fmt >= 0.0 { `+${vrad_fmt}` } else { `${vrad_fmt}` };
        scene.set(
            "stats-biomes",
            "text.content",
            `MODE FLIGHT  PROFILE ${stats_flight_profile_label}  ALT ${alt_fmt}km  SPD ${spd_fmt}km/s  VRAD ${vrad_txt}km/s  R ${planet_radius_real_km.round()}km/${surface_radius.round()}wu  G ${stats_surface_g_fmt}  BOOST[X] BRAKE[Z]  PROFILE[F9]  H:${stats_alt_hold_label} J:${stats_hdg_hold_label}`
        );
        if !body_patch_ok {
            scene.set("stats-biomes", "text.content", "MODE FLIGHT  BODY PATCH FAILED");
        }
    } else {
        scene.set("stats-biomes", "text.content", "MODE FLIGHT  SHIP PREFAB NOT AVAILABLE");
    }
}


