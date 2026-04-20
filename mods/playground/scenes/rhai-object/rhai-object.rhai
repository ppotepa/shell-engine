let tick = if state.contains("tick") { state["tick"] } else { 0 };
let next_tick = (tick + 1) % 120;
let phase = (time.scene_elapsed_ms / 300) % 2;
scene.object("object-halo-a").set("visible", phase == 0);
scene.object("object-halo-b").set("visible", phase == 1);
scene.object("object-spark").set("visible", (time.scene_elapsed_ms / 120) % 2 == 0);
scene.object("object-shadow-a").set("visible", phase == 0);
scene.object("object-shadow-b").set("visible", phase == 1);

let core = scene.object("object-core");
let bob = if ((next_tick / 15) % 2) == 0 { 0 } else { 1 };
core.set("position.y", bob);
let ry = core.get("region.y");
if ry.type_of() == "i64" {
    scene.object("object-status").set("text.content", `CORE y=${ry} tick=${next_tick}`);
}

#{ state: #{ tick: next_tick } }
