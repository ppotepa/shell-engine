fn stars(value) {
    let out = "";
    for _c in value.chars() {
        out += "*";
    }
    out
}

fn sanitize_input(value) {
    let out = "";
    for c in value.chars() {
        if c != "\n" && c != "\r" {
            out += c;
        }
    }
    out
}

let attempts = if state.contains("attempts") { state["attempts"] } else { 1 };
let mode = if state.contains("mode") { state["mode"] } else { "login" };
let transcript = if state.contains("transcript") {
    state["transcript"]
} else {
    "Minix 1.3  Copyright 1987, Prentice-Hall\nConsole ready\n\n"
};
let login_buf = if state.contains("login_buf") { state["login_buf"] } else { "" };
let pass_buf = if state.contains("pass_buf") { state["pass_buf"] } else { "" };
let success_at = if state.contains("success_at") { state["success_at"] } else { -1 };
let shake_until = if state.contains("shake_until") { state["shake_until"] } else { 0 };

if ui_has_change {
    let changed = sanitize_input(ui_change_text);
    if mode == "login" {
        login_buf = changed;
    } else if mode == "password" {
        pass_buf = changed;
    }
}

if ui_has_submit {
    let submitted = sanitize_input(ui_submit_text);
    if mode == "login" {
        login_buf = submitted;
        transcript += `kruuna login: ${login_buf}\n`;
        mode = "password";
        pass_buf = "";
    } else if mode == "password" {
        transcript += `Password: ${stars(submitted)}\n`;
        if login_buf == "linus" && submitted == "tux" {
            mode = "done";
            success_at = scene_elapsed_ms;
            transcript += "\n";
        } else {
            transcript += "Login incorrect\n\n";
            shake_until = scene_elapsed_ms + 420;
            attempts += 1;
            login_buf = "";
            pass_buf = "";
            if attempts > 3 {
                mode = "locked";
                transcript += "[yellow]Too many failed attempts.[/]\n";
            } else {
                mode = "login";
            }
        }
    }
}

let cursor = if ((scene_elapsed_ms / 320) % 2) == 0 { "_" } else { " " };
let view = transcript;

if mode == "login" {
    view += `kruuna login: ${login_buf}${cursor}`;
} else if mode == "password" {
    view += `Password: ${stars(pass_buf)}${cursor}`;
} else if mode == "done" {
    let dt = scene_elapsed_ms - success_at;
    if dt >= 300 {
        view += "Last login: Tue Sep 17 21:12\n";
    }
    if dt >= 600 {
        view += "[white]You have 1 new message.[/]\n";
    }
    if dt >= 900 {
        view += "\nA small system can still travel far.\n";
    }
    if dt >= 1200 {
        view += "Look around. Build only what is needed.\n";
    }
    if dt >= 1500 {
        view += "The network is slow. The server likes archives.\n";
    }
    if dt >= 1900 {
        view += "\n[white]kruuna:~$ _[/]";
    }
} else if mode == "locked" {
    view += `kruuna login: ${cursor}`;
}

scene.set("login-block", "text.content", view);
scene.set("login-hidden-prompt", "text.content", "");
scene.set("login-hidden-output", "text.content", "");

let shake_x = if scene_elapsed_ms < shake_until {
    let phase = (scene_elapsed_ms / 35) % 6;
    if phase == 0 { -2 }
    else if phase == 1 { 2 }
    else if phase == 2 { -1 }
    else if phase == 3 { 1 }
    else if phase == 4 { -1 }
    else { 1 }
} else {
    0
};
scene.set("login-block", "offset.x", shake_x);

#{ state: #{
    attempts: attempts,
    mode: mode,
    transcript: transcript,
    login_buf: login_buf,
    pass_buf: pass_buf,
    success_at: success_at,
    shake_until: shake_until
} }
