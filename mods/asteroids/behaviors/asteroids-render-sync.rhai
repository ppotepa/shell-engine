fn flash_fill_hex(flash_ms, flash_total_ms) {
    if flash_ms <= 0 { return "black"; }
    let total = if flash_total_ms <= 0 { 1 } else { flash_total_ms };
    let ratio = (flash_ms * 100) / total;
    if ratio > 66 { return "#808080"; }
    if ratio > 33 { return "#555555"; }
    "#2a2a2a"
}

fn asteroid_stroke_hex(flash_ms, flash_total_ms) {
    if flash_ms <= 0 { return "#cccccc"; }
    let total = if flash_total_ms <= 0 { 1 } else { flash_total_ms };
    let ratio = (flash_ms * 100) / total;
    if ratio > 66 { return "#ffffff"; }
    if ratio > 33 { return "#e2e2e2"; }
    "#d0d0d0"
}

fn smoke_colour_hex(ttl_ms, max_ttl_ms) {
    if ttl_ms <= 0 { return "#101010"; }
    let total = if max_ttl_ms <= 0 { 1 } else { max_ttl_ms };
    let ratio = (ttl_ms * 100) / total;
    if ratio > 75 { return "#8f8f8f"; }
    if ratio > 50 { return "#6f6f6f"; }
    if ratio > 25 { return "#4a4a4a"; }
    "#262626"
}

fn smoke_points(radius, ttl_ms, max_ttl_ms) {
    let total = if max_ttl_ms <= 0 { 1 } else { max_ttl_ms };
    let r = (radius * ttl_ms) / total;
    if r < 1 { r = 1; }
    if r == 1 {
        return [[0, -1], [1, 0], [0, 1], [-1, 0]];
    }
    if r == 2 {
        return [[0, -2], [1, -1], [2, 0], [1, 1], [0, 2], [-1, 1], [-2, 0], [-1, -1]];
    }
    [[0, -3], [2, -2], [3, 0], [2, 2], [0, 3], [-2, 2], [-3, 0], [-2, -2]]
}

fn wrap_heading32(v) {
    let out = v % 32;
    if out < 0 { out + 32 } else { out }
}

fn fragment_heading_offset(idx) {
    if idx == 0 { return 0; }
    if idx == 1 { return 11; }
    21
}

fn crack_visual_id(ast, idx) {
    if idx == 0 { return ast["crack_visual_0"]; }
    if idx == 1 { return ast["crack_visual_1"]; }
    ast["crack_visual_2"]
}

fn hide_crack_visuals(scene, ast) {
    for frag_idx in 0..3 {
        let frag_id = crack_visual_id(ast, frag_idx);
        if frag_id.type_of() == "string" {
            scene.set(frag_id, "visible", false);
        }
    }
}

let scale = 1024;
let ship_id = world.first_kind("ship");
let session_id = world.first_kind("session");
if ship_id <= 0 || session_id <= 0 {
    return ();
}

let ship = world.get(ship_id, "/");
let session = world.get(session_id, "/");
if ship.type_of() != "map" || session.type_of() != "map" {
    return ();
}

let ship_h = if session.contains("ship_h") { session["ship_h"] } else { 0 };
let ship_x_fp = if session.contains("ship_x_fp") { session["ship_x_fp"] } else { 0 };
let ship_y_fp = if session.contains("ship_y_fp") { session["ship_y_fp"] } else { 0 };
let ship_invuln_ms = if session.contains("ship_invuln_ms") { session["ship_invuln_ms"] } else { 0 };
let ship_respawn_ms = if session.contains("ship_respawn_ms") { session["ship_respawn_ms"] } else { 0 };
let score = if session.contains("score") { session["score"] } else { 0 };
let lives = if session.contains("lives") { session["lives"] } else { 0 };
let wave = if session.contains("wave") { session["wave"] } else { 1 };
let hit_flash_ms = if session.contains("hit_flash_ms") { session["hit_flash_ms"] } else { 150 };

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
        hide_crack_visuals(scene, ast);
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
    scene.set(asteroid_id, "vector.fg", asteroid_stroke_hex(flash_ms, flash_total_ms));
    scene.set(asteroid_id, "vector.bg", "black");
    if split_visible {
        for frag_idx in 0..3 {
            let frag_id = crack_visual_id(ast, frag_idx);
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
        hide_crack_visuals(scene, ast);
    }
}

let bullet_ids = world.query_kind("bullet");
for bullet_id in bullet_ids {
    let bullet = world.get(bullet_id, "/");
    if bullet.type_of() != "map" || bullet.contains("visual_id") == false {
        continue;
    }
    let sprite_id = bullet["visual_id"];
    if sprite_id.type_of() != "string" {
        continue;
    }
    if bullet.contains("x_fp") == false || bullet.contains("y_fp") == false {
        scene.set(sprite_id, "visible", false);
        continue;
    }
    scene.set(sprite_id, "visible", bullet["active"]);
    scene.set(sprite_id, "position.x", bullet["x_fp"] / scale);
    scene.set(sprite_id, "position.y", bullet["y_fp"] / scale);
}

let smoke_ids = world.query_kind("smoke");
for smoke_id in smoke_ids {
    let smoke = world.get(smoke_id, "/");
    if smoke.type_of() != "map" || smoke.contains("visual_id") == false {
        continue;
    }
    let sprite_id = smoke["visual_id"];
    if sprite_id.type_of() != "string" {
        continue;
    }
    if smoke.contains("x_fp") == false || smoke.contains("y_fp") == false {
        scene.set(sprite_id, "visible", false);
        continue;
    }
    let ttl_ms = if smoke.contains("ttl_ms") { smoke["ttl_ms"] } else { 0 };
    let max_ttl_ms = if smoke.contains("max_ttl_ms") { smoke["max_ttl_ms"] } else { 1 };
    let radius = if smoke.contains("radius") { smoke["radius"] } else { 2 };
    scene.set(sprite_id, "visible", smoke["active"]);
    scene.set(sprite_id, "position.x", smoke["x_fp"] / scale);
    scene.set(sprite_id, "position.y", smoke["y_fp"] / scale);
    scene.set(sprite_id, "vector.points", smoke_points(radius, ttl_ms, max_ttl_ms));
    scene.set(sprite_id, "vector.fg", smoke_colour_hex(ttl_ms, max_ttl_ms));
    scene.set(sprite_id, "vector.bg", "black");
}

scene.set("hud-score", "text.content", `SCORE: ${score}`);
scene.set("hud-lives", "text.content", `LIVES: ${lives}`);
scene.set("hud-wave", "text.content", `WAVE: ${wave}`);
