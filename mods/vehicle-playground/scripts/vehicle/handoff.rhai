import "std/math3" as math3;
import "vehicle/control" as control;
import "vehicle/environment" as environment;

fn producer_vehicle_handoff_state_path(producer_mod_id) {
    `/mods/${producer_mod_id}/vehicle/handoff`
}

fn resolve_handoff_packet_path(game) {
    let path = producer_vehicle_handoff_state_path("planet-generator");
    if game.has(path) {
        return path;
    }
    ""
}

fn build_vehicle_packet_basis_map(state) {
    #{
        normal: [
            state.snx ?? 0.0,
            state.sny ?? 0.0,
            state.snz ?? 0.0
        ],
        forward: [
            state.sfx ?? 0.0,
            state.sfy ?? 0.0,
            state.sfz ?? 0.0
        ],
        right: [
            state.srx ?? 0.0,
            state.sry ?? 0.0,
            state.srz ?? 0.0
        ]
    }
}

fn build_vehicle_packet_vehicle(state, vehicle) {
    vehicle.packet_vehicle_from_control(
        control::current_vehicle_control_state(state, vehicle),
        state.spawn_altitude_km ?? 0.0
    )
}

fn build_vehicle_packet_telemetry(state, vehicle, ship_x, ship_y, ship_vx, ship_vy) {
    vehicle
        .packet_telemetry_from(#{
            x: ship_x,
            y: ship_y,
            vx: ship_vx,
            vy: ship_vy,
            heading_deg: math3::rad_to_deg(state.ship_heading ?? state.vehicle_heading ?? 0.0),
            altitude_km: state.vehicle_altitude_km ?? 0.0,
            tangent_speed_kms: state.vehicle_speed_kms ?? 0.0,
            radial_speed_kms: state.vehicle_vrad_kms ?? 0.0,
            spawn_angle_deg: state.vehicle_spawn_angle_deg ?? 0.0,
            camera_sway: state.cam_sway ?? state.vehicle_camera_sway ?? 0.0,
            radius_wu: state.radius ?? 0.0,
            vfwd_wu_s: state.vfwd ?? 0.0,
            vright_wu_s: state.vright ?? 0.0,
            vrad_wu_s: state.vrad ?? 0.0,
            yaw_rate_rad_s: state.yaw_rate ?? 0.0,
            basis: build_vehicle_packet_basis_map(state),
            grounded: state.vehicle_grounded ?? false
        })
}

fn basis_component(axis, index, fallback) {
    if type_of(axis) == "array" && axis.len() > index {
        axis[index]
    } else {
        fallback
    }
}

fn raw_packet_basis_map(packet_map) {
    let telemetry = packet_map["telemetry"] ?? #{};
    if type_of(telemetry) != "map" {
        return ();
    }
    let basis = telemetry["basis"] ?? ();
    if type_of(basis) == "VehicleBasis3" {
        return basis;
    }
    ()
}

