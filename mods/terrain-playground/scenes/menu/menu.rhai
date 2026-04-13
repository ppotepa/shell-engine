// ── Terrain Playground Main Menu ─────────────────────────────────────────────

if type_of(local) == "()" { local = #{}; }
if !(local.initialized ?? false) {
    local.initialized = true;
    local.sel = 0;
}

let count = 5;
let sel   = local.sel;

// ── Navigation ────────────────────────────────────────────────────────────────
if input.just_pressed("Up") {
    sel = if sel == 0 { count - 1 } else { sel - 1 };
}
if input.just_pressed("Down") {
    sel = (sel + 1) % count;
}
local.sel = sel;

// ── Colours ───────────────────────────────────────────────────────────────────
let col_sel  = "#44cc88";
let col_idle = "#2a6e55";
let col_exit = "#1a3e35";

let t = time.scene_elapsed_ms;
let float_x = sin(t.to_float() / 400.0) * 3.0;

let items = ["item-0", "item-1", "item-2", "item-3", "item-4"];
for i in 0..count {
    let is_exit = (i == count - 1);
    if i == sel {
        scene.set(items[i], "style.fg", col_sel);
        scene.set(items[i], "position.x", float_x);
    } else {
        scene.set(items[i], "style.fg", if is_exit { col_exit } else { col_idle });
        scene.set(items[i], "position.x", 0.0);
    }
}

// ── Number shortcuts ──────────────────────────────────────────────────────────
if input.just_pressed("1") { sel = 0; local.sel = 0; }
if input.just_pressed("2") { sel = 1; local.sel = 1; }
if input.just_pressed("3") { sel = 2; local.sel = 2; }
if input.just_pressed("4") { sel = 3; local.sel = 3; }
if input.just_pressed("0") { sel = 4; local.sel = 4; }

// ── Confirm ───────────────────────────────────────────────────────────────────
let confirm = input.just_pressed("Enter") || input.just_pressed("Space");
if confirm {
    if sel == 0 { game.jump("terrain-plain"); }
    else if sel == 1 { game.jump("terrain-placeholder-heightmap"); }
    else if sel == 2 { game.jump("terrain-placeholder-erosion"); }
    else if sel == 3 { game.jump("terrain-placeholder-caves"); }
    else if sel == 4 { game.jump("playground-exit"); }
}
