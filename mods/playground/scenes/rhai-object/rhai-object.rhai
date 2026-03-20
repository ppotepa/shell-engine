let tick = if state.contains("tick") { state["tick"] } else { 0 };
let next_tick = (tick + 1) % 120;
let phase = (time.scene_elapsed_ms / 300) % 2;
scene.set("object-halo-a", "visible", phase == 0);
scene.set("object-halo-b", "visible", phase == 1);
scene.set("object-spark", "visible", (time.scene_elapsed_ms / 120) % 2 == 0);
scene.set("object-shadow-a", "visible", phase == 0);
scene.set("object-shadow-b", "visible", phase == 1);

let core = scene.get("object-core");
let bob = if ((next_tick / 15) % 2) == 0 { 0 } else { 1 };
core.set("position.y", bob);
let ry = core.get("region.y");
if ry.type_of() == "i64" {
    scene.set("object-status", "text.content", `CORE y=${ry} tick=${next_tick}`);
}

#{ state: #{ tick: next_tick } }
