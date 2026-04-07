use anyhow::Result;
use crossterm::terminal;
use std::io::{stdout, Write};
use super::state::MenuState;
use super::scanner::{MenuMod, MenuScene};

// Fixed layout metrics (rows used by chrome above the list)
const HEADER_ROWS: usize = 8;  // header + blank + flags(3) + blank + help + blank
const FOOTER_ROWS: usize = 3;  // status + scrollbar hint + blank

pub fn render_menu(state: &mut MenuState) -> Result<()> {
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let term_w = term_w as usize;
    let term_h = term_h as usize;

    let viewport_height = term_h.saturating_sub(HEADER_ROWS + FOOTER_ROWS);
    let viewport_height = viewport_height.max(1);

    state.ensure_visible(viewport_height);

    let mut out = String::with_capacity(4096);

    // Move to home, but don't clear — we'll overwrite each line
    out.push_str("\x1b[H");

    // ── Header ──────────────────────────────────────────────────────
    push_line(&mut out, &format!(
        "\x1b[1;36m  Shell Engine\x1b[0m\x1b[2m  —  interactive launcher\x1b[0m"
    ), term_w);

    push_blank(&mut out, term_w);

    // ── Flags ────────────────────────────────────────────────────────
    let f = &state.flags;
    push_line(&mut out, &format!(
        "  \x1b[2mFlags:\x1b[0m  {}  {}  {}  {}",
        fmt_flag(1, "SDL2",      f.sdl2),
        fmt_flag(2, "SkipSplash",f.skip_splash),
        fmt_flag(3, "Audio",     f.audio),
        fmt_flag(4, "CheckScene",f.check_scenes),
    ), term_w);
    push_line(&mut out, &format!(
        "           {}  {}  {}",
        fmt_flag(5, "Release",   f.release),
        fmt_flag(6, "Dev",       f.dev),
        fmt_flag(7, "AllOpt",    f.all_opt),
    ), term_w);

    push_blank(&mut out, term_w);

    // ── Help bar ─────────────────────────────────────────────────────
    if state.search_mode {
        push_line(&mut out, &format!(
            "  \x1b[2mSearch:\x1b[0m  \x1b[35m/{}\x1b[1;35m█\x1b[0m   \x1b[2mEnter confirm  Esc cancel  Backspace delete\x1b[0m",
            state.search
        ), term_w);
    } else if !state.search.is_empty() {
        push_line(&mut out, &format!(
            "  \x1b[2mFilter: \x1b[35m/{}\x1b[0m  \x1b[2m(/ to edit  Esc clear)\x1b[0m",
            state.search
        ), term_w);
    } else {
        push_line(&mut out, "  \x1b[2m↑↓/jk navigate   → expand   Enter launch   ← collapse   / search   1-7 flags   q quit\x1b[0m", term_w);
    }

    push_blank(&mut out, term_w);

    // ── List ─────────────────────────────────────────────────────────
    if state.filtered_indices.is_empty() {
        push_line(&mut out, "  \x1b[2mno matches\x1b[0m", term_w);
        // Fill remaining viewport rows
        for _ in 1..viewport_height {
            push_blank(&mut out, term_w);
        }
    } else {
        let total = state.filtered_indices.len();
        let end = (state.scroll + viewport_height).min(total);

        for row_idx in state.scroll..end {
            let (mod_idx, scene_idx) = state.filtered_indices[row_idx];
            let m = &state.mods[mod_idx];
            let is_selected = row_idx == state.cursor;

            let line = if let Some(si) = scene_idx {
                let is_last = {
                    let next = row_idx + 1;
                    next >= total || state.filtered_indices[next].0 != mod_idx
                };
                fmt_scene(&m.scenes[si], is_last, is_selected, term_w)
            } else {
                let expanded = state.expanded.contains(&mod_idx);
                fmt_mod(m, expanded, is_selected, term_w)
            };

            out.push_str(&line);
            out.push_str("\x1b[K\r\n"); // clear to EOL
        }

        // Fill remaining viewport rows if list is shorter
        for _ in end..state.scroll + viewport_height {
            push_blank(&mut out, term_w);
        }
    }

    push_blank(&mut out, term_w);

    // ── Status bar ───────────────────────────────────────────────────
    let status = build_status(state, viewport_height);
    push_line(&mut out, &status, term_w);

    // Clear any lines below (in case terminal is taller than our content)
    out.push_str("\x1b[J");

    print!("{}", out);
    stdout().flush()?;
    Ok(())
}

