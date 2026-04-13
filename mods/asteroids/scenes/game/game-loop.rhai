// ============================================================================
// ASTEROIDS — Surface Launch Simulation (minimal)
// ============================================================================
// Ship spawns on Earth's surface. Earth spins. Ship co-rotates with surface.
//
// SHIP MODEL: orbital basis vectors (sn / sf / sr).
//   sn = radial normal from planet centre (local up)
//   sf = ship nose-forward tangent on the local horizon
//   sr = ship right tangent (cross product of sf × sn)
//
// CONTROLS:
//   E           — main engine (forward thrust)
//   SPACE       — bottom engine (radial lift)
//   W/S / ↑↓   — prograde thrust / retro-brake
//   A/D / ←→   — yaw left / right (RCS)
//   Q           — lateral strafe left
// ============================================================================
import "rcs" as rcs;

if type_of(local) == "()" { local = #{}; }

let WORLD_W = 3200.0;
let WORLD_H = 1800.0;

// ======================== PALETTE ==========================================
let pal_ver = palette.version();
if pal_ver != (local.pal_ver ?? -1) {
    local.pal_ver = pal_ver;
    let PAL_BG = palette.get("bg"); if PAL_BG == "" { PAL_BG = "#060620"; }
    local.pal = #{
        bg: PAL_BG,
        ship: palette.get("ship"),
        thrust_init: palette.get("particle_thruster_init"),
        hot_init: palette.get("particle_hot_init"),
        hot_core: palette.get("particle_hot_core_init"),
        correct: palette.get("particle_correction"),
        pulse: palette.get("particle_pulse"),
        ramp_hot_engine: palette.particles("hot_engine"),
        ramp_brake: palette.particles("brake"),
        ramp_heat_trail: palette.particles("heat_trail"),
    };
    scene.set_bg(PAL_BG);
}
let col = local.pal ?? #{};

// ======================== UTILITY FUNCTIONS =================================

fn rotate_axis(vx, vy, vz, ax, ay, az, angle) {
    let c = cos(angle); let s = sin(angle);
    let d = vx*ax + vy*ay + vz*az;
    let cx = ay*vz - az*vy; let cy = az*vx - ax*vz; let cz = ax*vy - ay*vx;
    #{ x: vx*c + cx*s + ax*d*(1.0-c),
       y: vy*c + cy*s + ay*d*(1.0-c),
       z: vz*c + cz*s + az*d*(1.0-c) }
}

fn clamp_unit(v) { if v < -1.0 { return -1.0; } if v > 1.0 { return 1.0; } v }

fn clamp01(x) { if x < 0.0 { return 0.0; } if x > 1.0 { return 1.0; } x }

fn angle_from_cos_sin(c, s) {
    let a = acos(clamp_unit(c));
    if s < 0.0 { return -a; }
    a
}

fn nearest_angle(angle, reference) {
    let pi = 3.1415926535; let tau = 6.2831853070;
    let best = angle; let best_delta = absf(best - reference);
    let plus = angle + tau;
    if absf(plus - reference) < best_delta { best = plus; best_delta = absf(plus - reference); }
    let minus = angle - tau;
    if absf(minus - reference) < best_delta { best = minus; }
    while best > pi { best -= tau; }
    while best < -pi { best += tau; }
    best
}

fn heading_from_forward(fx, fy, fallback) {
    let fl = sqrtf(fx * fx + fy * fy);
    if fl < 0.0001 { return fallback; }
    let nfx = fx / fl; let nfy = fy / fl;
    let h = acos(clamp_unit(-nfy));
    if nfx < 0.0 { return -h; }
    h
}

fn ship_basis_euler(srx, sry, srz, sfx, sfy, sfz, snx, sny, snz, prev_pitch, prev_yaw, prev_roll) {
    let cy = sqrtf(srx * srx + sry * sry);
    let yaw = angle_from_cos_sin(cy, -srz);
    let pitch = prev_pitch; let roll = prev_roll;
    if cy > 0.0005 {
        pitch = angle_from_cos_sin(snz / cy, sfz / cy);
        roll = angle_from_cos_sin(srx / cy, sry / cy);
    } else {
        let combo = if srz >= 0.0 { angle_from_cos_sin(-snx, -sny) }
                    else { angle_from_cos_sin(snx, -sny) };
        pitch = if srz >= 0.0 { combo - roll } else { combo + roll };
    }
    #{
        pitch: nearest_angle(pitch, prev_pitch),
        yaw: nearest_angle(yaw, prev_yaw),
        roll: nearest_angle(roll, prev_roll),
    }
}

