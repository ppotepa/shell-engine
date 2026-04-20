fn normalize_vehicle_profile_name(vehicle, profile_name) {
    vehicle.normalize_profile_id(profile_name ?? vehicle.default_profile_id())
}

fn vehicle_profile_state_path() { "/mods/vehicle-playground/vehicle/profile" }
fn vehicle_assist_alt_state_path() { "/mods/vehicle-playground/vehicle/assist_alt_hold" }
fn vehicle_assist_heading_state_path() { "/mods/vehicle-playground/vehicle/assist_heading_hold" }

fn bool_to_i(v) { if v { 1 } else { 0 } }
fn i_to_bool(v) { v != 0 }

fn current_vehicle_control_state(state, vehicle) {
    let control = state.vehicle_control ?? ();
    if type_of(control) == "()" {
        return vehicle.control_state();
    }
    control.normalized()
}

fn current_vehicle_profile_name(state, vehicle) {
    current_vehicle_control_state(state, vehicle).profile_id ?? vehicle.default_profile_id()
}

fn current_vehicle_assist_state(state, vehicle) {
    current_vehicle_control_state(state, vehicle).assists
}

fn current_vehicle_target_altitude(state, vehicle) {
    let control = current_vehicle_control_state(state, vehicle);
    if control.has_target_altitude_km() {
        return control.target_altitude_km(0.0);
    }
    ()
}

fn current_vehicle_target_heading(state, vehicle) {
    let control = current_vehicle_control_state(state, vehicle);
    if control.has_target_heading_rad() {
        return control.target_heading_rad(0.0);
    }
    ()
}

fn sync_vehicle_control_state(state, control, _vehicle) {
    state.vehicle_control = control.normalized();
    state
}

fn set_vehicle_target_altitude(state, vehicle, target_altitude_km) {
    let control = current_vehicle_control_state(state, vehicle);
    state.vehicle_control =
        if control.assists.alt_hold {
            control.set_target_altitude_km(target_altitude_km)
        } else {
            control.clear_target_altitude_km()
        };
    state
}

fn set_vehicle_target_heading(state, vehicle, target_heading_rad) {
    let control = current_vehicle_control_state(state, vehicle);
    state.vehicle_control =
        if control.assists.heading_hold {
            control.set_target_heading_rad(target_heading_rad)
        } else {
            control.clear_target_heading_rad()
        };
    state
}

fn persist_vehicle_profile(state, game, vehicle) {
    let profile = current_vehicle_profile_name(state, vehicle);
    game.set(vehicle_profile_state_path(), profile);
    state
}

fn persist_vehicle_assists(state, game, vehicle) {
    let assists = current_vehicle_assist_state(state, vehicle);
    let alt_hold = assists.alt_hold;
    let heading_hold = assists.heading_hold;
    game.set(vehicle_assist_alt_state_path(), bool_to_i(alt_hold));
    game.set(vehicle_assist_heading_state_path(), bool_to_i(heading_hold));
    state
}

fn load_persisted_vehicle_state(state, game, vehicle) {
    let assists = vehicle.assist_state_with(
        i_to_bool(game.get_i(vehicle_assist_alt_state_path(), 0)),
        i_to_bool(game.get_i(vehicle_assist_heading_state_path(), 0))
    );
    state = sync_vehicle_control_state(
        state,
        vehicle.control_from_buttons(
            normalize_vehicle_profile_name(
                vehicle,
                game.get_s(vehicle_profile_state_path(), vehicle.default_profile_id())
            ),
            vehicle.button_input(),
            assists
        ),
        vehicle
    );
    state
}

fn persist_vehicle_state(state, game, vehicle) {
    state = persist_vehicle_profile(state, game, vehicle);
    state = persist_vehicle_assists(state, game, vehicle);
    state
}

fn toggle_vehicle_profile(state, game, vehicle) {
    let control = current_vehicle_control_state(state, vehicle);
    control.cycle_ship_profile();
    state = sync_vehicle_control_state(state, control, vehicle);
    state = persist_vehicle_profile(state, game, vehicle);
    state
}

fn toggle_vehicle_assist_alt_hold(state, game, vehicle, target_altitude_km) {
    let control = current_vehicle_control_state(state, vehicle);
    control.toggle_altitude_hold(target_altitude_km);
    state = sync_vehicle_control_state(state, control, vehicle);
    state = persist_vehicle_assists(state, game, vehicle);
    state
}

fn toggle_vehicle_assist_heading_hold(state, game, vehicle, target_heading_rad) {
    let control = current_vehicle_control_state(state, vehicle);
    control.toggle_heading_hold(target_heading_rad);
    state = sync_vehicle_control_state(state, control, vehicle);
    state = persist_vehicle_assists(state, game, vehicle);
    state
}

fn current_vehicle_buttons(vehicle, input) {
    vehicle.button_input_pressed(
        input.action_down("thrust"),
        input.action_down("brake"),
        input.action_down("strafe_left"),
        input.action_down("strafe_right"),
        input.action_down("lift"),
        input.action_down("turn_left"),
        input.action_down("turn_right"),
        input.action_down("boost"),
        input.action_down("boost")
    )
}

fn current_vehicle_frame_control(state, vehicle, input) {
    let control = vehicle
        .control_from_buttons(
            current_vehicle_profile_name(state, vehicle),
            current_vehicle_buttons(vehicle, input),
            current_vehicle_assist_state(state, vehicle)
        );
    let target_altitude = current_vehicle_target_altitude(state, vehicle);
    let target_heading = current_vehicle_target_heading(state, vehicle);
    let control =
        if type_of(target_altitude) != "()" {
            control.set_target_altitude_km(target_altitude)
        } else {
            control
        };
    if type_of(target_heading) != "()" {
        return control.set_target_heading_rad(target_heading);
    }
    control
}

fn current_ship_profile_tuning(state, vehicle) {
    vehicle.ship_profile_tuning(current_vehicle_profile_name(state, vehicle))
}
