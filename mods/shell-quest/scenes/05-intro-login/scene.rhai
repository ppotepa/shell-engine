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

// Returns hint line shown below the prompt based on quest state.
fn quest_hint(quest_done) {
    if quest_done {
        `[#555555]mail read. look around.[/]
`
    } else {
        `[#555555]tip: try  ls  or  cat mail/welcome.txt[/]
`
    }
}

fn handle_shell_command(command_line, transcript, g) {
    let submitted = sanitize_input(command_line).trim();
    if submitted.is_empty() {
        return transcript;
    }

    if submitted == "clear" {
        return "";
    }

    transcript += `[white]linus@kruuna:~$ ${submitted}[/]
`;

    let parts = submitted.split();
    let command = if parts.len() > 0 { parts[0] } else { "" };

    if command == "help" {
        transcript += `Commands: ls  cat <file>  clear
`;
    } else if command == "ls" {
        transcript += `mail/  notes/
`;
    } else if command == "cat" {
        if parts.len() < 2 {
            transcript += `usage: cat <file>
`;
        } else if parts[1] == "mail" || parts[1] == "mail/welcome.txt" {
            // First-message quest: mark completed when mail is read.
            if !g.has("/quests/first_message/completed") || !g.get("/quests/first_message/completed") {
                g.set("/quests/first_message/completed", true);
            }
            transcript += `From: Operator <op@kruuna>
Subject: Welcome

You made it in. Good.
Read the notes when you get a chance.

`;
        } else if parts[1] == "notes" || parts[1] == "notes/starter.txt" {
            transcript += `- type ls to look around
- type cat mail/welcome.txt to read your mail
- more coming soon
`;
        } else {
            transcript += `cat: ${parts[1]}: No such file or directory
`;
        }
    } else {
        transcript += `${command}: command not found
`;
    }

    transcript
}

let attempts    = if state.contains("attempts")    { state["attempts"]    } else { 1 };
let mode        = if state.contains("mode")        { state["mode"]        } else { "login" };
let transcript  = if state.contains("transcript")  { state["transcript"]  } else {
`Minix 1.3  Copyright 1987, Prentice-Hall
Console ready

`
};
let login_buf   = if state.contains("login_buf")   { state["login_buf"]   } else { "" };
let pass_buf    = if state.contains("pass_buf")    { state["pass_buf"]    } else { "" };
let shell_buf   = if state.contains("shell_buf")   { state["shell_buf"]   } else { "" };
let success_at  = if state.contains("success_at")  { state["success_at"]  } else { -1 };
let shake_until = if state.contains("shake_until") { state["shake_until"] } else { 0 };
let now = time.scene_elapsed_ms;

// Transition from done → shell after brief pause (login success messages appear).
if mode == "done" && now - success_at >= 300 {
    transcript += `Last login: Tue Sep 17 21:12
[white]You have 1 new message.[/]

`;
    mode = "shell";
}

if ui.has_change {
    let changed = sanitize_input(ui.change_text);
    if mode == "login" {
        login_buf = changed;
    } else if mode == "password" {
        pass_buf = changed;
    } else if mode == "shell" {
        shell_buf = changed;
    }
}

if ui.has_submit {
    let submitted = sanitize_input(ui.submit_text);
    if mode == "login" {
        login_buf = submitted;
        transcript += `kruuna login: ${login_buf}
`;
        mode = "password";
        pass_buf = "";
        scene.set("login-hidden-prompt", "text.content", "");
    } else if mode == "password" {
        transcript += `Password: ${stars(submitted)}
`;
        if login_buf == "linus" && submitted == "tux" {
            mode = "done";
            success_at = now;
            transcript += `
`;
            scene.set("login-hidden-prompt", "text.content", "");
            // Seed prologue GameState on first successful login.
            if !game.has("/session/user") {
                game.set("/session/user", "linus");
                game.set("/session/host", "kruuna");
                game.set("/vfs/files", ["mail/welcome.txt", "notes/starter.txt"]);
                game.set("/quests/first_message/completed", false);
            }
        } else {
            transcript += `Login incorrect

`;
            shake_until = now + 420;
            attempts += 1;
            login_buf = "";
            pass_buf = "";
            if attempts > 3 {
                mode = "locked";
                transcript += `[yellow]Too many failed attempts.[/]
`;
            } else {
                mode = "login";
            }
        }
    } else if mode == "shell" {
        transcript = handle_shell_command(submitted, transcript, game);
        shell_buf = "";
    }
}

let quest_done = game.has("/quests/first_message/completed") && game.get("/quests/first_message/completed");
let cursor = if ((now / 320) % 2) == 0 { "_" } else { " " };
let view = transcript;

if mode == "login" {
    view += `kruuna login: ${login_buf}${cursor}`;
} else if mode == "password" {
    view += `Password: ${stars(pass_buf)}${cursor}`;
} else if mode == "done" {
    view += " ";
} else if mode == "shell" {
    view += quest_hint(quest_done);
    view += `[white]linus@kruuna:~$ ${shell_buf}${cursor}[/]`;
} else if mode == "locked" {
    view += `kruuna login: ${cursor}`;
}

