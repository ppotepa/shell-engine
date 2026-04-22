import "std/flight_handoff" as flight_handoff;
import "std/planet_spec" as planet_spec;

fn apply_cockpit_planet_defaults(local) {
    let spec = planet_spec::snapshot(local);
    spec["seed"] = 2701.0;
    spec["mscale"] = 7.5;
    spec["mstr"] = 0.22;
    spec["mroct"] = 5.0;
    spec["disp"] = 0.0022;
    spec["ambient"] = 0.10;
    spec["sunaz"] = 220.0;
    spec["sunel"] = 30.0;
    spec["atmo_density"] = 0.64;
    spec["atmo_height"] = 0.18;
    spec["atmo_forward_scatter"] = 0.85;
    spec["atmo_absorption"] = 0.10;
    spec["atmo_rayleigh_falloff"] = 0.20;
    spec["atmo_haze_amount"] = 0.36;
    spec["atmo_haze_falloff"] = 0.54;
    spec["planet_radius_real_km"] = 6371.0;
    spec["planet_radius_scale_divisor"] = 1.0;
    spec["surface_gravity_mps2"] = 9.81;
    spec["atmo_top_km"] = 80.0;
    spec["atmo_dense_start_km"] = 12.0;
    spec["atmo_drag_max"] = 1.6;
    spec["rotspeed"] = 0.32;
    spec["subdiv"] = 64.0;
    planet_spec::apply_snapshot(local, spec)
}

fn apply_cockpit_visual_overrides(local) {
    let spec = planet_spec::snapshot(local);
    spec["rotspeed"] = 0.32;
    planet_spec::apply_snapshot(local, spec)
}

fn ensure_initialized(local, persist) {
    local = local ?? #{};
    if local.cockpitview_initialized ?? false {
        return local;
    }

    local.model_mode = 1;
    local.rotation_paused = false;
    local.render_push_throttle_ms = 0.0;
    local.push_ready = false;
    local.push_dirty = true;
    local.push_timer = 0.0;

    local = apply_cockpit_planet_defaults(local);

    local.cockpit_mesh_scale = 0.22;
    local.cockpit_eye_x = 0.0;
    local.cockpit_eye_y = 0.08;
    local.cockpit_eye_z = 3.75;
    local.cockpit_look_distance = 12.0;
    local.cockpit_mouse_yaw_limit_deg = 10.0;
    local.cockpit_mouse_pitch_limit_deg = 7.0;
    local.cockpit_mouse_lag_sec = 0.08;
    local.cockpit_look_yaw_deg = 0.0;
    local.cockpit_look_pitch_deg = 0.0;
    local.cockpit_center_x = 320.0;
    local.cockpit_center_y = 180.0;
    local.cockpit_title = "COCKPIT SIMULATOR";

    let launch_snapshot = flight_handoff::consume_launch_snapshot(persist);
    if launch_snapshot != () {
        local = flight_handoff::apply_launch_snapshot(local, launch_snapshot);
        local.rotation_paused = false;
        local = apply_cockpit_visual_overrides(local);
    }

    local.cockpitview_initialized = true;
    local
}
