// Safe fallbacks for preflight (menu map may be empty during validation)
let sel   = menu.selected_index ?? 0;
let count = menu.count ?? 2;

let items = ["menu-item-0", "menu-item-1"];

// Palette: selected = Honey Bronze (warm gold), idle = Tech Blue (dim)
let col_selected = "#f6ae2d";
let col_idle     = "#2e5eaa";

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
