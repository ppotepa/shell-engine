// High Scores Display Script
// NO functions that access 'game' - inline all game.get() calls

if type_of(local) == "()" { local = #{}; }

fn format_line(entry) {
    let rank_str = `${entry.rank}`;
    if rank_str.len() == 1 { rank_str = ` ${rank_str}`; }
    
    let score_str = `${entry.score}`;
    while score_str.len() < 6 { score_str = ` ${score_str}`; }
    
    let name_str = entry.name;
    while name_str.len() < 3 { name_str = `${name_str} `; }
    
    let wave_str = `${entry.wave}`;
    if wave_str.len() == 1 { wave_str = ` ${wave_str}`; }
    
    `${rank_str}    ${score_str}  ${name_str}    ${wave_str}`
}

// Load scores from persistence (game.get INLINE, not in function)
if !(local.initialized ?? false) {
    local.initialized = true;
    
    // Inline game.get - can't be in a function during preflight
    let saved = game.get("/highscores/scores");
    if type_of(saved) == "()" || type_of(saved) != "array" {
        local.scores = [
            #{ rank: 1, score: 5000, name: "AAA", wave: 3 },
            #{ rank: 2, score: 4000, name: "BBB", wave: 2 },
        ];
    } else {
        local.scores = saved;
    }
    
    local.highlight_rank = game.get("/highscores/last_entry_rank") ?? -1;
}

let scores = local.scores ?? [];
let highlight = local.highlight_rank ?? -1;

// Format and display scores
for i in 0..10 {
    if i < scores.len() {
        let line = format_line(scores[i]);
        scene.set(`row-${i}`, "text.content", line);
        
        // Highlight new entry
        if i == highlight {
            scene.set(`row-${i}`, "style.fg", "#f6ae2d");
        }
    } else {
        scene.set(`row-${i}`, "text.content", "");
    }
}

// ESC to return to title
if key.code == "escape" {
    game.set("/highscores/last_entry_rank", -1);
    game.jump("asteroids-mainmenu");
}

#{state: local}
