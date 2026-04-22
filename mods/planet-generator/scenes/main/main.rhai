import "generator/state" as state;
import "generator/input" as generator_input;
import "generator/gui_sync" as gui_sync;
import "generator/hud" as hud;
import "generator/render_push" as render_push;

local = state::ensure_initialized(local, game, vehicle);
local = generator_input::step(local, input, gui, game, vehicle, persist);
local = gui_sync::step(local, gui);
hud::render(runtime, gui, game, vehicle, local);
local = render_push::step(local, runtime, world, frame_ms);
