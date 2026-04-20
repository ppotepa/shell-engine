import "vehicle/state" as state;
import "vehicle/control" as control;
import "vehicle/environment" as environment;
import "vehicle/handoff" as handoff;
import "vehicle/flight" as flight;
import "vehicle/hud" as hud;

local = state::ensure_root(local);
let vehicle_state = state::load(local, vehicle);

if !state::initialized(vehicle_state) {
    vehicle_state = state::default_state(vehicle);
    scene.set_bg("#000010");
    input.load_profile("default");
    vehicle_state = control::load_persisted_vehicle_state(vehicle_state, game, vehicle);
    vehicle_state = handoff::consume_handoff_packet(vehicle_state, game, vehicle);
    vehicle_state = control::persist_vehicle_state(vehicle_state, game, vehicle);
    environment::apply_planet_state_to_scene(vehicle_state, runtime);
}

let body_ctx = environment::sync_body_context(vehicle_state, world);
let frame = flight::step(
    vehicle_state,
    runtime,
    world,
    input,
    vehicle,
    body_ctx,
    (frame_ms.to_float() / 1000.0).max(0.0001)
);
vehicle_state = frame["state"] ?? vehicle_state;
let ship_id = frame["ship_id"] ?? (vehicle_state.ship_id ?? 0);
let ship_x = frame["ship_x"] ?? 0.0;
let ship_y = frame["ship_y"] ?? 0.0;

environment::keep_planet_visible(vehicle_state, runtime);
hud::render(vehicle_state, runtime, vehicle, body_ctx);

if input.just_pressed("F9") || input.just_pressed("c") || input.just_pressed("C") {
    vehicle_state = control::toggle_vehicle_profile(vehicle_state, game, vehicle);
}
if input.just_pressed("h") {
    vehicle_state = control::toggle_vehicle_assist_alt_hold(
        vehicle_state,
        game,
        vehicle,
        if vehicle_state.vehicle_grounded ?? false { 0.0 } else { vehicle_state.vehicle_altitude_km ?? 0.0 }
    );
}
if input.just_pressed("j") {
    vehicle_state = control::toggle_vehicle_assist_heading_hold(
        vehicle_state,
        game,
        vehicle,
        vehicle_state.ship_heading ?? vehicle_state.vehicle_heading ?? 0.0
    );
}
if input.just_pressed("Escape") {
    let return_scene_id = handoff::resolve_return_scene_id(vehicle_state);
    let return_mod_ref = handoff::resolve_return_mod_ref(vehicle_state);
    if return_scene_id != "" && return_mod_ref != "" && (vehicle_state.handoff_source ?? "defaults") == "packet" {
        let ship_physics = world.physics(ship_id) ?? #{};
        vehicle_state = handoff::save_return_vehicle_handoff_packet(
            vehicle_state,
            game,
            vehicle,
            body_ctx["body_snapshot"],
            ship_x,
            ship_y,
            ship_physics["vx"] ?? 0.0,
            ship_physics["vy"] ?? 0.0
        );
        vehicle_state = control::persist_vehicle_state(vehicle_state, game, vehicle);
        local = state::store(local, vehicle_state);
        game.jump_mod(return_mod_ref, return_scene_id);
    }
}

local = state::store(local, vehicle_state);
