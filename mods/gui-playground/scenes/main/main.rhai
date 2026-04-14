// ── GUI Playground ────────────────────────────────────────────────────────────
// Tests every GUI widget type: Slider, Button, Toggle, Panel
// Left column: inputs  |  Right column: reactive outputs

// ── Layout Constants ─────────────────────────────────────────────────────────
const TRACK_W    = 190;
const SWATCH_N   = 5;
const LOG_LINES  = 8;

// ── Persistent state (survives across frames) ────────────────────────────────
if local.click_count == () { local.click_count = 0; }
if local.last_btn == ()    { local.last_btn = "---"; }
if local.log == ()         { local.log = []; for i in 0..LOG_LINES { local.log.push("---"); } }
if local.prev_r == ()      { local.prev_r = -1.0; }
if local.prev_g == ()      { local.prev_g = -1.0; }
if local.prev_b == ()      { local.prev_b = -1.0; }

// ── Helpers ──────────────────────────────────────────────────────────────────
fn to_hex_byte(v) {
    let n = v.round();
    if n < 0.0 { n = 0.0; }
    if n > 255.0 { n = 255.0; }
    let hi = (n / 16.0).floor().to_int();
    let lo = (n - hi.to_float() * 16.0).floor().to_int();
    let hex_chars = "0123456789ABCDEF";
    `${hex_chars.sub_string(hi, 1)}${hex_chars.sub_string(lo, 1)}`
}

fn push_log(log, msg, max_lines) {
    log.push(msg);
    if log.len() > max_lines { log.remove(0); }
}

fn lerp_byte(lo, hi, t) {
    (lo + (hi - lo) * t).round()
}

// ── Read all widget state ────────────────────────────────────────────────────
let r = gui.slider_value("slider-r");
let g = gui.slider_value("slider-g");
let b = gui.slider_value("slider-b");

let show_hex    = gui.toggle_on("toggle-hex");
let show_swatch = gui.toggle_on("toggle-swatch");
let invert      = gui.toggle_on("toggle-invert");

// Apply invert
let dr = if invert { 255.0 - r } else { r };
let dg = if invert { 255.0 - g } else { g };
let db = if invert { 255.0 - b } else { b };

let hex_r = to_hex_byte(dr);
let hex_g = to_hex_byte(dg);
let hex_b = to_hex_byte(db);
let hex_str = `#${hex_r}${hex_g}${hex_b}`;

// ── Slider fill tracks (real-time visual feedback) ───────────────────────────
// Update fill bar length proportional to slider value.
// This is the most impactful visual: colored bar grows as you drag.
let fill_r = (r / 255.0 * TRACK_W.to_float()).round().to_int();
let fill_g = (g / 255.0 * TRACK_W.to_float()).round().to_int();
let fill_b = (b / 255.0 * TRACK_W.to_float()).round().to_int();

if fill_r < 1 { fill_r = 1; }
if fill_g < 1 { fill_g = 1; }
if fill_b < 1 { fill_b = 1; }

scene.set("slider-r-fill", "vector.points", [[0, 0], [fill_r, 0]]);
scene.set("slider-g-fill", "vector.points", [[0, 0], [fill_g, 0]]);
scene.set("slider-b-fill", "vector.points", [[0, 0], [fill_b, 0]]);

// Tint fill bars: brighter as value increases
let r_hex = to_hex_byte(lerp_byte(40.0, 255.0, r / 255.0));
let g_hex = to_hex_byte(lerp_byte(40.0, 255.0, g / 255.0));
let b_hex = to_hex_byte(lerp_byte(40.0, 255.0, b / 255.0));
scene.set("slider-r-fill", "style.fg", `#${r_hex}2020`);
scene.set("slider-g-fill", "style.fg", `#20${g_hex}20`);
scene.set("slider-b-fill", "style.fg", `#2020${b_hex}`);

// ── Handle theming (state-aware + channel-tinted) ────────────────────────────
let handle_ids = ["slider-r-handle", "slider-g-handle", "slider-b-handle"];
let widget_ids = ["slider-r", "slider-g", "slider-b"];
let channel_hi = ["#ff8080", "#80ff80", "#8080ff"];
let channel_lo = ["#c06060", "#60c060", "#6060c0"];
for i in 0..3 {
    let color = if gui.widget_pressed(widget_ids[i]) { channel_hi[i] }
                else if gui.widget_hovered(widget_ids[i]) { channel_lo[i] }
                else { "#808090" };
    scene.set(handle_ids[i], "style.fg", color);
}

// ── Slider value labels (only update on change) ─────────────────────────────
if r != local.prev_r { scene.set("slider-r-val", "text.content", `${r.round()}`); }
if g != local.prev_g { scene.set("slider-g-val", "text.content", `${g.round()}`); }
if b != local.prev_b { scene.set("slider-b-val", "text.content", `${b.round()}`); }
local.prev_r = r;
local.prev_g = g;
local.prev_b = b;

