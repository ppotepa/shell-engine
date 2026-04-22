import "cockpit/state" as cockpit_state;
import "cockpit/view" as cockpit_view;
import "cockpit/hud" as cockpit_hud;
import "generator/body_sync" as body_sync;
import "generator/render_push" as render_push;
import "std/planet_spec" as planet_spec;

local = cockpit_state::ensure_initialized(local, persist);
local = planet_spec::sync_spec_from_legacy(local);

let body_sync_result = body_sync::sync_generated_body(local, world);
local = body_sync_result["local"] ?? local;

local = render_push::step(local, runtime, frame_ms);
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
