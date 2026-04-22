import "flight/state" as flight_state;
import "flight/flight" as flight;
import "flight/hud" as hud;
import "generator/body_sync" as body_sync;
import "generator/render_push" as render_push;
import "std/planet_spec" as planet_spec;

local = flight_state::ensure_initialized(local, persist);
local = planet_spec::sync_spec_from_legacy(local);

let body_sync_result = body_sync::sync_generated_body(local, world);
local = body_sync_result["local"] ?? local;

local = render_push::step(local, runtime, frame_ms);
local = flight::step(local, runtime, world, input, (frame_ms.to_float() / 1000.0).max(0.0001));
hud::render(runtime, local);