fn one_decimal_text(v) {
    let av = absf(v);
    let scaled = (av * 10.0).to_int();
    let whole = scaled / 10; let frac = scaled - whole * 10;
    whole + "." + frac
}

fn fmt_kms(label, value) {
    let sign = if value < 0.0 { "-" } else { "" };
    label + " " + sign + one_decimal_text(value) + " km/s"
}

fn ensure_ship(world, spawn_x, spawn_y) {
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    world.spawn_prefab("ship", #{ x: spawn_x, y: spawn_y, vx: 0.0, vy: 0.0 })
}

// ======================== CONSTANTS =========================================
let _body = world.body_info("main-planet");
let PLANET_X = if type_of(_body["center_x"]) != "()" { _body["center_x"] } else { WORLD_W * 0.5 };
let PLANET_Y = if type_of(_body["center_y"]) != "()" { _body["center_y"] } else { WORLD_H * 0.5 };
let SURFACE_RADIUS = if type_of(_body["surface_radius"]) != "()" { _body["surface_radius"] } else { 200.0 };
let BODY_RADIUS_PX = if type_of(_body["radius_px"]) != "()" { _body["radius_px"] } else { SURFACE_RADIUS };
let BODY_GRAVITY_MU = if type_of(_body["gravity_mu"]) != "()" { _body["gravity_mu"] } else { 87861.0 };
let BODY_RADIUS_KM = if type_of(_body["radius_km"]) != "()" { _body["radius_km"] } else { 66.85 };
let KM_PER_PX = if type_of(_body["km_per_px"]) != "()" { _body["km_per_px"].max(0.001) } else { BODY_RADIUS_KM / BODY_RADIUS_PX.max(1.0) };
let ATMO_TOP_ALT_PX =
    if type_of(_body["atmosphere_top_km"]) != "()" { (_body["atmosphere_top_km"] / KM_PER_PX).max(0.0) }
    else { 42.0 };
let ATMO_DENSE_ALT_PX =
    if type_of(_body["atmosphere_dense_start_km"]) != "()" { (_body["atmosphere_dense_start_km"] / KM_PER_PX).max(0.0) }
    else { 13.5 };
let ATMO_DRAG_MAX = if type_of(_body["atmosphere_drag_max"]) != "()" { _body["atmosphere_drag_max"] } else { 2.2 };

let PI = 3.1415926535;
let ORBIT_R = SURFACE_RADIUS + 1.0;
// v_circ at surface = sqrt(87861/200) = 20.96 px/s ≈ 7 km/s → 60s orbit.
let MAX_SPD        = 35.0;
let MAX_VRAD       = 20.0;
let ACC_FWD        = 1.8;
let ACC_SIDE       = 1.2;
let ACC_LIFT       = 5.0;    // ~2.3× surface g (2.2 px/s²)
let ACC_MAIN_ENG   = 3.2;
let LIN_DAMP       = 0.04;
let SIDE_TRIM      = 0.20;
let SIDE_THRUST_TRIM = 1.0;
let YAW_RESPONSE   = 7.2;
let YAW_DAMP       = 2.6;
let YAW_MAX        = 1.1;
let CAM_SWAY_TAU   = 0.50;
let CAM_SWAY_GAIN  = 0.28;
// Map physics surface radius to 3D unit sphere: surface → r=1.0
let SCENE_SCALE    = 1.0 / SURFACE_RADIUS;

let _ptype_id = if type_of(_body["planet_type"]) != "()" { _body["planet_type"] } else { "earth_like" };
let _ptype = world.planet_type_info(_ptype_id);
let PLANET_SPIN_DPS = _ptype["surface_spin_dps"];
let CLOUD_SPIN_DPS  = _ptype["cloud_spin_dps"];
let CLOUD2_SPIN_DPS = _ptype["cloud_spin_2_dps"];

let dt = frame_ms.to_float() / 1000.0;

// ======================== INITIALIZATION ====================================
if !(local.initialized ?? false) {
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    // Random spawn angle on the surface
    let _spawn_angle = rand() * 2.0 * PI;
    let _ca = cos(_spawn_angle); let _sa = sin(_spawn_angle);
    local.snx = _ca; local.sny = _sa; local.snz = 0.0;
    // Forward tangent perpendicular to radial (rotated 90° CCW in XY plane)
    local.sfx = -_sa; local.sfy = _ca; local.sfz = 0.0;
    local.srx = 0.0; local.sry = 0.0; local.srz = 1.0;
    local.radius = ORBIT_R;
    local.vrad = 0.0;
    local.vfwd = 0.0; local.vright = 0.0;
    local.yaw_rate = 0.0;
    local.ship_heading = 0.0;
    local.ship_vis_pitch = PI;
    local.ship_vis_yaw = -PI * 0.5;
    local.ship_vis_roll = 0.0;
    local.cam_nx = _ca; local.cam_ny = _sa; local.cam_nz = 0.0;
    local.cam_fx = -_sa; local.cam_fy = _ca; local.cam_fz = 0.0;
    local.cam_sway = 0.0;
    local.thrust_hold_ms = 0.0;
    local.thrust_release_ms = 9999.0;
    local.spawn_ship_x = PLANET_X + ORBIT_R * _ca;
    local.spawn_ship_y = PLANET_Y + ORBIT_R * _sa;
    input.load_profile("default");
}

// ======================== ORBITAL STATE =====================================
let snx = local.snx ?? 1.0; let sny = local.sny ?? 0.0; let snz = local.snz ?? 0.0;
let sfx = local.sfx ?? 0.0; let sfy = local.sfy ?? -1.0; let sfz = local.sfz ?? 0.0;
let srx = local.srx ?? 0.0; let sry = local.sry ?? 0.0; let srz = local.srz ?? 1.0;
let radius = local.radius ?? ORBIT_R;
let vrad = local.vrad ?? 0.0;
let vfwd = local.vfwd ?? 0.0;
let vright = local.vright ?? 0.0;
let yaw_rate = local.yaw_rate ?? 0.0;
let ship_heading = local.ship_heading ?? 0.0;
let ship_vis_pitch = local.ship_vis_pitch ?? PI;
let ship_vis_yaw = local.ship_vis_yaw ?? -PI * 0.5;
let ship_vis_roll = local.ship_vis_roll ?? 0.0;
let cam_nx = local.cam_nx ?? snx; let cam_ny = local.cam_ny ?? sny; let cam_nz = local.cam_nz ?? snz;
let cam_fx = local.cam_fx ?? sfx; let cam_fy = local.cam_fy ?? sfy; let cam_fz = local.cam_fz ?? sfz;
let cam_sway = local.cam_sway ?? 0.0;

// ======================== SHIP SPAWN ========================================
let ship_id = ensure_ship(world, PLANET_X + snx * radius, PLANET_Y + sny * radius);
local.ship_id = ship_id;
scene.set("ship-scene-slot", "visible", ship_id > 0);

let ship_x = local.spawn_ship_x ?? (PLANET_X + radius);
let ship_y = local.spawn_ship_y ?? PLANET_Y;

// Input axes
let thrust_axis = 0.0;
let strafe_axis = 0.0;
let yaw_axis = 0.0;
let main_engine_on = false;
let lift_on = false;

if ship_id > 0 {
    let xf = world.transform(ship_id) ?? #{};
    ship_x = xf["x"] ?? ship_x;
    ship_y = xf["y"] ?? ship_y;
    local.spawn_ship_x = ship_x;
    local.spawn_ship_y = ship_y;

    // ── Input ──────────────────────────────────────────────────────────────
    thrust_axis =
        if input.action_down("thrust") && !input.action_down("brake") { 1.0 }
        else if input.action_down("brake") && !input.action_down("thrust") { -1.0 }
        else { 0.0 };
    strafe_axis = if input.action_down("strafe_left") { -1.0 } else { 0.0 };
    yaw_axis =
        if input.action_down("turn_right") && !input.action_down("turn_left") { -1.0 }
        else if input.action_down("turn_left") && !input.action_down("turn_right") { 1.0 }
        else { 0.0 };
    main_engine_on = input.action_down("main_engine");
    lift_on = input.action_down("lift");

    // ── Yaw dynamics ───────────────────────────────────────────────────────
    let _yaw_target = yaw_axis * YAW_MAX;
    let _yaw_blend = (YAW_RESPONSE * dt).min(1.0);
    yaw_rate += (_yaw_target - yaw_rate) * _yaw_blend;
    if yaw_axis == 0.0 { yaw_rate *= 1.0 / (1.0 + YAW_DAMP * dt); }
    if absf(yaw_rate) > YAW_MAX { yaw_rate = if yaw_rate > 0.0 { YAW_MAX } else { -YAW_MAX }; }

    let _dyaw = yaw_rate * dt;
    if absf(_dyaw) > 0.00001 {
        let _rsf_yaw = rotate_axis(sfx, sfy, sfz, snx, sny, snz, _dyaw);
        sfx = _rsf_yaw.x; sfy = _rsf_yaw.y; sfz = _rsf_yaw.z;
        let _rsr_yaw = rotate_axis(srx, sry, srz, snx, sny, snz, _dyaw);
        srx = _rsr_yaw.x; sry = _rsr_yaw.y; srz = _rsr_yaw.z;
    }

    // ── Thrust ─────────────────────────────────────────────────────────────
    vfwd += thrust_axis * ACC_FWD * dt;
    vright += strafe_axis * ACC_SIDE * dt;
    if main_engine_on { vfwd += ACC_MAIN_ENG * dt; }

    let _trim = SIDE_TRIM + (thrust_axis.max(0.0) * SIDE_THRUST_TRIM);
    vright *= 1.0 / (1.0 + _trim * dt);
    vfwd *= 1.0 / (1.0 + LIN_DAMP * dt);
    vright *= 1.0 / (1.0 + LIN_DAMP * dt);

    // Atmospheric drag
    let _altitude_px = (radius - SURFACE_RADIUS).max(0.0);
    let _atmo_alpha = clamp01((ATMO_TOP_ALT_PX - _altitude_px) / ATMO_TOP_ALT_PX.max(0.1));
    let _drag = _atmo_alpha * _atmo_alpha * ATMO_DRAG_MAX;
    vrad *= 1.0 / (1.0 + _drag * dt * 1.2);
    vfwd *= 1.0 / (1.0 + _drag * dt);
    vright *= 1.0 / (1.0 + _drag * dt);

    // Speed caps
    if vfwd > MAX_SPD { vfwd = MAX_SPD; }
    if vfwd < -MAX_SPD * 0.5 { vfwd = -MAX_SPD * 0.5; }
    if vright > MAX_SPD { vright = MAX_SPD; }
    if vright < -MAX_SPD { vright = -MAX_SPD; }

    // ── Gravity + centrifugal + lift ───────────────────────────────────────
    let _vt2 = vfwd*vfwd + vright*vright;
    let _ship_world_x = PLANET_X + snx * radius;
    let _ship_world_y = PLANET_Y + sny * radius;
    let _ship_gravity = world.body_gravity("main-planet", _ship_world_x, _ship_world_y, snz * radius);
    let _grav_radial = (_ship_gravity["ax"] ?? 0.0) * snx
                     + (_ship_gravity["ay"] ?? 0.0) * sny
                     + (_ship_gravity["az"] ?? 0.0) * snz;
    let _centrifugal = _vt2 / radius.max(SURFACE_RADIUS + 1.0);
    let _lift_acc = if lift_on { ACC_LIFT } else { 0.0 };
    vrad += (_centrifugal + _grav_radial + _lift_acc) * dt;
    if vrad > MAX_VRAD { vrad = MAX_VRAD; }
    if vrad < -MAX_VRAD { vrad = -MAX_VRAD; }
    radius += vrad * dt;
    if radius < SURFACE_RADIUS + 1.0 {
        radius = SURFACE_RADIUS + 1.0;
        if vrad < 0.0 { vrad = 0.0; }
    }

    // ── Geodesic transport (move sn/sf/sr along sphere) ────────────────────
    let _vt2_safe = vfwd*vfwd + vright*vright;
    if _vt2_safe > 0.001 {
        let _delta = sqrtf(_vt2_safe) * dt / radius.max(SURFACE_RADIUS + 1.0);
        let _vt_l = sqrtf(_vt2_safe);
        let _tang_x = (sfx * vfwd + srx * vright) / _vt_l;
        let _tang_y = (sfy * vfwd + sry * vright) / _vt_l;
        let _tang_z = (sfz * vfwd + srz * vright) / _vt_l;
        let _rn = rotate_axis(snx, sny, snz, _tang_x, _tang_y, _tang_z, _delta);
        snx = _rn.x; sny = _rn.y; snz = _rn.z;
        let _rf = rotate_axis(sfx, sfy, sfz, _tang_x, _tang_y, _tang_z, _delta);
        sfx = _rf.x; sfy = _rf.y; sfz = _rf.z;
        let _rr = rotate_axis(srx, sry, srz, _tang_x, _tang_y, _tang_z, _delta);
        srx = _rr.x; sry = _rr.y; srz = _rr.z;
    }

    // Re-orthonormalize basis
    let _nl = sqrtf(snx*snx + sny*sny + snz*snz).max(0.0001);
    snx /= _nl; sny /= _nl; snz /= _nl;
    let _fd = sfx*snx + sfy*sny + sfz*snz;
    sfx -= snx*_fd; sfy -= sny*_fd; sfz -= snz*_fd;
    let _fl = sqrtf(sfx*sfx + sfy*sfy + sfz*sfz).max(0.0001);
    sfx /= _fl; sfy /= _fl; sfz /= _fl;
    srx = sfy*snz - sfz*sny;
    sry = sfz*snx - sfx*snz;
    srz = sfx*sny - sfy*snx;

    // ── 3D ship visual ─────────────────────────────────────────────────────
    let _ship_sprite = "ship-scene-slot";
    ship_heading = heading_from_forward(sfx, sfy, ship_heading);
    let _ship_euler = ship_basis_euler(
        srx, sry, srz, sfx, sfy, sfz, snx, sny, snz,
        ship_vis_pitch, ship_vis_yaw, ship_vis_roll
    );
    ship_vis_pitch = _ship_euler["pitch"] ?? ship_vis_pitch;
    ship_vis_yaw = _ship_euler["yaw"] ?? ship_vis_yaw;
    ship_vis_roll = _ship_euler["roll"] ?? ship_vis_roll;
    scene.set("ship-" + ship_id, "visible", false);
    scene.set(_ship_sprite, "obj.pitch", ship_vis_pitch * 57.29577951);
    scene.set(_ship_sprite, "obj.yaw",   ship_vis_yaw   * 57.29577951);
    scene.set(_ship_sprite, "obj.roll",  ship_vis_roll  * 57.29577951);
    let _ship_scene_r = radius * SCENE_SCALE;
    scene.set(_ship_sprite, "obj.world.x", snx * _ship_scene_r);
    scene.set(_ship_sprite, "obj.world.y", sny * _ship_scene_r);
    scene.set(_ship_sprite, "obj.world.z", snz * _ship_scene_r);
    world.entity(ship_id).set_heading(ship_heading);

    // ── Persist state ──────────────────────────────────────────────────────
    local.snx = snx; local.sny = sny; local.snz = snz;
    local.sfx = sfx; local.sfy = sfy; local.sfz = sfz;
    local.srx = srx; local.sry = sry; local.srz = srz;
    local.radius = radius;
    local.vrad = vrad;
    local.vfwd = vfwd; local.vright = vright;
    local.yaw_rate = yaw_rate;
    local.ship_heading = ship_heading;
    local.ship_vis_pitch = ship_vis_pitch;
    local.ship_vis_yaw = ship_vis_yaw;
    local.ship_vis_roll = ship_vis_roll;

    // ── Drive engine entity to sphere position ─────────────────────────────
    let _dx = PLANET_X + snx * radius;
    let _dy = PLANET_Y + sny * radius;
    world.set_physics(ship_id,
        (_dx - ship_x) / dt, (_dy - ship_y) / dt,
        0.0, 0.0, 0.0, 0.0);

    // ── RCS VFX ────────────────────────────────────────────────────────────
    let _thrust_now = thrust_axis > 0.0 || main_engine_on;
    let _hold_ms = if _thrust_now { (local.thrust_hold_ms ?? 0.0) + frame_ms.to_float() } else { 0.0 };
    let _release_ms = if _thrust_now { 0.0 }
        else { ((local.thrust_release_ms ?? 9999.0) + frame_ms.to_float()).min(9999.0) };
    local.thrust_hold_ms = _hold_ms;
    local.thrust_release_ms = _release_ms;
    let _ship_speed = sqrtf(vfwd * vfwd + vright * vright);
    rcs::update_orbital_vfx_original_style(
        world, ship_id, thrust_axis, yaw_axis, yaw_rate,
        _hold_ms, _release_ms, vfwd, vright, _ship_speed, col,
        PLANET_X, PLANET_Y
    );
}  // end if ship_id > 0

// ======================== 2D CAMERA =========================================
let _cam_ship_x = PLANET_X + snx * radius;
let _cam_ship_y = PLANET_Y + sny * radius;
world.set_camera(_cam_ship_x - 320.0, _cam_ship_y - 180.0);

// Adaptive zoom: surface → zoomed in, orbit → zoomed out
let _alt_px = (radius - SURFACE_RADIUS).max(0.0);
let ZOOM_SURFACE = 3.0;
let ZOOM_ORBIT   = 1.0;
let ZOOM_FADE_PX = SURFACE_RADIUS;
let _zoom_t = clamp01(_alt_px / ZOOM_FADE_PX);
let _zoom_t2 = 1.0 - (1.0 - _zoom_t) * (1.0 - _zoom_t);
world.set_camera_zoom(ZOOM_SURFACE + (ZOOM_ORBIT - ZOOM_SURFACE) * _zoom_t2);

// ======================== PLANET SPIN =======================================
local.planet_self_yaw  = (local.planet_self_yaw  ?? 0.0) + PLANET_SPIN_DPS  * dt;
local.cloud_self_yaw   = (local.cloud_self_yaw   ?? 0.0) + CLOUD_SPIN_DPS   * dt;
local.cloud_self_yaw_2 = (local.cloud_self_yaw_2 ?? 90.0) + CLOUD2_SPIN_DPS * dt;
scene.set("main-planet-view", "planet.spin_deg", local.planet_self_yaw ?? 0.0);
scene.set("main-planet-view", "planet.cloud_spin_deg", local.cloud_self_yaw ?? 0.0);
scene.set("main-planet-view", "planet.cloud2_spin_deg", local.cloud_self_yaw_2 ?? 90.0);

// ======================== 3D CHASE CAMERA ===================================
let _cam_valid = snx*snx + sny*sny + snz*snz > 0.01;
if _cam_valid {
    let _ship_scene_r = radius * SCENE_SCALE;
    // Altitude-adaptive camera: hugs the ship at surface, pulls back in orbit
    let _alt_frac = clamp01((radius - SURFACE_RADIUS) / SURFACE_RADIUS);
    let CAM_LIFT = 0.005 + _alt_frac * 0.04;
    let CAM_BACK = 0.04 + _alt_frac * 0.74;    // 0.04 at surface → 0.78 in orbit
    let CAM_AHEAD = 0.03 + _alt_frac * 0.55;   // 0.03 at surface → 0.58 in orbit

    let _alpha_n = dt / (0.30 + dt);
    cam_nx += (snx - cam_nx) * _alpha_n;
    cam_ny += (sny - cam_ny) * _alpha_n;
    cam_nz += (snz - cam_nz) * _alpha_n;
    let _cnl = sqrtf(cam_nx*cam_nx + cam_ny*cam_ny + cam_nz*cam_nz).max(0.0001);
    cam_nx /= _cnl; cam_ny /= _cnl; cam_nz /= _cnl;

    let _alpha_f = dt / (CAM_SWAY_TAU + dt);
    cam_fx += (sfx - cam_fx) * _alpha_f;
    cam_fy += (sfy - cam_fy) * _alpha_f;
    cam_fz += (sfz - cam_fz) * _alpha_f;
    let _fd = cam_fx*cam_nx + cam_fy*cam_ny + cam_fz*cam_nz;
    cam_fx -= cam_nx*_fd; cam_fy -= cam_ny*_fd; cam_fz -= cam_nz*_fd;
    let _cfl = sqrtf(cam_fx*cam_fx + cam_fy*cam_fy + cam_fz*cam_fz).max(0.0001);
    cam_fx /= _cfl; cam_fy /= _cfl; cam_fz /= _cfl;

    let _ex = cam_nx * (_ship_scene_r + CAM_LIFT) - cam_fx * CAM_BACK;
    let _ey = cam_ny * (_ship_scene_r + CAM_LIFT) - cam_fy * CAM_BACK;
    let _ez = cam_nz * (_ship_scene_r + CAM_LIFT) - cam_fz * CAM_BACK;
    let _lx = cam_nx * _ship_scene_r + cam_fx * CAM_AHEAD;
    let _ly = cam_ny * _ship_scene_r + cam_fy * CAM_AHEAD;
    let _lz = cam_nz * _ship_scene_r + cam_fz * CAM_AHEAD;

    let _target_sway = (-yaw_rate * CAM_SWAY_GAIN).max(-0.20).min(0.20);
    let _alpha_s = dt / (CAM_SWAY_TAU + dt);
    cam_sway += (_target_sway - cam_sway) * _alpha_s;
    let _sup = rotate_axis(cam_nx, cam_ny, cam_nz, cam_fx, cam_fy, cam_fz, cam_sway);

    world.set_camera_3d_look_at(_ex, _ey, _ez, _lx, _ly, _lz);
    world.set_camera_3d_up(_sup.x, _sup.y, _sup.z);
}
local.cam_nx = cam_nx; local.cam_ny = cam_ny; local.cam_nz = cam_nz;
local.cam_fx = cam_fx; local.cam_fy = cam_fy; local.cam_fz = cam_fz;
local.cam_sway = cam_sway;

// ======================== HUD ===============================================
let _unit_km_s = KM_PER_PX;
let _tspd_px_s = sqrtf(vfwd*vfwd + vright*vright);
let _altitude_px = (radius - SURFACE_RADIUS).max(0.0);
let _tspd_km_s = _tspd_px_s * _unit_km_s;
let _rspd_km_s = vrad * _unit_km_s;
let _alt_km = (_altitude_px * KM_PER_PX).to_int();
let _v_circ_km_s = sqrtf(BODY_GRAVITY_MU / radius.max(SURFACE_RADIUS + 1.0)) * _unit_km_s;
let _v_esc_km_s = sqrtf(2.0 * BODY_GRAVITY_MU / radius.max(SURFACE_RADIUS + 1.0)) * _unit_km_s;
scene.set("main-planet-view", "planet.observer_altitude_km", _alt_km.to_float());

let _atmo_alpha = clamp01((ATMO_TOP_ALT_PX - _altitude_px) / ATMO_TOP_ALT_PX.max(0.1));
let hud_status =
    if radius <= SURFACE_RADIUS + 1.5 && absf(vrad) < 0.5 { "SURFACE" }
    else if _tspd_km_s >= _v_esc_km_s * 0.95 { "ESCAPE" }
    else if _rspd_km_s > 0.05 { "CLIMB" }
    else if _rspd_km_s < -0.05 { "DECAY" }
    else if absf(_tspd_km_s - _v_circ_km_s) < 0.03 { "STABLE" }
    else { "ORBIT" };
let _status_colour =
    if hud_status == "ESCAPE" || hud_status == "DECAY" { "#ddd47d" }
    else if hud_status == "SURFACE" { "#7aff9e" }
    else { "#9ed8ff" };

scene.set("hud-orbital-alt",    "content", "ALT  " + _alt_km + " km");
scene.set("hud-orbital-status", "content", hud_status);
scene.set("hud-orbital-tspd",   "content", fmt_kms("TSPD", _tspd_km_s));
scene.set("hud-orbital-rspd",   "content", fmt_kms("RSPD", _rspd_km_s));
scene.set("hud-orbital-heat",   "content", "");
scene.set("hud-orbital-vxy",    "content", "REF  " + one_decimal_text(_v_circ_km_s) + "/" + one_decimal_text(_v_esc_km_s));
scene.set("hud-orbital-status", "style.fg", _status_colour);
scene.set("hud-orbital-vxy",    "style.fg", "#88a8c4");
