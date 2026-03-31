import "asteroids-shared" as h;

// ════════════════════════════════════════════════════════════════════════
// on_idle: Per-frame visual sync
//
// Engine's visual_sync_system handles position updates automatically.
// This script only handles: rotation visuals, flash effects, HUD text.
// ════════════════════════════════════════════════════════════════════════

if time.stage == "on_idle" {
    // ── Ship visual ──────────────────────────────────────────────────
    let ship_id = game.get_i("/ast/ship_id", 0);
    if ship_id > 0 {
        let ship = world.entity(ship_id);
        if ship.exists() {
            let meta = ship.get_metadata();
            let vid = if meta.contains("visual_id") { meta["visual_id"] } else { "ship" };
            let invuln = ship.status_has("invulnerable");
            let visible = if invuln { (time.scene_elapsed_ms / 120) % 2 == 0 } else { true };
            let heading = ship.heading();
            let t = ship.transform();

            scene.set(vid, "position.x", t.x);
            scene.set(vid, "position.y", t.y);
            scene.set_visible(vid, visible);
            scene.set(vid, "vector.points", rotate_points([[0, -7], [4, 5], [0, 2], [-4, 5]], heading));
            scene.set(vid, "vector.fg", "white");
            scene.set(vid, "vector.bg", "#1a1a1a");
        }
    }

    // ── Asteroid visuals ─────────────────────────────────────────────
    for ast_id in world.query_kind("asteroid") {
        let ast = world.entity(ast_id);
        if !ast.exists() { continue; }

        let vid = `asteroid-${ast_id}`;
        let active = ast.get_b("/active", true);
        let rot_phase = ast.get_i("/rot_phase", 0);
        let flash_ms = ast.get_i("/flash_ms", 0);
        let flash_total = ast.get_i("/flash_total_ms", 0);
        let shape = ast.get_i("/shape", 0);
        let size = ast.get_i("/size", 1);

        scene.set_visible(vid, active);
        let pts = rotate_points(asteroid_points(shape, size), rot_phase);
        if flash_ms > 0 {
            scene.set_vector(vid, pts, h::asteroid_stroke_hex(flash_ms, flash_total), h::flash_fill_hex(flash_ms, flash_total));
        } else {
            scene.set_vector(vid, pts, "#aaaaaa", "#333333");
        }

        // Crack overlays follow the asteroid
        let d = ast.data();
        let t = ast.transform();
        if active && flash_ms > 0 {
            for i in 0..3 {
                let frag_id = h::crack_visual_id(d, i);
                if frag_id.type_of() == "string" {
                    scene.set(frag_id, "position.x", t.x);
                    scene.set(frag_id, "position.y", t.y);
                    scene.set_vector(frag_id, pts, h::asteroid_stroke_hex(flash_ms, flash_total), h::flash_fill_hex(flash_ms, flash_total));
                    scene.set_visible(frag_id, true);
                }
            }
        } else {
            h::hide_crack_visuals(scene, d);
        }
    }

    // ── Bullet visibility ────────────────────────────────────────────
    for bid in world.query_kind("bullet") {
        if world.exists(bid) {
            scene.set_visible(`bullet-${bid}`, true);
        }
    }

    // ── Smoke particles ──────────────────────────────────────────────
    for sid in world.query_kind("smoke") {
        let s = world.entity(sid);
        if !s.exists() { continue; }
        let ttl = s.lifetime_remaining();
        let max_ttl = s.get_i("/max_ttl_ms", 1);
        let radius = s.get_i("/radius", 2);
        let vid = `smoke-${sid}`;

        scene.set_visible(vid, true);
        scene.set(vid, "vector.points", h::smoke_points(radius, ttl, max_ttl));
        scene.set(vid, "vector.fg", h::smoke_colour_hex(ttl, max_ttl));
    }

    // ── HUD (only update on change) ──────────────────────────────────
    let score = game.get_i("/ast/score", 0);
    let lives = game.get_i("/ast/lives", 0);
    let wave = game.get_i("/ast/wave", 0);

    let prev_score = if local.contains("hud_score") { local["hud_score"] } else { -1 };
    let prev_lives = if local.contains("hud_lives") { local["hud_lives"] } else { -1 };
    let prev_wave = if local.contains("hud_wave") { local["hud_wave"] } else { -1 };

    if score != prev_score { scene.set("hud-score", "text.content", `SCORE: ${score}`); local["hud_score"] = score; }
    if lives != prev_lives { scene.set("hud-lives", "text.content", `LIVES: ${lives}`); local["hud_lives"] = lives; }
    if wave != prev_wave { scene.set("hud-wave", "text.content", `WAVE: ${wave}`); local["hud_wave"] = wave; }
}
