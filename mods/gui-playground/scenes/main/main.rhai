// â”€â”€ GUI Playground â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Tests every GUI widget type: Slider, Button, Toggle, Panel
// Left column: inputs  |  Right column: reactive outputs

// â”€â”€ Layout Constants â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
const TRACK_W    = 164;
const SWATCH_N   = 5;
const LOG_LINES  = 8;
const MODE_LABELS = ["Surface", "Orbit", "Debug"];

// â”€â”€ Persistent state (survives across frames) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
if local.click_count == () { local.click_count = 0; }
if local.last_btn == ()    { local.last_btn = "---"; }
if local.log == ()         { local.log = []; for i in 0..LOG_LINES { local.log.push("---"); } }
if local.prev_r == ()      { local.prev_r = -1.0; }
if local.prev_g == ()      { local.prev_g = -1.0; }
if local.prev_b == ()      { local.prev_b = -1.0; }
if local.prev_name == ()   { local.prev_name = ""; }
if local.prev_density == () { local.prev_density = ""; }
if local.prev_mode == ()   { local.prev_mode = -1; }

// â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

fn scene_set(scene, target, path, value) {
    scene.object(target).set(path, value);
}

fn scene_text(scene, target, value) {
    scene_set(scene, target, "text.content", value);
}

fn scene_fg(scene, target, value) {
    scene_set(scene, target, "text.fg", value);
}

fn scene_style_fg(scene, target, value) {
    scene_set(scene, target, "style.fg", value);
}

fn scene_visible(scene, target, value) {
    scene_set(scene, target, "visible", value);
}

fn scene_vector(scene, target, points) {
    scene_set(scene, target, "vector.points", points);
}

// â”€â”€ Read all widget state â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let r = gui.slider_value("color-red-slider");
let g = gui.slider_value("color-green-slider");
let b = gui.slider_value("color-blue-slider");

let show_hex    = gui.toggle_on("display-hex-toggle");
let show_swatch = gui.toggle_on("display-swatch-toggle");
let invert      = gui.toggle_on("display-invert-toggle");
let typed_name = gui.text("planet-name-input");
let typed_density = gui.text("planet-density-input");
let density_value = gui.number_value("planet-density-input");
let mode_idx = gui.selected_index("view-mode-dropdown");
let mode_open = gui.widget_open("view-mode-dropdown");
let mode_label = if mode_idx >= 0 && mode_idx < MODE_LABELS.len() { MODE_LABELS[mode_idx] } else { "---" };

// Apply invert
let dr = if invert { 255.0 - r } else { r };
let dg = if invert { 255.0 - g } else { g };
let db = if invert { 255.0 - b } else { b };

let hex_r = to_hex_byte(dr);
let hex_g = to_hex_byte(dg);
let hex_b = to_hex_byte(db);
let hex_str = `#${hex_r}${hex_g}${hex_b}`;

// â”€â”€ Slider fill tracks (real-time visual feedback) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Update fill bar length proportional to slider value.
// This is the most impactful visual: colored bar grows as you drag.
let fill_r = (r / 255.0 * TRACK_W.to_float()).round().to_int();
let fill_g = (g / 255.0 * TRACK_W.to_float()).round().to_int();
let fill_b = (b / 255.0 * TRACK_W.to_float()).round().to_int();

if fill_r < 1 { fill_r = 1; }
if fill_g < 1 { fill_g = 1; }
if fill_b < 1 { fill_b = 1; }

scene_vector(scene, "color-red-slider-fill", [[0, 0], [fill_r, 0]]);
scene_vector(scene, "color-green-slider-fill", [[0, 0], [fill_g, 0]]);
scene_vector(scene, "color-blue-slider-fill", [[0, 0], [fill_b, 0]]);

// Tint fill bars: brighter as value increases
let r_hex = to_hex_byte(lerp_byte(40.0, 255.0, r / 255.0));
let g_hex = to_hex_byte(lerp_byte(40.0, 255.0, g / 255.0));
let b_hex = to_hex_byte(lerp_byte(40.0, 255.0, b / 255.0));
scene_style_fg(scene, "color-red-slider-fill", `#${r_hex}2020`);
scene_style_fg(scene, "color-green-slider-fill", `#20${g_hex}20`);
scene_style_fg(scene, "color-blue-slider-fill", `#2020${b_hex}`);

