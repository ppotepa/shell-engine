if type_of(local) == "()" { local = #{}; }

fn clamp_unit(v) {
    if v < -1.0 { return -1.0; }
    if v > 1.0 { return 1.0; }
    v
}

fn clamp01(v) {
    if v < 0.0 { return 0.0; }
    if v > 1.0 { return 1.0; }
    v
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

fn rotate_axis(vx, vy, vz, ax, ay, az, angle) {
    let c = cos(angle);
    let s = sin(angle);
    let d = vx * ax + vy * ay + vz * az;
    let cx = ay * vz - az * vy;
    let cy = az * vx - ax * vz;
    let cz = ax * vy - ay * vx;
    #{
        x: vx * c + cx * s + ax * d * (1.0 - c),
        y: vy * c + cy * s + ay * d * (1.0 - c),
        z: vz * c + cz * s + az * d * (1.0 - c)
    }
}

fn angle_from_cos_sin(c, s) {
    let a = acos(clamp_unit(c));
    if s < 0.0 { return -a; }
    a
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

fn sim_radius_km(real_radius_km, divisor) {
    let d = if divisor <= 0.0001 { 1.0 } else { divisor };
    real_radius_km / d
}

fn gravity_mu_km3_s2(surface_g_mps2, radius_km) {
    let r = radius_km.max(0.001);
    (surface_g_mps2.max(0.0) / 1000.0) * r * r
}

fn normalize_vehicle_profile_name(vehicle, profile_name) {
    vehicle.normalize_profile_id(profile_name ?? vehicle.default_profile_id())
}

fn atmo_color_hex(v) { if v < 0.5 { "#d9b36a" } else if v < 1.5 { "#cf7a8c" } else if v < 2.5 { "#7cc8ff" } else if v < 3.5 { "#9bcf86" } else if v < 4.5 { "#9ab8ff" } else { "#b79dff" } }
fn atmo_haze_hex(v) { if v < 0.5 { "#f6eddc" } else if v < 1.5 { "#f6ebef" } else if v < 2.5 { "#edf7ff" } else if v < 3.5 { "#eef7ea" } else if v < 4.5 { "#eef4ff" } else { "#f2eeff" } }
fn atmo_absorption_hex(v) { if v < 0.5 { "#ffbf7b" } else if v < 1.5 { "#ffc0ad" } else if v < 2.5 { "#ffd59d" } else if v < 3.5 { "#ffd08f" } else if v < 4.5 { "#ffd9b4" } else { "#ffd0dc" } }

fn producer_vehicle_handoff_state_path(producer_mod_id) {
    `/mods/${producer_mod_id}/vehicle/handoff`
}

fn vehicle_profile_state_path() { "/mods/vehicle-playground/vehicle/profile" }
fn vehicle_assist_alt_state_path() { "/mods/vehicle-playground/vehicle/assist_alt_hold" }
fn vehicle_assist_heading_state_path() { "/mods/vehicle-playground/vehicle/assist_heading_hold" }

fn bool_to_i(v) { if v { 1 } else { 0 } }
fn i_to_bool(v) { v != 0 }
fn hud_bool_label(v) { if v { "ON" } else { "OFF" } }

fn hud_profile_label(profile_name) {
    if profile_name == "sim-lite" { "SIM-LITE" } else { "ARCADE" }
}

fn hud_status_label(surface_mode, alt_km, speed_kms, radial_speed_kms) {
    if surface_mode == "grounded" { "SURFACE" }
    else if surface_mode == "surface_locked" { "LOCKED" }
    else if alt_km < 0.05 && speed_kms < 0.05 { "SURFACE" }
    else if alt_km > 25.0 { "ORBIT" }
    else if radial_speed_kms > 0.05 { "CLIMB" }
    else if radial_speed_kms < -0.05 { "DECAY" }
    else { "CRUISE" }
}

fn hud_body_colour(body_patch_ok) {
    if body_patch_ok { "#d8f0ff" } else { "#ddd47d" }
}

fn hud_body_label(body_type) {
    let label = body_type ?? "earth_like";
    if label == "earth_like" { "EARTH-LIKE" }
    else if label == "gas_giant" { "GAS GIANT" }
    else if label == "ice" { "ICE" }
    else if label == "ocean" { "OCEAN" }
    else if label == "desert" { "DESERT" }
    else if label == "volcanic" { "VOLCANIC" }
    else if label == "barren" { "BARREN" }
    else { "CUSTOM" }
}

fn current_vehicle_control_state(local, vehicle) {
    vehicle.control_state_from(
        local["vehicle_control"] ?? #{
            profile_id: vehicle.default_profile_id()
        }
    )
}

fn current_vehicle_control_map(local, vehicle) {
    current_vehicle_control_state(local, vehicle).to_map()
}

fn current_vehicle_profile_name(local, vehicle) {
    current_vehicle_control_map(local, vehicle)["profile_id"] ?? vehicle.default_profile_id()
}

fn current_vehicle_assist_state(local, vehicle) {
    vehicle.assist_state_from(current_vehicle_control_map(local, vehicle)["assists"] ?? #{})
}

fn current_vehicle_target_altitude(local, vehicle) {
    current_vehicle_control_map(local, vehicle)["target_altitude_km"]
}

fn current_vehicle_target_heading(local, vehicle) {
    current_vehicle_control_map(local, vehicle)["target_heading_rad"]
}

fn sync_vehicle_control_state(local, control, vehicle) {
    local["vehicle_control"] = vehicle.control_state_from(control.to_map()).to_map();
    local
}

fn set_vehicle_target_altitude(local, vehicle, target_altitude_km) {
    let control = current_vehicle_control_map(local, vehicle);
    control["target_altitude_km"] = target_altitude_km;
    local["vehicle_control"] = vehicle.control_state_from(control).to_map();
    local
}

fn set_vehicle_target_heading(local, vehicle, target_heading_rad) {
    let control = current_vehicle_control_map(local, vehicle);
    control["target_heading_rad"] = target_heading_rad;
    local["vehicle_control"] = vehicle.control_state_from(control).to_map();
    local
}

fn persist_vehicle_profile(local, game, vehicle) {
    let profile = current_vehicle_profile_name(local, vehicle);
    game.set(vehicle_profile_state_path(), profile);
    local
}

fn persist_vehicle_assists(local, game, vehicle) {
    let assists = current_vehicle_control_map(local, vehicle)["assists"] ?? #{};
    let alt_hold = assists["alt_hold"] ?? false;
    let heading_hold = assists["heading_hold"] ?? false;
    game.set(vehicle_assist_alt_state_path(), bool_to_i(alt_hold));
    game.set(vehicle_assist_heading_state_path(), bool_to_i(heading_hold));
    local
}

fn load_persisted_vehicle_state(local, game, vehicle) {
    local = sync_vehicle_control_state(
        local,
        vehicle.control_state_from(#{
            profile_id: normalize_vehicle_profile_name(
                vehicle,
                game.get_s(vehicle_profile_state_path(), vehicle.default_profile_id())
            ),
            assists: #{
                alt_hold: i_to_bool(game.get_i(vehicle_assist_alt_state_path(), 0)),
                heading_hold: i_to_bool(game.get_i(vehicle_assist_heading_state_path(), 0))
            }
        }),
        vehicle
    );
    local
}

