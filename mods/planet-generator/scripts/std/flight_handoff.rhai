import "std/planet_spec" as planet_spec;

fn flight_launch_path() {
    "/planet_generator/flight_launch"
}

fn build_launch_snapshot(local) {
    planet_spec::snapshot(local)
}

fn store_launch_snapshot(persist, local) {
    persist.set(flight_launch_path(), build_launch_snapshot(local))
}

fn consume_launch_snapshot(persist) {
    let snapshot = persist.get(flight_launch_path());
    if snapshot != () {
        persist.remove(flight_launch_path());
    }
    snapshot
}

fn apply_launch_snapshot(local, snapshot) {
    local = planet_spec::apply_snapshot(local, snapshot);

    local.model_mode = 1;
    local.rotation_paused = true;
    local.push_ready = false;
    local.push_dirty = true;
    local.push_timer = 0.0;
    local.ship_seeded = false;
    local["render_world_observed"] = ();
    local["render_visual_cache"] = ();
    local["render_atmo_cache"] = ();
    local["render_light_cache"] = ();
    local
}
