import "generator/body_sync" as body_sync;

fn normalize_vehicle_profile_name(profile_name, vehicle) {
    vehicle.normalize_profile_id(profile_name ?? vehicle.default_profile_id())
}

fn build_vehicle_control_state(profile_name, alt_hold, heading_hold, vehicle) {
    let normalized = normalize_vehicle_profile_name(profile_name, vehicle);
    let default_profile = vehicle.default_profile_id();

    if normalized != default_profile && normalized != "sim-lite" {
        return vehicle.control_state_with(normalized, alt_hold, heading_hold);
    }

    let control = vehicle.control_state();
    if normalized != default_profile {
        control.cycle_ship_profile();
    }
    if alt_hold {
        control.toggle_altitude_hold();
    }
    if heading_hold {
        control.toggle_heading_hold();
    }
    control
}

fn default_vehicle_session(vehicle) {
    let control = vehicle.control_state();
    vehicle.session_state().with_control(control).with_spawn(0.0, 0.0)
}

fn legacy_vehicle_session(local, vehicle) {
    let control = build_vehicle_control_state(
        local["vehicle_profile"] ?? vehicle.default_profile_id(),
        local["vehicle_assist_alt_hold"] ?? false,
        local["vehicle_assist_heading_hold"] ?? false,
        vehicle
    );
    default_vehicle_session(vehicle)
        .with_control(control)
        .with_spawn(local["spawn_altitude_km"] ?? 0.0, 0.0)
}

fn current_vehicle_session(local, vehicle) {
    let session = local["vehicle_session"] ?? ();
    if type_of(session) == "VehicleSessionState" {
        return session.normalized();
    }
    legacy_vehicle_session(local, vehicle)
}

fn store_vehicle_session(local, session) {
    local["vehicle_session"] = session.normalized();
    local
}

fn ensure_vehicle_session(local, vehicle) {
    store_vehicle_session(local, current_vehicle_session(local, vehicle))
}

fn current_vehicle_profile_name(local, vehicle) {
    current_vehicle_session(local, vehicle).control.profile_id
}

fn current_vehicle_assist_alt_hold(local, vehicle) {
    current_vehicle_session(local, vehicle).control.assists.alt_hold
}

fn current_vehicle_assist_heading_hold(local, vehicle) {
    current_vehicle_session(local, vehicle).control.assists.heading_hold
}

fn current_vehicle_profile_hud_label(local, vehicle) {
    vehicle_profile_hud_label(current_vehicle_profile_name(local, vehicle), vehicle)
}

fn vehicle_profile_hud_label(profile_name, vehicle) {
    if normalize_vehicle_profile_name(profile_name, vehicle) == "sim-lite" {
        "SIM-L"
    } else {
        "ARC"
    }
}

fn vehicle_profile_state_path() {
    "/mods/planet-generator/vehicle/profile"
}

fn vehicle_assist_alt_state_path() {
    "/mods/planet-generator/vehicle/assist_alt_hold"
}

fn vehicle_assist_heading_state_path() {
    "/mods/planet-generator/vehicle/assist_heading_hold"
}

fn vehicle_handoff_state_path() {
    "/mods/planet-generator/vehicle/handoff"
}

fn vehicle_producer_mod_id() {
    "planet-generator"
}

fn vehicle_scene_id() {
    "planet-generator-main"
}

fn vehicle_return_packet_kind(vehicle) {
    vehicle.return_packet().envelope.packet_kind
}

fn vehicle_target_mod_state_path() {
    "/mods/planet-generator/vehicle/target_mod_ref"
}

fn vehicle_target_scene_state_path() {
    "/mods/planet-generator/vehicle/target_scene_id"
}

fn default_vehicle_target_mod_ref() {
    "vehicle-playground"
}

fn default_vehicle_target_scene_id() {
    "vehicle-playground-vehicle"
}

fn resolve_vehicle_target_mod_ref(game) {
    let configured = game.get_s(vehicle_target_mod_state_path(), "");
    if configured == "" {
        default_vehicle_target_mod_ref()
    } else {
        configured
    }
}

fn resolve_vehicle_target_scene_id(game) {
    let configured = game.get_s(vehicle_target_scene_state_path(), "");
    if configured == "" {
        default_vehicle_target_scene_id()
    } else {
        configured
    }
}

fn bool_to_i(v) {
    if v { 1 } else { 0 }
}

fn i_to_bool(v) {
    v != 0
}

fn hud_on_off(v) {
    if v { "ON" } else { "OFF" }
}

