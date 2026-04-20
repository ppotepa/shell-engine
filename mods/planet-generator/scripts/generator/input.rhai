import "generator/presets" as presets;
import "generator/vehicle_handoff" as vehicle_handoff;

fn step(local, input, gui, game, vehicle) {
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

    if input.just_pressed("F10") || gui.button_clicked("planet-vehicle-button") {
        local = vehicle_handoff::request_vehicle_launch(local);
    }

    if input.just_pressed("F9") || input.just_pressed("c") || input.just_pressed("C") {
        local = vehicle_handoff::toggle_vehicle_profile_state(local, game, vehicle);
    }
    if input.just_pressed("h") {
        local = vehicle_handoff::toggle_vehicle_assist_alt_hold(local, game, vehicle);
    }
    if input.just_pressed("j") {
        local = vehicle_handoff::toggle_vehicle_assist_heading_hold(local, game, vehicle);
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

    local
}