fn persist_vehicle_state(local, game, vehicle) {
    local = persist_vehicle_profile(local, game, vehicle);
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn toggle_vehicle_profile(local, game, vehicle) {
    let control = current_vehicle_control_state(local, vehicle);
    control.cycle_ship_profile();
    local = sync_vehicle_control_state(local, control, vehicle);
    local = persist_vehicle_profile(local, game, vehicle);
    local
}

fn toggle_vehicle_assist_alt_hold(local, game, vehicle, target_altitude_km) {
    let control = current_vehicle_control_state(local, vehicle);
    control.toggle_altitude_hold(target_altitude_km);
    local = sync_vehicle_control_state(local, control, vehicle);
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn toggle_vehicle_assist_heading_hold(local, game, vehicle, target_heading_rad) {
    let control = current_vehicle_control_state(local, vehicle);
    control.toggle_heading_hold(target_heading_rad);
    local = sync_vehicle_control_state(local, control, vehicle);
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn current_vehicle_buttons(vehicle, input) {
    vehicle.button_input_from(#{
        forward: input.action_down("thrust"),
        reverse: input.action_down("brake"),
        strafe_left: input.action_down("strafe_left"),
        strafe_right: input.action_down("strafe_right"),
        lift_up: input.action_down("lift"),
        yaw_left: input.action_down("turn_left"),
        yaw_right: input.action_down("turn_right"),
        boost: input.action_down("boost"),
        main_engine: input.action_down("boost")
    })
}

fn current_vehicle_frame_control(local, vehicle, input) {
    let control = vehicle
        .control_from_buttons(
            current_vehicle_profile_name(local, vehicle),
            current_vehicle_buttons(vehicle, input),
            current_vehicle_assist_state(local, vehicle)
        )
        .to_map();
    control["target_altitude_km"] = current_vehicle_target_altitude(local, vehicle);
    control["target_heading_rad"] = current_vehicle_target_heading(local, vehicle);
    vehicle.control_state_from(control)
}

fn current_ship_profile_tuning(local, vehicle) {
    vehicle
        .ship_profile_tuning(current_vehicle_profile_name(local, vehicle))
        .to_map()
}

fn build_vehicle_packet_vehicle_map(local, vehicle) {
    vehicle
        .packet_vehicle_from_control(
            current_vehicle_control_state(local, vehicle),
            local["spawn_altitude_km"] ?? 0.0
        )
        .to_map()
}

fn build_vehicle_packet_basis_map(local) {
    #{
        normal: [
            local["snx"] ?? 0.0,
            local["sny"] ?? 0.0,
            local["snz"] ?? 0.0
        ],
        forward: [
            local["sfx"] ?? 0.0,
            local["sfy"] ?? 0.0,
            local["sfz"] ?? 0.0
        ],
        right: [
            local["srx"] ?? 0.0,
            local["sry"] ?? 0.0,
            local["srz"] ?? 0.0
        ]
    }
}

fn build_vehicle_packet_telemetry_map(local, vehicle, ship_x, ship_y, ship_vx, ship_vy) {
    vehicle
        .packet_telemetry_from(#{
            x: ship_x,
            y: ship_y,
            vx: ship_vx,
            vy: ship_vy,
            heading_deg: (local["ship_heading"] ?? local["vehicle_heading"] ?? 0.0) * 57.29577951,
            altitude_km: local["vehicle_altitude_km"] ?? 0.0,
            tangent_speed_kms: local["vehicle_speed_kms"] ?? 0.0,
            radial_speed_kms: local["vehicle_vrad_kms"] ?? 0.0,
            spawn_angle_deg: local["vehicle_spawn_angle_deg"] ?? 0.0,
            camera_sway: local["cam_sway"] ?? local["vehicle_camera_sway"] ?? 0.0,
            radius_wu: local["radius"] ?? 0.0,
            vfwd_wu_s: local["vfwd"] ?? 0.0,
            vright_wu_s: local["vright"] ?? 0.0,
            vrad_wu_s: local["vrad"] ?? 0.0,
            yaw_rate_rad_s: local["yaw_rate"] ?? 0.0,
            basis: build_vehicle_packet_basis_map(local),
            grounded: local["vehicle_grounded"] ?? false
        })
        .to_map()
}

fn build_vehicle_environment_map(local, body) {
    #{
        body: body,
        real_radius_km: local["planet_radius_real_km"] ?? 6371.0,
        scale_divisor: local["planet_radius_scale_divisor"] ?? 1.0,
        surface_gravity_mps2: local["surface_gravity_mps2"] ?? 9.81,
        atmo_top_km: local["atmo_top_km"] ?? 80.0,
        atmo_dense_start_km: local["atmo_dense_start_km"] ?? 12.0,
        atmo_drag_max: local["atmo_drag_max"] ?? 2.0,
        seed: local["seed"] ?? 0.0,
        ocean: local["ocean"] ?? 0.55,
        cscale: local["cscale"] ?? 2.5,
        cwarp: local["cwarp"] ?? 0.65,
        coct: local["coct"] ?? 5.0,
        mscale: local["mscale"] ?? 6.0,
        mstr: local["mstr"] ?? 0.45,
        mroct: local["mroct"] ?? 5.0,
        moisture: local["moisture"] ?? 3.0,
        ice: local["ice"] ?? 1.0,
        lapse: local["lapse"] ?? 0.6,
        rain: local["rain"] ?? 0.4,
        disp: local["disp"] ?? 0.22,
        subdiv: local["subdiv"] ?? 32.0,
        coloring: local["coloring"] ?? 0.0,
        rotspeed: local["rotspeed"] ?? 3.0,
        sunaz: local["sunaz"] ?? 210.0,
        sunel: local["sunel"] ?? 40.0,
        ambient: local["ambient"] ?? 0.12,
        atmo_color: local["atmo_color"] ?? 2.0,
        atmo_density: local["atmo_density"] ?? 0.48,
        atmo_height: local["atmo_height"] ?? 0.11,
        atmo_forward_scatter: local["atmo_forward_scatter"] ?? 0.78,
        atmo_absorption: local["atmo_absorption"] ?? 0.12,
        atmo_rayleigh_falloff: local["atmo_rayleigh_falloff"] ?? 0.22,
        atmo_haze_amount: local["atmo_haze_amount"] ?? 0.22,
        atmo_haze_falloff: local["atmo_haze_falloff"] ?? 0.46
    }
}

fn build_vehicle_runtime_environment_map(local, body_info, body) {
    #{
        body_id: body_info["id"] ?? body["id"] ?? "generated-planet",
        body_kind: body_info["planet_type"] ?? body["planet_type"] ?? local["planet_type"] ?? "earth_like",
        body_center_x: body_info["center_x"] ?? body["center_x"] ?? 0.0,
        body_center_y: body_info["center_y"] ?? body["center_y"] ?? 0.0,
        render_radius_wu: body_info["radius_px"] ?? body["radius_px"] ?? body_info["surface_radius"] ?? body["surface_radius"] ?? 0.0,
        surface_radius_wu: body_info["surface_radius"] ?? body["surface_radius"] ?? 0.0,
        real_radius_km: body_info["radius_km"] ?? body["radius_km"] ?? local["planet_radius_real_km"] ?? 6371.0,
        scale_divisor: local["planet_radius_scale_divisor"] ?? 1.0,
        gravity_mu_km3_s2: body_info["gravity_mu_km3_s2"] ?? body["gravity_mu_km3_s2"] ?? 0.0,
        surface_gravity_mps2: local["surface_gravity_mps2"] ?? 9.81,
        atmosphere_top_km: body_info["atmosphere_top_km"] ?? body["atmosphere_top_km"] ?? local["atmo_top_km"] ?? 0.0,
        atmosphere_dense_start_km: body_info["atmosphere_dense_start_km"] ?? body["atmosphere_dense_start_km"] ?? local["atmo_dense_start_km"] ?? 0.0,
        atmosphere_drag_max: body_info["atmosphere_drag_max"] ?? body["atmosphere_drag_max"] ?? local["atmo_drag_max"] ?? 0.0,
        cloud_bottom_km: body_info["cloud_bottom_km"] ?? body["cloud_bottom_km"] ?? 0.0,
        cloud_top_km: body_info["cloud_top_km"] ?? body["cloud_top_km"] ?? 0.0
    }
}

fn basis_component(axis, index, fallback) {
    if type_of(axis) == "array" && axis.len() > index {
        axis[index]
    } else {
        fallback
    }
}

fn build_generated_body_map(
    real_radius_km,
    scale_divisor,
    surface_gravity_mps2,
    atmo_top_km,
    atmo_dense_start_km,
    atmo_drag_max,
    planet_type
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
        cloud_top_km: (atmo_top_km.max(0.0) * 0.35),
        planet_type: planet_type ?? "earth_like"
    }
}

fn sync_generated_body(local, world) {
    let real_radius_km = local["planet_radius_real_km"] ?? 6371.0;
    let scale_divisor = local["planet_radius_scale_divisor"] ?? 1.0;
    let surface_gravity_mps2 = local["surface_gravity_mps2"] ?? 9.81;
    let atmo_top_km = local["atmo_top_km"] ?? 80.0;
    let atmo_dense_start_km = local["atmo_dense_start_km"] ?? 12.0;
    let atmo_drag_max = local["atmo_drag_max"] ?? 2.0;
    let planet_type = local["planet_type"] ?? "earth_like";
    let body = build_generated_body_map(
        real_radius_km,
        scale_divisor,
        surface_gravity_mps2,
        atmo_top_km,
        atmo_dense_start_km,
        atmo_drag_max,
        planet_type
    );

    let patched = world.body_patch("generated-planet", body);
    let info = world.body_info("generated-planet");
    #{
        body: body,
        info: info,
        patched: patched
    }
}

fn ensure_ship(world, spawn_x, spawn_y, heading) {
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    world.spawn_prefab("ship", #{
        x: spawn_x,
        y: spawn_y,
        heading: heading,
        vx: 0.0,
        vy: 0.0
    })
}

fn body_km_per_wu(body_info, fallback_real_radius_km, fallback_surface_radius) {
    let radius_km = body_info["radius_km"] ?? fallback_real_radius_km;
    let surface_radius = body_info["surface_radius"] ?? fallback_surface_radius;
    let km_per_px = body_info["km_per_px"] ?? (radius_km / surface_radius.max(0.001));
    km_per_px.max(0.0001)
}

fn resolve_handoff_packet_path(game) {
    let path = producer_vehicle_handoff_state_path("planet-generator");
    if game.has(path) {
        return path;
    }
    ""
}

