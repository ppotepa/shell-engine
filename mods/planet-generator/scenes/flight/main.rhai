import "flight/state" as flight_state;
import "flight/flight" as flight;
import "flight/hud" as hud;
import "generator/render_push" as render_push;

local = flight_state::ensure_initialized(local, persist);
local = render_push::step(local, runtime, world, frame_ms);
local = flight::step(local, runtime, world, input, (frame_ms.to_float() / 1000.0).max(0.0001));
hud::render(runtime, local);