fn vehicle_handoff_hud_label(game) {
    let target_mod_ref = resolve_vehicle_target_mod_ref(game);
    let target_scene_id = resolve_vehicle_target_scene_id(game);
    if target_mod_ref == "" || target_scene_id == "" {
        "LINK:OFF"
    } else {
        "LINK:OK"
    }
}

fn toggle_vehicle_profile(local, vehicle) {
    let session = current_vehicle_session(local, vehicle);
    let control = session.control;
    control.cycle_ship_profile();
    store_vehicle_session(local, session.with_control(control))
}

fn persist_vehicle_profile(local, game, vehicle) {
    let profile = normalize_vehicle_profile_name(current_vehicle_profile_name(local, vehicle), vehicle);
    game.set(vehicle_profile_state_path(), profile);
    local
}

fn persist_vehicle_assists(local, game, vehicle) {
    let alt_hold = current_vehicle_assist_alt_hold(local, vehicle);
    let heading_hold = current_vehicle_assist_heading_hold(local, vehicle);
    game.set(vehicle_assist_alt_state_path(), bool_to_i(alt_hold));
    game.set(vehicle_assist_heading_state_path(), bool_to_i(heading_hold));
    local
}

fn restore_persisted_vehicle_state(local, game, vehicle) {
    let control = build_vehicle_control_state(
        game.get_s(vehicle_profile_state_path(), vehicle.default_profile_id()),
        i_to_bool(game.get_i(vehicle_assist_alt_state_path(), 0)),
        i_to_bool(game.get_i(vehicle_assist_heading_state_path(), 0)),
        vehicle
    );
    store_vehicle_session(
        local,
        current_vehicle_session(local, vehicle).with_control(control)
    )
}

fn persist_vehicle_state(local, game, vehicle) {
    local = persist_vehicle_profile(local, game, vehicle);
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn toggle_vehicle_profile_state(local, game, vehicle) {
    local = toggle_vehicle_profile(local, vehicle);
    local = persist_vehicle_profile(local, game, vehicle);
    local
}

fn toggle_vehicle_assist_alt_hold(local, game, vehicle) {
    let session = current_vehicle_session(local, vehicle);
    let control = session.control;
    control.toggle_altitude_hold();
    local = store_vehicle_session(local, session.with_control(control));
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn toggle_vehicle_assist_heading_hold(local, game, vehicle) {
    let session = current_vehicle_session(local, vehicle);
    let control = session.control;
    control.toggle_heading_hold();
    local = store_vehicle_session(local, session.with_control(control));
    local = persist_vehicle_assists(local, game, vehicle);
    local
}

fn build_vehicle_packet_vehicle_map(local, vehicle) {
    let packet_vehicle = current_vehicle_session(local, vehicle).packet_vehicle();
    #{
        profile_id: packet_vehicle.profile_id,
        assist_alt_hold: packet_vehicle.assist_alt_hold,
        assist_heading_hold: packet_vehicle.assist_heading_hold,
        spawn_altitude_km: packet_vehicle.spawn_altitude_km
    }
}

fn build_vehicle_body_map(local, body_snapshot) {
    if type_of(body_snapshot) != "WorldBodySnapshot" || !body_snapshot.exists {
        return body_sync::build_generated_body_from_local(local);
    }

    #{
        id: body_snapshot.id,
        center_x: body_snapshot.center_x,
        center_y: body_snapshot.center_y,
        radius_px: body_snapshot.radius_px,
        km_per_px: body_snapshot.km_per_px,
        surface_radius: body_snapshot.surface_radius,
        radius_km: body_snapshot.radius_km,
        gravity_mu_km3_s2: body_snapshot.gravity_mu_km3_s2,
        atmosphere_top_km: body_snapshot.atmosphere_top_km,
        atmosphere_dense_start_km: body_snapshot.atmosphere_dense_start_km,
        atmosphere_drag_max: body_snapshot.atmosphere_drag_max,
        cloud_bottom_km: body_snapshot.cloud_bottom_km,
        cloud_top_km: body_snapshot.cloud_top_km,
        planet_type: body_snapshot.planet_type
    }
}

