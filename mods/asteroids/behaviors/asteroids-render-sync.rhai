import "asteroids-shared" as h;

// Find ship entity — FAIL FAST if doesn't exist
let ship_id = world.first_kind("ship");
if ship_id <= 0 {
    return ();
}

let ship_ref = world.entity(ship_id);
if !ship_ref.exists() {
    return ();
}

// Ship visibility (invuln flash)
let invuln_ms = ship_ref.get_i("/invuln_ms", 0);
if invuln_ms > 0 {
    let flash_phase = (time.scene_elapsed_ms / 100) % 2;
    scene.set("ship", "vector.visible", flash_phase == 0);
} else {
    scene.set("ship", "vector.visible", true);
}

// Ship heading (rotate vector.points)
let ship_heading = ship_ref.get_i("/h", 0);
let heading_rad = h::heading32_to_rad(ship_heading);
let ship_points = [[0, -7], [4, 5], [0, 2], [-4, 5]];
scene.set("ship", "vector.points", rotate_points(ship_points, ship_heading));

// Asteroids: rotation + flash color + crack visuals
let ast_ids = world.query_kind("asteroid");
for ast_id in ast_ids {
    let ast = world.get(ast_id, "/");
    if ast.type_of() != "map" {
        continue;
    }
    
    let rot_phase = if ast.contains("rot_phase") { ast["rot_phase"] } else { 0 };
    let flash_ms = if ast.contains("flash_ms") { ast["flash_ms"] } else { 0 };
    let flash_total_ms = if ast.contains("flash_total_ms") { ast["flash_total_ms"] } else { 0 };
    
    scene.set(`asteroid-${ast_id}`, "vector.visible", ast["active"]);
    scene.set(`asteroid-${ast_id}`, "vector.points", rotate_points(
        asteroid_points(ast["shape"], ast["size"]), 
        rot_phase
    ));
    
    if flash_ms > 0 {
        scene.set(`asteroid-${ast_id}`, "vector.fg", h::flash_fill_hex(flash_ms, flash_total_ms));
    } else {
        scene.set(`asteroid-${ast_id}`, "vector.fg", h::asteroid_stroke_hex(flash_ms, flash_total_ms));
    }
    
    let split_visible = ast.contains("active") && ast["active"] && flash_ms > 0;
    if split_visible {
        for frag_idx in 0..3 {
            let frag_id = h::crack_visual_id(ast, frag_idx);
            if frag_id.type_of() == "string" {
                scene.set(frag_id, "vector.visible", true);
            }
        }
    } else {
        h::hide_crack_visuals(scene, ast);
    }
}

// Bullets: visible if active
let bullet_ids = world.query_kind("bullet");
for bullet_id in bullet_ids {
    let bullet = world.get(bullet_id, "/");
    if bullet.type_of() != "map" {
        continue;
    }
    scene.set(`bullet-${bullet_id}`, "vector.visible", bullet["active"]);
}

// Smoke: only update vector.points (fade) + vector.fg (color)
let smoke_ids = world.query_kind("smoke");
for smoke_id in smoke_ids {
    let smoke = world.get(smoke_id, "/");
    if smoke.type_of() != "map" {
        continue;
    }
    
    let ttl_ms = if smoke.contains("ttl_ms") { smoke["ttl_ms"] } else { 0 };
    let max_ttl_ms = if smoke.contains("max_ttl_ms") { smoke["max_ttl_ms"] } else { 1 };
    let radius = if smoke.contains("radius") { smoke["radius"] } else { 2 };
    
    scene.set(`smoke-${smoke_id}`, "vector.visible", smoke["active"]);
    scene.set(`smoke-${smoke_id}`, "vector.points", h::smoke_points(radius, ttl_ms, max_ttl_ms));
    scene.set(`smoke-${smoke_id}`, "vector.fg", h::smoke_colour_hex(ttl_ms, max_ttl_ms));
}

// HUD (score, lives, wave)
let score = game.get("/ast/score");
let lives = game.get("/ast/lives");
let wave = game.get("/ast/wave");
scene.set("hud-score", "text.content", `SCORE: ${score}`);
scene.set("hud-lives", "text.content", `LIVES: ${lives}`);
scene.set("hud-wave", "text.content", `WAVE: ${wave}`);
