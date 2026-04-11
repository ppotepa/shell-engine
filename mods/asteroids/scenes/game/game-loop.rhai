// ============================================================================
// ASTEROIDS — Orbital RCS Simulation
// ============================================================================
// SHIP MODEL: orbital basis vectors (sn / sf / sr).
//   sn = ship local up / radial normal from planet centre
//   sf = ship nose-forward tangent on the local horizon
//   sr = ship right tangent    (cross product of sf × sn)
//   vfwd / vright = tangential speed in sf / sr directions (px/s)
//
// MOVEMENT: geodesic transport on sphere via Rodrigues rotation each frame.
//   Yaw rotates sf/sr around sn.  Translation moves sn along great-circle arc.
//
// GAMEPLAY:
//   SPACE/Fire  — shoot bullets
//   W/S / ↑↓   — prograde thrust / retro-brake
//   A/D / ←→   — yaw left / right
//   Q/E         — lateral strafe
// ============================================================================
import "rcs" as rcs;

if type_of(local) == "()" { local = #{}; }

let WORLD_W = 3200.0;
let WORLD_H = 1800.0;

// ======================== PALETTE SETUP ====================================
// Only re-read palette values when the active palette has changed.
// palette.version() is bumped by the engine every time set_active/cycle is called.
let pal_ver = palette.version();
if pal_ver != (local.pal_ver ?? -1) {
    local.pal_ver = pal_ver;

    let PAL_BG           = palette.get("bg");           if PAL_BG           == "" { PAL_BG           = "#060620"; }
    let PAL_SHIP         = palette.get("ship");         if PAL_SHIP         == "" { PAL_SHIP         = "white"; }
    let PAL_ASTEROID     = palette.get("asteroid");     if PAL_ASTEROID     == "" { PAL_ASTEROID     = "white"; }
    let PAL_BULLET       = palette.get("bullet");       if PAL_BULLET       == "" { PAL_BULLET       = "yellow"; }
    let PAL_P_THRUST_INIT  = palette.get("particle_thruster_init"); if PAL_P_THRUST_INIT  == "" { PAL_P_THRUST_INIT  = "white"; }
    let PAL_P_HOT_INIT     = palette.get("particle_hot_init");      if PAL_P_HOT_INIT     == "" { PAL_P_HOT_INIT     = "#66FFFF"; }
    let PAL_P_HOT_CORE     = palette.get("particle_hot_core_init"); if PAL_P_HOT_CORE     == "" { PAL_P_HOT_CORE     = "#AAFFFF"; }
    let PAL_P_CORRECT      = palette.get("particle_correction");    if PAL_P_CORRECT      == "" { PAL_P_CORRECT      = "#AACCFF"; }
    let PAL_P_PULSE        = palette.get("particle_pulse");         if PAL_P_PULSE        == "" { PAL_P_PULSE        = "#00FFFF"; }

    local.pal = #{
        bg:           PAL_BG,
        ship:         PAL_SHIP,
        asteroid:     PAL_ASTEROID,
        bullet:       PAL_BULLET,
        thrust_init:  PAL_P_THRUST_INIT,
        hot_init:     PAL_P_HOT_INIT,
        hot_core:     PAL_P_HOT_CORE,
        pulse:        PAL_P_PULSE,
        correct:      PAL_P_CORRECT,
        ramp_hot_engine: palette.particles("hot_engine"),
        ramp_brake:      palette.particles("brake"),
        ramp_heat_trail: palette.particles("heat_trail"),
        ramp_bullet_trail: palette.particles("bullet_trail"),
    };

    // Apply palette-driven static styles (only needed on change)
    scene.set_bg(PAL_BG);
}

let col  = local.pal ?? #{};



// Give an asteroid a random tumble (constant spin, no braking).
fn add_tumble(world, id, size) {
    let max_spin = if size == 3 { 1.5 } else if size == 2 { 2.5 } else { 4.0 };
    let spin = (rand() - 0.5) * max_spin * 2.0;
    if absf(spin) < 0.3 { spin = if spin >= 0.0 { 0.3 } else { -0.3 }; }
    world.angular_body_attach(id, #{
        accel: 0.0,
        max: max_spin + 1.0,
        deadband: 0.0,
        auto_brake: false,
        angular_vel: spin,
    });
}

// ── Rodrigues axis-angle rotation ─────────────────────────────────────────────
// Rotate vector (vx,vy,vz) by `angle` radians around unit axis (ax,ay,az).
fn rotate_axis(vx, vy, vz, ax, ay, az, angle) {
    let c = cos(angle); let s = sin(angle);
    let d = vx*ax + vy*ay + vz*az;
    let cx = ay*vz - az*vy; let cy = az*vx - ax*vz; let cz = ax*vy - ay*vx;
    #{ x: vx*c + cx*s + ax*d*(1.0-c),
       y: vy*c + cy*s + ay*d*(1.0-c),
       z: vz*c + cz*s + az*d*(1.0-c) }
}

fn clamp_unit(v) {
    if v < -1.0 { return -1.0; }
    if v > 1.0 { return 1.0; }
    v
}

fn angle_from_cos_sin(c, s) {
    let a = acos(clamp_unit(c));
    if s < 0.0 { return -a; }
    a
}

fn nearest_angle(angle, reference) {
    let pi = 3.1415926535;
    let tau = 6.2831853070;
    let best = angle;
    let best_delta = absf(best - reference);
    let plus = angle + tau;
    if absf(plus - reference) < best_delta {
        best = plus;
        best_delta = absf(plus - reference);
    }
    let minus = angle - tau;
    if absf(minus - reference) < best_delta {
        best = minus;
    }
    while best > pi { best -= tau; }
    while best < -pi { best += tau; }
    best
}

fn heading_from_forward(fx, fy, fallback) {
    let fl = sqrtf(fx * fx + fy * fy);
    if fl < 0.0001 { return fallback; }
    let nfx = fx / fl;
    let nfy = fy / fl;
    let h = acos(clamp_unit(-nfy));
    if nfx < 0.0 { return -h; }
    h
}

fn heading_dir_xy(heading) {
    #{ x: sin(heading), y: -cos(heading) }
}

fn ship_basis_euler(srx, sry, srz, sfx, sfy, sfz, snx, sny, snz, prev_pitch, prev_yaw, prev_roll) {
    let cy = sqrtf(srx * srx + sry * sry);
    let yaw = angle_from_cos_sin(cy, -srz);
    let pitch = prev_pitch;
    let roll = prev_roll;

    if cy > 0.0005 {
        pitch = angle_from_cos_sin(snz / cy, sfz / cy);
        roll = angle_from_cos_sin(srx / cy, sry / cy);
    } else {
        let combo = if srz >= 0.0 {
            angle_from_cos_sin(-snx, -sny)
        } else {
            angle_from_cos_sin(snx, -sny)
        };
        pitch = if srz >= 0.0 { combo - roll } else { combo + roll };
    }

    #{
        pitch: nearest_angle(pitch, prev_pitch),
        yaw: nearest_angle(yaw, prev_yaw),
        roll: nearest_angle(roll, prev_roll),
    }
}

fn hide_asteroid_slots(scene, prefix, count) {
    for i in 0..count {
        scene.set(prefix + i, "visible", false);
    }
}

fn init_asteroid_slot(scene, slot_id) {
    scene.set(slot_id, "obj.yaw",         rand() * 360.0);
    scene.set(slot_id, "obj.pitch",       rand() * 360.0);
    scene.set(slot_id, "obj.roll",        rand() * 360.0);
    scene.set(slot_id, "obj.orbit_speed", 30.0 + rand() * 110.0);
}

