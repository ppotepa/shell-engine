import "std/runtime_scene" as scene_ui;
import "std/math" as math;
import "generator/params" as params;

fn near_map_number(cache, key, value, epsilon) {
    math::near(cache[key] ?? value, value, epsilon)
}

fn push_throttle_ms(local) {
    (local.render_push_throttle_ms ?? 0.0).max(0.0)
}

fn current_world_state(local) {
    #{
        model_mode: local.model_mode ?? 1,
        seed: local.seed ?? 0.0,
        has_ocean: local.has_ocean ?? true,
        ocean: local.ocean ?? 0.55,
        cscale: local.cscale ?? 2.5,
        cwarp: local.cwarp ?? 0.65,
        coct: local.coct ?? 5.0,
        mscale: local.mscale ?? 6.0,
        mstr: local.mstr ?? 0.45,
        mroct: local.mroct ?? 5.0,
        moisture: local.moisture ?? 3.0,
        ice: local.ice ?? 1.0,
        lapse: local.lapse ?? 0.6,
        rain: local.rain ?? 0.4,
        disp: local.disp ?? 0.22,
        subdiv: local.subdiv ?? 32.0,
        coloring: local.coloring ?? 0.0
    }
}

fn world_state_changed(previous, current) {
    (previous["model_mode"] ?? current["model_mode"]) != current["model_mode"]
        || (previous["seed"] ?? current["seed"]) != current["seed"]
        || (previous["has_ocean"] ?? current["has_ocean"]) != current["has_ocean"]
        || (previous["ocean"] ?? current["ocean"]) != current["ocean"]
        || (previous["cscale"] ?? current["cscale"]) != current["cscale"]
        || (previous["cwarp"] ?? current["cwarp"]) != current["cwarp"]
        || (previous["coct"] ?? current["coct"]) != current["coct"]
        || (previous["mscale"] ?? current["mscale"]) != current["mscale"]
        || (previous["mstr"] ?? current["mstr"]) != current["mstr"]
        || (previous["mroct"] ?? current["mroct"]) != current["mroct"]
        || (previous["moisture"] ?? current["moisture"]) != current["moisture"]
        || (previous["ice"] ?? current["ice"]) != current["ice"]
        || (previous["lapse"] ?? current["lapse"]) != current["lapse"]
        || (previous["rain"] ?? current["rain"]) != current["rain"]
        || (previous["disp"] ?? current["disp"]) != current["disp"]
        || (previous["subdiv"] ?? current["subdiv"]) != current["subdiv"]
        || (previous["coloring"] ?? current["coloring"]) != current["coloring"]
}

fn current_visual_state(local) {
    let rotspeed = local.rotspeed ?? 3.0;
    #{
        effective_rotspeed: if local.rotation_paused ?? false { 0.0 } else { rotspeed },
        ambient: local.ambient ?? 0.12
    }
}

fn visual_state_changed(previous, current) {
    !near_map_number(previous, "effective_rotspeed", current["effective_rotspeed"], 0.0001)
        || !near_map_number(previous, "ambient", current["ambient"], 0.0001)
}

fn current_atmo_state(local) {
    let atmo_color = local.atmo_color ?? 2.0;
    let atmo_density = local.atmo_density ?? 0.48;
    let atmo_height = local.atmo_height ?? 0.11;
    let atmo_forward_scatter = local.atmo_forward_scatter ?? 0.78;
    let atmo_absorption = local.atmo_absorption ?? 0.12;
    let atmo_rayleigh_falloff = local.atmo_rayleigh_falloff ?? 0.22;
    let atmo_haze_amount = local.atmo_haze_amount ?? 0.22;
    let atmo_haze_falloff = local.atmo_haze_falloff ?? 0.46;

    let height_rel = (0.02 + atmo_height * 0.88).max(0.02).min(0.90);
    let density = atmo_density.max(0.0).min(1.0);
    let forward = atmo_forward_scatter.max(0.0).min(1.0);
    let limb = (0.95 + atmo_density * 1.7 + atmo_haze_amount * 1.15 + atmo_height * 0.55).max(0.2).min(4.0);
    let term_soft = (0.75 + atmo_haze_amount * 1.55 + atmo_density * 0.35).max(0.05).min(4.0);
    let absorb = atmo_absorption.max(0.0).min(1.0);
    let ray_amount = (0.08 + atmo_density * 0.72).max(0.0).min(1.0);
    let absorb_height = (0.70 + atmo_height * 0.10).max(0.50).min(0.92);
    let absorb_width = (0.10 + atmo_haze_amount * 0.18 + atmo_absorption * 0.10).max(0.06).min(0.32);

    #{
        height: height_rel,
        density: density,
        rayleigh_amount: ray_amount,
        rayleigh_color: params::atmo_color_hex(atmo_color),
        rayleigh_falloff: atmo_rayleigh_falloff,
        haze_amount: atmo_haze_amount,
        haze_color: params::atmo_haze_hex(atmo_color),
        haze_falloff: atmo_haze_falloff,
        absorption_amount: absorb,
        absorption_color: params::atmo_absorption_hex(atmo_color),
        absorption_height: absorb_height,
        absorption_width: absorb_width,
        forward_scatter: forward,
        limb_boost: limb,
        terminator_softness: term_soft,
        night_glow: 0.0
    }
}

