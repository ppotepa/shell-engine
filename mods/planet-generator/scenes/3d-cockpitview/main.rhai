import "cockpit/state" as cockpit_state;
import "cockpit/view" as cockpit_view;
import "cockpit/hud" as cockpit_hud;
import "generator/render_push" as render_push;

local = cockpit_state::ensure_initialized(local, persist);
local = render_push::step(local, runtime, world, frame_ms);
local = cockpit_view::step(
    local,
    runtime,
    world,
    input,
    game,
    persist,
    (frame_ms.to_float() / 1000.0).max(0.0001),
    scene_elapsed_ms.to_float()
);
cockpit_hud::render(runtime, local);
