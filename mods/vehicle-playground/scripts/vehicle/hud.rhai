import "std/runtime_scene" as scene_rt;
import "vehicle/control" as control;
import "vehicle/flight" as flight;
import "vehicle/handoff" as handoff;
import "vehicle/state" as vehicle_state;

fn hud_bool_label(v) { if v { "ON" } else { "OFF" } }

fn hud_profile_label(profile_name) {
    if profile_name == "sim-lite" { "SIM-LITE" } else { "ARCADE" }
}

fn hud_status_label(phase) {
    if phase == "pad" { "PAD" }
    else if phase == "hover" { "HOVER" }
    else if phase == "orbit" { "ORBIT" }
    else { "FLIGHT" }
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

fn render(state, runtime, vehicle, body_ctx) {
    let body_snapshot = body_ctx["body_snapshot"];
    let body_patch_ok = body_ctx["body_patch_ok"] ?? false;
    let stats_vehicle_model_label = vehicle_state::vehicle_model_label(state);
    let stats_vehicle_control = control::current_vehicle_control_state(state, vehicle);
    let stats_vehicle_assists = stats_vehicle_control.assists;
    let stats_vehicle_profile = stats_vehicle_control.profile_id ?? control::current_vehicle_profile_name(state, vehicle);
    let stats_vehicle_profile_label = hud_profile_label(stats_vehicle_profile);
    let stats_alt_hold_label = hud_bool_label(stats_vehicle_assists.alt_hold);
    let stats_hdg_hold_label = hud_bool_label(stats_vehicle_assists.heading_hold);
    let stats_surface_g_fmt = ((state.surface_gravity_mps2 ?? 9.81) * 10.0).round() / 10.0;
    let stats_alt = (state.vehicle_altitude_km ?? 0.0).max(0.0);
    let stats_spd = (state.vehicle_speed_kms ?? 0.0).abs();
    let stats_vrad = state.vehicle_vrad_kms ?? 0.0;
    let stats_status = hud_status_label(flight::current_vehicle_phase(state, vehicle));

    let body_colour = hud_body_colour(body_patch_ok);
    let body_type_label =
        if body_snapshot.exists {
            hud_body_label(body_snapshot.planet_type)
        } else {
            hud_body_label(state.planet_type ?? "earth_like")
        };
    let stats_speed_fmt = (stats_spd * 100.0).round() / 100.0;
    let stats_vrad_fmt = (stats_vrad * 100.0).round() / 100.0;
    let stats_sway_fmt = ((state.vehicle_camera_sway ?? 0.0) * 100.0).round() / 100.0;
    let handoff_label = handoff::hud_handoff_label(state);
    let escape_label = handoff::hud_escape_label(state);

    scene_rt::text(runtime, "vehicle-hud-flight-state-value", stats_status);
    scene_rt::text(runtime, "vehicle-hud-flight-alt-value", `${stats_alt.round()}`);
    scene_rt::text(runtime, "vehicle-hud-flight-spd-value", `${stats_speed_fmt}`);
    scene_rt::text(runtime, "vehicle-hud-flight-vrad-value", `${stats_vrad_fmt}`);
    scene_rt::text(runtime, "vehicle-hud-vehicle-model-value", stats_vehicle_model_label);
    scene_rt::text(runtime, "vehicle-hud-vehicle-profile-value", stats_vehicle_profile_label);
    scene_rt::text(runtime, "vehicle-hud-vehicle-alt-hold-value", stats_alt_hold_label);
    scene_rt::text(runtime, "vehicle-hud-vehicle-hdg-hold-value", stats_hdg_hold_label);
    scene_rt::text(runtime, "vehicle-hud-flow-escape-value", escape_label);
    scene_rt::text(runtime, "vehicle-hud-world-body-value", body_type_label);
    scene_rt::text(runtime, "vehicle-hud-flow-source-value", handoff_label);
    scene_rt::text(runtime, "vehicle-hud-world-g-value", `${stats_surface_g_fmt}`);
    scene_rt::text(runtime, "vehicle-hud-vehicle-sway-value", `${stats_sway_fmt}`);
    scene_rt::fg(runtime, "vehicle-hud-flight-state-value", body_colour);
    scene_rt::fg(runtime, "vehicle-hud-world-body-value", body_colour);
    scene_rt::fg(runtime, "vehicle-hud-world-g-value", body_colour);
    scene_rt::text(runtime, "vehicle-hud-controls-escape-value", escape_label);
}