fn current_ship_runtime_state(local, vehicle) {
    vehicle.ship_runtime_state_from(local["vehicle_ship_runtime"] ?? vehicle.ship_runtime_state().to_map())
}

fn current_vehicle_surface_mode(local, vehicle) {
    current_ship_runtime_state(local, vehicle).to_map()["surface_mode"] ?? "detached"
}

fn build_vehicle_runtime_telemetry_map(local, ship_x, ship_y, ship_vx, ship_vy) {
    #{
        reference: #{
            heading_rad: local["ship_heading"] ?? local["vehicle_heading"] ?? 0.0
        },
        position_x: ship_x,
        position_y: ship_y,
        motion: #{
            velocity_x: ship_vx,
            velocity_y: ship_vy
        },
        altitude_km: local["vehicle_altitude_km"] ?? 0.0,
        tangent_speed_kms: local["vehicle_speed_kms"] ?? 0.0,
        radial_speed_kms: local["vehicle_vrad_kms"] ?? 0.0,
        spawn_angle_deg: local["vehicle_spawn_angle_deg"] ?? 0.0,
        camera_sway: local["cam_sway"] ?? local["vehicle_camera_sway"] ?? 0.0,
        radius_wu: local["radius"] ?? 0.0,
        forward_speed_wu_s: local["vfwd"] ?? 0.0,
        lateral_speed_wu_s: local["vright"] ?? 0.0,
        radial_speed_wu_s: local["vrad"] ?? 0.0,
        yaw_rate_rad_s: local["yaw_rate"] ?? 0.0,
        basis: build_vehicle_packet_basis_map(local),
        grounded: local["vehicle_grounded"] ?? false
    }
}

fn sync_vehicle_ship_runtime(
    local,
    vehicle,
    body_info,
    body,
    control,
    ship_x,
    ship_y,
    ship_vx,
    ship_vy,
    surface_contact,
    request_detach
) {
    let step = vehicle
        .ship_runtime_step(
            current_vehicle_profile_name(local, vehicle),
            current_ship_runtime_state(local, vehicle),
            vehicle.ship_runtime_input_from(#{
                control: control.to_map(),
                telemetry: build_vehicle_runtime_telemetry_map(local, ship_x, ship_y, ship_vx, ship_vy),
                environment: build_vehicle_runtime_environment_map(local, body_info, body),
                surface_contact: surface_contact,
                request_detach: request_detach
            })
        )
        .to_map();
    let state = step["state"] ?? #{};
    let telemetry = step["telemetry"] ?? #{};
    local["vehicle_ship_runtime"] = state;
    local["vehicle_grounded"] = telemetry["grounded"] ?? local["vehicle_grounded"] ?? false;
    local
}

fn consume_handoff_packet(local, game, vehicle) {
    let path = resolve_handoff_packet_path(game);
    if path == "" {
        local["handoff_source"] = "defaults";
        local["handoff_return_scene_id"] = "";
        local["handoff_producer_mod_id"] = "";
        return local;
    }

    let packet = game.get(path);
    if type_of(packet) != "map" {
        local["handoff_source"] = "defaults";
        return local;
    }
    let packet = vehicle.launch_packet_from(packet).to_map();

    local["handoff_source"] = "packet";
    local["handoff_packet_path"] = path;
    local["handoff_producer_mod_id"] = packet["producer_mod_id"] ?? "planet-generator";
    local["handoff_return_scene_id"] = packet["return_scene_id"] ?? "";

    let environment = packet["planet"] ?? #{};
    if type_of(environment) == "map" {
        let body = environment["body"] ?? #{};
        local["planet_radius_real_km"] = environment["real_radius_km"] ?? local["planet_radius_real_km"] ?? 6371.0;
        local["planet_radius_scale_divisor"] = environment["scale_divisor"] ?? local["planet_radius_scale_divisor"] ?? 1.0;
        local["surface_gravity_mps2"] = environment["surface_gravity_mps2"] ?? local["surface_gravity_mps2"] ?? 9.81;
        local["atmo_top_km"] = environment["atmo_top_km"] ?? local["atmo_top_km"] ?? 80.0;
        local["atmo_dense_start_km"] = environment["atmo_dense_start_km"] ?? local["atmo_dense_start_km"] ?? 12.0;
        local["atmo_drag_max"] = environment["atmo_drag_max"] ?? local["atmo_drag_max"] ?? 2.0;
        local["planet_type"] = body["planet_type"] ?? local["planet_type"] ?? "earth_like";
        local["seed"] = environment["seed"] ?? local["seed"] ?? 0.0;
        local["ocean"] = environment["ocean"] ?? local["ocean"] ?? 0.55;
        local["cscale"] = environment["cscale"] ?? local["cscale"] ?? 2.5;
        local["cwarp"] = environment["cwarp"] ?? local["cwarp"] ?? 0.65;
        local["coct"] = environment["coct"] ?? local["coct"] ?? 5.0;
        local["mscale"] = environment["mscale"] ?? local["mscale"] ?? 6.0;
        local["mstr"] = environment["mstr"] ?? local["mstr"] ?? 0.45;
        local["mroct"] = environment["mroct"] ?? local["mroct"] ?? 5.0;
        local["moisture"] = environment["moisture"] ?? local["moisture"] ?? 3.0;
        local["ice"] = environment["ice"] ?? local["ice"] ?? 1.0;
        local["lapse"] = environment["lapse"] ?? local["lapse"] ?? 0.6;
        local["rain"] = environment["rain"] ?? local["rain"] ?? 0.4;
        local["disp"] = environment["disp"] ?? local["disp"] ?? 0.22;
        local["subdiv"] = environment["subdiv"] ?? local["subdiv"] ?? 32.0;
        local["coloring"] = environment["coloring"] ?? local["coloring"] ?? 0.0;
        local["rotspeed"] = environment["rotspeed"] ?? local["rotspeed"] ?? 3.0;
        local["sunaz"] = environment["sunaz"] ?? local["sunaz"] ?? 210.0;
        local["sunel"] = environment["sunel"] ?? local["sunel"] ?? 40.0;
        local["ambient"] = environment["ambient"] ?? local["ambient"] ?? 0.12;
        local["atmo_color"] = environment["atmo_color"] ?? local["atmo_color"] ?? 2.0;
        local["atmo_density"] = environment["atmo_density"] ?? local["atmo_density"] ?? 0.48;
        local["atmo_height"] = environment["atmo_height"] ?? local["atmo_height"] ?? 0.11;
        local["atmo_forward_scatter"] = environment["atmo_forward_scatter"] ?? local["atmo_forward_scatter"] ?? 0.78;
        local["atmo_absorption"] = environment["atmo_absorption"] ?? local["atmo_absorption"] ?? 0.12;
        local["atmo_rayleigh_falloff"] = environment["atmo_rayleigh_falloff"] ?? local["atmo_rayleigh_falloff"] ?? 0.22;
        local["atmo_haze_amount"] = environment["atmo_haze_amount"] ?? local["atmo_haze_amount"] ?? 0.22;
        local["atmo_haze_falloff"] = environment["atmo_haze_falloff"] ?? local["atmo_haze_falloff"] ?? 0.46;
    }

    let packet_vehicle = packet["vehicle"] ?? #{};
    if type_of(packet_vehicle) == "map" {
        let packet_vehicle = vehicle.packet_vehicle_from(packet_vehicle);
        local = sync_vehicle_control_state(local, packet_vehicle.to_control_state(), vehicle);
        let packet_vehicle = packet_vehicle.to_map();
        local["spawn_altitude_km"] = packet_vehicle["spawn_altitude_km"] ?? local["spawn_altitude_km"] ?? 0.0;
    }

    let packet_telemetry = packet["telemetry"] ?? #{};
    if type_of(packet_telemetry) == "map" {
        let telemetry = vehicle.packet_telemetry_from(packet_telemetry).to_map();
        local["spawn_altitude_km"] = telemetry["altitude_km"] ?? local["spawn_altitude_km"] ?? 0.0;
        local["vehicle_heading"] = (telemetry["heading_deg"] ?? 0.0) * 0.01745329252;
        local["vehicle_speed_kms"] = telemetry["tangent_speed_kms"] ?? local["vehicle_speed_kms"] ?? 0.0;
        local["vehicle_vrad_kms"] = telemetry["radial_speed_kms"] ?? local["vehicle_vrad_kms"] ?? 0.0;
        local["vehicle_spawn_angle_deg"] = telemetry["spawn_angle_deg"] ?? local["vehicle_spawn_angle_deg"] ?? 0.0;
        local["vehicle_camera_sway"] = telemetry["camera_sway"] ?? local["vehicle_camera_sway"] ?? 0.0;
        local["radius"] = telemetry["radius_wu"] ?? local["radius"];
        local["vfwd"] = telemetry["vfwd_wu_s"] ?? local["vfwd"];
        local["vright"] = telemetry["vright_wu_s"] ?? local["vright"];
        local["vrad"] = telemetry["vrad_wu_s"] ?? local["vrad"];
        local["yaw_rate"] = telemetry["yaw_rate_rad_s"] ?? local["yaw_rate"];
        let basis = telemetry["basis"] ?? #{};
        if type_of(basis) == "map" {
            let normal = basis["normal"] ?? [];
            let forward = basis["forward"] ?? [];
            let right = basis["right"] ?? [];
            local["snx"] = basis_component(normal, 0, local["snx"]);
            local["sny"] = basis_component(normal, 1, local["sny"]);
            local["snz"] = basis_component(normal, 2, local["snz"]);
            local["sfx"] = basis_component(forward, 0, local["sfx"]);
            local["sfy"] = basis_component(forward, 1, local["sfy"]);
            local["sfz"] = basis_component(forward, 2, local["sfz"]);
            local["srx"] = basis_component(right, 0, local["srx"]);
            local["sry"] = basis_component(right, 1, local["sry"]);
            local["srz"] = basis_component(right, 2, local["srz"]);
            if type_of(basis["normal"]) == "array" && type_of(basis["forward"]) == "array" {
                local["vehicle_orbit_seeded"] = true;
            }
        }
        local["cam_sway"] = telemetry["camera_sway"] ?? local["cam_sway"];
        local["vehicle_grounded"] = telemetry["grounded"] ?? local["vehicle_grounded"] ?? false;
    }

    game.remove(path);
    let producer_mod_id = local["handoff_producer_mod_id"] ?? "planet-generator";
    let producer_vehicle_path = producer_vehicle_handoff_state_path(producer_mod_id);
    if path != producer_vehicle_path && game.has(producer_vehicle_path) {
        game.remove(producer_vehicle_path);
    }
    local
}

