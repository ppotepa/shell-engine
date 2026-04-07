// ── Asteroids Main Menu ───────────────────────────────────────────────────────
// Selected item: bright colour + gentle horizontal sine-wave float.
// Idle items: dim colour, reset to x=0.

if type_of(local) == "()" { local = #{}; }
if !(local.initialized ?? false) {
    local.initialized = true;
    local.sel = 0;
}

let count = 3;
let sel   = local.sel;

// ── Navigation ────────────────────────────────────────────────────────────────
if input.just_pressed("Up") {
    sel = if sel == 0 { count - 1 } else { sel - 1 };
}
if input.just_pressed("Down") {
    sel = (sel + 1) % count;
}
local.sel = sel;

// ── Palette colours ───────────────────────────────────────────────────────────
let col_sel   = palette.get("menu_sel");  if col_sel   == "" { col_sel   = "#f6ae2d"; }
let col_idle  = palette.get("menu_idle"); if col_idle  == "" { col_idle  = "#2e5eaa"; }

// ── Floaty offset for selected item ──────────────────────────────────────────
let t = time.scene_elapsed_ms;
let float_x = sin(t.to_float() / 400.0) * 3.0;

let items = ["item-0", "item-1", "item-2"];
for i in 0..count {
    if i == sel {
        scene.set(items[i], "style.fg", col_sel);
        scene.set(items[i], "position.x", float_x);
    } else {
        scene.set(items[i], "style.fg", col_idle);
        scene.set(items[i], "position.x", 0.0);
    }
}

// ── Confirm selection ─────────────────────────────────────────────────────────
if input.just_pressed("Enter") || input.just_pressed("Space") {
    if sel == 0 {
        game.jump("asteroids-game");
    } else if sel == 1 {
        game.jump("asteroids-highscores");
    } else if sel == 2 {
        scene.quit();
    }
}
