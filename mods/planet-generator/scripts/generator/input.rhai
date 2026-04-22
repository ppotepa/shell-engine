import "generator/presets" as presets;
import "std/flight_handoff" as flight_handoff;
import "std/planet_spec" as planet_spec;

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

fn seed_code_text(local) {
    let spec = local["planet_spec"] ?? planet_spec::snapshot(local);
    `SEED:${(spec["seed"] ?? 0.0).round()} ${body_type_label(spec["planet_type"] ?? "earth_like")}`
}

fn step(local, input, gui, game, vehicle, persist) {
    let tab = local.tab ?? 0;
    if input.just_pressed("q") {
        local.tab = 0;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 0.0);
    }
    if input.just_pressed("w") {
        local.tab = 1;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 1.0);
    }
    if input.just_pressed("e") {
        local.tab = 2;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 2.0);
    }
    if input.just_pressed("y") {
        local.tab = 3;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 3.0);
    }
    if input.just_pressed("u") {
        local.tab = 4;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 4.0);
    }

    let gui_tab = gui.selected_index("planet-parameter-tabs");
    if gui_tab >= 0 && gui_tab != tab {
        local.tab = gui_tab;
        local.tab_sync_needed = true;
    }

    let model_mode = local.model_mode ?? 1;
    let model_changed = false;

    let gui_model = gui.selected_index("planet-model-selector");
    if gui_model >= 0 {
        let next_model = gui_model + 1;
        if next_model != model_mode {
            local.model_mode = next_model;
            model_changed = true;
        }
    }

    if model_changed {
        local.push_dirty = true;
        local.push_timer = local.render_push_throttle_ms ?? 0.0;
    }

    if input.just_pressed("r") {
        local.rotation_paused = !(local.rotation_paused ?? false);
    }

    let preset_choice = local.preset_choice ?? 0;
    if input.just_pressed("1") || input.just_pressed("F1") {
        local = presets::apply_choice(local, 0);
        gui.set_widget_value("planet-preset-dropdown", 0.0);
    }
    if input.just_pressed("2") || input.just_pressed("F2") {
        local = presets::apply_choice(local, 1);
        gui.set_widget_value("planet-preset-dropdown", 1.0);
    }
    if input.just_pressed("3") || input.just_pressed("F3") {
        local = presets::apply_choice(local, 2);
        gui.set_widget_value("planet-preset-dropdown", 2.0);
    }
    if input.just_pressed("4") || input.just_pressed("F4") {
        local = presets::apply_choice(local, 3);
        gui.set_widget_value("planet-preset-dropdown", 3.0);
    }
    if input.just_pressed("5") || input.just_pressed("F5") {
        local = presets::apply_choice(local, 4);
        gui.set_widget_value("planet-preset-dropdown", 4.0);
    }
    if input.just_pressed("6") || input.just_pressed("F6") {
        local = presets::apply_choice(local, 5);
        gui.set_widget_value("planet-preset-dropdown", 5.0);
    }
    if input.just_pressed("7") || input.just_pressed("F7") {
        local = presets::apply_choice(local, 6);
        gui.set_widget_value("planet-preset-dropdown", 6.0);
    }

    let gui_preset = gui.selected_index("planet-preset-dropdown");
    if gui_preset >= 0 && gui_preset != preset_choice {
        local = presets::apply_choice(local, gui_preset);
    }

    if gui.button_clicked("planet-randomize-button") {
        local = presets::randomize(local);
    }

    if input.just_pressed("Delete") || gui.button_clicked("planet-reset-button") {
        local = presets::reset_to_earth(local);
        local.tab = 0;
        local.tab_sync_needed = true;
        gui.set_widget_value("planet-parameter-tabs", 0.0);
    }

    if local.planet_spec_dirty ?? false {
        local = planet_spec::sync_spec_from_legacy(local);
        local.planet_spec_dirty = false;
    }

    if input.just_pressed("c") {
        if ui.copy_to_clipboard(seed_code_text(local)) {
            ui.flash_message("SEED HAS BEEN COPIED", 1600);
        }
    }

    if input.just_pressed("F10") {
        flight_handoff::store_launch_snapshot(persist, local);
        game.jump("/scenes/flight/scene.yml");
    }
    if input.just_pressed("F11") {
        flight_handoff::store_launch_snapshot(persist, local);
        game.jump("/scenes/3d-cockpitview/scene.yml");
    }

    local
}
