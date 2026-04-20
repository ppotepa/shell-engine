import "std/math3" as math3;
import "std/runtime_scene" as scene_rt;

fn atmo_color_hex(v) { if v < 0.5 { "#d9b36a" } else if v < 1.5 { "#cf7a8c" } else if v < 2.5 { "#7cc8ff" } else if v < 3.5 { "#9bcf86" } else if v < 4.5 { "#9ab8ff" } else { "#b79dff" } }
fn atmo_haze_hex(v) { if v < 0.5 { "#f6eddc" } else if v < 1.5 { "#f6ebef" } else if v < 2.5 { "#edf7ff" } else if v < 3.5 { "#eef7ea" } else if v < 4.5 { "#eef4ff" } else { "#f2eeff" } }
fn atmo_absorption_hex(v) { if v < 0.5 { "#ffbf7b" } else if v < 1.5 { "#ffc0ad" } else if v < 2.5 { "#ffd59d" } else if v < 3.5 { "#ffd08f" } else if v < 4.5 { "#ffd9b4" } else { "#ffd0dc" } }

fn snapshot_body_map(state, body_snapshot) {
    if body_snapshot.exists {
        return body_snapshot.inspect();
    }
    build_generated_body_map(
        state.planet_radius_real_km ?? 6371.0,
        state.planet_radius_scale_divisor ?? 1.0,
        state.surface_gravity_mps2 ?? 9.81,
        state.atmo_top_km ?? 80.0,
        state.atmo_dense_start_km ?? 12.0,
        state.atmo_drag_max ?? 2.0,
        state.planet_type ?? "earth_like"
    )
}

fn build_vehicle_environment_map(state, body_snapshot) {
    #{
        body: snapshot_body_map(state, body_snapshot),
        real_radius_km: state.planet_radius_real_km ?? 6371.0,
        scale_divisor: state.planet_radius_scale_divisor ?? 1.0,
        surface_gravity_mps2: state.surface_gravity_mps2 ?? 9.81,
        atmo_top_km: state.atmo_top_km ?? 80.0,
        atmo_dense_start_km: state.atmo_dense_start_km ?? 12.0,
        atmo_drag_max: state.atmo_drag_max ?? 2.0,
        seed: state.seed ?? 0.0,
        ocean: state.ocean ?? 0.55,
        cscale: state.cscale ?? 2.5,
        cwarp: state.cwarp ?? 0.65,
        coct: state.coct ?? 5.0,
        mscale: state.mscale ?? 6.0,
        mstr: state.mstr ?? 0.45,
        mroct: state.mroct ?? 5.0,
        moisture: state.moisture ?? 3.0,
        ice: state.ice ?? 1.0,
        lapse: state.lapse ?? 0.6,
        rain: state.rain ?? 0.4,
        disp: state.disp ?? 0.22,
        subdiv: state.subdiv ?? 32.0,
        coloring: state.coloring ?? 0.0,
        rotspeed: state.rotspeed ?? 3.0,
        sunaz: state.sunaz ?? 210.0,
        sunel: state.sunel ?? 40.0,
        ambient: state.ambient ?? 0.12,
        atmo_color: state.atmo_color ?? 2.0,
        atmo_density: state.atmo_density ?? 0.48,
        atmo_height: state.atmo_height ?? 0.11,
        atmo_forward_scatter: state.atmo_forward_scatter ?? 0.78,
        atmo_absorption: state.atmo_absorption ?? 0.12,
        atmo_rayleigh_falloff: state.atmo_rayleigh_falloff ?? 0.22,
        atmo_haze_amount: state.atmo_haze_amount ?? 0.22,
        atmo_haze_falloff: state.atmo_haze_falloff ?? 0.46
    }
}

fn build_vehicle_runtime_environment_map(state, body_snapshot) {
    let snapshot_exists = body_snapshot.exists;
    let body = snapshot_body_map(state, body_snapshot);
    #{
        body_id: if snapshot_exists { body_snapshot.id } else { body["id"] ?? "generated-planet" },
        body_kind: if snapshot_exists { body_snapshot.planet_type } else { body["planet_type"] ?? state.planet_type ?? "earth_like" },
        body_center_x: if snapshot_exists { body_snapshot.center_x } else { body["center_x"] ?? 0.0 },
        body_center_y: if snapshot_exists { body_snapshot.center_y } else { body["center_y"] ?? 0.0 },
        render_radius_wu:
            if snapshot_exists {
                body_snapshot.radius_px
            } else {
                body["radius_px"] ?? body["surface_radius"] ?? 0.0
            },
        surface_radius_wu: if snapshot_exists { body_snapshot.surface_radius } else { body["surface_radius"] ?? 0.0 },
        real_radius_km: if snapshot_exists { body_snapshot.radius_km } else { body["radius_km"] ?? state.planet_radius_real_km ?? 6371.0 },
        scale_divisor: state.planet_radius_scale_divisor ?? 1.0,
        gravity_mu_km3_s2: if snapshot_exists { body_snapshot.gravity_mu_km3_s2 } else { body["gravity_mu_km3_s2"] ?? 0.0 },
        surface_gravity_mps2: state.surface_gravity_mps2 ?? 9.81,
        atmosphere_top_km:
            if snapshot_exists {
                body_snapshot.atmosphere_top_km
            } else {
                body["atmosphere_top_km"] ?? state.atmo_top_km ?? 0.0
            },
        atmosphere_dense_start_km:
            if snapshot_exists {
                body_snapshot.atmosphere_dense_start_km
            } else {
                body["atmosphere_dense_start_km"] ?? state.atmo_dense_start_km ?? 0.0
            },
        atmosphere_drag_max:
            if snapshot_exists {
                body_snapshot.atmosphere_drag_max
            } else {
                body["atmosphere_drag_max"] ?? state.atmo_drag_max ?? 0.0
            },
        cloud_bottom_km: if snapshot_exists { body_snapshot.cloud_bottom_km } else { body["cloud_bottom_km"] ?? 0.0 },
        cloud_top_km: if snapshot_exists { body_snapshot.cloud_top_km } else { body["cloud_top_km"] ?? 0.0 }
    }
}

