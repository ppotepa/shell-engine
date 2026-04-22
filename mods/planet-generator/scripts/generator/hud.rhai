import "std/runtime_scene" as scene_ui;
import "std/planet_spec" as planet_spec;
import "generator/params" as params;
import "generator/body_sync" as body_sync;

fn body_type_label(body_type) {
    let label = body_type ?? "earth_like";
    if label == "earth_like" { "EARTH" }
    else if label == "ocean" { "OCEAN" }
    else if label == "desert" { "DESERT" }
    else if label == "ice" { "ICE" }
    else if label == "volcanic" { "VOLCANIC" }
    else if label == "barren" { "BARREN" }
    else { "CUSTOM" }
}

fn render(runtime, gui, game, vehicle, local) {
    let spec = planet_spec::snapshot(local);
    let tab = local.tab ?? 0;
    let model_mode = local.model_mode ?? 1;

    scene_ui::fg(runtime, "tab-label-continents", if tab == 0 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "tab-label-mountains", if tab == 1 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "tab-label-climate", if tab == 2 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "tab-label-visual", if tab == 3 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "tab-label-atmo", if tab == 4 { "#aaffcc" } else { "#2a9440" });

    scene_ui::fg(runtime, "model-label-planet", if model_mode == 1 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "model-label-sphere", if model_mode == 2 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "model-label-cube", if model_mode == 3 { "#aaffcc" } else { "#2a9440" });
    scene_ui::fg(runtime, "model-label-suzanne", if model_mode == 4 { "#aaffcc" } else { "#2a9440" });

    scene_ui::fg(runtime, "planet-randomize-button-label", if gui.widget_hovered("planet-randomize-button") { "#ffe060" } else { "#ffc030" });
    scene_ui::fg(runtime, "planet-reset-button-label", if gui.widget_hovered("planet-reset-button") { "#80b0ff" } else { "#4080ff" });

    let preset_hover = if gui.widget_hovered("planet-preset-dropdown") || gui.widget_open("planet-preset-dropdown") { "#8cc08c" } else { "#507050" };
    scene_ui::fg(runtime, "preset-prefix-label", preset_hover);
    scene_ui::fg(runtime, "preset-arrow-label", preset_hover);
    scene_ui::fg(runtime, "preset-current-label", if gui.widget_open("planet-preset-dropdown") { "#aaffcc" } else { "#8cc08c" });

    let num_params = params::num_params(tab);
    for i in 0..8 {
        let visible = i < num_params;
        scene_ui::visible(runtime, `param-label-${i}`, visible);
        scene_ui::visible(runtime, `param-value-${i}`, visible);
        scene_ui::visible(runtime, `param-track-${i}`, visible);
        scene_ui::visible(runtime, `param-handle-${i}`, visible);
    }

    scene_ui::text(runtime, "param-title", params::tab_title(tab));
    scene_ui::text(runtime, "param-summary", params::tab_summary(tab));
    for i in 0..num_params {
        scene_ui::text(runtime, `param-label-${i}`, params::slot_label(tab, i));
        scene_ui::text(runtime, `param-value-${i}`, params::slot_value(local, tab, i));
    }

    scene_ui::text(
        runtime,
        "stats-seed",
        `SEED:${(spec["seed"] ?? 0.0).round()} ${body_type_label(spec["planet_type"] ?? "earth_like")}`
    );
    let ps = planet_last_stats();
    let stats_radius_real = spec["planet_radius_real_km"] ?? 6371.0;
    let stats_radius_sim = body_sync::sim_radius_km(stats_radius_real, spec["planet_radius_scale_divisor"] ?? 1.0);
    let stats_surface_g = spec["surface_gravity_mps2"] ?? 9.81;
    let stats_surface_g_fmt = (stats_surface_g * 100.0).round() / 100.0;
    let stats_alt = (local.return_altitude_km ?? 0.0).max(0.0);
    let stats_spd = (local.return_speed_kms ?? 0.0).abs();
    let stats_vrad = local.return_vrad_kms ?? 0.0;
    let stats_radius_sim_fmt = (stats_radius_sim * 100.0).round() / 100.0;
    let stats_spd_fmt = (stats_spd * 100.0).round() / 100.0;
    let stats_vrad_fmt = (stats_vrad * 100.0).round() / 100.0;
    let stats_atmo_top = (spec["atmo_top_km"] ?? 80.0).max(0.0);
    scene_ui::text(runtime, "stats-radius-real", `REAL:${stats_radius_real.round()}km`);
    scene_ui::text(runtime, "stats-radius-sim", `SIM:${stats_radius_sim_fmt}wu`);
    scene_ui::text(runtime, "stats-gravity", `G:${stats_surface_g_fmt}m/s2`);
    scene_ui::text(runtime, "stats-alt", `ALT:${stats_alt.round()}km`);
    scene_ui::text(runtime, "stats-spd", `SPD:${stats_spd_fmt}km/s`);
    scene_ui::text(runtime, "stats-vrad", `VR:${stats_vrad_fmt}km/s`);
    scene_ui::text(runtime, "stats-atmo-top", `ATM:${stats_atmo_top.round()}km`);

    if ps.len() > 0 {
        let ocean_pct = (ps["ocean"] * 100.0).round();
        let forest_pct = (ps["forest"] * 100.0).round();
        let desert_pct = (ps["desert"] * 100.0).round();
        let cold_pct = (ps["cold"] * 100.0).round();
        scene_ui::text(
            runtime,
            "stats-biome-ocean",
            if spec["has_ocean"] ?? true { `OCEAN:${ocean_pct}%` } else { "OCEAN:OFF" }
        );
        scene_ui::text(runtime, "stats-biome-forest", `FOREST:${forest_pct}%`);
        scene_ui::text(runtime, "stats-biome-desert", `DESERT:${desert_pct}%`);
        scene_ui::text(runtime, "stats-biome-snow", `SNOW:${cold_pct}%`);
    } else {
        scene_ui::text(runtime, "stats-biome-ocean", "OCEAN:--%");
        scene_ui::text(runtime, "stats-biome-forest", "FOREST:--%");
        scene_ui::text(runtime, "stats-biome-desert", "DESERT:--%");
        scene_ui::text(runtime, "stats-biome-snow", "SNOW:--%");
    }
}
