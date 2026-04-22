import "generator/state" as state;
import "generator/input" as generator_input;
import "generator/gui_sync" as gui_sync;
import "generator/body_sync" as body_sync;
import "generator/hud" as hud;
import "generator/render_push" as render_push;
import "std/planet_spec" as planet_spec;

local = state::ensure_initialized(local, game, vehicle);
local = planet_spec::sync_spec_from_legacy(local);
local = state::refresh_derived(local);
local = generator_input::step(local, input, gui, game, vehicle, persist);
local = state::refresh_derived(local);
local = gui_sync::step(local, gui);
local = state::refresh_derived(local);

let body_sync_result = body_sync::sync_generated_body(local, world);
local = body_sync_result["local"] ?? local;

hud::render(runtime, gui, game, vehicle, local);
local = render_push::step(local, runtime, frame_ms);