fn project_world_to_scene(x, y, planet_x, planet_y, scene_scale) {
    #{
        x: (x - planet_x) * scene_scale,
        y: (y - planet_y) * scene_scale,
        z: 0.0,
    }
}

fn prune_dead_asteroid_slots(world, ast_slots) {
    let next_slots = #{};
    for key in ast_slots.keys() {
        let eid = key.parse_int();
        if type_of(eid) == "i64" && world.exists(eid) {
            next_slots[key] = ast_slots[key];
        }
    }
    next_slots
}

fn sync_asteroid_slot_group(scene, world, ast_slots, live_ids, prefix, count, planet_x, planet_y, scene_scale) {
    let next_slots = ast_slots;
    let used_slots = [];
    for aid in live_ids {
        let key = aid + "";
        let slot = next_slots[key];
        if type_of(slot) == "string" { used_slots.push(slot); }
    }

    let next_free = 0;
    for aid in live_ids {
        // Hide the prefab-cloned object layer: asteroid visuals are rendered via
        // the dedicated generator slot pool, not the empty runtime object layer.
        scene.set("asteroid-" + aid, "visible", false);
        let key = aid + "";
        let slot = next_slots[key];
        if type_of(slot) != "string" {
            while next_free < count {
                let candidate = prefix + next_free;
                if !used_slots.contains(candidate) {
                    slot = candidate;
                    next_slots[key] = slot;
                    used_slots.push(slot);
                    init_asteroid_slot(scene, slot);
                    break;
                }
                next_free += 1;
            }
        }

        if type_of(slot) != "string" { continue; }

        let axf = world.transform(aid) ?? #{};
        let pos = project_world_to_scene(
            axf["x"] ?? planet_x,
            axf["y"] ?? planet_y,
            planet_x,
            planet_y,
            scene_scale
        );
        scene.set(slot, "obj.world.x", pos["x"]);
        scene.set(slot, "obj.world.y", pos["y"]);
        scene.set(slot, "obj.world.z", pos["z"]);
        scene.set(slot, "visible", true);
    }

    next_slots
}

fn spawn_wave(world, wave) {
    let WORLD_W = world.world_width();
    let WORLD_H = world.world_height();
    let count = 3 + wave;
    for i in 0..count {
        let side = i % 4;
        let x = if side == 0 { -32.0 } else if side == 1 { WORLD_W + 32.0 } else { 20.0 + rand() * (WORLD_W - 40.0) };
        let y = if side == 2 { -32.0 } else if side == 3 { WORLD_H + 32.0 } else { 20.0 + rand() * (WORLD_H - 40.0) };
        if absf(x - WORLD_W * 0.5) < 60.0 && absf(y - WORLD_H * 0.5) < 60.0 {
            x = if rand() > 0.5 { -32.0 } else { WORLD_W + 32.0 };
        }
        let base_speed = 30.0 + to_float(wave) * 5.0;
        let vx = (rand() - 0.5) * base_speed * 2.0;
        let vy = (rand() - 0.5) * base_speed * 2.0;
        if absf(vx) < 10.0 { vx = if vx >= 0.0 { 10.0 } else { -10.0 }; }
        if absf(vy) < 10.0 { vy = if vy >= 0.0 { 10.0 } else { -10.0 }; }
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-large", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 { add_tumble(world, id, 3); }
    }
}

fn spawn_medium_asteroids(world, x, y, parent_vx, parent_vy) {
    for i in 0..2 {
        let speed = 40.0 + rand() * 30.0;
        let vx = parent_vx * 0.5 + speed * (if i == 0 { 1.0 } else { -1.0 }) * 0.707;
        let vy = parent_vy * 0.5 + speed * (if i == 0 { 0.707 } else { -0.707 });
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-medium", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 { add_tumble(world, id, 2); }
    }
}

fn spawn_small_asteroids(world, x, y, parent_vx, parent_vy) {
    for i in 0..2 {
        let speed = 55.0 + rand() * 25.0;
        let vx = parent_vx * 0.4 + speed * (if i == 0 { 1.0 } else { -1.0 });
        let vy = parent_vy * 0.4 + speed * (if i == 0 { 0.5 } else { -0.5 });
        let heading = rand() * 6.2832;
        let id = world.spawn_prefab("asteroid-small", #{ x: x, y: y, vx: vx, vy: vy, heading: heading });
        if id > 0 { add_tumble(world, id, 1); }
    }
}

// Spawn shrapnel: real physics entities that can ricochet and damage the ship.
// Evenly spread around 360° with jitter, inheriting asteroid + bullet momentum.
fn spawn_shrapnel(world, ax, ay, bvx, bvy, avx, avy, count, base_speed) {
    for i in range(0, count) {
        let angle = (i.to_float() / count.to_float()) * 6.2832 + (rand() - 0.5) * 1.2;
        let spd   = base_speed + (rand() - 0.5) * 40.0;
        let vx    = sin(angle)  * spd + avx * 0.4 + bvx * 0.2;
        let vy    = -cos(angle) * spd + avy * 0.4 + bvy * 0.2;
        let ttl   = 1200 + (rand() * 600.0).to_int();
        world.spawn_prefab("shrapnel", #{
            x: ax, y: ay, vx: vx, vy: vy, ttl_ms: ttl, heading: angle
        });
    }
}

