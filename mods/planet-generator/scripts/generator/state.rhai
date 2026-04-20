import "std/bootstrap" as bootstrap;
import "generator/presets" as presets;
import "generator/vehicle_handoff" as vehicle_handoff;
import "generator/body_sync" as body_sync;

fn refresh_derived(local) {
    local["planet_radius_sim_km"] = body_sync::sim_radius_km(
        local["planet_radius_real_km"] ?? 6371.0,
        local["planet_radius_scale_divisor"] ?? 1.0
    );
    local
}

fn apply_generator_defaults(local) {
    local.planet_radius_real_km = 6371.0;
    local.planet_radius_scale_divisor = 1.0;
    local.surface_gravity_mps2 = 9.81;
    local.atmo_top_km = 80.0;
    local.atmo_dense_start_km = 12.0;
    local.atmo_drag_max = 2.0;
    local.subdiv = 32.0;
    local
}

fn apply_generator_ui_defaults(local) {
    local.tab = 0;
    local.model_mode = 1;
    local.preset_choice = 0;
    local.rotation_paused = false;
    local.render_push_throttle_ms = 0.0;
    local["vehicle_launch_requested"] = false;
    local
}

fn initialize_generator_state(local, game, vehicle) {
    local.initialized = true;
    local = apply_generator_defaults(local);
    local = apply_generator_ui_defaults(local);
    local = vehicle_handoff::restore_persisted_vehicle_state(local, game, vehicle);
    local = presets::apply_choice(local, 0);
    local = apply_generator_ui_defaults(local);
    local = vehicle_handoff::persist_vehicle_state(local, game, vehicle);
    local
}

fn ensure_initialized(local, game, vehicle) {
    local = bootstrap::ensure_local(local);

    if !(local.initialized ?? false) {
        local = initialize_generator_state(local, game, vehicle);
    }

    local = vehicle_handoff::ensure_vehicle_session(local, vehicle);
    refresh_derived(local)
}
