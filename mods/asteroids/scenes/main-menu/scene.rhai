fn mode_label(level_id) {
    if level_id == "asteroids.easy" { return "EASY"; }
    if level_id == "asteroids.hard" { return "HARD"; }
    "DEFAULT"
}

fn mode_colour(level_id) {
    if level_id == "asteroids.easy" { return "#9de36f"; }
    if level_id == "asteroids.hard" { return "#ff8a8a"; }
    "#7bd1ff"
}

fn try_select(level_api, level_id) {
    if level_api.select(level_id) {
    } else {
        // no-op when select fails; keep menu stable
    }
}

if key.pressed == true {
    let code = if key.code.type_of() == "string" { key.code } else { "" };
    if code == "q" || code == "Q" || code == "F5" {
        try_select(level, "asteroids.easy");
    } else if code == "w" || code == "W" || code == "F6" {
        try_select(level, "asteroids.default");
    } else if code == "e" || code == "E" || code == "F7" {
        try_select(level, "asteroids.hard");
    }
}

let current = level.current();
if local.contains("menu_song_started") == false {
    audio.play_song("asteroids.menu-loop");
    local["menu_song_started"] = true;
}
scene.set("mode-current", "text.content", `MODE: ${mode_label(current)}`);
scene.set("mode-current", "style.fg", mode_colour(current));
