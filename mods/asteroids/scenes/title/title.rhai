// Safe fallbacks for preflight (menu map may be empty during validation)
let sel   = menu.selected_index ?? 0;
let count = menu.count ?? 2;

let items = ["menu-item-0", "menu-item-1"];

// Colours: selected = bright white, idle = dim green
let col_selected = "#ffffff";
let col_idle     = "#2d5c2d";

// Update item colours based on current selection
for i in 0..count {
    let col = if i == sel { col_selected } else { col_idle };
    scene.set(items[i], "style.fg", col);
}

// Horizontal float for the selected item (sin wave ±3 cells)
let float_x = (to_float(time.scene_elapsed_ms) / 650.0 * 6.28318).sin() * 3.0;
scene.set(items[sel], "offset.x", float_x);

// Reset offset for non-selected items
for i in 0..count {
    if i != sel {
        scene.set(items[i], "offset.x", 0);
    }
}

// Light streak: sweeps in from the left, crosses the menu, exits right
let streak_period = 3800;
let streak_phase  = time.scene_elapsed_ms % streak_period;
let streak_t      = to_float(streak_phase) / to_float(streak_period);

let streak_x = if streak_t < 0.55 {
    // Ease-out sweep from off-left (-340) across to off-right (+360)
    let p = streak_t / 0.55;
    let eased = 1.0 - (1.0 - p) * (1.0 - p);
    -340.0 + eased * 700.0
} else {
    // Hold off-screen right until next cycle
    360.0
};

scene.set("light-streak", "offset.x", streak_x);