fn resolve_return_scene_id(local) {
    local["handoff_return_scene_id"] ?? ""
}

fn resolve_return_mod_ref(local) {
    local["handoff_producer_mod_id"] ?? ""
}

fn hud_handoff_label(local) {
    if (local.handoff_source ?? "defaults") == "packet" { "PACKET" } else { "DEFAULTS" }
}

fn hud_escape_label(local) {
    if resolve_return_scene_id(local) != "" && resolve_return_mod_ref(local) != "" && (local.handoff_source ?? "defaults") == "packet" {
        "RETURN"
    } else {
        "RESET"
    }
}

fn save_return_vehicle_handoff_packet(local, game, vehicle, body, ship_x, ship_y, ship_vx, ship_vy) {
    let producer_mod_id = local["handoff_producer_mod_id"] ?? "planet-generator";
    let return_scene_id = resolve_return_scene_id(local);
    let packet_vehicle = build_vehicle_packet_vehicle_map(local, vehicle);
    let telemetry = build_vehicle_packet_telemetry_map(local, vehicle, ship_x, ship_y, ship_vx, ship_vy);
    let packet = vehicle.return_packet_from(#{
        producer_mod_id: "vehicle-playground",
        source_scene_id: "vehicle-playground-vehicle",
        target_mod_ref: producer_mod_id,
        target_scene_id: return_scene_id,
        return_scene_id: return_scene_id,
        consumer_hint: "generator-return",
        environment: build_vehicle_environment_map(local, body),
        vehicle: packet_vehicle,
        telemetry: telemetry
    }).to_map();
    game.set(producer_vehicle_handoff_state_path(producer_mod_id), packet);
    local
}

fn apply_planet_state_to_scene(local, scene) {
    let coloring = local["coloring"] ?? 0.0;
    let coloring_str = if coloring < 0.5 { "biome" } else if coloring < 1.5 { "elevation" } else { "moisture" };
    scene.set("vehicle-planet-view", "obj.source", "world://32");
    scene.set("vehicle-planet-view", "world.seed", local["seed"] ?? 0.0);
    scene.set("vehicle-planet-view", "world.ocean_fraction", local["ocean"] ?? 0.55);
    scene.set("vehicle-planet-view", "world.continent_scale", local["cscale"] ?? 2.5);
    scene.set("vehicle-planet-view", "world.continent_warp", local["cwarp"] ?? 0.65);
    scene.set("vehicle-planet-view", "world.continent_octaves", local["coct"] ?? 5.0);
    scene.set("vehicle-planet-view", "world.mountain_scale", local["mscale"] ?? 6.0);
    scene.set("vehicle-planet-view", "world.mountain_strength", local["mstr"] ?? 0.45);
    scene.set("vehicle-planet-view", "world.mountain_ridge_octaves", local["mroct"] ?? 5.0);
    scene.set("vehicle-planet-view", "world.moisture_scale", local["moisture"] ?? 3.0);
    scene.set("vehicle-planet-view", "world.ice_cap_strength", local["ice"] ?? 1.0);
    scene.set("vehicle-planet-view", "world.lapse_rate", local["lapse"] ?? 0.6);
    scene.set("vehicle-planet-view", "world.rain_shadow", local["rain"] ?? 0.4);
    scene.set("vehicle-planet-view", "world.displacement_scale", local["disp"] ?? 0.22);
    scene.set("vehicle-planet-view", "world.subdivisions", local["subdiv"] ?? 32.0);
    scene.set("vehicle-planet-view", "world.shape", "sphere");
    scene.set("vehicle-planet-view", "world.base", "cube");
    scene.set("vehicle-planet-view", "world.coloring", coloring_str);
    scene.set("vehicle-planet-view", "obj.surface_mode", "material");
    scene.set("vehicle-planet-view", "obj.rotation-speed", local["rotspeed"] ?? 3.0);
    scene.set("vehicle-planet-view", "obj.ambient", local["ambient"] ?? 0.12);

    let atmo_height = local["atmo_height"] ?? 0.11;
    let atmo_density = local["atmo_density"] ?? 0.48;
    let atmo_forward_scatter = local["atmo_forward_scatter"] ?? 0.78;
    let atmo_absorption = local["atmo_absorption"] ?? 0.12;
    let atmo_rayleigh_falloff = local["atmo_rayleigh_falloff"] ?? 0.22;
    let atmo_haze_amount = local["atmo_haze_amount"] ?? 0.22;
    let atmo_haze_falloff = local["atmo_haze_falloff"] ?? 0.46;
    let atmo_color = local["atmo_color"] ?? 2.0;

    let atmo_height_rel = (0.02 + atmo_height * 0.88).max(0.02).min(0.90);
    let atmo_density_profile = atmo_density.max(0.0).min(1.0);
    let atmo_forward = atmo_forward_scatter.max(0.0).min(1.0);
    let atmo_limb = (0.95 + atmo_density * 1.7 + atmo_haze_amount * 1.15 + atmo_height * 0.55).max(0.2).min(4.0);
    let atmo_term_soft = (0.75 + atmo_haze_amount * 1.55 + atmo_density * 0.35).max(0.05).min(4.0);
    let atmo_absorb = atmo_absorption.max(0.0).min(1.0);
    let atmo_ray_amount = (0.08 + atmo_density * 0.72).max(0.0).min(1.0);
    let atmo_absorb_height = (0.70 + atmo_height * 0.10).max(0.50).min(0.92);
    let atmo_absorb_width = (0.10 + atmo_haze_amount * 0.18 + atmo_absorb * 0.10).max(0.06).min(0.32);
    scene.set("vehicle-planet-view", "obj.atmo.height", atmo_height_rel);
    scene.set("vehicle-planet-view", "obj.atmo.density", atmo_density_profile);
    scene.set("vehicle-planet-view", "obj.atmo.rayleigh_amount", atmo_ray_amount);
    scene.set("vehicle-planet-view", "obj.atmo.rayleigh_color", atmo_color_hex(atmo_color));
    scene.set("vehicle-planet-view", "obj.atmo.rayleigh_falloff", atmo_rayleigh_falloff);
    scene.set("vehicle-planet-view", "obj.atmo.haze_amount", atmo_haze_amount);
    scene.set("vehicle-planet-view", "obj.atmo.haze_color", atmo_haze_hex(atmo_color));
    scene.set("vehicle-planet-view", "obj.atmo.haze_falloff", atmo_haze_falloff);
    scene.set("vehicle-planet-view", "obj.atmo.absorption_amount", atmo_absorb);
    scene.set("vehicle-planet-view", "obj.atmo.absorption_color", atmo_absorption_hex(atmo_color));
    scene.set("vehicle-planet-view", "obj.atmo.absorption_height", atmo_absorb_height);
    scene.set("vehicle-planet-view", "obj.atmo.absorption_width", atmo_absorb_width);
    scene.set("vehicle-planet-view", "obj.atmo.forward_scatter", atmo_forward);
    scene.set("vehicle-planet-view", "obj.atmo.limb_boost", atmo_limb);
    scene.set("vehicle-planet-view", "obj.atmo.terminator_softness", atmo_term_soft);
    scene.set("vehicle-planet-view", "obj.atmo.night_glow", 0.0);

    let az_rad = (local["sunaz"] ?? 210.0) * 0.01745329;
    let el_rad = (local["sunel"] ?? 40.0) * 0.01745329;
    scene.set("vehicle-planet-view", "obj.light.x", -(az_rad.cos() * el_rad.cos()));
    scene.set("vehicle-planet-view", "obj.light.y", el_rad.sin());
    scene.set("vehicle-planet-view", "obj.light.z", -(az_rad.sin() * el_rad.cos()));
}

