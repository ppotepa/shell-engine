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

fn first_message_done(g) {
    if !g.has("/quests/first_message/completed") {
        return false;
    }
    g.get("/quests/first_message/completed")
}

fn is_mail_target(value) {
    if value == "mail" {
        return true;
    }
    value == "mail/welcome.txt"
}

fn is_notes_target(value) {
    if value == "notes" {
        return true;
    }
    value == "notes/starter.txt"
}

fn login_success(user, password) {
    if user != "linus" {
        return false;
    }
    password == "tux"
}

fn seed_session(g) {
    if g.has("/session/user") {
        return;
    }
    g.set("/session/user", "linus");
    g.set("/session/host", "kruuna");
    g.set("/quests/first_message/completed", false);
    g.set("/vfs/files", []);
    g.push("/vfs/files", "mail/welcome.txt");
    g.push("/vfs/files", "notes/starter.txt");
}

fn is_blank_text(value) {
    for c in value.chars() {
        if c != " " && c != "\t" {
            return false;
        }
    }
    true
}

fn handle_cat(parts, transcript, g) {
    if parts.len() < 2 {
        transcript += `usage: cat <file>
`;
        return transcript;
    }

    let target = parts[1];
    if is_mail_target(target) {
        // First-message quest: mark completed when mail is read.
        if !first_message_done(g) {
            g.set("/quests/first_message/completed", true);
        }
        transcript += `From: Operator <op@kruuna>
Subject: Welcome

You made it in. Good.
Read the notes when you get a chance.

`;
        return transcript;
    }

    if is_notes_target(target) {
        transcript += `- type ls to look around
- type cat mail/welcome.txt to read your mail
- more coming soon
`;
        return transcript;
    }

    transcript += `cat: ${target}: No such file or directory
`;
    transcript
}

fn handle_shell_command(command_line, transcript, g) {
    let submitted = sanitize_input(command_line);
    if is_blank_text(submitted) {
        return transcript;
    }

    if submitted == "clear" {
        return "";
    }

    transcript += `[white]linus@kruuna:~$ ${submitted}[/]
`;

    let parts = submitted.split();
    if parts.len() == 0 {
        return transcript;
    }
    let command = parts[0];

    if command == "help" {
        transcript += `Commands: ls  cat <file>  clear
`;
        return transcript;
    }

    if command == "ls" {
        transcript += `mail/  notes/
`;
        return transcript;
    }

    if command == "cat" {
        return handle_cat(parts, transcript, g);
    }

    transcript += `${command}: command not found
`;
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
if mode == "done" {
    if now - success_at >= 300 {
        transcript += `Last login: Tue Sep 17 21:12
[white]You have 1 new message.[/]

`;
        mode = "shell";
    }
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
        if login_success(login_buf, submitted) {
            mode = "done";
            success_at = now;
            transcript += `
`;
            scene.set("login-hidden-prompt", "text.content", "");
            // Seed prologue GameState on first successful login.
            seed_session(game);
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

let quest_done = first_message_done(game);
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