fn atmo_state_changed(previous, current) {
    !near_map_number(previous, "height", current["height"], 0.0001)
        || !near_map_number(previous, "density", current["density"], 0.0001)
        || !near_map_number(previous, "rayleigh_amount", current["rayleigh_amount"], 0.0001)
        || (previous["rayleigh_color"] ?? current["rayleigh_color"]) != current["rayleigh_color"]
        || !near_map_number(previous, "rayleigh_falloff", current["rayleigh_falloff"], 0.0001)
        || !near_map_number(previous, "haze_amount", current["haze_amount"], 0.0001)
        || (previous["haze_color"] ?? current["haze_color"]) != current["haze_color"]
        || !near_map_number(previous, "haze_falloff", current["haze_falloff"], 0.0001)
        || !near_map_number(previous, "absorption_amount", current["absorption_amount"], 0.0001)
        || (previous["absorption_color"] ?? current["absorption_color"]) != current["absorption_color"]
        || !near_map_number(previous, "absorption_height", current["absorption_height"], 0.0001)
        || !near_map_number(previous, "absorption_width", current["absorption_width"], 0.0001)
        || !near_map_number(previous, "forward_scatter", current["forward_scatter"], 0.0001)
        || !near_map_number(previous, "limb_boost", current["limb_boost"], 0.0001)
        || !near_map_number(previous, "terminator_softness", current["terminator_softness"], 0.0001)
}

fn current_light_state(local) {
    let az_rad = (local.sunaz ?? 210.0) * 0.01745329;
    let el_rad = (local.sunel ?? 40.0) * 0.01745329;
    #{
        x: -(az_rad.cos() * el_rad.cos()),
        y: el_rad.sin(),
        z: -(az_rad.sin() * el_rad.cos())
    }
}

fn light_state_changed(previous, current) {
    !near_map_number(previous, "x", current["x"], 0.0001)
        || !near_map_number(previous, "y", current["y"], 0.0001)
        || !near_map_number(previous, "z", current["z"], 0.0001)
}