fn push_line(out: &mut String, content: &str, _term_w: usize) {
    out.push_str(content);
    out.push_str("\x1b[K\r\n");
}

fn push_blank(out: &mut String, term_w: usize) {
    push_line(out, "", term_w);
}

fn fmt_flag(n: u8, label: &str, checked: bool) -> String {
    let box_char = if checked { "\x1b[32m✓\x1b[0m" } else { "\x1b[31m✗\x1b[0m" };
    format!("[{}] \x1b[0m{}\x1b[2m({})\x1b[0m", box_char, label, n)
}

fn fmt_mod(m: &MenuMod, expanded: bool, selected: bool, _term_w: usize) -> String {
    let arrow = if expanded { "▼" } else { "▶" };

    let meta = {
        let colors = if m.colors > 0 { format!("{} col", m.colors) } else { String::new() };
        let parts: Vec<&str> = [
            colors.as_str(),
            m.render_size.as_str(),
            m.policy.as_str(),
            m.backend.as_str(),
        ].iter().filter(|s: &&&str| !s.is_empty()).copied().collect();
        parts.join("  ")
    };

    if selected {
        format!(
            "  \x1b[1;7m {} {:<22}\x1b[0m\x1b[2m  {}\x1b[0m",
            arrow, m.name, meta
        )
    } else {
        format!(
            "  \x1b[1m {} {:<22}\x1b[0m\x1b[2m  {}\x1b[0m",
            arrow, m.name, meta
        )
    }
}

fn fmt_scene(scene: &MenuScene, is_last: bool, selected: bool, _term_w: usize) -> String {
    let tree_char = if is_last { "└─" } else { "├─" };
    let label = scene.id.as_deref().unwrap_or(&scene.dir_name);
    let title_part = scene.title.as_deref()
        .map(|t| format!("\x1b[2m  —  {}\x1b[0m", t))
        .unwrap_or_default();

    if selected {
        format!("     \x1b[1;32m{} {}{}\x1b[0m", tree_char, label, title_part)
    } else {
        format!("     \x1b[36m{}\x1b[0m \x1b[37m{}{}\x1b[0m", tree_char, label, title_part)
    }
}

fn build_status(state: &MenuState, viewport_height: usize) -> String {
    let total = state.filtered_indices.len();

    if state.search_mode {
        let scene_count = state.filtered_indices.iter().filter(|(_, s)| s.is_some()).count();
        return format!(
            "  \x1b[35m/{}\x1b[0m\x1b[2m  {} scene{} matched\x1b[0m",
            state.search,
            scene_count,
            if scene_count != 1 { "s" } else { "" }
        );
    }

    if total == 0 {
        return "  \x1b[2mno items\x1b[0m".to_string();
    }

    let (mod_idx, scene_idx) = state.filtered_indices[state.cursor];
    let m = &state.mods[mod_idx];

    let location = if let Some(si) = scene_idx {
        let scene = &m.scenes[si];
        format!("scene  \x1b[37m{}\x1b[0m  in mod  \x1b[37m{}\x1b[0m", 
            scene.id.as_deref().unwrap_or(&scene.dir_name),
            m.name)
    } else {
        let scene_count = m.scenes.len();
        format!("mod  \x1b[37m{}\x1b[0m  \x1b[2m({} scenes)\x1b[0m", m.name, scene_count)
    };

    // Scroll indicator
    let scroll_info = if total > viewport_height {
        let pct = (state.cursor * 100) / total.max(1);
        format!("  \x1b[2m[{}/{}  {}%]\x1b[0m", state.cursor + 1, total, pct)
    } else {
        String::new()
    };

    format!("  \x1b[2m{}\x1b[0m{}", location, scroll_info)
}