fn spawn_state(local, surface_radius, km_per_wu, cx, cy) {
    let spawn_angle_deg = local["vehicle_spawn_angle_deg"] ?? 0.0;
    let spawn_angle = spawn_angle_deg * 0.01745329252;
    let spawn_altitude_km = local["spawn_altitude_km"] ?? 1.5;
    let spawn_radius_wu = surface_radius + (spawn_altitude_km / km_per_wu.max(0.0001));
    let tangent_x = -spawn_angle.sin();
    let tangent_y = spawn_angle.cos();
    let radial_x = spawn_angle.cos();
    let radial_y = spawn_angle.sin();
    #{
        x: cx + spawn_radius_wu * radial_x,
        y: cy + spawn_radius_wu * radial_y,
        tangent_x: tangent_x,
        tangent_y: tangent_y,
        radial_x: radial_x,
        radial_y: radial_y,
        angle_deg: spawn_angle_deg,
        altitude_km: spawn_altitude_km
    }
}

fn seed_vehicle_orbit_state(local, surface_radius, km_per_wu) {
    let pi = 3.1415926535;
    let spawn_angle_deg = local["vehicle_spawn_angle_deg"] ?? 0.0;
    let spawn_angle = spawn_angle_deg * 0.01745329252;
    let grounded = local["vehicle_grounded"] ?? true;
    let spawn_altitude_km = if grounded { 0.0 } else { local["spawn_altitude_km"] ?? 1.5 };
    let radius = surface_radius + (spawn_altitude_km / km_per_wu.max(0.0001));
    let heading = local["vehicle_heading"] ?? 0.0;

    let snx = spawn_angle.cos();
    let sny = spawn_angle.sin();
    let snz = 0.0;
    let sfx = -spawn_angle.sin();
    let sfy = spawn_angle.cos();
    let sfz = 0.0;
    let srx = 0.0;
    let sry = 0.0;
    let srz = 1.0;

    let rotated_forward = rotate_axis(sfx, sfy, sfz, snx, sny, snz, heading);
    let rotated_right = rotate_axis(srx, sry, srz, snx, sny, snz, heading);

    local["snx"] = snx;
    local["sny"] = sny;
    local["snz"] = snz;
    local["sfx"] = rotated_forward["x"] ?? sfx;
    local["sfy"] = rotated_forward["y"] ?? sfy;
    local["sfz"] = rotated_forward["z"] ?? sfz;
    local["srx"] = rotated_right["x"] ?? srx;
    local["sry"] = rotated_right["y"] ?? sry;
    local["srz"] = rotated_right["z"] ?? srz;
    local["radius"] = local["radius"] ?? radius;
    local["vfwd"] = local["vfwd"] ?? ((local["vehicle_speed_kms"] ?? 0.0) / km_per_wu.max(0.0001));
    local["vright"] = local["vright"] ?? 0.0;
    local["vrad"] = local["vrad"] ?? ((local["vehicle_vrad_kms"] ?? 0.0) / km_per_wu.max(0.0001));
    local["yaw_rate"] = local["yaw_rate"] ?? 0.0;
    local["ship_heading"] = local["ship_heading"] ?? heading;
    local["ship_vis_pitch"] = local["ship_vis_pitch"] ?? pi;
    local["ship_vis_yaw"] = local["ship_vis_yaw"] ?? (-pi * 0.5);
    local["ship_vis_roll"] = local["ship_vis_roll"] ?? 0.0;
    local["cam_nx"] = local["cam_nx"] ?? snx;
    local["cam_ny"] = local["cam_ny"] ?? sny;
    local["cam_nz"] = local["cam_nz"] ?? snz;
    local["cam_fx"] = local["cam_fx"] ?? (rotated_forward["x"] ?? sfx);
    local["cam_fy"] = local["cam_fy"] ?? (rotated_forward["y"] ?? sfy);
    local["cam_fz"] = local["cam_fz"] ?? (rotated_forward["z"] ?? sfz);
    local["cam_sway"] = local["cam_sway"] ?? local["vehicle_camera_sway"] ?? 0.0;
    local["vehicle_grounded"] = local["vehicle_grounded"] ?? true;
    local["vehicle_orbit_seeded"] = true;
    local
}

if !(local.initialized ?? false) {
    local.initialized = true;
    local.mode = "vehicle";
    local.vehicle_orbit_seeded = false;
    local.vehicle_grounded = true;
    local.vehicle_control = vehicle.control_state().to_map();
    local.vehicle_ship_runtime = vehicle.ship_runtime_state().to_map();
    local.handoff_source = "defaults";
    local.handoff_packet_path = "";
    local.handoff_producer_mod_id = "";
    local.handoff_return_scene_id = "";
    local = load_persisted_vehicle_state(local, game, vehicle);
    local.vehicle_heading = 0.0;
    local = set_vehicle_target_heading(local, vehicle, 0.0);
    local = set_vehicle_target_altitude(local, vehicle, 0.0);
    local.vehicle_altitude_km = 0.0;
    local.vehicle_speed_kms = 0.0;
    local.vehicle_vrad_kms = 0.0;
    local.planet_radius_real_km = 6371.0;
    local.planet_radius_scale_divisor = 1.0;
    local.surface_gravity_mps2 = 9.81;
    local.spawn_altitude_km = 0.0;
    local.atmo_top_km = 80.0;
    local.atmo_dense_start_km = 12.0;
    local.atmo_drag_max = 2.0;
    local.planet_type = "earth_like";
    local.sun_az_deg = 210.0;
    local.sun_el_deg = 40.0;
    local.vehicle_spawn_angle_deg = 0.0;
    local.ship_id = 0;
    local.vehicle_camera_sway = 0.0;
    local.seed = 1847.0;
    local.ocean = 0.55;
    local.cscale = 2.5;
    local.cwarp = 0.65;
    local.coct = 5.0;
    local.mscale = 6.0;
    local.mstr = 0.45;
    local.mroct = 5.0;
    local.moisture = 3.0;
    local.ice = 1.0;
    local.lapse = 0.6;
    local.rain = 0.4;
    local.disp = 0.22;
    local.subdiv = 32.0;
    local.coloring = 0.0;
    local.rotspeed = 3.0;
    local.sunaz = 210.0;
    local.sunel = 40.0;
    local.ambient = 0.12;
    local.atmo_color = 2.0;
    local.atmo_density = 0.48;
    local.atmo_height = 0.11;
    local.atmo_forward_scatter = 0.78;
    local.atmo_absorption = 0.12;
    local.atmo_rayleigh_falloff = 0.22;
    local.atmo_haze_amount = 0.22;
    local.atmo_haze_falloff = 0.46;
    scene.set_bg("#000010");
    input.load_profile("default");
    local = consume_handoff_packet(local, game, vehicle);
    local = persist_vehicle_state(local, game, vehicle);
    apply_planet_state_to_scene(local, scene);
}

let dt = (frame_ms.to_float() / 1000.0).max(0.0001);
let ship_id = if (local.ship_id ?? 0) > 0 { local.ship_id ?? 0 } else { vehicle.active() };
if ship_id > 0 {
    local.ship_id = ship_id;
}
let body_sync = sync_generated_body(local, world);
let body = body_sync["body"] ?? #{};
let body_info = body_sync["info"] ?? body;
let body_patch_ok = body_sync["patched"] ?? false;

let cx = body_info["center_x"] ?? 0.0;
let cy = body_info["center_y"] ?? 0.0;
let surface_radius = (body_info["surface_radius"] ?? body["surface_radius"] ?? 1.0).max(0.1);
let real_radius_km = (body_info["radius_km"] ?? body["radius_km"] ?? local.planet_radius_real_km ?? 6371.0).max(0.1);
let km_per_wu = body_km_per_wu(body_info, real_radius_km, surface_radius);
let gravity_mu = (body_info["gravity_mu"] ?? 0.0).max(0.0);
let surface_gravity_wu = if gravity_mu > 0.0 {
    gravity_mu / (surface_radius.max(0.1) * surface_radius.max(0.1))
} else {
    0.01
};
let surface_circular_speed = if gravity_mu > 0.0 {
    sqrtf(gravity_mu / surface_radius.max(0.1))
} else {
    1.0
};
let ship_tuning = current_ship_profile_tuning(local, vehicle);
let acc_fwd = surface_gravity_wu * (ship_tuning["forward_accel_g"] ?? 1.10);
let acc_side = surface_gravity_wu * (ship_tuning["side_accel_g"] ?? 0.72);
let acc_lift = surface_gravity_wu * (ship_tuning["lift_accel_g"] ?? 2.90);
let acc_main = surface_gravity_wu * (ship_tuning["main_engine_g"] ?? 1.95);
let max_spd = (surface_circular_speed * (ship_tuning["max_speed_ratio"] ?? 1.85)).max(0.5);
let max_vrad = (surface_circular_speed * (ship_tuning["max_vrad_ratio"] ?? 1.05)).max(0.25);
let lin_damp = ship_tuning["linear_damp"] ?? 0.06;
let side_trim = ship_tuning["side_trim"] ?? 0.28;
let side_thrust_trim = ship_tuning["side_thrust_trim"] ?? 1.10;
let yaw_response = ship_tuning["yaw_response"] ?? 8.4;
let yaw_damp = ship_tuning["yaw_damp"] ?? 3.2;
let yaw_max = ship_tuning["yaw_max"] ?? 1.35;
let cam_sway_tau = ship_tuning["camera_sway_tau"] ?? 0.42;
let cam_sway_gain = ship_tuning["camera_sway_gain"] ?? 0.24;
let surface_contact_altitude_km = ship_tuning["surface_contact_altitude_threshold_km"] ?? 0.05;
let takeoff_lift_threshold = ship_tuning["takeoff_lift_threshold"] ?? 0.001;
let atmo_top_alt_wu =
    if type_of(body_info["atmosphere_top_km"]) != "()" {
        (body_info["atmosphere_top_km"] / km_per_wu.max(0.0001)).max(0.0)
    } else {
        0.0
    };
