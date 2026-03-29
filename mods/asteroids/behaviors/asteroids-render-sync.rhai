import "asteroids-shared" as h;

let ship_id = world.first_kind("ship");
if ship_id > 0 {
    let ship_ref = world.entity(ship_id);
    if ship_ref.exists() {
        let invuln_ms = ship_ref.get_i("/invuln_ms", 0);
        let visible = if invuln_ms > 0 { (time.scene_elapsed_ms / 120) % 2 == 0 } else { true };
        let heading = ship_ref.get_i("/h", 0);
        scene.set("ship", "vector.visible", visible);
        scene.set("ship", "vector.points", rotate_points([[0, -7], [4, 5], [0, 2], [-4, 5]], heading));
        scene.set("ship", "vector.fg", "#d0d0d0");
        scene.set("ship", "vector.bg", "white");
    }
}

for ast_id in world.query_kind("asteroid") {
    let ast_ref = world.entity(ast_id);
    if !ast_ref.exists() { continue; }
    let d = ast_ref.data();
    let vid = `asteroid-${ast_id}`;

    let active = if d.contains("active") { d["active"] } else { true };
    let rot_phase = if d.contains("rot_phase") { d["rot_phase"] } else { 0 };
    let flash_ms = if d.contains("flash_ms") { d["flash_ms"] } else { 0 };
    let flash_total = if d.contains("flash_total_ms") { d["flash_total_ms"] } else { 0 };
    let shape = if d.contains("shape") { d["shape"] } else { 0 };
    let size = if d.contains("size") { d["size"] } else { 1 };

    scene.set(vid, "vector.visible", active);
    scene.set(vid, "vector.points", rotate_points(asteroid_points(shape, size), rot_phase));
    if flash_ms > 0 {
        scene.set(vid, "vector.fg", h::asteroid_stroke_hex(flash_ms, flash_total));
        scene.set(vid, "vector.bg", h::flash_fill_hex(flash_ms, flash_total));
    } else {
        scene.set(vid, "vector.fg", "#aaaaaa");
        scene.set(vid, "vector.bg", "#333333");
    }

    let show_cracks = active && flash_ms > 0;
    if show_cracks {
        for i in 0..3 {
            let frag_id = h::crack_visual_id(d, i);
            if frag_id.type_of() == "string" {
                scene.set(frag_id, "vector.visible", true);
            }
        }
    } else {
        h::hide_crack_visuals(scene, d);
    }
}

for bid in world.query_kind("bullet") {
    let bref = world.entity(bid);
    if !bref.exists() { continue; }
    scene.set(`bullet-${bid}`, "vector.visible", true);
}

for sid in world.query_kind("smoke") {
    let sref = world.entity(sid);
    if !sref.exists() { continue; }
    let d = sref.data();
    let ttl = if d.contains("ttl_ms") { d["ttl_ms"] } else { 0 };
    let max_ttl = if d.contains("max_ttl_ms") { d["max_ttl_ms"] } else { 1 };
    let radius = if d.contains("radius") { d["radius"] } else { 2 };

    scene.set(`smoke-${sid}`, "vector.visible", true);
    scene.set(`smoke-${sid}`, "vector.points", h::smoke_points(radius, ttl, max_ttl));
    scene.set(`smoke-${sid}`, "vector.fg", h::smoke_colour_hex(ttl, max_ttl));
}

let score = game.get("/ast/score");
let lives = game.get("/ast/lives");
let wave = game.get("/ast/wave");
scene.set("hud-score", "text.content", `SCORE: ${score}`);
scene.set("hud-lives", "text.content", `LIVES: ${lives}`);
scene.set("hud-wave",  "text.content", `WAVE: ${wave}`);