fn build_vehicle_runtime_environment_binding(state, body_snapshot, vehicle) {
    vehicle.environment_binding_from(
        build_vehicle_runtime_environment_map(state, body_snapshot)
    )
}

fn build_generated_body_map(
    real_radius_km,
    scale_divisor,
    surface_gravity_mps2,
    atmo_top_km,
    atmo_dense_start_km,
    atmo_drag_max,
    planet_type
) {
    let sim_radius = math3::sim_radius_km(real_radius_km, scale_divisor);
    let mu_km3 = math3::gravity_mu_km3_s2(surface_gravity_mps2, sim_radius);
    #{
        id: "generated-planet",
        center_x: 0.0,
        center_y: 0.0,
        radius_px: sim_radius,
        surface_radius: sim_radius,
        radius_km: sim_radius,
        gravity_mu_km3_s2: mu_km3,
        atmosphere_top_km: atmo_top_km.max(0.0),
        atmosphere_dense_start_km: atmo_dense_start_km.max(0.0),
        atmosphere_drag_max: atmo_drag_max.max(0.0),
        cloud_bottom_km: 0.0,
        cloud_top_km: (atmo_top_km.max(0.0) * 0.35),
        planet_type: planet_type ?? "earth_like"
    }
}

fn sync_generated_body(state, world) {
    let body = build_generated_body_map(
        state.planet_radius_real_km ?? 6371.0,
        state.planet_radius_scale_divisor ?? 1.0,
        state.surface_gravity_mps2 ?? 9.81,
        state.atmo_top_km ?? 80.0,
        state.atmo_dense_start_km ?? 12.0,
        state.atmo_drag_max ?? 2.0,
        state.planet_type ?? "earth_like"
    );

    let patched = world.body_patch("generated-planet", body);
    let body_snapshot = world.body("generated-planet");
    #{
        body_snapshot: body_snapshot,
        patched: patched
    }
}

fn ensure_ship(world, spawn_x, spawn_y, heading) {
    let existing = world.first_kind("ship");
    if existing > 0 { return existing; }
    world.spawn_prefab("ship", #{
        x: spawn_x,
        y: spawn_y,
        heading: heading,
        vx: 0.0,
        vy: 0.0
    })
}

fn body_km_per_wu(body_snapshot, fallback_real_radius_km, fallback_surface_radius) {
    let radius_km = if body_snapshot.exists { body_snapshot.radius_km } else { fallback_real_radius_km };
    let surface_radius = if body_snapshot.exists { body_snapshot.surface_radius } else { fallback_surface_radius };
    let km_per_px =
        if body_snapshot.exists && body_snapshot.km_per_px > 0.0 {
            body_snapshot.km_per_px
        } else {
            radius_km / surface_radius.max(0.001)
        };
    km_per_px.max(0.0001)
}

fn sync_body_context(state, world) {
    let body_sync = sync_generated_body(state, world);
    let body_snapshot = body_sync["body_snapshot"];
    let body = snapshot_body_map(state, body_snapshot);
    let surface_radius =
        if body_snapshot.exists {
            body_snapshot.surface_radius.max(0.1)
        } else {
            (body["surface_radius"] ?? 1.0).max(0.1)
        };
    let real_radius_km =
        if body_snapshot.exists {
            body_snapshot.radius_km.max(0.1)
        } else {
            (body["radius_km"] ?? state.planet_radius_real_km ?? 6371.0).max(0.1)
        };
    let km_per_wu = body_km_per_wu(body_snapshot, real_radius_km, surface_radius);
    #{
        body_snapshot: body_snapshot,
        body_patch_ok: body_sync["patched"] ?? false,
        cx: if body_snapshot.exists { body_snapshot.center_x } else { body["center_x"] ?? 0.0 },
        cy: if body_snapshot.exists { body_snapshot.center_y } else { body["center_y"] ?? 0.0 },
        surface_radius: surface_radius,
        real_radius_km: real_radius_km,
        km_per_wu: km_per_wu
    }
}