let atmo_drag_max = body_info["atmosphere_drag_max"] ?? body["atmosphere_drag_max"] ?? local["atmo_drag_max"] ?? 0.0;
let surface_clearance_wu =
    ((ship_tuning["surface_clearance_km"] ?? 0.35) / km_per_wu.max(0.0001))
        .max(ship_tuning["surface_clearance_min_wu"] ?? 0.05);
let surface_spin_omega = (local["rotspeed"] ?? 0.0) * 0.01745329252;

if !(local["vehicle_orbit_seeded"] ?? false) {
    local = seed_vehicle_orbit_state(local, surface_radius, km_per_wu);
}

let snx = local["snx"] ?? 1.0;
let sny = local["sny"] ?? 0.0;
let snz = local["snz"] ?? 0.0;
let sfx = local["sfx"] ?? 0.0;
let sfy = local["sfy"] ?? 1.0;
let sfz = local["sfz"] ?? 0.0;
let srx = local["srx"] ?? 0.0;
let sry = local["sry"] ?? 0.0;
let srz = local["srz"] ?? 1.0;
let radius = (local["radius"] ?? (surface_radius + surface_clearance_wu)).max(surface_radius + surface_clearance_wu);
let vrad = local["vrad"] ?? 0.0;
let vfwd = local["vfwd"] ?? 0.0;
let vright = local["vright"] ?? 0.0;
let yaw_rate = local["yaw_rate"] ?? 0.0;
let ship_heading = local["ship_heading"] ?? (local["vehicle_heading"] ?? 0.0);
let ship_vis_pitch = local["ship_vis_pitch"] ?? 3.1415926535;
let ship_vis_yaw = local["ship_vis_yaw"] ?? -1.57079632675;
let ship_vis_roll = local["ship_vis_roll"] ?? 0.0;
let cam_nx = local["cam_nx"] ?? snx;
let cam_ny = local["cam_ny"] ?? sny;
let cam_nz = local["cam_nz"] ?? snz;
let cam_fx = local["cam_fx"] ?? sfx;
let cam_fy = local["cam_fy"] ?? sfy;
let cam_fz = local["cam_fz"] ?? sfz;
let cam_sway = local["cam_sway"] ?? local["vehicle_camera_sway"] ?? 0.0;

