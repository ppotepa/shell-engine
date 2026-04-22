import "std/planet_spec" as planet_spec;

fn apply_generator_defaults(local) {
    planet_spec::apply_snapshot(local, planet_spec::defaults())
}

fn apply_generator_ui_defaults(local) {
    local.tab = 0;
    local.model_mode = 1;
    local.preset_choice = 0;
    local.rotation_paused = false;
    local.render_push_throttle_ms = 0.0;
    local
}

fn initialize_generator_state(local, game, vehicle) {
    local.initialized = true;
    local = apply_generator_defaults(local);
    local = apply_generator_ui_defaults(local);
    local.tab_sync_needed = true;
    local.push_ready = false;
    local.push_dirty = true;
    local.push_timer = 0.0;
    local
}

fn ensure_initialized(local, game, vehicle) {
    local = local ?? #{};

    if !(local.initialized ?? false) {
        local = initialize_generator_state(local, game, vehicle);
    }

    local
}