fn consume_handoff_packet(state, game, vehicle) {
    let path = resolve_handoff_packet_path(game);
    if path == "" {
        state.handoff_source = "defaults";
        state.handoff_return_scene_id = "";
        state.handoff_producer_mod_id = "";
        return state;
    }

    let packet_map = game.get(path);
    if type_of(packet_map) != "map" {
        state.handoff_source = "defaults";
        return state;
    }
    let packet = vehicle.launch_packet_from(packet_map);
    let session = vehicle.session_state_from_launch_packet(packet);

    state.handoff_source = "packet";
    state.handoff_packet_path = path;
    state.handoff_producer_mod_id = packet.envelope.producer_mod_id ?? "planet-generator";
    state.handoff_return_scene_id = packet.envelope.return_scene_id ?? "";

    let environment_binding = packet.environment_binding();
    state.planet_radius_real_km = environment_binding.real_radius_km ?? state.planet_radius_real_km ?? 6371.0;
    state.planet_radius_scale_divisor = environment_binding.scale_divisor ?? state.planet_radius_scale_divisor ?? 1.0;
    state.surface_gravity_mps2 = environment_binding.surface_gravity_mps2 ?? state.surface_gravity_mps2 ?? 9.81;
    state.atmo_top_km = environment_binding.atmosphere_top_km ?? state.atmo_top_km ?? 80.0;
    state.atmo_dense_start_km = environment_binding.atmosphere_dense_start_km ?? state.atmo_dense_start_km ?? 12.0;
    state.atmo_drag_max = environment_binding.atmosphere_drag_max ?? state.atmo_drag_max ?? 2.0;
    state.planet_type = packet.environment.body.body_kind ?? state.planet_type ?? "earth_like";
    let extras = packet.environment.extras;
    state.seed = extras["seed"] ?? state.seed ?? 0.0;
    state.has_ocean = extras["has_ocean"] ?? state.has_ocean ?? true;
    state.ocean = extras["ocean"] ?? state.ocean ?? 0.55;
    state.cscale = extras["cscale"] ?? state.cscale ?? 2.5;
    state.cwarp = extras["cwarp"] ?? state.cwarp ?? 0.65;
    state.coct = extras["coct"] ?? state.coct ?? 5.0;
    state.mscale = extras["mscale"] ?? state.mscale ?? 6.0;
    state.mstr = extras["mstr"] ?? state.mstr ?? 0.45;
    state.mroct = extras["mroct"] ?? state.mroct ?? 5.0;
    state.moisture = extras["moisture"] ?? state.moisture ?? 3.0;
    state.ice = extras["ice"] ?? state.ice ?? 1.0;
    state.lapse = extras["lapse"] ?? state.lapse ?? 0.6;
    state.rain = extras["rain"] ?? state.rain ?? 0.4;
    state.disp = extras["disp"] ?? state.disp ?? 0.22;
    state.subdiv = extras["subdiv"] ?? state.subdiv ?? 32.0;
    state.coloring = extras["coloring"] ?? state.coloring ?? 0.0;
    state.rotspeed = extras["rotspeed"] ?? state.rotspeed ?? 3.0;
    state.sunaz = extras["sunaz"] ?? state.sunaz ?? 210.0;
    state.sunel = extras["sunel"] ?? state.sunel ?? 40.0;
    state.ambient = extras["ambient"] ?? state.ambient ?? 0.12;
    state.atmo_color = extras["atmo_color"] ?? state.atmo_color ?? 2.0;
    state.atmo_density = extras["atmo_density"] ?? state.atmo_density ?? 0.48;
    state.atmo_height = extras["atmo_height"] ?? state.atmo_height ?? 0.11;
    state.atmo_forward_scatter = extras["atmo_forward_scatter"] ?? state.atmo_forward_scatter ?? 0.78;
    state.atmo_absorption = extras["atmo_absorption"] ?? state.atmo_absorption ?? 0.12;
    state.atmo_rayleigh_falloff = extras["atmo_rayleigh_falloff"] ?? state.atmo_rayleigh_falloff ?? 0.22;
    state.atmo_haze_amount = extras["atmo_haze_amount"] ?? state.atmo_haze_amount ?? 0.22;
    state.atmo_haze_falloff = extras["atmo_haze_falloff"] ?? state.atmo_haze_falloff ?? 0.46;

    state = control::sync_vehicle_control_state(state, session.control, vehicle);
    state.spawn_altitude_km = session.spawn_altitude_km ?? state.spawn_altitude_km ?? 0.0;

    if packet.has_telemetry() {
        let telemetry = session.telemetry;
        state.spawn_altitude_km = telemetry.altitude_km ?? state.spawn_altitude_km ?? 0.0;
        state.vehicle_heading = telemetry.heading_rad();
        state.vehicle_speed_kms = telemetry.tangent_speed_kms ?? state.vehicle_speed_kms ?? 0.0;
        state.vehicle_vrad_kms = telemetry.radial_speed_kms ?? state.vehicle_vrad_kms ?? 0.0;
        state.vehicle_spawn_angle_deg = telemetry.spawn_angle_deg ?? state.vehicle_spawn_angle_deg ?? 0.0;
        state.vehicle_camera_sway = telemetry.camera_sway ?? state.vehicle_camera_sway ?? 0.0;
        state.radius = telemetry.radius_wu ?? state.radius;
        state.vfwd = telemetry.forward_speed_wu_s ?? state.vfwd;
        state.vright = telemetry.lateral_speed_wu_s ?? state.vright;
        state.vrad = telemetry.radial_speed_wu_s ?? state.vrad;
        state.yaw_rate = telemetry.yaw_rate_rad_s ?? state.yaw_rate;
        let basis = raw_packet_basis_map(packet_map);
        if type_of(basis) == "VehicleBasis3" {
            let normal = basis.normal;
            let forward = basis.forward;
            let right = basis.right;
            state.snx = basis_component(normal, 0, state.snx);
            state.sny = basis_component(normal, 1, state.sny);
            state.snz = basis_component(normal, 2, state.snz);
            state.sfx = basis_component(forward, 0, state.sfx);
            state.sfy = basis_component(forward, 1, state.sfy);
            state.sfz = basis_component(forward, 2, state.sfz);
            state.srx = basis_component(right, 0, state.srx);
            state.sry = basis_component(right, 1, state.sry);
            state.srz = basis_component(right, 2, state.srz);
            state.vehicle_orbit_seeded = true;
        }
        state.cam_sway = telemetry.camera_sway ?? state.cam_sway;
        state.vehicle_grounded = telemetry.grounded ?? state.vehicle_grounded ?? false;
    }

    game.remove(path);
    let producer_mod_id = state.handoff_producer_mod_id ?? "planet-generator";
    let producer_vehicle_path = producer_vehicle_handoff_state_path(producer_mod_id);
    if path != producer_vehicle_path && game.has(producer_vehicle_path) {
        game.remove(producer_vehicle_path);
    }
    state
}

fn resolve_return_scene_id(state) {
    state.handoff_return_scene_id ?? ""
}

fn resolve_return_mod_ref(state) {
    state.handoff_producer_mod_id ?? ""
}

fn hud_handoff_label(state) {
    if (state.handoff_source ?? "defaults") == "packet" { "PACKET" } else { "DEFAULTS" }
}

fn hud_escape_label(state) {
    if resolve_return_scene_id(state) != "" && resolve_return_mod_ref(state) != "" && (state.handoff_source ?? "defaults") == "packet" {
        "RETURN"
    } else {
        "RESET"
    }
}

fn save_return_vehicle_handoff_packet(state, game, vehicle, body_snapshot, ship_x, ship_y, ship_vx, ship_vy) {
    let producer_mod_id = state.handoff_producer_mod_id ?? "planet-generator";
    let return_scene_id = resolve_return_scene_id(state);
    let packet_vehicle = build_vehicle_packet_vehicle(state, vehicle);
    let telemetry = build_vehicle_packet_telemetry(state, vehicle, ship_x, ship_y, ship_vx, ship_vy);
    let packet = #{
        producer_mod_id: "vehicle-playground",
        source_scene_id: "vehicle-playground-vehicle",
        target_mod_ref: producer_mod_id,
        target_scene_id: return_scene_id,
        return_scene_id: return_scene_id,
        consumer_hint: "generator-return",
        environment: environment::build_vehicle_environment_map(state, body_snapshot),
        vehicle: packet_vehicle.to_map(),
        telemetry: telemetry.to_map()
    };
    game.set(producer_vehicle_handoff_state_path(producer_mod_id), packet);
    state
}
