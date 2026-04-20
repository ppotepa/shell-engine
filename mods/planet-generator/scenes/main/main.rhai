import "generator/state" as state;
import "generator/input" as generator_input;
import "generator/gui_sync" as gui_sync;
import "generator/body_sync" as body_sync;
import "generator/hud" as hud;
import "generator/render_push" as render_push;
import "generator/vehicle_handoff" as vehicle_handoff;

local = state::ensure_initialized(local, game, vehicle);
local = vehicle_handoff::consume_vehicle_return_packet(local, game, vehicle);
local = state::refresh_derived(local);
local = generator_input::step(local, input, gui, game, vehicle);
local = state::refresh_derived(local);
local = gui_sync::step(local, gui);
local = state::refresh_derived(local);

let body_sync_result = body_sync::sync_generated_body(local, world);
local = body_sync_result["local"] ?? local;
let launch_request = vehicle_handoff::consume_vehicle_launch_request(local);
local = launch_request["local"] ?? local;
if launch_request["requested"] ?? false {
    local = vehicle_handoff::launch_vehicle_scene(local, game, vehicle, world.body("generated-planet"));
}

hud::render(runtime, gui, game, vehicle, local);
local = render_push::step(local, runtime, frame_ms);
