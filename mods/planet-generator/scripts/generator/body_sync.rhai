fn sim_radius_km(real_radius_km, divisor) {
    let d = if divisor <= 0.0001 { 1.0 } else { divisor };
    real_radius_km / d
}

fn gravity_mu_km3_s2(surface_g_mps2, radius_km) {
    let r = radius_km.max(0.001);
    (surface_g_mps2.max(0.0) / 1000.0) * r * r
}

fn build_generated_body_map(
    real_radius_km,
    scale_divisor,
    surface_gravity_mps2,
    atmo_top_km,
    atmo_dense_start_km,
    atmo_drag_max
) {
    let sim_radius = sim_radius_km(real_radius_km, scale_divisor);
    let mu_km3 = gravity_mu_km3_s2(surface_gravity_mps2, sim_radius);
    #{
        id: "generated-planet",
        center_x: 0.0,
        center_y: 0.0,
        // Keep the generator preview on a unit shell while gameplay/physics
        // continue to use the scaled simulation radius in world units.
        radius_px: 1.0,
        km_per_px: 1.0,
        surface_radius: sim_radius,
        radius_km: sim_radius,
        gravity_mu_km3_s2: mu_km3,
        atmosphere_top_km: atmo_top_km.max(0.0),
        atmosphere_dense_start_km: atmo_dense_start_km.max(0.0),
        atmosphere_drag_max: atmo_drag_max.max(0.0),
        cloud_bottom_km: 0.0,
        cloud_top_km: (atmo_top_km.max(0.0) * 0.35)
    }
}

fn build_generated_body_from_local(local) {
    build_generated_body_map(
        local["planet_radius_real_km"] ?? 6371.0,
        local["planet_radius_scale_divisor"] ?? 1.0,
        local["surface_gravity_mps2"] ?? 9.81,
        local["atmo_top_km"] ?? 80.0,
        local["atmo_dense_start_km"] ?? 12.0,
        local["atmo_drag_max"] ?? 2.0
    )
}

fn sync_generated_body(local, world) {
    local["planet_radius_sim_km"] = sim_radius_km(
        local["planet_radius_real_km"] ?? 6371.0,
        local["planet_radius_scale_divisor"] ?? 1.0
    );

    let body = build_generated_body_from_local(local);
    let patched = world.body_patch("generated-planet", body);
    let body_snapshot = world.body("generated-planet");

    #{
        local: local,
        body: body,
        body_snapshot: body_snapshot,
        patched: patched
    }
}