fn build_vehicle_environment_map(local, body) {
    let extras = current_generator_environment_extras(local);
    #{
        body: body,
        real_radius_km: local["planet_radius_real_km"] ?? 6371.0,
        scale_divisor: local["planet_radius_scale_divisor"] ?? 1.0,
        surface_gravity_mps2: local["surface_gravity_mps2"] ?? 9.81,
        atmo_top_km: local["atmo_top_km"] ?? 80.0,
        atmo_dense_start_km: local["atmo_dense_start_km"] ?? 12.0,
        atmo_drag_max: local["atmo_drag_max"] ?? 2.0,
        seed: extras["seed"],
        ocean: extras["ocean"],
        cscale: extras["cscale"],
        cwarp: extras["cwarp"],
        coct: extras["coct"],
        mscale: extras["mscale"],
        mstr: extras["mstr"],
        mroct: extras["mroct"],
        moisture: extras["moisture"],
        ice: extras["ice"],
        lapse: extras["lapse"],
        rain: extras["rain"],
        disp: extras["disp"],
        subdiv: extras["subdiv"],
        coloring: extras["coloring"],
        rotspeed: extras["rotspeed"],
        sunaz: extras["sunaz"],
        sunel: extras["sunel"],
        ambient: extras["ambient"],
        atmo_color: extras["atmo_color"],
        atmo_density: extras["atmo_density"],
        atmo_height: extras["atmo_height"],
        atmo_forward_scatter: extras["atmo_forward_scatter"],
        atmo_absorption: extras["atmo_absorption"],
        atmo_rayleigh_falloff: extras["atmo_rayleigh_falloff"],
        atmo_haze_amount: extras["atmo_haze_amount"],
        atmo_haze_falloff: extras["atmo_haze_falloff"]
    }
}

fn current_generator_ui_state(local) {
    #{
        tab: local["tab"] ?? 0,
        model_mode: local["model_mode"] ?? 1,
        preset_choice: local["preset_choice"] ?? 0,
        rotation_paused: local["rotation_paused"] ?? false
    }
}

fn current_generator_environment_extras(local) {
    #{
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

fn apply_generator_environment_extras(local, extras) {
    local["seed"] = extras["seed"] ?? local["seed"] ?? 0.0;
    local["ocean"] = extras["ocean"] ?? local["ocean"] ?? 0.55;
    local["cscale"] = extras["cscale"] ?? local["cscale"] ?? 2.5;
    local["cwarp"] = extras["cwarp"] ?? local["cwarp"] ?? 0.65;
    local["coct"] = extras["coct"] ?? local["coct"] ?? 5.0;
    local["mscale"] = extras["mscale"] ?? local["mscale"] ?? 6.0;
    local["mstr"] = extras["mstr"] ?? local["mstr"] ?? 0.45;
    local["mroct"] = extras["mroct"] ?? local["mroct"] ?? 5.0;
    local["moisture"] = extras["moisture"] ?? local["moisture"] ?? 3.0;
    local["ice"] = extras["ice"] ?? local["ice"] ?? 1.0;
    local["lapse"] = extras["lapse"] ?? local["lapse"] ?? 0.6;
    local["rain"] = extras["rain"] ?? local["rain"] ?? 0.4;
    local["disp"] = extras["disp"] ?? local["disp"] ?? 0.22;
    local["subdiv"] = extras["subdiv"] ?? local["subdiv"] ?? 32.0;
    local["coloring"] = extras["coloring"] ?? local["coloring"] ?? 0.0;
    local["rotspeed"] = extras["rotspeed"] ?? local["rotspeed"] ?? 3.0;
    local["sunaz"] = extras["sunaz"] ?? local["sunaz"] ?? 210.0;
    local["sunel"] = extras["sunel"] ?? local["sunel"] ?? 40.0;
    local["ambient"] = extras["ambient"] ?? local["ambient"] ?? 0.12;
    local["atmo_color"] = extras["atmo_color"] ?? local["atmo_color"] ?? 2.0;
    local["atmo_density"] = extras["atmo_density"] ?? local["atmo_density"] ?? 0.48;
    local["atmo_height"] = extras["atmo_height"] ?? local["atmo_height"] ?? 0.11;
    local["atmo_forward_scatter"] = extras["atmo_forward_scatter"] ?? local["atmo_forward_scatter"] ?? 0.78;
    local["atmo_absorption"] = extras["atmo_absorption"] ?? local["atmo_absorption"] ?? 0.12;
    local["atmo_rayleigh_falloff"] = extras["atmo_rayleigh_falloff"] ?? local["atmo_rayleigh_falloff"] ?? 0.22;
    local["atmo_haze_amount"] = extras["atmo_haze_amount"] ?? local["atmo_haze_amount"] ?? 0.22;
    local["atmo_haze_falloff"] = extras["atmo_haze_falloff"] ?? local["atmo_haze_falloff"] ?? 0.46;
    local
}

fn apply_vehicle_return_environment(local, environment, extras) {
    local["planet_radius_real_km"] = environment.real_radius_km;
    local["planet_radius_scale_divisor"] = environment.scale_divisor;
    local["surface_gravity_mps2"] = environment.surface_gravity_mps2;
    local["atmo_top_km"] = environment.atmosphere_top_km;
    local["atmo_dense_start_km"] = environment.atmosphere_dense_start_km;
    local["atmo_drag_max"] = environment.atmosphere_drag_max;
    local = apply_generator_environment_extras(local, extras);
    local["planet_radius_sim_km"] = body_sync::sim_radius_km(
        local["planet_radius_real_km"] ?? 6371.0,
        local["planet_radius_scale_divisor"] ?? 1.0
    );
    local
}

fn apply_vehicle_return_vehicle_state(local, session) {
    store_vehicle_session(local, session)
}

fn apply_vehicle_return_telemetry(local, telemetry) {
    local["return_altitude_km"] = telemetry.altitude_km;
    local["return_speed_kms"] = telemetry.tangent_speed_kms;
    local["return_vrad_kms"] = telemetry.radial_speed_kms;
    local
}

fn apply_generator_ui_state(local, ui) {
    local["tab"] = ui["tab"] ?? local["tab"] ?? 0;
    local["model_mode"] = ui["model_mode"] ?? local["model_mode"] ?? 1;
    local["preset_choice"] = ui["preset_choice"] ?? local["preset_choice"] ?? 0;
    local["rotation_paused"] = ui["rotation_paused"] ?? local["rotation_paused"] ?? false;
    local
}

fn apply_vehicle_return_ui(local, ui) {
    if type_of(ui) != "map" {
        return local;
    }
    apply_generator_ui_state(local, ui)
}

fn mark_return_applied(local) {
    local["tab_sync_needed"] = true;
    local["push_dirty"] = true;
    local["push_timer"] = 150.0;
    local
}

fn request_vehicle_launch(local) {
    local["vehicle_launch_requested"] = true;
    local
}

fn consume_vehicle_launch_request(local) {
    let requested = local["vehicle_launch_requested"] ?? false;
    local["vehicle_launch_requested"] = false;
    #{
        local: local,
        requested: requested
    }
}

