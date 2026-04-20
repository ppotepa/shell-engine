fn ensure_root(local) {
    if type_of(local) == "()" { return #{}; }
    local
}

fn normalize_typed_vehicle_state(state, vehicle) {
    let control = state["vehicle_control"] ?? ();
    if type_of(control) == "map" {
        state.vehicle_control = vehicle.control_state_from(control);
    } else if type_of(control) != "()" {
        state.vehicle_control = control.normalized();
    }

    let runtime_state = state["vehicle_ship_runtime"] ?? ();
    if type_of(runtime_state) == "map" {
        state.vehicle_ship_runtime = vehicle.ship_runtime_state_from(runtime_state);
    } else if type_of(runtime_state) != "()" {
        state.vehicle_ship_runtime = runtime_state.normalized();
    }
    state
}

fn load(local, vehicle) {
    let root = ensure_root(local);
    let state = root["state"] ?? ();
    if type_of(state) == "map" {
        return normalize_typed_vehicle_state(state, vehicle);
    }
    if type_of(root["initialized"]) != "()" || type_of(root["vehicle_control"]) != "()" {
        return normalize_typed_vehicle_state(root, vehicle);
    }
    #{}
}

fn store(_local, state) {
    #{ state: state }
}

fn initialized(state) {
    state.initialized ?? false
}

fn default_state(vehicle) {
    let state = #{
        initialized: true,
        mode: "vehicle",
        vehicle_orbit_seeded: false,
        vehicle_grounded: true,
        handoff_source: "defaults",
        handoff_packet_path: "",
        handoff_producer_mod_id: "",
        handoff_return_scene_id: "",
        vehicle_heading: 0.0,
        vehicle_altitude_km: 0.0,
        vehicle_speed_kms: 0.0,
        vehicle_vrad_kms: 0.0,
        planet_radius_real_km: 6371.0,
        planet_radius_scale_divisor: 1.0,
        surface_gravity_mps2: 9.81,
        spawn_altitude_km: 0.0,
        atmo_top_km: 80.0,
        atmo_dense_start_km: 12.0,
        atmo_drag_max: 2.0,
        planet_type: "earth_like",
        sun_az_deg: 210.0,
        sun_el_deg: 40.0,
        vehicle_spawn_angle_deg: 0.0,
        ship_id: 0,
        vehicle_camera_sway: 0.0,
        vehicle_camera_zoom: 0.0,
        seed: 1847.0,
        ocean: 0.55,
        cscale: 2.5,
        cwarp: 0.65,
        coct: 5.0,
        mscale: 6.0,
        mstr: 0.45,
        mroct: 5.0,
        moisture: 3.0,
        ice: 1.0,
        lapse: 0.6,
        rain: 0.4,
        disp: 0.22,
        subdiv: 32.0,
        coloring: 0.0,
        rotspeed: 3.0,
        sunaz: 210.0,
        sunel: 40.0,
        ambient: 0.12,
        atmo_color: 2.0,
        atmo_density: 0.48,
        atmo_height: 0.11,
        atmo_forward_scatter: 0.78,
        atmo_absorption: 0.12,
        atmo_rayleigh_falloff: 0.22,
        atmo_haze_amount: 0.22,
        atmo_haze_falloff: 0.46
    };
    state.vehicle_control = vehicle.control_state();
    state.vehicle_ship_runtime = vehicle.ship_runtime_state();
    state
}
