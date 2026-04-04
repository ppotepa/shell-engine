// Cycle palette with TAB
if key.code == "tab" {
    palette.cycle();
}

// Debug: 1-9 switches palette by index
if debug_enabled {
    let digit = if key.code == "1" { 0 }
           else if key.code == "2" { 1 }
           else if key.code == "3" { 2 }
           else if key.code == "4" { 3 }
           else if key.code == "5" { 4 }
           else if key.code == "6" { 5 }
           else if key.code == "7" { 6 }
           else if key.code == "8" { 7 }
           else if key.code == "9" { 8 }
           else { -1 };
    if digit >= 0 {
        let ids = palette.list();
        if digit < ids.len() {
            palette.set_active(ids[digit]);
        }
    }
}

// Safe fallbacks for preflight (menu map may be empty during validation)
let sel   = menu.selected_index ?? 0;
let count = menu.count ?? 2;

let items = ["menu-item-0", "menu-item-1"];

// Palette-driven colors — fall back to hardcoded values for preflight safety
let col_bg       = palette.get("bg");       if col_bg       == "" { col_bg       = "#060620"; }
let col_selected = palette.get("menu_sel"); if col_selected == "" { col_selected = "#f6ae2d"; }
let col_idle     = palette.get("menu_idle"); if col_idle    == "" { col_idle     = "#2e5eaa"; }
let col_hint     = palette.get("hint");     if col_hint     == "" { col_hint     = "#172e55"; }
let col_title    = palette.get("title");    if col_title    == "" { col_title    = "#5bc0be"; }
let col_ship_art = palette.get("ship_art"); if col_ship_art == "" { col_ship_art = "#5bc0be"; }
let col_star     = palette.get("star");     if col_star     == "" { col_star     = "#5bc0be"; }

// Apply scene background
scene.set_bg(col_bg);

// Update item colours based on current selection
for i in 0..count {
    let col = if i == sel { col_selected } else { col_idle };
    scene.set(items[i], "style.fg", col);
}

// Apply palette to title text, ship art, hint
scene.set("title-asteroids", "style.fg", col_title);
scene.set("ship-art",        "style.fg", col_ship_art);
scene.set("menu-hint",       "style.fg", col_hint);
scene.set("menu-credit",     "style.fg", col_hint);

// Stars tinted toward palette accent color
let star_ids = [
    "star-0","star-1","star-2","star-3","star-4","star-5","star-6","star-7","star-8","star-9",
    "star-10","star-11","star-12","star-13","star-14","star-15","star-16","star-17","star-18","star-19"
];
for sid in star_ids {
    scene.set(sid, "style.fg", col_star);
}

// Show active palette name in the hint
let pname = palette.name();
if pname != "" {
    scene.set("menu-hint", "text.content",
        `ARROWS  NAVIGATE     ENTER  SELECT     TAB  PALETTE: ${pname}`);
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