scene.set("login-block", "text.content", view);
scene.set("login-hidden-prompt", "text.content", "");
scene.set("login-hidden-output", "text.content", "");

let shake_x = if now < shake_until {
    let phase = (now / 35) % 6;
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
    attempts:    attempts,
    mode:        mode,
    transcript:  transcript,
    login_buf:   login_buf,
    pass_buf:    pass_buf,
    shell_buf:   shell_buf,
    success_at:  success_at,
    shake_until: shake_until
} }

    for c in value.chars() {
        if c != "\n" && c != "\r" {
            out += c;
        }
    }
    out
}

fn handle_shell_command(command_line, transcript) {
    let submitted = sanitize_input(command_line).trim();
    if submitted.is_empty() {
        return transcript;
    }

    if submitted == "clear" {
        return "";
    }

    transcript += `[white]kruuna:~$ ${submitted}[/]
`;

    let parts = submitted.split();
    let command = if parts.len() > 0 { parts[0] } else { "" };

    if command == "help" {
        transcript += "Available commands: help, ls, cat, clear
";
    } else if command == "ls" {
        transcript += "mail  notes
";
    } else if command == "cat" {
        if parts.len() < 2 {
            transcript += "usage: cat <file>
";
        } else if parts[1] == "mail" || parts[1] == "mail/welcome.txt" {
            transcript += `From: Operator
Subject: Welcome

You made it in. Read the notes and look around.
`;
        } else if parts[1] == "notes" || parts[1] == "notes/starter.txt" {
            transcript += `- type ls to look around
- type cat mail/welcome.txt to read mail
`;
        } else {
            transcript += `cat: ${parts[1]}: No such file
`;
        }
    } else {
        transcript += `command not found: ${command}
`;
    }

    transcript
}

let attempts = if state.contains("attempts") { state["attempts"] } else { 1 };
let mode = if state.contains("mode") { state["mode"] } else { "login" };
let transcript = if state.contains("transcript") {
    state["transcript"]
} else {
    `Minix 1.3  Copyright 1987, Prentice-Hall
Console ready

`
};
let login_buf = if state.contains("login_buf") { state["login_buf"] } else { "" };
let pass_buf = if state.contains("pass_buf") { state["pass_buf"] } else { "" };
let shell_buf = if state.contains("shell_buf") { state["shell_buf"] } else { "" };
let success_at = if state.contains("success_at") { state["success_at"] } else { -1 };
let shake_until = if state.contains("shake_until") { state["shake_until"] } else { 0 };
let now = time.scene_elapsed_ms;

if mode == "done" && now - success_at >= 300 {
    transcript += `Last login: Tue Sep 17 21:12
[white]You have 1 new message.[/]

`;
    mode = "shell";
}

if ui.has_change {
    let changed = sanitize_input(ui.change_text);
    if mode == "login" {
        login_buf = changed;
    } else if mode == "password" {
        pass_buf = changed;
    } else if mode == "shell" {
        shell_buf = changed;
    }
}

if ui.has_submit {
    let submitted = sanitize_input(ui.submit_text);
    if mode == "login" {
        login_buf = submitted;
        transcript += `kruuna login: ${login_buf}
`;
        mode = "password";
        pass_buf = "";
        scene.set("login-hidden-prompt", "text.content", "");
    } else if mode == "password" {
        transcript += `Password: ${stars(submitted)}
`;
        if login_buf == "linus" && submitted == "tux" {
            mode = "done";
            success_at = now;
            transcript += `
`;
            scene.set("login-hidden-prompt", "text.content", "");
        } else {
            transcript += `Login incorrect

`;
            shake_until = now + 420;
            attempts += 1;
            login_buf = "";
            pass_buf = "";
            if attempts > 3 {
                mode = "locked";
                transcript += `[yellow]Too many failed attempts.[/]
`;
            } else {
                mode = "login";
            }
        }
    } else if mode == "shell" {
        transcript = handle_shell_command(submitted, transcript);
        shell_buf = "";
    }
}

let cursor = if ((now / 320) % 2) == 0 { "_" } else { " " };
let view = transcript;

if mode == "login" {
    view += `kruuna login: ${login_buf}${cursor}`;
} else if mode == "password" {
    view += `Password: ${stars(pass_buf)}${cursor}`;
} else if mode == "done" {
    view += " ";
} else if mode == "shell" {
    view += `[white]kruuna:~$ ${shell_buf}${cursor}[/]`;
} else if mode == "locked" {
    view += `kruuna login: ${cursor}`;
}

scene.set("login-block", "text.content", view);
scene.set("login-hidden-prompt", "text.content", "");
scene.set("login-hidden-output", "text.content", "");

let shake_x = if now < shake_until {
    let phase = (now / 35) % 6;
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
    shell_buf: shell_buf,
    success_at: success_at,
    shake_until: shake_until
} }