fn consume_vehicle_return_packet(local, game, vehicle) {
    let path = vehicle_handoff_state_path();
    if !game.has(path) {
        return local;
    }

    let packet_doc = game.get(path);
    if type_of(packet_doc) != "map" {
        return local;
    }

    let packet = vehicle.return_packet_from(packet_doc);
    if !packet.is_vehicle_return() || packet.envelope.packet_kind != vehicle_return_packet_kind(vehicle) {
        return local;
    }

    let telemetry = packet.telemetry_snapshot();
    let session = vehicle
        .session_state_from_return_packet(packet)
        .with_spawn(telemetry.altitude_km, telemetry.spawn_angle_deg);
    local = apply_vehicle_return_environment(local, packet.environment, packet.environment.extras);
    local = apply_vehicle_return_vehicle_state(local, session);
    local = apply_vehicle_return_telemetry(local, telemetry);
    local = apply_vehicle_return_ui(local, packet.ui);
    local = persist_vehicle_state(local, game, vehicle);
    local = mark_return_applied(local);
    game.remove(path);
    local
}

fn build_vehicle_launch_packet(local, game, vehicle, body_snapshot) {
    let target_mod_ref = resolve_vehicle_target_mod_ref(game);
    let target_scene_id = resolve_vehicle_target_scene_id(game);
    let environment = build_vehicle_environment_map(
        local,
        build_vehicle_body_map(local, body_snapshot)
    );

    vehicle.launch_packet_from(#{
        producer_mod_id: vehicle_producer_mod_id(),
        source_scene_id: vehicle_scene_id(),
        target_mod_ref: target_mod_ref,
        target_scene_id: target_scene_id,
        return_scene_id: vehicle_scene_id(),
        consumer_hint: "vehicle-runtime",
        environment: environment,
        vehicle: build_vehicle_packet_vehicle_map(local, vehicle),
        ui: current_generator_ui_state(local)
    }).to_map()
}

fn publish_vehicle_launch_packet(local, game, vehicle, body_snapshot) {
    let packet = build_vehicle_launch_packet(local, game, vehicle, body_snapshot);
    game.set(vehicle_handoff_state_path(), packet);
    local
}

fn launch_vehicle_scene(local, game, vehicle, body_snapshot) {
    local = persist_vehicle_state(local, game, vehicle);
    local = publish_vehicle_launch_packet(local, game, vehicle, body_snapshot);
    local["vehicle_launch_requested"] = false;
    game.jump_mod(resolve_vehicle_target_mod_ref(game), resolve_vehicle_target_scene_id(game));
    local
}
