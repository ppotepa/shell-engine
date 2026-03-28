if local.contains("loaded") == false {
    local["loaded"] = true;
    let saved = persist.get("/asteroids/highscores");
    if saved.type_of() == "array" {
        let output = "";
        let idx = 0;
        for score in saved {
            if idx >= 8 { break; }
            let points = if score.contains("score") { score["score"] } else { 0 };
            let wave = if score.contains("wave") { score["wave"] } else { 1 };
            if idx > 0 {
                output += "\n";
            }
            output += `#${idx + 1}  SCORE ${points}  WAVE ${wave}`;
            idx += 1;
        }
        if idx > 0 {
            scene.set("scores-lines", "text.content", output);
        } else {
            scene.set("scores-lines", "text.content", "No scores yet");
        }
    } else {
        scene.set("scores-lines", "text.content", "No scores yet");
    }
}

if key.pressed == false {
    return ();
}

let code = if key.code.type_of() == "string" { key.code } else { "" };
if code == "Enter" || code == "Esc" {
    game.jump("asteroids.mainmenu");
} else if code == "r" || code == "R" {
    game.jump("asteroids.game");
}
