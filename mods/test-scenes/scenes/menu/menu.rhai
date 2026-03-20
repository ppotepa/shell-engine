fn abs_i(v) {
    if v < 0 { -v } else { v }
}

fn wrapped_distance(index, selected, total) {
    let raw = index - selected;
    if total <= 1 {
        return raw;
    }
    let a = raw;
    let b = raw - total;
    let c = raw + total;
    let mut best = a;
    if abs_i(b) < abs_i(best) { best = b; }
    if abs_i(c) < abs_i(best) { best = c; }
    best
}

let target = "menu-grid";
let item_prefix = "menu-item-";
let window = 3;
let step_y = 2;
let endless = true;
let count = menu.count;

if !regions.contains(target) {
    return ();
}

let target_region = regions[target];
let center_y = target_region["y"] + ((target_region["h"] - 1) / 2);
let half_window = (window - 1) / 2;

for idx in 0..count {
    let alias = `${item_prefix}${idx}`;
    let relative = if endless {
        wrapped_distance(idx, menu.selected_index, count)
    } else {
        idx - menu.selected_index
    };
    if abs_i(relative) > half_window {
        scene.set(alias, "visible", false);
        continue;
    }
    scene.set(alias, "visible", true);
    if !regions.contains(alias) {
        continue;
    }
    let item_region = regions[alias];
    let item_height = if item_region["h"] > 0 { item_region["h"] } else { 1 };
    let min_step = item_height + 1;
    let effective_step = if step_y > min_step { step_y } else { min_step };
    let desired_y = center_y + relative * effective_step;
    let dy = desired_y - item_region["y"];
    scene.set(alias, "offset.x", 0);
    scene.set(alias, "offset.y", dy);
}