fn apply_planet_state_to_scene(state, runtime) {
    let coloring = state.coloring ?? 0.0;
    let coloring_str = if coloring < 0.5 { "biome" } else if coloring < 1.5 { "elevation" } else { "moisture" };
    scene_rt::set(runtime, "vehicle-planet-view", "obj.source", "world://32");
    scene_rt::render_generator(runtime, "vehicle-planet-view", "seed", state.seed ?? 0.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "ocean_fraction", state.ocean ?? 0.55);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "continent_scale", state.cscale ?? 2.5);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "continent_warp", state.cwarp ?? 0.65);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "continent_octaves", state.coct ?? 5.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "mountain_scale", state.mscale ?? 6.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "mountain_strength", state.mstr ?? 0.45);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "mountain_ridge_octaves", state.mroct ?? 5.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "moisture_scale", state.moisture ?? 3.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "ice_cap_strength", state.ice ?? 1.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "lapse_rate", state.lapse ?? 0.6);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "rain_shadow", state.rain ?? 0.4);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "displacement_scale", state.disp ?? 0.22);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "subdivisions", state.subdiv ?? 32.0);
    scene_rt::render_generator(runtime, "vehicle-planet-view", "shape", "sphere");
    scene_rt::render_generator(runtime, "vehicle-planet-view", "base", "cube");
    scene_rt::render_generator(runtime, "vehicle-planet-view", "coloring", coloring_str);
    scene_rt::render_surface_mode(runtime, "vehicle-planet-view", "material");
    scene_rt::set(runtime, "vehicle-planet-view", "obj.rotation-speed", state.rotspeed ?? 3.0);
    scene_rt::set(runtime, "vehicle-planet-view", "obj.ambient", state.ambient ?? 0.12);

    let atmo_height = state.atmo_height ?? 0.11;
    let atmo_density = state.atmo_density ?? 0.48;
    let atmo_forward_scatter = state.atmo_forward_scatter ?? 0.78;
    let atmo_absorption = state.atmo_absorption ?? 0.12;
    let atmo_rayleigh_falloff = state.atmo_rayleigh_falloff ?? 0.22;
    let atmo_haze_amount = state.atmo_haze_amount ?? 0.22;
    let atmo_haze_falloff = state.atmo_haze_falloff ?? 0.46;
    let atmo_color = state.atmo_color ?? 2.0;

    let atmo_height_rel = (0.02 + atmo_height * 0.88).max(0.02).min(0.90);
    let atmo_density_profile = atmo_density.max(0.0).min(1.0);
    let atmo_forward = atmo_forward_scatter.max(0.0).min(1.0);
    let atmo_limb = (0.95 + atmo_density * 1.7 + atmo_haze_amount * 1.15 + atmo_height * 0.55).max(0.2).min(4.0);
    let atmo_term_soft = (0.75 + atmo_haze_amount * 1.55 + atmo_density * 0.35).max(0.05).min(4.0);
    let atmo_absorb = atmo_absorption.max(0.0).min(1.0);
    let atmo_ray_amount = (0.08 + atmo_density * 0.72).max(0.0).min(1.0);
    let atmo_absorb_height = (0.70 + atmo_height * 0.10).max(0.50).min(0.92);
    let atmo_absorb_width = (0.10 + atmo_haze_amount * 0.18 + atmo_absorb * 0.10).max(0.06).min(0.32);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "height", atmo_height_rel);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "density", atmo_density_profile);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "rayleigh_amount", atmo_ray_amount);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "rayleigh_color", atmo_color_hex(atmo_color));
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "rayleigh_falloff", atmo_rayleigh_falloff);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "haze_amount", atmo_haze_amount);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "haze_color", atmo_haze_hex(atmo_color));
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "haze_falloff", atmo_haze_falloff);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "absorption_amount", atmo_absorb);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "absorption_color", atmo_absorption_hex(atmo_color));
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "absorption_height", atmo_absorb_height);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "absorption_width", atmo_absorb_width);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "forward_scatter", atmo_forward);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "limb_boost", atmo_limb);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "terminator_softness", atmo_term_soft);
    scene_rt::render_atmosphere(runtime, "vehicle-planet-view", "night_glow", 0.0);

    let az_rad = math3::deg_to_rad(state.sunaz ?? 210.0);
    let el_rad = math3::deg_to_rad(state.sunel ?? 40.0);
    scene_rt::set(runtime, "vehicle-planet-view", "obj.light.x", -(az_rad.cos() * el_rad.cos()));
    scene_rt::set(runtime, "vehicle-planet-view", "obj.light.y", el_rad.sin());
    scene_rt::set(runtime, "vehicle-planet-view", "obj.light.z", -(az_rad.sin() * el_rad.cos()));
}

fn keep_planet_visible(state, runtime) {
    scene_rt::set(runtime, "vehicle-planet-view", "obj.rotation-speed", state.rotspeed ?? 3.0);
    scene_rt::visible(runtime, "vehicle-planet-view", true);
}
