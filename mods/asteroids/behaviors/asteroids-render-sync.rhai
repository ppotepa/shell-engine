import "asteroids-shared" as h;

let scale = 1024;
let ship_id = world.first_kind("ship");
let session_id = world.first_kind("session");
if ship_id <= 0 || session_id <= 0 {
    return ();
}

let session_ref = world.entity(session_id);
if !session_ref.exists() || !world.exists(ship_id) {
    return ();
}

let ship_h = session_ref.get_i("/ship_h", 0);
let ship_x_fp = session_ref.get_i("/ship_x_fp", 0);
let ship_y_fp = session_ref.get_i("/ship_y_fp", 0);
let ship_invuln_ms = session_ref.get_i("/ship_invuln_ms", 0);
let ship_respawn_ms = session_ref.get_i("/ship_respawn_ms", 0);
let score = session_ref.get_i("/score", 0);
let lives = session_ref.get_i("/lives", 0);
let wave = session_ref.get_i("/wave", 1);
let hit_flash_ms = session_ref.get_i("/hit_flash_ms", 150);

let ship_visible = if ship_respawn_ms > 0 {
    false
} else if ship_invuln_ms > 0 {
    ((time.scene_elapsed_ms / 120) % 2) == 0
} else {
    true
};
scene.set("ship", "visible", ship_visible);
scene.set("ship", "position.x", ship_x_fp / scale);
scene.set("ship", "position.y", ship_y_fp / scale);
scene.set("ship", "vector.points", ship_points(ship_h));

let asteroid_ids = world.query_kind("asteroid");
for ast_id in asteroid_ids {
    let ast = world.get(ast_id, "/");
    if ast.type_of() != "map" || ast.contains("visual_id") == false {
        continue;
    }
    let asteroid_id = ast["visual_id"];
    if asteroid_id.type_of() != "string" {
        continue;
    }
    if ast.contains("x") == false || ast.contains("y") == false {
        scene.set(asteroid_id, "visible", false);
        h::hide_crack_visuals(scene, ast);
        continue;
    }
    let flash_ms = if ast.contains("flash_ms") { ast["flash_ms"] } else { 0 };
    let flash_total_ms =
        if ast.contains("flash_total_ms") && ast["flash_total_ms"] > 0 {
            ast["flash_total_ms"]
        } else {
            hit_flash_ms
        };
    let rot_phase = if ast.contains("rot_phase") { ast["rot_phase"] } else { 0 };
    let split_visible = ast["active"] && flash_ms > 0;
    scene.set(asteroid_id, "visible", ast["active"]);
    scene.set(asteroid_id, "position.x", ast["x"]);
    scene.set(asteroid_id, "position.y", ast["y"]);
    scene.set(
        asteroid_id,
        "vector.points",
        rotate_points(asteroid_points(ast["shape"], ast["size"]), rot_phase)
    );
    scene.set(asteroid_id, "vector.fg", h::asteroid_stroke_hex(flash_ms, flash_total_ms));
    scene.set(asteroid_id, "vector.bg", "black");
    if split_visible {
        for frag_idx in 0..3 {
            let frag_id = h::crack_visual_id(ast, frag_idx);
            if frag_id.type_of() != "string" {
                continue;
            }
            scene.set(frag_id, "visible", true);
            scene.set(frag_id, "position.x", ast["x"]);
            scene.set(frag_id, "position.y", ast["y"]);
            scene.set(
                frag_id,
                "vector.points",
                rotate_points(
                    asteroid_fragment_points(ast["shape"], ast["size"], frag_idx),
                    rot_phase
                )
            );
            scene.set(frag_id, "vector.closed", false);
            scene.set(frag_id, "vector.fg", "#8f8f8f");
            scene.set(frag_id, "vector.bg", "black");
        }
    } else {
        h::hide_crack_visuals(scene, ast);
    }
}

let bullet_ids = world.query_kind("bullet");
for bullet_id in bullet_ids {
    let bref = world.entity(bullet_id);
    if !bref.exists() { continue; }
    let sprite_id = bref.get("/visual_id");
    if sprite_id.type_of() != "string" { continue; }
    let x_fp = bref.get_i("/x_fp", -99999);
    let y_fp = bref.get_i("/y_fp", -99999);
    if x_fp == -99999 || y_fp == -99999 {
        scene.set(sprite_id, "visible", false);
        continue;
    }
    scene.set(sprite_id, "visible", bref.get_b("/active", false));
    scene.set(sprite_id, "position.x", x_fp / scale);
    scene.set(sprite_id, "position.y", y_fp / scale);
}

let smoke_ids = world.query_kind("smoke");
for smoke_id in smoke_ids {
    let sref = world.entity(smoke_id);
    if !sref.exists() { continue; }
    let sprite_id = sref.get("/visual_id");
    if sprite_id.type_of() != "string" { continue; }
    let x_fp = sref.get_i("/x_fp", -99999);
    let y_fp = sref.get_i("/y_fp", -99999);
    if x_fp == -99999 || y_fp == -99999 {
        scene.set(sprite_id, "visible", false);
        continue;
    }
    let ttl_ms = sref.get_i("/ttl_ms", 0);
    let max_ttl_ms = sref.get_i("/max_ttl_ms", 1);
    let radius = sref.get_i("/radius", 2);
    scene.set(sprite_id, "visible", sref.get_b("/active", false));
    scene.set(sprite_id, "position.x", x_fp / scale);
    scene.set(sprite_id, "position.y", y_fp / scale);
    scene.set(sprite_id, "vector.points", h::smoke_points(radius, ttl_ms, max_ttl_ms));
    scene.set(sprite_id, "vector.fg", h::smoke_colour_hex(ttl_ms, max_ttl_ms));
    scene.set(sprite_id, "vector.bg", "black");
}

scene.set("hud-score", "text.content", `SCORE: ${score}`);
scene.set("hud-lives", "text.content", `LIVES: ${lives}`);
scene.set("hud-wave", "text.content", `WAVE: ${wave}`);
