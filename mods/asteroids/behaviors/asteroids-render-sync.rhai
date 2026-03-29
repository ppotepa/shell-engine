import "asteroids-shared" as h;

// visual_sync_system now handles Transform2D → scene position auto-sync.
// Render-sync only needs to sync visual properties and game-logic state.

// ── Ship visual ────────────────────────────────────────────
let ship_id = world.first_kind("ship");
if ship_id > 0 {
    let ship = world.entity(ship_id);
    if ship.exists() {
        let t = ship.transform();
        if t.len() > 0 {
            scene.set("ship", "position.x", to_i(t.x));
            scene.set("ship", "position.y", to_i(t.y));
        }

        // Invulnerability flash
        let invuln_ms = ship.get_i("/invuln_ms", 0);
        let respawn_ms = game.get("/ast/ship_respawn_ms");
        if respawn_ms > 0 {
            scene.set("ship", "vector.visible", false);
        } else if invuln_ms > 0 {
            let flash_phase = (time.scene_elapsed_ms / 100) % 2;
            scene.set("ship", "vector.visible", flash_phase == 0);
        } else {
            scene.set("ship", "vector.visible", true);
        }

        // Heading rotation
        let heading = ship.get_i("/h", 0);
        scene.set("ship", "vector.points", rotate_points(
            [[0, -7], [4, 5], [0, 2], [-4, 5]], heading
        ));
    }
}

// ── Asteroids visual ───────────────────────────────────────
for ast_id in world.query_kind("asteroid") {
    let e = world.entity(ast_id);
    if !e.exists() { continue; }
    let d = e.data();
    let vid = if d.contains("visual_id") { d["visual_id"] } else { continue; };

    let active = if d.contains("active") { d["active"] } else { false };
    let rot_phase = if d.contains("rot_phase") { d["rot_phase"] } else { 0 };
    let flash_ms = if d.contains("flash_ms") { d["flash_ms"] } else { 0 };
    let flash_total = if d.contains("flash_total_ms") { d["flash_total_ms"] } else { 0 };
    let shape = if d.contains("shape") { d["shape"] } else { 0 };
    let size = if d.contains("size") { d["size"] } else { 1 };

    let t = e.transform();
    if t.len() > 0 {
        let px = to_i(t.x);
        let py = to_i(t.y);
        scene.set(vid, "position.x", px);
        scene.set(vid, "position.y", py);
        for i in 0..3 {
            let frag_id = h::crack_visual_id(d, i);
            if frag_id.type_of() == "string" {
                scene.set(frag_id, "position.x", px);
                scene.set(frag_id, "position.y", py);
            }
        }
    }

    scene.set(vid, "vector.visible", active);
    scene.set(vid, "vector.points", rotate_points(asteroid_points(shape, size), rot_phase));
    if flash_ms > 0 {
        scene.set(vid, "vector.fg", h::flash_fill_hex(flash_ms, flash_total));
    } else {
        scene.set(vid, "vector.fg", h::asteroid_stroke_hex(flash_ms, flash_total));
    }

    // Crack overlays
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

// ── Bullets visual ─────────────────────────────────────────
for bid in world.query_kind("bullet") {
    let e = world.entity(bid);
    if !e.exists() { continue; }
    scene.set(`bullet-${bid}`, "vector.visible", true);
}

// ── Smoke visual ───────────────────────────────────────────
for sid in world.query_kind("smoke") {
    let e = world.entity(sid);
    if !e.exists() { continue; }
    let d = e.data();
    let ttl = if d.contains("ttl_ms") { d["ttl_ms"] } else { 0 };
    let max_ttl = if d.contains("max_ttl_ms") { d["max_ttl_ms"] } else { 1 };
    let radius = if d.contains("radius") { d["radius"] } else { 2 };

    scene.set(`smoke-${sid}`, "vector.visible", true);
    scene.set(`smoke-${sid}`, "vector.points", h::smoke_points(radius, ttl, max_ttl));
    scene.set(`smoke-${sid}`, "vector.fg", h::smoke_colour_hex(ttl, max_ttl));
}

// ── HUD ────────────────────────────────────────────────────
scene.set("hud-score", "text.content", `SCORE: ${game.get("/ast/score")}`);
scene.set("hud-lives", "text.content", `LIVES: ${game.get("/ast/lives")}`);
scene.set("hud-wave",  "text.content", `WAVE: ${game.get("/ast/wave")}`);
