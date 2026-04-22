import "generator/presets" as presets;
import "std/flight_handoff" as flight_handoff;

fn ensure_initialized(local, persist) {
    local = local ?? #{};
    if local.cockpitview_initialized ?? false {
        return local;
    }

    local = presets::apply_choice(local, 0);

    local.model_mode = 1;
    local.rotation_paused = false;
    local.rotspeed = 0.32;
    local.subdiv = 64.0;
    local.render_push_throttle_ms = 0.0;
    local.push_ready = false;
    local.push_dirty = true;
    local.push_timer = 0.0;

    local.seed = 2701.0;
    local.mscale = 7.5;
    local.mstr = 0.22;
    local.mroct = 5.0;
    local.disp = 0.0022;
    local.ambient = 0.10;
    local.sunaz = 220.0;
    local.sunel = 30.0;
    local.atmo_density = 0.64;
    local.atmo_height = 0.18;
    local.atmo_forward_scatter = 0.85;
    local.atmo_absorption = 0.10;
    local.atmo_rayleigh_falloff = 0.20;
    local.atmo_haze_amount = 0.36;
    local.atmo_haze_falloff = 0.54;

    local.planet_radius_real_km = 6371.0;
    local.planet_radius_scale_divisor = 1.0;
    local.surface_gravity_mps2 = 9.81;
    local.atmo_top_km = 80.0;
    local.atmo_dense_start_km = 12.0;
    local.atmo_drag_max = 1.6;

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
        local.rotspeed = 0.32;
    }

    local.cockpitview_initialized = true;
    local
}
