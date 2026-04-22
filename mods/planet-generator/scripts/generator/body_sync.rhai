import "std/planet_spec" as planet_spec;

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
    atmo_drag_max,
    planet_type
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
        km_per_px: sim_radius,
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

fn build_generated_body_from_local(local) {
    let spec = planet_spec::snapshot(local);
    build_generated_body_map(
        spec["planet_radius_real_km"] ?? 6371.0,
        spec["planet_radius_scale_divisor"] ?? 1.0,
        spec["surface_gravity_mps2"] ?? 9.81,
        spec["atmo_top_km"] ?? 80.0,
        spec["atmo_dense_start_km"] ?? 12.0,
        spec["atmo_drag_max"] ?? 2.0,
        spec["planet_type"] ?? "earth_like"
    )
}

fn generated_body_changed(previous, current) {
    (previous["center_x"] ?? current["center_x"]) != current["center_x"]
        || (previous["center_y"] ?? current["center_y"]) != current["center_y"]
        || (previous["radius_px"] ?? current["radius_px"]) != current["radius_px"]
        || (previous["km_per_px"] ?? current["km_per_px"]) != current["km_per_px"]
        || (previous["surface_radius"] ?? current["surface_radius"]) != current["surface_radius"]
        || (previous["radius_km"] ?? current["radius_km"]) != current["radius_km"]
        || (previous["gravity_mu_km3_s2"] ?? current["gravity_mu_km3_s2"]) != current["gravity_mu_km3_s2"]
        || (previous["atmosphere_top_km"] ?? current["atmosphere_top_km"]) != current["atmosphere_top_km"]
        || (previous["atmosphere_dense_start_km"] ?? current["atmosphere_dense_start_km"]) != current["atmosphere_dense_start_km"]
        || (previous["atmosphere_drag_max"] ?? current["atmosphere_drag_max"]) != current["atmosphere_drag_max"]
        || (previous["cloud_bottom_km"] ?? current["cloud_bottom_km"]) != current["cloud_bottom_km"]
        || (previous["cloud_top_km"] ?? current["cloud_top_km"]) != current["cloud_top_km"]
        || (previous["planet_type"] ?? current["planet_type"]) != current["planet_type"]
}