// â”€â”€ Handle theming (state-aware + channel-tinted) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let handle_ids = ["color-red-slider-handle", "color-green-slider-handle", "color-blue-slider-handle"];
let widget_ids = ["color-red-slider", "color-green-slider", "color-blue-slider"];
let channel_hi = ["#ff8080", "#80ff80", "#8080ff"];
let channel_lo = ["#c06060", "#60c060", "#6060c0"];
for i in 0..3 {
    let color = if gui.widget_pressed(widget_ids[i]) { channel_hi[i] }
                else if gui.widget_hovered(widget_ids[i]) { channel_lo[i] }
                else { "#808090" };
    scene_style_fg(scene, handle_ids[i], color);
}

// â”€â”€ Slider value labels (only update on change) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
if r != local.prev_r { scene_text(scene, "color-red-slider-value", `${r.round()}`); }
if g != local.prev_g { scene_text(scene, "color-green-slider-value", `${g.round()}`); }
if b != local.prev_b { scene_text(scene, "color-blue-slider-value", `${b.round()}`); }
local.prev_r = r;
local.prev_g = g;
local.prev_b = b;

// â”€â”€ Track events â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
if gui.has_change() {
    let wid = gui.changed_widget();

    if wid == "color-red-slider" { push_log(local.log, `slider R -> ${r.round()}`, LOG_LINES); }
    if wid == "color-green-slider" { push_log(local.log, `slider G -> ${g.round()}`, LOG_LINES); }
    if wid == "color-blue-slider" { push_log(local.log, `slider B -> ${b.round()}`, LOG_LINES); }
    if wid == "display-hex-toggle" { push_log(local.log, `toggle hex -> ${if show_hex { "ON" } else { "OFF" }}`, LOG_LINES); }
    if wid == "display-swatch-toggle" { push_log(local.log, `toggle swatch -> ${if show_swatch { "ON" } else { "OFF" }}`, LOG_LINES); }
    if wid == "display-invert-toggle" { push_log(local.log, `toggle invert -> ${if invert { "ON" } else { "OFF" }}`, LOG_LINES); }
    if wid == "view-mode-dropdown" { push_log(local.log, `mode -> ${mode_label}`, LOG_LINES); }
    if wid == "planet-name-input" && gui.submitted("planet-name-input") { push_log(local.log, `name submit -> ${typed_name}`, LOG_LINES); }
    if wid == "planet-density-input" && gui.submitted("planet-density-input") { push_log(local.log, `density submit -> ${gui.text("planet-density-input")}`, LOG_LINES); }
}

// â”€â”€ Button clicks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
if gui.button_clicked("playground-reset-button") {
    local.click_count += 1;
    local.last_btn = "reset";
    push_log(local.log, "btn RESET clicked", LOG_LINES);
    gui.set_widget_value("color-red-slider", 128.0);
    gui.set_widget_value("color-green-slider", 128.0);
    gui.set_widget_value("color-blue-slider", 128.0);
}

if gui.button_clicked("playground-randomize-button") {
    local.click_count += 1;
    local.last_btn = "random";
    push_log(local.log, "btn RANDOMIZE clicked", LOG_LINES);
    let rr = (scene_elapsed_ms % 256).to_float();
    let rg = ((scene_elapsed_ms * 7 + 53) % 256).to_float();
    let rb = ((scene_elapsed_ms * 13 + 97) % 256).to_float();
    gui.set_widget_value("color-red-slider", rr);
    gui.set_widget_value("color-green-slider", rg);
    gui.set_widget_value("color-blue-slider", rb);
}

// â”€â”€ Toggle visuals â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
scene_text(scene, "display-hex-toggle-label", `${if show_hex { "[x]" } else { "[ ]" }} Show Hex`);
scene_text(scene, "display-swatch-toggle-label", `${if show_swatch { "[x]" } else { "[ ]" }} Show Swatch`);
scene_text(scene, "display-invert-toggle-label", `${if invert { "[x]" } else { "[ ]" }} Invert Colors`);

scene_fg(scene, "display-hex-toggle-label", if show_hex { "#90c090" } else { "#708070" });
scene_fg(scene, "display-swatch-toggle-label", if show_swatch { "#90c090" } else { "#708070" });
scene_fg(scene, "display-invert-toggle-label", if invert { "#90c090" } else { "#708070" });