// ── Track events ─────────────────────────────────────────────────────────────
if gui.has_change() {
    let wid = gui.changed_widget();

    if wid == "slider-r" { push_log(local.log, `slider R -> ${r.round()}`, LOG_LINES); }
    if wid == "slider-g" { push_log(local.log, `slider G -> ${g.round()}`, LOG_LINES); }
    if wid == "slider-b" { push_log(local.log, `slider B -> ${b.round()}`, LOG_LINES); }
    if wid == "toggle-hex" { push_log(local.log, `toggle hex -> ${if show_hex { "ON" } else { "OFF" }}`, LOG_LINES); }
    if wid == "toggle-swatch" { push_log(local.log, `toggle swatch -> ${if show_swatch { "ON" } else { "OFF" }}`, LOG_LINES); }
    if wid == "toggle-invert" { push_log(local.log, `toggle invert -> ${if invert { "ON" } else { "OFF" }}`, LOG_LINES); }
}

// ── Button clicks ────────────────────────────────────────────────────────────
if gui.button_clicked("btn-reset") {
    local.click_count += 1;
    local.last_btn = "reset";
    push_log(local.log, "btn RESET clicked", LOG_LINES);
    gui.set_widget_value("slider-r", 128.0);
    gui.set_widget_value("slider-g", 128.0);
    gui.set_widget_value("slider-b", 128.0);
}

if gui.button_clicked("btn-randomize") {
    local.click_count += 1;
    local.last_btn = "random";
    push_log(local.log, "btn RANDOMIZE clicked", LOG_LINES);
    let rr = (scene_elapsed_ms % 256).to_float();
    let rg = ((scene_elapsed_ms * 7 + 53) % 256).to_float();
    let rb = ((scene_elapsed_ms * 13 + 97) % 256).to_float();
    gui.set_widget_value("slider-r", rr);
    gui.set_widget_value("slider-g", rg);
    gui.set_widget_value("slider-b", rb);
}

// ── Toggle visuals ───────────────────────────────────────────────────────────
scene.set("toggle-hex-label", "text.content", `${if show_hex { "[✓]" } else { "[ ]" }} Show Hex`);
scene.set("toggle-swatch-label", "text.content", `${if show_swatch { "[✓]" } else { "[ ]" }} Show Swatch`);
scene.set("toggle-invert-label", "text.content", `${if invert { "[✓]" } else { "[ ]" }} Invert Colors`);

scene.set("toggle-hex-label", "text.fg", if show_hex { "#90c090" } else { "#708070" });
scene.set("toggle-swatch-label", "text.fg", if show_swatch { "#90c090" } else { "#708070" });
scene.set("toggle-invert-label", "text.fg", if invert { "#90c090" } else { "#708070" });

// ── Button hover feedback ────────────────────────────────────────────────────
let reset_hov = gui.widget_hovered("btn-reset");
let rand_hov  = gui.widget_hovered("btn-randomize");
scene.set("btn-reset-label", "text.fg", if reset_hov { "#80c0ff" } else { "#4080ff" });
scene.set("btn-randomize-label", "text.fg", if rand_hov { "#ffe060" } else { "#ffc030" });

// ── Color swatch (real-time) ─────────────────────────────────────────────────
for i in 0..SWATCH_N {
    scene.set(`swatch-${i}`, "visible", show_swatch);
    scene.set(`swatch-${i}`, "text.fg", hex_str);
}

// Hex/RGB readout
scene.set("hex-readout", "visible", show_hex);
scene.set("hex-readout", "text.content", `HEX: ${hex_str}`);
scene.set("hex-readout", "text.fg", hex_str);
scene.set("rgb-readout", "text.content", `RGB: ${dr.round()}, ${dg.round()}, ${db.round()}`);

// Click counter
scene.set("click-counter", "text.content", `Clicks: ${local.click_count}   Last: ${local.last_btn}`);

// ── State readout ────────────────────────────────────────────────────────────
let all_ids = ["slider-r", "slider-g", "slider-b", "toggle-hex", "toggle-swatch", "toggle-invert", "btn-reset", "btn-randomize"];

let hover_name = "---";
let pressed_name = "---";
for wid in all_ids {
    if gui.widget_hovered(wid) { hover_name = wid; }
    if gui.widget_pressed(wid) { pressed_name = wid; }
}

scene.set("state-hover",   "text.content", `Hover:   ${hover_name}`);
scene.set("state-pressed", "text.content", `Pressed: ${pressed_name}`);
scene.set("state-mouse",   "text.content", `Mouse:   ${gui.mouse_x}, ${gui.mouse_y}`);
scene.set("state-lmb",     "text.content", `LMB:     ${if gui.mouse_left_down { "DOWN" } else { "up" }}`);

if gui.has_change() {
    scene.set("state-changed", "text.content", `Changed: ${gui.changed_widget()}`);
    scene.set("state-changed", "text.fg", "#c0c060");
} else {
    scene.set("state-changed", "text.fg", "#808090");
}

// ── Event log ────────────────────────────────────────────────────────────────
for i in 0..LOG_LINES {
    let line = if i < local.log.len() { local.log[local.log.len() - LOG_LINES + i] } else { "---" };
    scene.set(`log-${i}`, "text.content", line);
    let age_frac = (LOG_LINES - 1 - i).to_float() / (LOG_LINES - 1).to_float();
    let bright = 96 - (age_frac * 48.0).round();
    let hb = to_hex_byte(bright);
    scene.set(`log-${i}`, "text.fg", `#${hb}${hb}${hb}`);
}