fn step(local, runtime, frame_ms) {
    let throttle_ms = push_throttle_ms(local);
    let world_state = current_world_state(local);
    let previous_world_state = local["render_world_observed"] ?? world_state;
    let world_changed = world_state_changed(previous_world_state, world_state);
    local["render_world_observed"] = world_state;

    if !(local.push_ready ?? false) {
        local.push_ready = true;
        local.push_timer = throttle_ms;
        local.push_dirty = true;
    }

    local.push_timer = (local.push_timer ?? throttle_ms) + frame_ms.to_float();
    if world_changed {
        local.push_dirty = true;
    }

    let should_push = (local.push_dirty ?? false) && (throttle_ms <= 0.0 || local.push_timer >= throttle_ms);
    if should_push {
        local.push_dirty = false;
        local.push_timer = 0.0;

        let model_mode = world_state["model_mode"] ?? 1;
        scene_ui::set(runtime, "planet-mesh", "obj.source", params::model_source(model_mode));
        scene_ui::render_surface_mode(runtime, "planet-mesh", "material");

        if model_mode == 1 {
            scene_ui::render_generator(runtime, "planet-mesh", "seed", world_state["seed"] ?? 0.0);
            scene_ui::render_generator(runtime, "planet-mesh", "has_ocean", world_state["has_ocean"] ?? true);
            scene_ui::render_generator(runtime, "planet-mesh", "ocean_fraction", world_state["ocean"] ?? 0.55);
            scene_ui::render_generator(runtime, "planet-mesh", "continent_scale", world_state["cscale"] ?? 2.5);
            scene_ui::render_generator(runtime, "planet-mesh", "continent_warp", world_state["cwarp"] ?? 0.65);
            scene_ui::render_generator(runtime, "planet-mesh", "continent_octaves", world_state["coct"] ?? 5.0);
            scene_ui::render_generator(runtime, "planet-mesh", "mountain_scale", world_state["mscale"] ?? 6.0);
            scene_ui::render_generator(runtime, "planet-mesh", "mountain_strength", world_state["mstr"] ?? 0.45);
            scene_ui::render_generator(runtime, "planet-mesh", "mountain_ridge_octaves", world_state["mroct"] ?? 5.0);
            scene_ui::render_generator(runtime, "planet-mesh", "moisture_scale", world_state["moisture"] ?? 3.0);
            scene_ui::render_generator(runtime, "planet-mesh", "ice_cap_strength", world_state["ice"] ?? 1.0);
            scene_ui::render_generator(runtime, "planet-mesh", "lapse_rate", world_state["lapse"] ?? 0.6);
            scene_ui::render_generator(runtime, "planet-mesh", "rain_shadow", world_state["rain"] ?? 0.4);
            scene_ui::render_generator(runtime, "planet-mesh", "displacement_scale", world_state["disp"] ?? 0.22);
            scene_ui::render_generator(runtime, "planet-mesh", "subdivisions", world_state["subdiv"] ?? 32.0);
            scene_ui::render_generator(runtime, "planet-mesh", "shape", "sphere");
            scene_ui::render_generator(runtime, "planet-mesh", "base", "cube");
            scene_ui::render_generator(runtime, "planet-mesh", "coloring", params::coloring_mode(world_state["coloring"] ?? 0.0));
        }
    }

    let visual_state = current_visual_state(local);
    let previous_visual_state = local["render_visual_cache"] ?? #{};
    if visual_state_changed(previous_visual_state, visual_state) {
        scene_ui::render_visual(
            runtime,
            "planet-mesh",
            visual_state["effective_rotspeed"] ?? 0.0,
            visual_state["ambient"] ?? 0.12
        );
        local["render_visual_cache"] = visual_state;
    }

    let atmo_state = current_atmo_state(local);
    let previous_atmo_state = local["render_atmo_cache"] ?? #{};
    if atmo_state_changed(previous_atmo_state, atmo_state) {
        scene_ui::render_atmosphere(runtime, "planet-mesh", "height", atmo_state["height"] ?? 0.02);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "density", atmo_state["density"] ?? 0.0);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "rayleigh_amount", atmo_state["rayleigh_amount"] ?? 0.0);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "rayleigh_color", atmo_state["rayleigh_color"] ?? "#7cc8ff");
        scene_ui::render_atmosphere(runtime, "planet-mesh", "rayleigh_falloff", atmo_state["rayleigh_falloff"] ?? 0.22);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "haze_amount", atmo_state["haze_amount"] ?? 0.22);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "haze_color", atmo_state["haze_color"] ?? "#edf7ff");
        scene_ui::render_atmosphere(runtime, "planet-mesh", "haze_falloff", atmo_state["haze_falloff"] ?? 0.46);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "absorption_amount", atmo_state["absorption_amount"] ?? 0.12);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "absorption_color", atmo_state["absorption_color"] ?? "#ffd59d");
        scene_ui::render_atmosphere(runtime, "planet-mesh", "absorption_height", atmo_state["absorption_height"] ?? 0.70);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "absorption_width", atmo_state["absorption_width"] ?? 0.10);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "forward_scatter", atmo_state["forward_scatter"] ?? 0.78);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "limb_boost", atmo_state["limb_boost"] ?? 0.95);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "terminator_softness", atmo_state["terminator_softness"] ?? 0.75);
        scene_ui::render_atmosphere(runtime, "planet-mesh", "night_glow", atmo_state["night_glow"] ?? 0.0);
        local["render_atmo_cache"] = atmo_state;
    }

    let light_state = current_light_state(local);
    let previous_light_state = local["render_light_cache"] ?? #{};
    if light_state_changed(previous_light_state, light_state) {
        scene_ui::render_light(
            runtime,
            "planet-mesh",
            light_state["x"] ?? 0.0,
            light_state["y"] ?? 0.0,
            light_state["z"] ?? 0.0
        );
        local["render_light_cache"] = light_state;
    }

    local
}
