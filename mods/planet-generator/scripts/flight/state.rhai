import "std/flight_handoff" as flight_handoff;
import "std/planet_spec" as planet_spec;

fn apply_flight_planet_defaults(local) {
    let spec = planet_spec::snapshot(local);
    spec["seed"] = 2701.0;
    spec["mscale"] = 7.5;
    spec["mstr"] = 0.22;
    spec["mroct"] = 5.0;
    spec["disp"] = 0.0022;
    spec["ambient"] = 0.09;
    spec["sunaz"] = 224.0;
    spec["sunel"] = 34.0;
    spec["atmo_density"] = 0.62;
    spec["atmo_height"] = 0.17;
    spec["atmo_forward_scatter"] = 0.84;
    spec["atmo_absorption"] = 0.10;
    spec["atmo_rayleigh_falloff"] = 0.20;
    spec["atmo_haze_amount"] = 0.34;
    spec["atmo_haze_falloff"] = 0.52;
    spec["planet_radius_real_km"] = 6371.0;
    spec["planet_radius_scale_divisor"] = 1.0;
    spec["surface_gravity_mps2"] = 9.81;
    spec["atmo_top_km"] = 80.0;
    spec["atmo_dense_start_km"] = 12.0;
    spec["atmo_drag_max"] = 1.6;
    spec["rotspeed"] = 0.0;
    spec["subdiv"] = 64.0;
    planet_spec::apply_snapshot(local, spec)
}

fn ensure_initialized(local, persist) {
    local = local ?? #{};
    if local.flight_initialized ?? false {
        return local;
    }

    local.model_mode = 1;
    local.rotation_paused = true;
    local.render_push_throttle_ms = 0.0;
    local.push_ready = false;
    local.push_dirty = true;
    local.push_timer = 0.0;

    // The generator preset uses very exaggerated displacement meant for a
    // preview sphere. Flight mode still needs restrained relief so the camera
    // and spawn point stay sane, but the previous values ended up looking
    // almost flat from orbital altitude.
    local = apply_flight_planet_defaults(local);

    local.camera_mode = "cockpit";
    local.ship_seeded = false;
    local.ship_spawn_use_finder = true;
    local.ship_spawn_biomes = ["beach", "grassland", "desert"];
    local.ship_spawn_angle_deg = 0.0;
    local.ship_spawn_latitude_deg = 0.0;
    local.ship_spawn_biome = "unknown";
    local.ship_spawn_surface_offset = 0.0;
    local.ship_heading_deg = 0.0;
    local.ship_size_m = 5.0;
    local.ship_spawn_altitude_m = 500000.0;
    local.ship_ground_clearance_m = 1.5;
    local.ship_surface_altitude = 0.0;
    local.ship_visual_scale = 0.0;
    local.cockpit_model_scale_m = 1.0;
    local.cockpit_eye_forward_m = 0.92;
    local.cockpit_eye_lift_m = 0.36;
    local.cockpit_look_distance_m = 4000.0;
    local.cockpit_forward_lag_sec = 0.06;
    local.cockpit_up_lag_sec = 0.08;
    local.cockpit_roll_lag_sec = 0.14;
    local.cockpit_sway_lag_sec = 0.16;
    local.cockpit_side_sway_m = 0.18;
    local.cockpit_lift_sway_m = 0.05;
    local.cockpit_roll_gain = 0.08;
    local.cockpit_roll_max_deg = 3.8;
    local.cockpit_look_up_bias_m = 14.0;
    local.flight_mouse_deg_per_px = 0.12;
    local.flight_mouse_lag_sec = 0.05;
    local.flight_mouse_damp = 7.5;
    local.flight_mouse_max_rate_deg = 180.0;
    local.flight_horizon_assist = 0.28;
    local.flight_thrust_forward_mps2 = 42.0;
    local.flight_thrust_strafe_mps2 = 26.0;
    local.flight_lift_thrust_mps2 = 24.0;
    local.flight_vacuum_drag = 0.02;
    local.flight_atmo_drag = 0.70;
    local.flight_idle_drag = 0.05;
    local.flight_max_speed_vacuum_mps = 3600.0;
    local.flight_max_speed_atmo_mps = 950.0;
    local.ship_display_altitude_km = 0.0;
    local.ship_display_speed_kms = 0.0;
    local.ship_display_altitude_m = 0.0;
    local.ship_display_speed_mps = 0.0;

    let launch_snapshot = flight_handoff::consume_launch_snapshot(persist);
    if launch_snapshot != () {
        local = flight_handoff::apply_launch_snapshot(local, launch_snapshot);
    }

    local.flight_initialized = true;

    local
}