// Shared death visual sequence — hull pieces + shockwave + shake + whiteout flash.
// Call BEFORE despawning the ship so ship_id is still a valid entity.
fn do_ship_death_fx(world, effects, col, ship_id) {
    world.emit("ship.hull_pieces", ship_id, #{ count: 22, speed:  95.0, spread: 6.28, fg: col.ship ?? "white" });
    world.emit("ship.hull_pieces", ship_id, #{ count:  9, speed: 195.0, spread: 6.28, fg: col.ship ?? "white" });
    world.emit("ship.shockwave",   ship_id, #{ count: 20, speed: 320.0, spread: 6.28 });
    world.emit("asteroids.impact_sparks", ship_id, #{ count: 14, speed: 55.0, spread: 6.28, fg: col.ship ?? "white" });
    effects.shake(600, 4.0, 3.0, 8.0);
    effects.trigger("whiteout", 300, #{ intensity: 0.85 });
}

fn clear_round_entities(world) {
    for id in world.query_tag("asteroid") { world.despawn(id); }
    for id in world.query_tag("bullet")   { world.despawn(id); }
    for id in world.query_tag("shrapnel") { world.despawn(id); }
}

fn merge_highscores(saved, final_score, wave) {
    let scores = if type_of(saved) == "array" { saved } else { [] };
    let rank = -1;
    for i in 0..scores.len() {
        if final_score > scores[i].score { rank = i; break; }
    }
    if rank < 0 && scores.len() < 10 { rank = scores.len(); }
    if rank >= 0 {
        let entry = #{ rank: rank + 1, score: final_score, name: "---", wave: wave };
        scores.insert(rank, entry);
        while scores.len() > 10 { scores.pop(); }
        for i in 0..scores.len() { scores[i].rank = i + 1; }
    }
    #{ scores: scores, rank: rank }
}

fn ship_hull_polygon(heading) {
    let c = cos(heading);
    let s = sin(heading);
    let base = [[0.0, -6.9], [2.65, 5.39], [7.4, 6.9], [0.0, 2.7], [-7.4, 6.9], [-2.65, 5.39]];
    [
        [base[0][0] * c - base[0][1] * s, base[0][0] * s + base[0][1] * c],
        [base[1][0] * c - base[1][1] * s, base[1][0] * s + base[1][1] * c],
        [base[2][0] * c - base[2][1] * s, base[2][0] * s + base[2][1] * c],
        [base[3][0] * c - base[3][1] * s, base[3][0] * s + base[3][1] * c],
        [base[4][0] * c - base[4][1] * s, base[4][0] * s + base[4][1] * c],
        [base[5][0] * c - base[5][1] * s, base[5][0] * s + base[5][1] * c],
    ]
}

fn ensure_ship(world) {
    let WORLD_W = world.world_width();
    let WORLD_H = world.world_height();
    let PLANET_X = WORLD_W * 0.5;
    let PLANET_Y = WORLD_H * 0.5;
    let ORBIT_R  = 450.0;

    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    // Spawn at 3-o'clock to match sn=(1,0,0) initial orbital state.
    let ship_id = world.spawn_prefab("ship", #{
        x: PLANET_X + ORBIT_R,
        y: PLANET_Y,
        vx: 0.0,
        vy: 0.0,
    });
    ship_id
}

// ======================== ORBITAL CONSTANTS ===================================
// Read body and planet type from catalog; fall back to hardcoded defaults if
// catalog not loaded (e.g. during tests or before first-frame catalog init).
let PLANET_X       = WORLD_W * 0.5;   // world-space planet centre X (1600)
let PLANET_Y       = WORLD_H * 0.5;   // world-space planet centre Y (900)
let ORBIT_R        = 450.0;           // Constant orbital radius (px)
let PI             = 3.1415926535;
let ORBIT_PERIOD_S = 300.0;           // Target full orbit time: 5 minutes
let ORBIT_V0       = (2.0 * PI * ORBIT_R) / ORBIT_PERIOD_S;  // ~9.42 px/s at R=450
let MAX_SPD        = 28.0;            // Speed cap tuned for slower cinematic orbit
let ACC_FWD        = 0.9;             // Forward/back acceleration (px/s²)
let ACC_SIDE       = 0.6;             // Lateral acceleration (px/s²)
let LIN_DAMP       = 0.08;            // Gentle vacuum stabilization
let YAW_ACCEL      = 1.95;            // rad/s² — slightly snappier turn-on
let YAW_DAMP       = 2.6;             // yaw-rate damping — slightly faster settle
let YAW_MAX        = 1.1;             // rad/s
let CAM_SWAY_TAU   = 0.50;            // forward-lag / sway smoothing (seconds)
let CAM_SWAY_GAIN  = 0.28;            // sway amount from yaw rate — banking feel
let SHIP_RENDER_R  = 1.25;            // shared ship orbital radius in 3D camera space
let ORBIT_PREVIEW_ONLY = true;        // ship + planet chase preview only
let GAMEPLAY_ENABLED   = !ORBIT_PREVIEW_ONLY;
let AST_SLOT_L_COUNT = 8;
let AST_SLOT_M_COUNT = 8;
let AST_SLOT_S_COUNT = 12;
let AST_SLOT_L_PREFIX = "ast-slot-L-";
let AST_SLOT_M_PREFIX = "ast-slot-M-";
let AST_SLOT_S_PREFIX = "ast-slot-S-";
// HUD scale: KM_PER_PX converts px distances to km at Earth scale.
// WORLD_TIME_SCALE: 1 game-second ≈ WORLD_TIME_SCALE real-seconds for km/h display.
// Tuned so ORBIT_V0 at R=450 reads near orbital-class speeds while gameplay stays readable.
let KM_PER_PX      = 55.4;           // 6371 km / 115 px (Earth radius / surface radius)
let WORLD_TIME_SCALE = 83.0;
let SCENE_SCALE    = SHIP_RENDER_R / ORBIT_R;  // 2D world px -> shared chase-camera scene units

// Read body-specific constants from catalog (surface radius, spin rates).
// world.body_info / world.planet_type_info return maps with all defaults pre-filled.
let _body = world.body_info("main-planet");
let SURFACE_RADIUS = _body["surface_radius"];
let _ptype_id = if type_of(_body["planet_type"]) != "()" { _body["planet_type"] } else { "earth_like" };
let _ptype = world.planet_type_info(_ptype_id);
let PLANET_SPIN_DPS  = _ptype["surface_spin_dps"];
let CLOUD_SPIN_DPS   = _ptype["cloud_spin_dps"];
let CLOUD2_SPIN_DPS  = _ptype["cloud_spin_2_dps"];

// Use raw frame dt — engine pause steps cap tick rate to ~16ms, no need to clamp here.
let dt = frame_ms.to_float() / 1000.0;

// ======================== INITIALIZATION ===================================

if !(local.initialized ?? false) {
    world.set_world_bounds(0.0, 0.0, WORLD_W, WORLD_H);
    local.initialized = true;
    local.ship_id = 0;
    local.score = 0;
    local.lives = 3;
    local.ship_hp = 3;
    local.wave = 1;
    local.game_over = false;
    // Orbital ship state: sn = sphere normal (position), sf/sr = ship forward/right tangents.
    // yaw_rate rotates sf/sr around sn. Camera has separate inertial lag state (cam_n, cam_up).
    // Spawn at 3-o'clock (sn=+X), forward north (-Y on screen), right toward +Z.
    // The ship starts flat on the local tangent plane: nose follows sf, dorsal side follows sn.
    // vfwd = ORBIT_V0 (prograde north) gives circular XY orbit visible in 2D.
    local.snx = 1.0; local.sny = 0.0; local.snz = 0.0;
    local.sfx = 0.0; local.sfy = -1.0; local.sfz = 0.0;
    local.srx = 0.0; local.sry = 0.0; local.srz = 1.0;
    local.vfwd = ORBIT_V0; local.vright = 0.0;
    local.yaw_rate = 0.0;
    local.ship_heading = 0.0;
    local.ship_vis_pitch = PI;
    local.ship_vis_yaw = -PI * 0.5;
    local.ship_vis_roll = 0.0;
    local.cam_nx = 1.0; local.cam_ny =  0.0; local.cam_nz = 0.0;
    local.cam_fx = 0.0; local.cam_fy = -1.0; local.cam_fz = 0.0;  // lagged sf
    local.cam_sway = 0.0;
    local.thrust_hold_ms = 0.0;
    local.thrust_release_ms = 9999.0;
    local.bullets = #{};   // orbital bullet tracking: id_str → {snx,sny,snz,dirx,diry,dirz,speed}
    local.ast_slots = #{};  // generator slot map: entity_id_str → slot_id
    input.load_profile("default");
    if GAMEPLAY_ENABLED {
        spawn_wave(world, 1);
    }
}

// ======================== MAIN LOOP ========================================

// ── Spherical ship state (spec §3) ────────────────────────────────────────────
// sn  = unit normal from planet center to ship (all 3 axes, always length 1)
// vfwd/vright = speed in the local forward/right tangent directions (px/s)
// ── Orbital state ─────────────────────────────────────────────────────────────
// sn = unit normal from planet to ship. sf/sr = ship local forward/right tangents.
// vfwd/vright are ship-frame tangential speeds; yaw_rate rotates sf/sr around sn.
let snx = local.snx ?? 1.0;
let sny = local.sny ?? 0.0;
let snz = local.snz ?? 0.0;
let sfx = local.sfx ?? 0.0;
let sfy = local.sfy ?? -1.0;
let sfz = local.sfz ?? 0.0;
let srx = local.srx ?? 0.0;
let sry = local.sry ?? 0.0;
let srz = local.srz ?? 1.0;
let vfwd   = local.vfwd   ?? 0.0;
let vright = local.vright ?? 0.0;
let yaw_rate = local.yaw_rate ?? 0.0;
let ship_heading = local.ship_heading ?? heading_from_forward(sfx, sfy, 0.0);
let ship_vis_pitch = local.ship_vis_pitch ?? PI;
let ship_vis_yaw = local.ship_vis_yaw ?? -PI * 0.5;
let ship_vis_roll = local.ship_vis_roll ?? 0.0;
let cam_nx = local.cam_nx ?? snx;
let cam_ny = local.cam_ny ?? sny;
let cam_nz = local.cam_nz ?? snz;
let cam_fx = local.cam_fx ?? sfx;
let cam_fy = local.cam_fy ?? sfy;
let cam_fz = local.cam_fz ?? sfz;
let cam_sway = local.cam_sway ?? 0.0;

let score     = local.score ?? 0;
let lives     = local.lives ?? 3;
let ship_hp   = local.ship_hp ?? 3;
let wave      = local.wave ?? 1;
let game_over = local.game_over ?? false;
let respawning = local.respawning ?? false;
let invincible = local.invincible ?? false;

// ── Engine timer resolution ───────────────────────────────────────────────────
if world.timer_fired("respawn_done") {
    local.respawning = false;
    respawning = false;
    local.invincible = true;
    invincible = true;
    local.just_spawned = true;
    world.after_ms("invincible_done", 2000);
}
if world.timer_fired("invincible_done") {
    local.invincible = false;
    invincible = false;
}
if world.timer_fired("hit_invincible_done") {
    local.invincible = false;
    invincible = false;
}

// ── HUD game state sync ───────────────────────────────────────────────────────
game.set("/score", score);
game.set("/wave", wave);
scene.set("heart-1", "visible", lives >= 1);
scene.set("heart-2", "visible", lives >= 2);
scene.set("heart-3", "visible", lives >= 3);

// ── ESC key: return to main menu ──────────────────────────────────────────────
if !game_over && (key.code == "Escape" || key.code == "escape") {
    world.reset_dynamic_entities();
    game.jump("asteroids-mainmenu");
}

// ── Game-over state ─────────────────────────────────────────────────────────
if game_over {
    // Scene transition already queued from the collision handler; just wait.
    #{state: local}
} else {

// ── Ship controls ────────────────────────────────────────────────────────────
let ship_id = if !respawning { ensure_ship(world) } else { 0 };
local.ship_id = ship_id;
scene.set("ship-scene-slot", "visible", ship_id > 0);

// ── Ship tracking — ALWAYS run (even during respawn) ─────────────────────────
// Use last known ship position so planet stays centred during the 3-second respawn window.
let ship_x = local.last_ship_x ?? 1600.0;
let ship_y = local.last_ship_y ?? 900.0;
// Input axes: declared in outer scope so both ship_id blocks can use them.
let thrust_axis = 0.0;
let strafe_axis = 0.0;
let yaw_axis = 0.0;

if ship_id > 0 {
    let xf = world.transform(ship_id) ?? #{};
    ship_x = xf["x"] ?? ship_x;
    ship_y = xf["y"] ?? ship_y;
    local.last_ship_x = ship_x;
    local.last_ship_y = ship_y;

    // ── Respawn reset: restore orbital state to spawn values ──────────────────
    if local.just_spawned ?? false {
        snx = 1.0; sny = 0.0; snz = 0.0;
        sfx = 0.0; sfy = -1.0; sfz = 0.0;
        srx = 0.0; sry = 0.0; srz = 1.0;
        vfwd = ORBIT_V0; vright = 0.0;
        yaw_rate = 0.0; ship_heading = 0.0;
        ship_vis_pitch = PI; ship_vis_yaw = -PI * 0.5; ship_vis_roll = 0.0;
        cam_nx = snx; cam_ny = sny; cam_nz = snz;
        cam_fx = sfx; cam_fy = sfy; cam_fz = sfz;
        cam_sway = 0.0;
        local.thrust_hold_ms = 0.0;
        local.thrust_release_ms = 9999.0;
        local.snx = snx; local.sny = sny; local.snz = snz;
        local.sfx = sfx; local.sfy = sfy; local.sfz = sfz;
        local.srx = srx; local.sry = sry; local.srz = srz;
        local.vfwd = vfwd; local.vright = vright;
        local.yaw_rate = yaw_rate;
        local.ship_heading = ship_heading;
        local.ship_vis_pitch = ship_vis_pitch;
        local.ship_vis_yaw = ship_vis_yaw;
        local.ship_vis_roll = ship_vis_roll;
        local.cam_nx = cam_nx; local.cam_ny = cam_ny; local.cam_nz = cam_nz;
        local.cam_fx = cam_fx; local.cam_fy = cam_fy; local.cam_fz = cam_fz;
        local.bullets = #{};
    }

    // ── Step 1: read pilot axes (yaw + thrust + strafe) ───────────────────────
    thrust_axis =
        if input.action_down("thrust") && !input.action_down("brake") { 1.0 }
        else if input.action_down("brake") && !input.action_down("thrust") { -1.0 }
        else { 0.0 };
    strafe_axis =
        if input.action_down("strafe_right") && !input.action_down("strafe_left") { 1.0 }
        else if input.action_down("strafe_left") && !input.action_down("strafe_right") { -1.0 }
        else { 0.0 };
    // Sign intentionally inverted to match expected on-screen turn direction.
    yaw_axis =
        if input.action_down("turn_right") && !input.action_down("turn_left") { -1.0 }
        else if input.action_down("turn_left") && !input.action_down("turn_right") { 1.0 }
        else { 0.0 };

    // ── Step 2: RCS yaw dynamics (source of truth for rotation) ───────────────
    yaw_rate += yaw_axis * YAW_ACCEL * dt;
    yaw_rate *= 1.0 / (1.0 + YAW_DAMP * dt);
    if yaw_rate > YAW_MAX { yaw_rate = YAW_MAX; }
    if yaw_rate < -YAW_MAX { yaw_rate = -YAW_MAX; }
    let _dyaw = yaw_rate * dt;
    if absf(_dyaw) > 0.000001 {
        let _rsf_yaw = rotate_axis(sfx, sfy, sfz, snx, sny, snz, _dyaw);
        sfx = _rsf_yaw.x; sfy = _rsf_yaw.y; sfz = _rsf_yaw.z;
        let _rsr_yaw = rotate_axis(srx, sry, srz, snx, sny, snz, _dyaw);
        srx = _rsr_yaw.x; sry = _rsr_yaw.y; srz = _rsr_yaw.z;
    }

    // ── Step 3: translational thrust/strafe in ship tangent basis ─────────────
    vfwd   += thrust_axis * ACC_FWD * dt;
    vright += strafe_axis * ACC_SIDE * dt;
    vfwd   *= 1.0 / (1.0 + LIN_DAMP * dt);
    vright *= 1.0 / (1.0 + LIN_DAMP * dt);
    if vfwd   >  MAX_SPD        { vfwd   =  MAX_SPD; }
    if vfwd   < -MAX_SPD * 0.5  { vfwd   = -MAX_SPD * 0.5; }
    if vright >  MAX_SPD        { vright =  MAX_SPD; }
    if vright < -MAX_SPD        { vright = -MAX_SPD; }

    // ── Step 4: move on sphere via geodesic transport ─────────────────────────
    let vel3x = sfx * vfwd + srx * vright;
    let vel3y = sfy * vfwd + sry * vright;
    let vel3z = sfz * vfwd + srz * vright;
    let _v2 = vel3x*vel3x + vel3y*vel3y + vel3z*vel3z;
    if _v2 > 0.001 {
        // Rotation axis = normalize(cross(sn, v_tan))
        let _ax = sny*vel3z - snz*vel3y;
        let _ay = snz*vel3x - snx*vel3z;
        let _az = snx*vel3y - sny*vel3x;
        let _al = sqrtf(_ax*_ax + _ay*_ay + _az*_az).max(0.0001);
        _ax /= _al; _ay /= _al; _az /= _al;
        let _delta = sqrtf(_v2) * dt / ORBIT_R;
        // Parallel transport: rotate sn and sf by same geodesic step
        let _rsn = rotate_axis(snx, sny, snz, _ax, _ay, _az, _delta);
        snx = _rsn.x; sny = _rsn.y; snz = _rsn.z;
        let _rsf = rotate_axis(sfx, sfy, sfz, _ax, _ay, _az, _delta);
        sfx = _rsf.x; sfy = _rsf.y; sfz = _rsf.z;
        // Normalize sn
        let _nl = sqrtf(snx*snx + sny*sny + snz*snz).max(0.0001);
        snx /= _nl; sny /= _nl; snz /= _nl;
        // Re-orthonormalize sf against sn and rebuild sr
        let _dsf = sfx*snx + sfy*sny + sfz*snz;
        sfx -= snx * _dsf; sfy -= sny * _dsf; sfz -= snz * _dsf;
        let _sfl = sqrtf(sfx*sfx + sfy*sfy + sfz*sfz).max(0.0001);
        sfx /= _sfl; sfy /= _sfl; sfz /= _sfl;
        srx = sfy*snz - sfz*sny;
        sry = sfz*snx - sfx*snz;
        srz = sfx*sny - sfy*snx;
        let _srl = sqrtf(srx*srx + sry*sry + srz*srz).max(0.0001);
        srx /= _srl; sry /= _srl; srz /= _srl;
    }

    // ── Step 5: reproject velocity onto updated tangent plane ──────────────────
    let _dot_new = vel3x*snx + vel3y*sny + vel3z*snz;
    let _vpx = vel3x - snx*_dot_new;
    let _vpy = vel3y - sny*_dot_new;
    let _vpz = vel3z - snz*_dot_new;
    vfwd   = _vpx*sfx + _vpy*sfy + _vpz*sfz;
    vright = _vpx*srx + _vpy*sry + _vpz*srz;

    // ── Step 6: ship transform follows the full orbital basis ──────────────────
    // Mesh local axes: +Y = nose, +X = starboard, +Z = dorsal/up.
    // World mapping:    sf = nose-forward tangent, sr = starboard tangent,
    //                   sn = radial up away from the planet.
    let _ship_layer = "ship-" + ship_id;
    let _ship_sprite = "ship-scene-slot";
    ship_heading = heading_from_forward(sfx, sfy, ship_heading);
    let _ship_euler = ship_basis_euler(
        srx, sry, srz,
        sfx, sfy, sfz,
        snx, sny, snz,
        ship_vis_pitch, ship_vis_yaw, ship_vis_roll
    );
    ship_vis_pitch = _ship_euler["pitch"] ?? ship_vis_pitch;
    ship_vis_yaw = _ship_euler["yaw"] ?? ship_vis_yaw;
    ship_vis_roll = _ship_euler["roll"] ?? ship_vis_roll;
    scene.set(_ship_layer, "visible", false);
    scene.set(_ship_sprite, "obj.pitch", ship_vis_pitch * 57.29577951);
    scene.set(_ship_sprite, "obj.yaw",   ship_vis_yaw   * 57.29577951);
    scene.set(_ship_sprite, "obj.roll",  ship_vis_roll  * 57.29577951);
    scene.set(_ship_sprite, "obj.world.x", snx * SHIP_RENDER_R);
    scene.set(_ship_sprite, "obj.world.y", sny * SHIP_RENDER_R);
    scene.set(_ship_sprite, "obj.world.z", snz * SHIP_RENDER_R);
    // 2D support systems still use the projected XY heading from the same forward basis.
    world.entity(ship_id).set_heading(ship_heading);

    // ── Step 7: persist orbital state ──────────────────────────────────────────
    local.snx = snx; local.sny = sny; local.snz = snz;
    local.sfx = sfx; local.sfy = sfy; local.sfz = sfz;
    local.srx = srx; local.sry = sry; local.srz = srz;
    local.vfwd = vfwd; local.vright = vright;
    local.yaw_rate = yaw_rate;
    local.ship_heading = ship_heading;
    local.ship_vis_pitch = ship_vis_pitch;
    local.ship_vis_yaw = ship_vis_yaw;
    local.ship_vis_roll = ship_vis_roll;

    // ── Step 8: drive engine entity to sphere position ─────────────────────────
    // Set vel = (desired - current) / dt so engine's physics step lands ship on sphere.
    let _dx = PLANET_X + snx * ORBIT_R;
    let _dy = PLANET_Y + sny * ORBIT_R;
    world.set_physics(ship_id,
        (_dx - ship_x) / dt,
        (_dy - ship_y) / dt,
        0.0, 0.0, 0.0, 0.0);
}  // end if ship_id > 0

// Camera follows ship — ship is always at screen centre (320, 180).
// Use the freshly-computed orbital position (snx/sny updated this frame),
// not the old transform position, so 2D camera matches the 3D planet camera exactly.
let _cam_ship_x = PLANET_X + snx * ORBIT_R;
let _cam_ship_y = PLANET_Y + sny * ORBIT_R;
world.set_camera(_cam_ship_x - 320.0, _cam_ship_y - 180.0);


// ── Planet axial self-rotation (independent of camera/orbit) ──────────────────
local.planet_self_yaw  = (local.planet_self_yaw  ?? 0.0) + PLANET_SPIN_DPS  * dt;
local.cloud_self_yaw   = (local.cloud_self_yaw   ?? 0.0) + CLOUD_SPIN_DPS   * dt;
local.cloud_self_yaw_2 = (local.cloud_self_yaw_2 ?? 90.0) + CLOUD2_SPIN_DPS * dt;

// ── Chase (behind-ship) 3D camera ─────────────────────────────────────────────
// Camera sits behind the ship and stays close to the local tangent plane.
// sn = local up (radial away from planet), sf = ship forward tangent.
let CAM_LIFT      = 0.08;  // small rise above the hull, but still tangent-dominant
let CAM_BACK      = 0.62;  // pull the camera a bit farther behind the ship
let CAM_AHEAD     = 0.42;  // look deeper into the flight path
let CAM_LOOK_LIFT = 0.02;  // keep target near the ship instead of aiming down at the planet
let _cam_valid = snx*snx + sny*sny + snz*snz > 0.01;
if _cam_valid {
    // Lag cam_n toward sn (smooth orbital position — no instant snapping)
    let _alpha_n = dt / (0.30 + dt);
    cam_nx += (snx - cam_nx) * _alpha_n;
    cam_ny += (sny - cam_ny) * _alpha_n;
    cam_nz += (snz - cam_nz) * _alpha_n;
    let _cnl = sqrtf(cam_nx*cam_nx + cam_ny*cam_ny + cam_nz*cam_nz).max(0.0001);
    cam_nx /= _cnl; cam_ny /= _cnl; cam_nz /= _cnl;

    // Lag cam_f toward sf (smooth forward direction — banks on yaw)
    let _alpha_f = dt / (CAM_SWAY_TAU + dt);
    cam_fx += (sfx - cam_fx) * _alpha_f;
    cam_fy += (sfy - cam_fy) * _alpha_f;
    cam_fz += (sfz - cam_fz) * _alpha_f;
    // Re-orthogonalize cam_f against cam_n (keep it tangential to sphere)
    let _fd = cam_fx*cam_nx + cam_fy*cam_ny + cam_fz*cam_nz;
    cam_fx -= cam_nx*_fd; cam_fy -= cam_ny*_fd; cam_fz -= cam_nz*_fd;
    let _cfl = sqrtf(cam_fx*cam_fx + cam_fy*cam_fy + cam_fz*cam_fz).max(0.0001);
    cam_fx /= _cfl; cam_fy /= _cfl; cam_fz /= _cfl;

    // Eye: behind ship with only a slight lift so the shot reads as "behind"
    // rather than "hovering above and looking down".
    let _ex = cam_nx * (SHIP_RENDER_R + CAM_LIFT) - cam_fx * CAM_BACK;
    let _ey = cam_ny * (SHIP_RENDER_R + CAM_LIFT) - cam_fy * CAM_BACK;
    let _ez = cam_nz * (SHIP_RENDER_R + CAM_LIFT) - cam_fz * CAM_BACK;

    // Look-at: target the ship's orbital position and a point farther ahead on
    // the tangent so the hull reads flat against the horizon.
    let _lx = cam_nx * (SHIP_RENDER_R + CAM_LOOK_LIFT) + cam_fx * CAM_AHEAD;
    let _ly = cam_ny * (SHIP_RENDER_R + CAM_LOOK_LIFT) + cam_fy * CAM_AHEAD;
    let _lz = cam_nz * (SHIP_RENDER_R + CAM_LOOK_LIFT) + cam_fz * CAM_AHEAD;

    // Camera-up = radial direction (sn = "sky"), with banking sway around cam_f
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

// Self-rotation only: obj.yaw = planet's own axial spin (no orbital omega_z needed).
// Planet axial tilt is fixed at -14° pitch (set in YAML; don't override it here).
let surface_yaw  = local.planet_self_yaw  ?? 0.0;
let cloud_yaw    = local.cloud_self_yaw   ?? 0.0;
let cloud_yaw_2  = local.cloud_self_yaw_2 ?? 90.0;
scene.set("planet-bg-sphere", "obj.yaw", surface_yaw);
scene.set("planet-clouds",    "obj.yaw", cloud_yaw);
scene.set("planet-clouds-2",  "obj.yaw", cloud_yaw_2);

// ── Gravity on asteroid bodies ────────────────────────────────────────────────
// Simple centripetal pull toward planet. G chosen so v≈56 at r=180.
let AST_G = 565000.0;
for grav_id in world.query_tag("asteroid") {
    let gxf = world.transform(grav_id) ?? #{};
    let egx = PLANET_X - (gxf["x"] ?? PLANET_X);
    let egy = PLANET_Y - (gxf["y"] ?? PLANET_Y);
    let edist = sqrtf(egx * egx + egy * egy).max(SURFACE_RADIUS);
    let egrav = AST_G / (edist * edist);
    let ephys = world.physics(grav_id);
    if type_of(ephys) == "map" {
        world.set_physics(grav_id,
            (ephys["vx"] ?? 0.0) + egx / edist * egrav * dt,
            (ephys["vy"] ?? 0.0) + egy / edist * egrav * dt,
            ephys["ax"] ?? 0.0, ephys["ay"] ?? 0.0,
            ephys["drag"] ?? 0.0, ephys["max_speed"] ?? 0.0);
    }
}

// ── Asteroid generator — map physics entities to 3D scene slot pool ───────────
// Each frame: hide all slots, then show+position only those with a live entity.
// Slot assignment is sticky (stored in local.ast_slots) so tumble rotation is
// continuous — obj.orbit_speed drives the renderer spin from elapsed_ms.
hide_asteroid_slots(scene, AST_SLOT_L_PREFIX, AST_SLOT_L_COUNT);
hide_asteroid_slots(scene, AST_SLOT_M_PREFIX, AST_SLOT_M_COUNT);
hide_asteroid_slots(scene, AST_SLOT_S_PREFIX, AST_SLOT_S_COUNT);

// Gather live asteroid IDs by size.
let _live_large  = [];
let _live_medium = [];
let _live_small  = [];
for _aid in world.query_tag("asteroid") {
    if      world.tag_has(_aid, "asteroid-large")  { _live_large.push(_aid);  }
    else if world.tag_has(_aid, "asteroid-medium") { _live_medium.push(_aid); }
    else                                            { _live_small.push(_aid);  }
}

// Persistent slot map: entity_id_str → slot_id.
let _ast_slots = prune_dead_asteroid_slots(world, local.ast_slots ?? #{});

if _live_large.len() > AST_SLOT_L_COUNT {
    if !(local.ast_slot_warn_large ?? false) {
        diag.warn(`[AST SLOT] large overflow: live=${_live_large.len()} slots=${AST_SLOT_L_COUNT}`);
    }
    local.ast_slot_warn_large = true;
} else {
    local.ast_slot_warn_large = false;
}
if _live_medium.len() > AST_SLOT_M_COUNT {
    if !(local.ast_slot_warn_medium ?? false) {
        diag.warn(`[AST SLOT] medium overflow: live=${_live_medium.len()} slots=${AST_SLOT_M_COUNT}`);
    }
    local.ast_slot_warn_medium = true;
} else {
    local.ast_slot_warn_medium = false;
}
if _live_small.len() > AST_SLOT_S_COUNT {
    if !(local.ast_slot_warn_small ?? false) {
        diag.warn(`[AST SLOT] small overflow: live=${_live_small.len()} slots=${AST_SLOT_S_COUNT}`);
    }
    local.ast_slot_warn_small = true;
} else {
    local.ast_slot_warn_small = false;
}

_ast_slots = sync_asteroid_slot_group(
    scene, world, _ast_slots, _live_large,
    AST_SLOT_L_PREFIX, AST_SLOT_L_COUNT,
    PLANET_X, PLANET_Y, SCENE_SCALE
);
_ast_slots = sync_asteroid_slot_group(
    scene, world, _ast_slots, _live_medium,
    AST_SLOT_M_PREFIX, AST_SLOT_M_COUNT,
    PLANET_X, PLANET_Y, SCENE_SCALE
);
_ast_slots = sync_asteroid_slot_group(
    scene, world, _ast_slots, _live_small,
    AST_SLOT_S_PREFIX, AST_SLOT_S_COUNT,
    PLANET_X, PLANET_Y, SCENE_SCALE
);

local.ast_slots = _ast_slots;

// Other gameplay entities now rely on particles or scene-layer visuals rather
// than prefab sprite layers, so keep their runtime visual layers hidden.
for _sid in world.query_tag("shrapnel") { scene.set("shrapnel-" + _sid, "visible", false); }
for _did in world.query_tag("debris")   { scene.set("debris-" + _did,   "visible", false); }


// Speed in km/h using world scale: 1 px = KM_PER_PX km, time compressed by WORLD_TIME_SCALE.
let _speed_px_s = sqrtf(vfwd*vfwd + vright*vright);
let _speed_kmh  = (_speed_px_s * KM_PER_PX / WORLD_TIME_SCALE * 3600.0).to_int();
let _alt_km     = ((ORBIT_R - SURFACE_RADIUS) * KM_PER_PX).to_int();
let hud_alt     = "ALT  " + _alt_km + " km";
let hud_spd     = "SPD  " + _speed_kmh + " km/h";
// Orbital status baseline: circular orbit at ORBIT_R with 5-minute period.
let _v_circ_kmh = (ORBIT_V0 * KM_PER_PX / WORLD_TIME_SCALE * 3600.0).to_int();
let hud_status = if _speed_kmh > _v_circ_kmh * 2 { "FAST!" }
                 else if _speed_kmh > _v_circ_kmh + 5000 { "FAST" }
                 else if _speed_kmh < 1000 { "SLOW" }
                 else { "ORBIT ✓" };
scene.set("hud-orbital-alt",    "content", hud_alt);
scene.set("hud-orbital-spd",    "content", hud_spd);
scene.set("hud-orbital-status", "content", hud_status);

if ship_id > 0 {
    world.entity(ship_id).physics.set_collider_polygon(ship_hull_polygon(ship_heading), 1, 6);

    // ── Warp-in materialisation on first frame after respawn ──────────────────
    if local.just_spawned ?? false {
        local.just_spawned = false;
        world.emit("ship.warp_in", ship_id, #{ count: 26, speed: 120.0, spread: 6.28, fg: col.ship ?? "white" });
        world.emit("ship.warp_in", ship_id, #{ count: 10, speed:  60.0, spread: 6.28, fg: col.ship ?? "white" });
        world.emit("asteroids.impact_sparks", ship_id, #{ count: 10, speed: 70.0, spread: 3.14, fg: "#AADDFF" });
        effects.shake(300, 2.5, 1.8, 10.0);
        effects.trigger("whiteout", 200, #{ intensity: 0.5 });
    }

    // ── Original-style orbital VFX orchestration (main/bow/port/starboard) ────
    let _thrust_now  = thrust_axis > 0.0;
    let _hold_ms = if _thrust_now {
        (local.thrust_hold_ms ?? 0.0) + frame_ms.to_float()
    } else { 0.0 };
    let _release_ms = if _thrust_now {
        0.0
    } else {
        ((local.thrust_release_ms ?? 9999.0) + frame_ms.to_float()).min(9999.0)
    };
    local.thrust_hold_ms = _hold_ms;
    local.thrust_release_ms = _release_ms;
    let _ship_speed = sqrtf(vfwd * vfwd + vright * vright);

    rcs::update_orbital_vfx_original_style(
        world,
        ship_id,
        thrust_axis,
        yaw_axis,
        yaw_rate,
        _hold_ms,
        _release_ms,
        vfwd,
        vright,
        _ship_speed,
        col
    );

    // ── Shooting ─────────────────────────────────────────────────────────────
    if GAMEPLAY_ENABLED && input.action_just_pressed("fire") && world.entity(ship_id).cooldown_ready("fire") {
        let _ship_xy = heading_dir_xy(ship_heading);
        // Spawn bullet from the projected nose position used by 2D support systems.
        let bullet_id = world.spawn_prefab("bullet", #{
            x: ship_x + (_ship_xy["x"] ?? 0.0) * 17.0,
            y: ship_y + (_ship_xy["y"] ?? -1.0) * 17.0,
            vx: 0.0, vy: 0.0,
            heading: ship_heading, ttl_ms: 1800,
        });
        // Store orbital state: bullet carries full ship tangential velocity + muzzle speed.
        // Full vel3 = sf*(muzzle + vfwd) + sr*vright, then normalize to get direction and speed.
        let _bvx = sfx * (24.0 + vfwd) + srx * vright;
        let _bvy = sfy * (24.0 + vfwd) + sry * vright;
        let _bvz = sfz * (24.0 + vfwd) + srz * vright;
        let _bspd = sqrtf(_bvx*_bvx + _bvy*_bvy + _bvz*_bvz).max(0.1);
        let _bullets = local.bullets ?? #{};
        _bullets[bullet_id + ""] = #{
            snx: snx, sny: sny, snz: snz,
            dirx: _bvx / _bspd, diry: _bvy / _bspd, dirz: _bvz / _bspd,
            speed: _bspd,
        };
        local.bullets = _bullets;
        world.entity(ship_id).cooldown_start("fire", 180);

        // Recoil: backward kick in orbital tangent frame
        vfwd -= 14.0;
        if vfwd < -MAX_SPD * 0.5 { vfwd = -MAX_SPD * 0.5; }
        local.vfwd = vfwd;
    }

    // ── Orbital bullet update — move each bullet along the sphere ─────────────
    let _alive_bullets = world.query_tag("bullet");
    let _old_b = local.bullets ?? #{};
    let _new_b = #{};
    for _bid in _alive_bullets {
        scene.set("bullet-" + _bid, "visible", false);
        let _bkey = _bid + "";
        let _bs = _old_b[_bkey] ?? ();
        if type_of(_bs) != "map" { continue; }  // not an orbital bullet, skip

        let _bsnx = _bs["snx"] ?? 0.0;
        let _bsny = _bs["sny"] ?? -1.0;
        let _bsnz = _bs["snz"] ?? 0.0;
        let _bdx  = _bs["dirx"] ?? 1.0;
        let _bdy  = _bs["diry"] ?? 0.0;
        let _bdz  = _bs["dirz"] ?? 0.0;
        let _bspd = _bs["speed"] ?? 28.0;

        // Normalize sn
        let _bnl = sqrtf(_bsnx*_bsnx + _bsny*_bsny + _bsnz*_bsnz).max(0.0001);
        _bsnx /= _bnl; _bsny /= _bnl; _bsnz /= _bnl;
        // Project dir onto tangent plane
        let _bdd = _bdx*_bsnx + _bdy*_bsny + _bdz*_bsnz;
        _bdx -= _bsnx*_bdd; _bdy -= _bsny*_bdd; _bdz -= _bsnz*_bdd;
        let _bdl = sqrtf(_bdx*_bdx + _bdy*_bdy + _bdz*_bdz).max(0.0001);
        _bdx /= _bdl; _bdy /= _bdl; _bdz /= _bdl;

        // Geodesic rotation axis = cross(sn, dir) (already tangential, just normalize)
        let _bax = _bsny*_bdz - _bsnz*_bdy;
        let _bay = _bsnz*_bdx - _bsnx*_bdz;
        let _baz = _bsnx*_bdy - _bsny*_bdx;
        let _bal = sqrtf(_bax*_bax + _bay*_bay + _baz*_baz).max(0.0001);
        _bax /= _bal; _bay /= _bal; _baz /= _bal;

        let _bdang = _bspd * dt / ORBIT_R;
        let _brsn = rotate_axis(_bsnx, _bsny, _bsnz, _bax, _bay, _baz, _bdang);
        _bsnx = _brsn.x; _bsny = _brsn.y; _bsnz = _brsn.z;
        let _brdir = rotate_axis(_bdx, _bdy, _bdz, _bax, _bay, _baz, _bdang);
        _bdx = _brdir.x; _bdy = _brdir.y; _bdz = _brdir.z;

        let _bx = PLANET_X + _bsnx * ORBIT_R;
        let _by = PLANET_Y + _bsny * ORBIT_R;
        let _bxf = world.transform(_bid) ?? #{};
        let _bpx = _bxf["x"] ?? _bx;
        let _bpy = _bxf["y"] ?? _by;
        world.set_physics(_bid, (_bx - _bpx) / dt, (_by - _bpy) / dt, 0.0, 0.0, 0.0, 0.0);

        // Trail VFX
        world.emit("bullet.trail", _bid, #{ count: 4, color_ramp: col.ramp_bullet_trail });

        _new_b[_bkey] = #{
            snx: _bsnx, sny: _bsny, snz: _bsnz,
            dirx: _bdx, diry: _bdy, dirz: _bdz,
            speed: _bspd,
        };
    }
    local.bullets = _new_b;
}

// ── Wave complete check ───────────────────────────────────────────────────────
if GAMEPLAY_ENABLED && world.count_kind("asteroid") == 0 && world.count_tag("bullet") == 0 && !respawning {
    wave = wave + 1;
    local.wave = wave;
    spawn_wave(world, wave);
}

// ── Collision handling ────────────────────────────────────────────────────────
// bullet hits asteroid — emit debris, score, split
for hit in collision.enters("bullet", "asteroid") {
    let bullet_id   = hit["bullet"];
    let asteroid_id = hit["asteroid"];
    if !world.exists(bullet_id) || !world.exists(asteroid_id) { continue; }
    let axf = world.transform(asteroid_id);
    let ax = if type_of(axf) == "map" { axf["x"] ?? 0.0 } else { 0.0 };
    let ay = if type_of(axf) == "map" { axf["y"] ?? 0.0 } else { 0.0 };
    let aphys = world.physics(asteroid_id);
    let avx = if type_of(aphys) == "map" { aphys["vx"] ?? 0.0 } else { 0.0 };
    let avy = if type_of(aphys) == "map" { aphys["vy"] ?? 0.0 } else { 0.0 };

    // Engine-level elastic impulse already applied before scripts run.
    // Read bullet velocity for shrapnel spawn direction.
    let bphys = world.physics(bullet_id);
    let bvx = if type_of(bphys) == "map" { bphys["vx"] ?? 0.0 } else { 0.0 };
    let bvy = if type_of(bphys) == "map" { bphys["vy"] ?? 0.0 } else { 0.0 };

    // Shrapnel: real physics entities — ricochet off asteroids, damage ship
    let shard_count = if world.tag_has(asteroid_id, "asteroid-large") { 5 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 4 }
                 else { 3 };
    let shard_speed = if world.tag_has(asteroid_id, "asteroid-large") { 220.0 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 280.0 }
                 else { 340.0 };
    spawn_shrapnel(world, ax, ay, bvx, bvy, avx, avy, shard_count, shard_speed);

    // Fragment halo: glowing dust cloud that fades to background over 2s (visual only)
    let frag_count = shard_count - 1;
    let frag_speed = shard_speed * 0.4;
    world.emit("asteroid.impact_fragments", asteroid_id, #{
        count: frag_count, speed: frag_speed, spread: 6.28, fg: col.asteroid ?? "white"
    });

    // Impact sparks and debris
    let spark_count = if world.tag_has(asteroid_id, "asteroid-large") { 12 }
                 else if world.tag_has(asteroid_id, "asteroid-medium") { 8 }
                 else { 5 };
    world.emit("asteroids.impact_sparks", asteroid_id, #{
        count: spark_count, speed: 60.0, spread: 3.14, fg: col.asteroid ?? "white"
    });
    world.emit("asteroid.debris", asteroid_id, #{
        count: spark_count / 2, speed: 35.0, spread: 6.28, fg: col.asteroid ?? "white"
    });

    // Score + split
    if world.tag_has(asteroid_id, "asteroid-large") {
        score = score + 20;
        world.despawn(asteroid_id);
        spawn_medium_asteroids(world, ax, ay, avx, avy);
    } else if world.tag_has(asteroid_id, "asteroid-medium") {
        score = score + 50;
        world.despawn(asteroid_id);
        spawn_small_asteroids(world, ax, ay, avx, avy);
    } else if world.tag_has(asteroid_id, "asteroid-small") {
        score = score + 100;
        world.despawn(asteroid_id);
    }
    local.score = score;
    // Let bullet ricochet briefly instead of instant despawn — engine impulse already deflected it
    world.set_lifetime(bullet_id, 250);
}

// shrapnel hits asteroid — ricochet: reflect velocity × 0.5, nudge asteroid
for hit in collision.enters("shrapnel", "asteroid") {
    let s_id = hit["shrapnel"];
    let a_id = hit["asteroid"];
    if !world.exists(s_id) || !world.exists(a_id) { continue; }
    let sphys = world.physics(s_id);
    let svx = if type_of(sphys) == "map" { sphys["vx"] ?? 0.0 } else { 0.0 };
    let svy = if type_of(sphys) == "map" { sphys["vy"] ?? 0.0 } else { 0.0 };
    // Reflect and bleed 50% speed
    if type_of(sphys) == "map" {
        world.set_physics(s_id,
            -svx * 0.5, -svy * 0.5,
            sphys["ax"] ?? 0.0, sphys["ay"] ?? 0.0,
            sphys["drag"] ?? 0.0, sphys["max_speed"] ?? 0.0);
    }
    // 3% momentum nudge to the asteroid
    let aphys = world.physics(a_id);
    if type_of(aphys) == "map" {
        let push = 0.03;
        world.set_physics(a_id,
            (aphys["vx"] ?? 0.0) + svx * push,
            (aphys["vy"] ?? 0.0) + svy * push,
            aphys["ax"] ?? 0.0, aphys["ay"] ?? 0.0,
            aphys["drag"] ?? 0.0, aphys["max_speed"] ?? 0.0);
    }
    // Spark flash at ricochet
    world.emit("asteroids.impact_sparks", a_id, #{
        count: 3, speed: 40.0, spread: 2.0, fg: col.asteroid ?? "white"
    });
}

// shrapnel hits ship — 3-hit HP system: lose a life only on 3rd hit
if !invincible && ship_id > 0 {
    for hit in collision.enters("shrapnel", "ship") {
        let s_id      = hit["shrapnel"];
        let hit_ship  = hit["ship"];
        if hit_ship != ship_id || !world.exists(s_id) { continue; }
        world.despawn(s_id);

        ship_hp = ship_hp - 1;
        local.ship_hp = ship_hp;

        if ship_hp <= 0 {
            // Full death sequence
            do_ship_death_fx(world, effects, col, ship_id);
            world.despawn(ship_id);
            local.ship_id = 0;
            ship_id = 0;
            local.ship_hp = 3;
            ship_hp = 3;
            lives = lives - 1;
            local.lives = lives;
            if lives <= 0 {
                local.game_over = true;
                local.lives = 0;
                let final_score = score;
                let result = merge_highscores(game.get("/highscores/scores"), final_score, wave);
                let rank = result["rank"];
                if rank >= 0 {
                    game.set("/highscores/scores", result["scores"]);
                    game.set("/highscores/last_entry_rank", rank);
                } else {
                    game.set("/highscores/last_entry_rank", -1);
                }
                game.set("/score", final_score);
                clear_round_entities(world);
                game.jump("asteroids-highscores");
            } else {
                local.respawning = true;
                world.after_ms("respawn_done", 3000);
            }
        } else {
            // Hit flash — brief invincibility, small spark burst only
            world.emit("asteroids.impact_sparks", ship_id, #{
                count: 6, speed: 50.0, spread: 6.28, fg: col.ship ?? "white"
            });
            local.invincible = true;
            invincible = true;
            world.after_ms("hit_invincible_done", 800);
        }
        break;
    }
}

// ship hits asteroid — 3-hit HP system: lose a life only on 3rd hit
if !invincible && ship_id > 0 {
    for hit in collision.enters("ship", "asteroid") {
        let hit_ship_id = hit["ship"];
        if hit_ship_id != ship_id { continue; }

        ship_hp = ship_hp - 1;
        local.ship_hp = ship_hp;

        if ship_hp <= 0 {
            // Full death sequence
            do_ship_death_fx(world, effects, col, ship_id);
            world.despawn(ship_id);
            local.ship_id = 0;
            ship_id = 0;
            local.ship_hp = 3;
            ship_hp = 3;
            lives = lives - 1;
            local.lives = lives;
            if lives <= 0 {
                local.game_over = true;
                local.lives = 0;
                let final_score = score;
                let result = merge_highscores(game.get("/highscores/scores"), final_score, wave);
                let rank = result["rank"];
                if rank >= 0 {
                    game.set("/highscores/scores", result["scores"]);
                    game.set("/highscores/last_entry_rank", rank);
                } else {
                    game.set("/highscores/last_entry_rank", -1);
                }
                game.set("/score", final_score);
                clear_round_entities(world);
                game.jump("asteroids-highscores");
            } else {
                local.respawning = true;
                world.after_ms("respawn_done", 3000);
            }
        } else {
            // Hit flash — brief invincibility, small spark burst only
            world.emit("asteroids.impact_sparks", ship_id, #{
                count: 8, speed: 60.0, spread: 6.28, fg: col.ship ?? "white"
            });
            local.invincible = true;
            invincible = true;
            world.after_ms("hit_invincible_done", 800);
        }
        break;
    }
}

#{state: local}
} // end game_over else block
