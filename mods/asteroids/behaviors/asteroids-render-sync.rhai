import "asteroids-shared" as h;

// ════════════════════════════════════════════════════════════════════════
// on_idle: Per-frame render sync (runs each frame while scene is idle)
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_idle" {
    let ship_id = world.first_kind("ship");
    if ship_id > 0 {
        let ship_ref = world.entity(ship_id);
        if ship_ref.exists() {
            let is_invulnerable = ship_ref.status_has("invulnerable");
            let visible = if is_invulnerable { (time.scene_elapsed_ms / 120) % 2 == 0 } else { true };
            let heading = ship_ref.heading();
            let t = ship_ref.transform();
            scene.set_visible("ship", visible);
            scene.set("ship", "vector.points", rotate_points([[0, -7], [4, 5], [0, 2], [-4, 5]], heading));
            scene.set("ship", "vector.fg", "white");
            scene.set("ship", "vector.bg", "#1a1a1a");
        }
    }

    for ast_id in world.query_kind("asteroid") {
        let ast_ref = world.entity(ast_id);
        if !ast_ref.exists() { continue; }
        let d = ast_ref.data();
        let vid = `asteroid-${ast_id}`;

        let active = ast_ref.get_b("/active", true);
        let rot_phase = ast_ref.get_i("/rot_phase", 0);
        let flash_ms = ast_ref.get_i("/flash_ms", 0);
        let flash_total = ast_ref.get_i("/flash_total_ms", 0);
        let shape = ast_ref.get_i("/shape", 0);
        let size = ast_ref.get_i("/size", 1);

        scene.set_visible(vid, active);
        let pts = rotate_points(asteroid_points(shape, size), rot_phase);
        if flash_ms > 0 {
            scene.set_vector(vid, pts, h::asteroid_stroke_hex(flash_ms, flash_total), h::flash_fill_hex(flash_ms, flash_total));
        } else {
            scene.set_vector(vid, pts, "#aaaaaa", "#333333");
        }

        let show_cracks = active && flash_ms > 0;
        if show_cracks {
            for i in 0..3 {
                let frag_id = h::crack_visual_id(d, i);
                if frag_id.type_of() == "string" {
                    scene.set_visible(frag_id, true);
                }
            }
        } else {
            h::hide_crack_visuals(scene, d);
        }
    }

    for bid in world.query_kind("bullet") {
        let bref = world.entity(bid);
        if !bref.exists() { continue; }
        scene.set_visible(`bullet-${bid}`, true);
    }

    for sid in world.query_kind("smoke") {
        let sref = world.entity(sid);
        if !sref.exists() { continue; }
        let ttl = sref.get_i("/ttl_ms", 0);
        let max_ttl = sref.get_i("/max_ttl_ms", 1);
        let radius = sref.get_i("/radius", 2);

        scene.set_visible(`smoke-${sid}`, true);
        scene.set(`smoke-${sid}`, "vector.points", h::smoke_points(radius, ttl, max_ttl));
        scene.set(`smoke-${sid}`, "vector.fg", h::smoke_colour_hex(ttl, max_ttl));
    }

    let score = game.get("/ast/score");
    let lives = game.get("/ast/lives");
    let wave = game.get("/ast/wave");
    scene.set("hud-score", "text.content", `SCORE: ${score}`);
    scene.set("hud-lives", "text.content", `LIVES: ${lives}`);
    scene.set("hud-wave",  "text.content", `WAVE: ${wave}`);
}