// â”€â”€ Button hover feedback â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let reset_hov = gui.widget_hovered("playground-reset-button");
let rand_hov  = gui.widget_hovered("playground-randomize-button");
scene_fg(scene, "playground-reset-button-label", if reset_hov { "#80c0ff" } else { "#4080ff" });
scene_fg(scene, "playground-randomize-button-label", if rand_hov { "#ffe060" } else { "#ffc030" });

// â”€â”€ Dropdown visuals â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
scene_text(scene, "view-mode-dropdown-arrow", if mode_open { "^" } else { "v" });
scene_fg(scene, "view-mode-dropdown-arrow", if mode_open { "#cfe6ff" } else { "#8aa2c4" });
for i in 0..MODE_LABELS.len() {
    let visible = mode_open;
    scene_visible(scene, `view-mode-dropdown-option-${i}`, visible);
    scene_text(scene, `view-mode-dropdown-option-${i}`, `${if i == mode_idx { ">" } else { " " }} ${MODE_LABELS[i]}`);
    scene_fg(scene, `view-mode-dropdown-option-${i}`, if i == mode_idx { "#e9f2ff" } else { "#8ea0bb" });
}

// â”€â”€ Color swatch (real-time) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
for i in 0..SWATCH_N {
    scene_visible(scene, `swatch-${i}`, show_swatch);
    scene_fg(scene, `swatch-${i}`, hex_str);
}

// Hex/RGB readout
scene_visible(scene, "hex-readout", show_hex);
scene_text(scene, "hex-readout", `HEX: ${hex_str}`);
scene_fg(scene, "hex-readout", hex_str);
scene_text(scene, "rgb-readout", `RGB: ${dr.round()}, ${dg.round()}, ${db.round()}`);

// Click counter
scene_text(scene, "click-counter", `Clicks: ${local.click_count}   Last: ${local.last_btn}`);

// â”€â”€ State readout â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let all_ids = ["color-red-slider", "color-green-slider", "color-blue-slider", "display-hex-toggle", "display-swatch-toggle", "display-invert-toggle", "playground-reset-button", "playground-randomize-button", "planet-name-input", "planet-density-input", "view-mode-dropdown"];

let hover_name = "---";
let pressed_name = "---";
for wid in all_ids {
    if gui.widget_hovered(wid) { hover_name = wid; }
    if gui.widget_pressed(wid) { pressed_name = wid; }
}

scene_text(scene, "state-hover", `Hover   : ${hover_name}`);
scene_text(scene, "state-pressed", `Pressed : ${pressed_name}`);
scene_text(scene, "state-mouse", `Mouse   : ${gui.mouse_x}, ${gui.mouse_y}`);
scene_text(scene, "state-lmb", `LMB     : ${if gui.mouse_left_down { "DOWN" } else { "up" }}`);
if typed_name != local.prev_name {
    scene_text(scene, "state-name", `Name    : ${typed_name}`);
    local.prev_name = typed_name;
}
if gui.text("planet-density-input") != local.prev_density {
    scene_text(scene, "state-density", `Density : ${gui.text("planet-density-input")} (${density_value})`);
    local.prev_density = gui.text("planet-density-input");
}
if mode_idx != local.prev_mode {
    scene_text(scene, "state-mode", `Mode    : ${mode_label}`);
    local.prev_mode = mode_idx;
}

if gui.has_change() {
    scene_text(scene, "state-changed", `Changed : ${gui.changed_widget()}`);
    scene_fg(scene, "state-changed", "#c0c060");
} else {
    scene_fg(scene, "state-changed", "#808090");
}

// â”€â”€ Event log â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
for i in 0..LOG_LINES {
    let line = if i < local.log.len() { local.log[local.log.len() - LOG_LINES + i] } else { "---" };
    scene_text(scene, `log-${i}`, line);
    let age_frac = (LOG_LINES - 1 - i).to_float() / (LOG_LINES - 1).to_float();
    let bright = 96 - (age_frac * 48.0).round();
    let hb = to_hex_byte(bright);
    scene_fg(scene, `log-${i}`, `#${hb}${hb}${hb}`);
}