if ship_id <= 0 {
    let ship_spawn_x = cx + snx * radius;
    let ship_spawn_y = cy + sny * radius;
    ship_id = ensure_ship(world, ship_spawn_x, ship_spawn_y, ship_heading);
    local.ship_id = ship_id;
    if ship_id > 0 {
        vehicle.set_active(ship_id);
        world.set_transform(ship_id, ship_spawn_x, ship_spawn_y, ship_heading);
        world.set_physics(ship_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
} else if vehicle.active() <= 0 {
    vehicle.set_active(ship_id);
}

let ship_x = cx + snx * radius;
let ship_y = cy + sny * radius;
let thrust_axis = 0.0;
let strafe_axis = 0.0;
let yaw_axis = 0.0;
let main_engine_on = false;
let lift_on = false;

if ship_id > 0 {
    let xf = world.transform(ship_id) ?? #{};
    ship_x = xf["x"] ?? ship_x;
    ship_y = xf["y"] ?? ship_y;

    let frame_control = current_vehicle_frame_control(local, vehicle, input);
    let frame_control_map = frame_control.to_map();
    let frame_control_assists = frame_control_map["assists"] ?? #{};
    let vehicle_input = frame_control.intent().to_map();
    thrust_axis = vehicle_input["throttle"] ?? 0.0;
    strafe_axis = vehicle_input["strafe"] ?? 0.0;
    yaw_axis = vehicle_input["yaw"] ?? 0.0;
    main_engine_on = vehicle_input["main_engine"] ?? false;
    lift_on = (vehicle_input["lift"] ?? 0.0) > takeoff_lift_threshold;

    let altitude_wu = (radius - surface_radius).max(0.0);
    let altitude_km = altitude_wu * km_per_wu;
    let grounded = local["vehicle_grounded"] ?? false;

    let yaw_target =
        if absf(yaw_axis) > 0.001 {
            local = set_vehicle_target_heading(local, vehicle, ship_heading);
            yaw_axis * yaw_max
        } else if frame_control_assists["heading_hold"] ?? false {
            let target_heading = nearest_angle(frame_control_map["target_heading_rad"] ?? ship_heading, ship_heading);
            let heading_error = target_heading - ship_heading;
            clamp_unit(heading_error * (ship_tuning["heading_hold_kp"] ?? 2.2)) * yaw_max
        } else {
            0.0
        };
    let yaw_blend = (yaw_response * dt).min(1.0);
    yaw_rate += (yaw_target - yaw_rate) * yaw_blend;
    if absf(yaw_axis) <= 0.001 && !(frame_control_assists["heading_hold"] ?? false) {
        yaw_rate *= 1.0 / (1.0 + yaw_damp * dt);
    }
    if absf(yaw_rate) > yaw_max {
        yaw_rate = if yaw_rate > 0.0 { yaw_max } else { -yaw_max };
    }

    let dyaw = yaw_rate * dt;
    if absf(dyaw) > 0.00001 {
        let rotated_forward = rotate_axis(sfx, sfy, sfz, snx, sny, snz, dyaw);
        sfx = rotated_forward["x"] ?? sfx;
        sfy = rotated_forward["y"] ?? sfy;
        sfz = rotated_forward["z"] ?? sfz;
        let rotated_right = rotate_axis(srx, sry, srz, snx, sny, snz, dyaw);
        srx = rotated_right["x"] ?? srx;
        sry = rotated_right["y"] ?? sry;
        srz = rotated_right["z"] ?? srz;
    }

    if grounded && lift_on {
        local["vehicle_grounded"] = false;
    }

    if !(local["vehicle_grounded"] ?? false) {
        vfwd += thrust_axis * acc_fwd * dt;
        vright += strafe_axis * acc_side * dt;
        if main_engine_on { vfwd += acc_main * dt; }

        let trim = side_trim + thrust_axis.max(0.0) * side_thrust_trim;
        vright *= 1.0 / (1.0 + trim * dt);
        vfwd *= 1.0 / (1.0 + lin_damp * dt);
        vright *= 1.0 / (1.0 + lin_damp * dt);
    } else {
        vfwd = 0.0;
        vright = 0.0;
        vrad = 0.0;
        radius = surface_radius + surface_clearance_wu;
        local = set_vehicle_target_altitude(local, vehicle, 0.0);
    }

    if atmo_top_alt_wu > 0.0 {
        let atmo_alpha = clamp01((atmo_top_alt_wu - altitude_wu) / atmo_top_alt_wu.max(0.1));
        let atmo_drag = atmo_alpha * atmo_alpha * atmo_drag_max.max(0.0);
        vrad *= 1.0 / (1.0 + atmo_drag * dt * 1.2);
        vfwd *= 1.0 / (1.0 + atmo_drag * dt);
        vright *= 1.0 / (1.0 + atmo_drag * dt);
    }

    if vfwd > max_spd { vfwd = max_spd; }
    if vfwd < -max_spd * 0.5 { vfwd = -max_spd * 0.5; }
    if vright > max_spd { vright = max_spd; }
    if vright < -max_spd { vright = -max_spd; }

    let ship_world_x = cx + snx * radius;
    let ship_world_y = cy + sny * radius;
    let vt2 = vfwd * vfwd + vright * vright;
    if !(local["vehicle_grounded"] ?? false) {
        let ship_gravity = world.body_gravity("generated-planet", ship_world_x, ship_world_y, snz * radius);
        let grav_radial = (ship_gravity["ax"] ?? 0.0) * snx
            + (ship_gravity["ay"] ?? 0.0) * sny
            + (ship_gravity["az"] ?? 0.0) * snz;
        let centrifugal = vt2 / radius.max(surface_radius + surface_clearance_wu);

        let assist_radial_acc =
            if (frame_control_assists["alt_hold"] ?? false) && !lift_on {
                let target_altitude_km = frame_control_map["target_altitude_km"] ?? altitude_km;
                let altitude_error_km = target_altitude_km - altitude_km;
                (altitude_error_km * (ship_tuning["alt_hold_kp"] ?? 0.018)
                    - (vrad * km_per_wu) * (ship_tuning["alt_hold_kd"] ?? 0.095))
                    .max(-acc_lift * 0.75)
                    .min(acc_lift * 0.75)
            } else {
                0.0
            };
        let lift_acc = if lift_on { acc_lift } else { 0.0 };

        vrad += (centrifugal + grav_radial + lift_acc + assist_radial_acc) * dt;
        if vrad > max_vrad { vrad = max_vrad; }
        if vrad < -max_vrad { vrad = -max_vrad; }
        radius += vrad * dt;
        if radius < surface_radius + surface_clearance_wu {
            radius = surface_radius + surface_clearance_wu;
            if vrad < 0.0 { vrad = 0.0; }
            if !lift_on {
                local["vehicle_grounded"] = true;
                vfwd = 0.0;
                vright = 0.0;
                local = set_vehicle_target_altitude(local, vehicle, 0.0);
            }
        }
    } else {
        vrad = 0.0;
        radius = surface_radius + surface_clearance_wu;
    }

    let spin_tang_x_raw = -sny;
    let spin_tang_y_raw = snx;
    let spin_tang_z_raw = 0.0;
    let spin_tang_len = sqrtf(
        spin_tang_x_raw * spin_tang_x_raw
        + spin_tang_y_raw * spin_tang_y_raw
        + spin_tang_z_raw * spin_tang_z_raw
    );
    let spin_tang_x = if spin_tang_len > 0.0001 { spin_tang_x_raw / spin_tang_len } else { sfx };
    let spin_tang_y = if spin_tang_len > 0.0001 { spin_tang_y_raw / spin_tang_len } else { sfy };
    let spin_tang_z = if spin_tang_len > 0.0001 { spin_tang_z_raw / spin_tang_len } else { sfz };
    let carrier_speed = surface_spin_omega * radius * spin_tang_len;
    let carrier_x = spin_tang_x * carrier_speed;
    let carrier_y = spin_tang_y * carrier_speed;
    let carrier_z = spin_tang_z * carrier_speed;
    let total_tang_x = carrier_x + sfx * vfwd + srx * vright;
    let total_tang_y = carrier_y + sfy * vfwd + sry * vright;
    let total_tang_z = carrier_z + sfz * vfwd + srz * vright;
    let tangential_speed = sqrtf(total_tang_x * total_tang_x + total_tang_y * total_tang_y + total_tang_z * total_tang_z);
    if tangential_speed > 0.001 {
        let delta = tangential_speed * dt / radius.max(surface_radius + surface_clearance_wu);
        let tang_x = total_tang_x / tangential_speed;
        let tang_y = total_tang_y / tangential_speed;
        let tang_z = total_tang_z / tangential_speed;

        let rotated_normal = rotate_axis(snx, sny, snz, tang_x, tang_y, tang_z, delta);
        snx = rotated_normal["x"] ?? snx;
        sny = rotated_normal["y"] ?? sny;
        snz = rotated_normal["z"] ?? snz;
        let rotated_forward = rotate_axis(sfx, sfy, sfz, tang_x, tang_y, tang_z, delta);
        sfx = rotated_forward["x"] ?? sfx;
        sfy = rotated_forward["y"] ?? sfy;
        sfz = rotated_forward["z"] ?? sfz;
        let rotated_right = rotate_axis(srx, sry, srz, tang_x, tang_y, tang_z, delta);
        srx = rotated_right["x"] ?? srx;
        sry = rotated_right["y"] ?? sry;
        srz = rotated_right["z"] ?? srz;
    }

    let normal_len = sqrtf(snx * snx + sny * sny + snz * snz).max(0.0001);
    snx /= normal_len;
    sny /= normal_len;
    snz /= normal_len;
    let forward_dot = sfx * snx + sfy * sny + sfz * snz;
    sfx -= snx * forward_dot;
    sfy -= sny * forward_dot;
    sfz -= snz * forward_dot;
    let forward_len = sqrtf(sfx * sfx + sfy * sfy + sfz * sfz).max(0.0001);
    sfx /= forward_len;
    sfy /= forward_len;
    sfz /= forward_len;
    srx = sfy * snz - sfz * sny;
    sry = sfz * snx - sfx * snz;
    srz = sfx * sny - sfy * snx;

    ship_heading = heading_from_forward(sfx, sfy, ship_heading);
    let ship_euler = ship_basis_euler(
        srx, sry, srz,
        sfx, sfy, sfz,
        snx, sny, snz,
        ship_vis_pitch, ship_vis_yaw, ship_vis_roll
    );
    ship_vis_pitch = ship_euler["pitch"] ?? ship_vis_pitch;
    ship_vis_yaw = ship_euler["yaw"] ?? ship_vis_yaw;
    ship_vis_roll = ship_euler["roll"] ?? ship_vis_roll;

    let ship_scene_r = radius / surface_radius.max(0.0001);
    scene.set("vehicle-slot", "visible", true);
    scene.set("vehicle-slot", "obj.pitch", ship_vis_pitch * 57.29577951);
    scene.set("vehicle-slot", "obj.yaw", ship_vis_yaw * 57.29577951);
    scene.set("vehicle-slot", "obj.roll", ship_vis_roll * 57.29577951);
    scene.set("vehicle-slot", "obj.world.x", snx * ship_scene_r);
    scene.set("vehicle-slot", "obj.world.y", sny * ship_scene_r);
    scene.set("vehicle-slot", "obj.world.z", snz * ship_scene_r);
    scene.set(`ship-${ship_id}`, "visible", false);
    world.entity(ship_id).set_heading(ship_heading);

    local["snx"] = snx;
    local["sny"] = sny;
    local["snz"] = snz;
    local["sfx"] = sfx;
    local["sfy"] = sfy;
    local["sfz"] = sfz;
    local["srx"] = srx;
    local["sry"] = sry;
    local["srz"] = srz;
    local["radius"] = radius;
    local["vrad"] = vrad;
    local["vfwd"] = vfwd;
    local["vright"] = vright;
    local["yaw_rate"] = yaw_rate;
    local["ship_heading"] = ship_heading;
    local["vehicle_heading"] = ship_heading;
    local["ship_vis_pitch"] = ship_vis_pitch;
    local["ship_vis_yaw"] = ship_vis_yaw;
    local["ship_vis_roll"] = ship_vis_roll;

    let next_ship_x = cx + snx * radius;
    let next_ship_y = cy + sny * radius;
    world.set_physics(ship_id, (next_ship_x - ship_x) / dt, (next_ship_y - ship_y) / dt, 0.0, 0.0, 0.0, 0.0);
    ship_x = next_ship_x;
    ship_y = next_ship_y;

    let altitude_px_frac = clamp01((radius - surface_radius) / surface_radius.max(0.1));
    let cam_lift = 0.005 + altitude_px_frac * 0.04;
    let cam_back = 0.04 + altitude_px_frac * 0.74;
    let cam_ahead = 0.03 + altitude_px_frac * 0.55;

    let alpha_n = dt / (0.30 + dt);
    cam_nx += (snx - cam_nx) * alpha_n;
    cam_ny += (sny - cam_ny) * alpha_n;
    cam_nz += (snz - cam_nz) * alpha_n;
    let cam_n_len = sqrtf(cam_nx * cam_nx + cam_ny * cam_ny + cam_nz * cam_nz).max(0.0001);
    cam_nx /= cam_n_len;
    cam_ny /= cam_n_len;
    cam_nz /= cam_n_len;

    let alpha_f = dt / (cam_sway_tau + dt);
    cam_fx += (sfx - cam_fx) * alpha_f;
    cam_fy += (sfy - cam_fy) * alpha_f;
    cam_fz += (sfz - cam_fz) * alpha_f;
    let cam_forward_dot = cam_fx * cam_nx + cam_fy * cam_ny + cam_fz * cam_nz;
    cam_fx -= cam_nx * cam_forward_dot;
    cam_fy -= cam_ny * cam_forward_dot;
    cam_fz -= cam_nz * cam_forward_dot;
    let cam_f_len = sqrtf(cam_fx * cam_fx + cam_fy * cam_fy + cam_fz * cam_fz).max(0.0001);
    cam_fx /= cam_f_len;
    cam_fy /= cam_f_len;
    cam_fz /= cam_f_len;

    let eye_x = cam_nx * (ship_scene_r + cam_lift) - cam_fx * cam_back;
    let eye_y = cam_ny * (ship_scene_r + cam_lift) - cam_fy * cam_back;
    let eye_z = cam_nz * (ship_scene_r + cam_lift) - cam_fz * cam_back;
    let look_x = cam_nx * ship_scene_r + cam_fx * cam_ahead;
    let look_y = cam_ny * ship_scene_r + cam_fy * cam_ahead;
    let look_z = cam_nz * ship_scene_r + cam_fz * cam_ahead;

    let sway_target = (-yaw_rate * cam_sway_gain).max(-0.20).min(0.20);
    let alpha_s = dt / (cam_sway_tau + dt);
    cam_sway += (sway_target - cam_sway) * alpha_s;
    let sway_up = rotate_axis(cam_nx, cam_ny, cam_nz, cam_fx, cam_fy, cam_fz, cam_sway);

    world.set_camera_3d_look_at(eye_x, eye_y, eye_z, look_x, look_y, look_z);
    world.set_camera_3d_up(sway_up["x"] ?? cam_nx, sway_up["y"] ?? cam_ny, sway_up["z"] ?? cam_nz);

    local["cam_nx"] = cam_nx;
    local["cam_ny"] = cam_ny;
    local["cam_nz"] = cam_nz;
    local["cam_fx"] = cam_fx;
    local["cam_fy"] = cam_fy;
    local["cam_fz"] = cam_fz;
    local["cam_sway"] = cam_sway;
    local["vehicle_camera_sway"] = cam_sway;
    local["vehicle_altitude_km"] = altitude_km;
    local["vehicle_speed_kms"] = sqrtf(vfwd * vfwd + vright * vright) * km_per_wu;
    local["vehicle_vrad_kms"] = vrad * km_per_wu;

    let orbit_xy_len = sqrtf(snx * snx + sny * sny).max(0.0001);
    let spawn_angle_rad = angle_from_cos_sin(clamp_unit(snx / orbit_xy_len), sny / orbit_xy_len);
    local["vehicle_spawn_angle_deg"] = nearest_angle(spawn_angle_rad, 0.0) * 57.29577951;

    if input.just_pressed("Escape") && !(resolve_return_scene_id(local) != "" && (local.handoff_source ?? "defaults") == "packet") {
        local["radius"] = ();
        local["vfwd"] = ();
        local["vright"] = ();
        local["vrad"] = ();
        local["yaw_rate"] = ();
        local["snx"] = ();
        local["sny"] = ();
        local["snz"] = ();
        local["sfx"] = ();
        local["sfy"] = ();
        local["sfz"] = ();
        local["srx"] = ();
        local["sry"] = ();
        local["srz"] = ();
        local["cam_nx"] = ();
        local["cam_ny"] = ();
        local["cam_nz"] = ();
        local["cam_fx"] = ();
        local["cam_fy"] = ();
        local["cam_fz"] = ();
        local["cam_sway"] = ();
        local["vehicle_heading"] = 0.0;
        local["vehicle_speed_kms"] = 0.0;
        local["vehicle_vrad_kms"] = 0.0;
        local["vehicle_camera_sway"] = 0.0;
        local["vehicle_orbit_seeded"] = false;
        local["vehicle_grounded"] = true;
        local["vehicle_ship_runtime"] = vehicle.ship_runtime_state().to_map();
        local = set_vehicle_target_heading(local, vehicle, 0.0);
        local = set_vehicle_target_altitude(local, vehicle, 0.0);
        local = seed_vehicle_orbit_state(local, surface_radius, km_per_wu);
        world.set_transform(ship_id, cx + (local["snx"] ?? 1.0) * (local["radius"] ?? surface_radius), cy + (local["sny"] ?? 0.0) * (local["radius"] ?? surface_radius), 0.0);
        world.set_physics(ship_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
}

if ship_id > 0 {
    let frame_control = current_vehicle_frame_control(local, vehicle, input);
    let ship_transform = world.transform(ship_id) ?? #{};
    let ship_physics = world.physics(ship_id) ?? #{};
    local = sync_vehicle_ship_runtime(
        local,
        vehicle,
        body_info,
        body,
        frame_control,
        ship_transform["x"] ?? ship_x,
        ship_transform["y"] ?? ship_y,
        ship_physics["vx"] ?? 0.0,
        ship_physics["vy"] ?? 0.0,
        (local["vehicle_altitude_km"] ?? 0.0) <= surface_contact_altitude_km || (local["vehicle_grounded"] ?? false),
        input.action_down("lift")
    );
}

scene.set("vehicle-planet-view", "obj.rotation-speed", local["rotspeed"] ?? 3.0);
scene.set("vehicle-planet-view", "visible", true);

let stats_vehicle_control = current_vehicle_control_map(local, vehicle);
let stats_vehicle_assists = stats_vehicle_control["assists"] ?? #{};
let stats_vehicle_profile = current_vehicle_profile_name(local, vehicle);
let stats_vehicle_profile_label = hud_profile_label(stats_vehicle_profile);
let stats_alt_hold_label = hud_bool_label(stats_vehicle_assists["alt_hold"] ?? false);
let stats_hdg_hold_label = hud_bool_label(stats_vehicle_assists["heading_hold"] ?? false);
let stats_surface_g_fmt = ((local.surface_gravity_mps2 ?? 9.81) * 10.0).round() / 10.0;
let stats_alt = (local.vehicle_altitude_km ?? 0.0).max(0.0);
let stats_spd = (local.vehicle_speed_kms ?? 0.0).abs();
let stats_vrad = local.vehicle_vrad_kms ?? 0.0;
let stats_status = hud_status_label(current_vehicle_surface_mode(local, vehicle), stats_alt, stats_spd, stats_vrad);

let body_colour = hud_body_colour(body_patch_ok);
let body_type_label = hud_body_label(body_info["planet_type"] ?? local.planet_type ?? "earth_like");
let stats_speed_fmt = (stats_spd * 100.0).round() / 100.0;
let stats_vrad_fmt = (stats_vrad * 100.0).round() / 100.0;
let stats_sway_fmt = ((local.vehicle_camera_sway ?? 0.0) * 100.0).round() / 100.0;
let handoff_label = hud_handoff_label(local);
let escape_label = hud_escape_label(local);
scene.set("vehicle-hud-flight-state-value", "text.content", stats_status);
scene.set("vehicle-hud-flight-alt-value", "text.content", `${stats_alt.round()}`);
scene.set("vehicle-hud-flight-spd-value", "text.content", `${stats_speed_fmt}`);
scene.set("vehicle-hud-flight-vrad-value", "text.content", `${stats_vrad_fmt}`);
scene.set("vehicle-hud-vehicle-profile-value", "text.content", stats_vehicle_profile_label);
scene.set("vehicle-hud-vehicle-alt-hold-value", "text.content", stats_alt_hold_label);
scene.set("vehicle-hud-vehicle-hdg-hold-value", "text.content", stats_hdg_hold_label);
scene.set("vehicle-hud-flow-escape-value", "text.content", escape_label);
scene.set("vehicle-hud-world-body-value", "text.content", body_type_label);
scene.set("vehicle-hud-flow-source-value", "text.content", handoff_label);
scene.set("vehicle-hud-world-g-value", "text.content", `${stats_surface_g_fmt}`);
scene.set("vehicle-hud-vehicle-sway-value", "text.content", `${stats_sway_fmt}`);
scene.set("vehicle-hud-flight-state-value", "text.fg", body_colour);
scene.set("vehicle-hud-world-body-value", "text.fg", body_colour);
scene.set("vehicle-hud-world-g-value", "text.fg", body_colour);
scene.set("vehicle-hud-controls-escape-value", "text.content", escape_label);

if input.just_pressed("F9") || input.just_pressed("c") || input.just_pressed("C") {
    local = toggle_vehicle_profile(local, game, vehicle);
}
if input.just_pressed("h") {
    local = toggle_vehicle_assist_alt_hold(
        local,
        game,
        vehicle,
        if local["vehicle_grounded"] ?? false { 0.0 } else { local["vehicle_altitude_km"] ?? 0.0 }
    );
}
if input.just_pressed("j") {
    local = toggle_vehicle_assist_heading_hold(
        local,
        game,
        vehicle,
        local["ship_heading"] ?? local["vehicle_heading"] ?? 0.0
    );
}
if input.just_pressed("Escape") {
    let return_scene_id = resolve_return_scene_id(local);
    let return_mod_ref = resolve_return_mod_ref(local);
    if return_scene_id != "" && return_mod_ref != "" && (local.handoff_source ?? "defaults") == "packet" {
        let ship_physics = world.physics(ship_id) ?? #{};
        local = save_return_vehicle_handoff_packet(
            local,
            game,
            vehicle,
            body_info,
            ship_x,
            ship_y,
            ship_physics["vx"] ?? 0.0,
            ship_physics["vy"] ?? 0.0
        );
        local = persist_vehicle_state(local, game, vehicle);
        game.jump_mod(return_mod_ref, return_scene_id);
    }
}
