// ── Planet Generator ─────────────────────────────────────────────────────────
// 1/2/3/4/5  → switch tab (Continents / Mountains / Climate / Visual / Atmosphere)
// F1-F7    → load preset (Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago)
// Mouse drag → orbit camera around planet (always active; drag outside HUD)
// +/-      → zoom orbit camera in/out
// Ctrl+F   → toggle free-look fly camera (WASD/QE to fly, mouse to look)
// R        → toggle planet auto-rotation on/off
// Delete   → reset to Earth defaults
//
// 8 multiplexed slider slots: content remapped per active tab.
// All gui widgets use min=0, max=1 (normalized). Rhai maps to/from param domains.

// ── Preset definitions ────────────────────────────────────────────────────────
fn preset_earth() {
    #{
        seed: 0.0, ocean: 0.55, cscale: 2.5, cwarp: 0.65, coct: 5.0,
        mscale: 6.0, mstr: 0.45, mroct: 5.0,
        moisture: 3.0, ice: 1.0, lapse: 0.6, rain: 0.4,
        disp: 0.22, coloring: 0.0, rotspeed: 3.0,
        sunaz: 210.0, sunel: 40.0, ambient: 0.12,
        atmo_color: 2.0, atmo_strength: 0.55, atmo_rim_power: 3.2, atmo_haze_strength: 0.38, atmo_haze_power: 1.5
    }
}
fn preset_mars() {
    #{
        seed: 42.0, ocean: 0.05, cscale: 3.5, cwarp: 0.3, coct: 4.0,
        mscale: 8.0, mstr: 0.65, mroct: 6.0,
        moisture: 1.0, ice: 0.1, lapse: 0.3, rain: 0.05,
        disp: 0.38, coloring: 1.0, rotspeed: 3.0,
        sunaz: 190.0, sunel: 30.0, ambient: 0.08,
        atmo_color: 0.0, atmo_strength: 0.20, atmo_rim_power: 5.0, atmo_haze_strength: 0.14, atmo_haze_power: 2.2
    }
}
fn preset_ocean() {
    #{
        seed: 7.0, ocean: 0.88, cscale: 1.5, cwarp: 1.2, coct: 3.0,
        mscale: 4.0, mstr: 0.15, mroct: 3.0,
        moisture: 5.0, ice: 0.8, lapse: 0.7, rain: 0.6,
        disp: 0.08, coloring: 0.0, rotspeed: 2.0,
        sunaz: 220.0, sunel: 50.0, ambient: 0.15,
        atmo_color: 2.0, atmo_strength: 0.65, atmo_rim_power: 3.0, atmo_haze_strength: 0.45, atmo_haze_power: 1.4
    }
}
fn preset_desert() {
    #{
        seed: 13.0, ocean: 0.15, cscale: 4.0, cwarp: 0.2, coct: 3.0,
        mscale: 5.0, mstr: 0.3, mroct: 4.0,
        moisture: 1.2, ice: 0.0, lapse: 0.4, rain: 0.05,
        disp: 0.3, coloring: 0.0, rotspeed: 4.0,
        sunaz: 200.0, sunel: 25.0, ambient: 0.18,
        atmo_color: 0.0, atmo_strength: 0.42, atmo_rim_power: 3.8, atmo_haze_strength: 0.28, atmo_haze_power: 1.9
    }
}
fn preset_ice() {
    #{
        seed: 99.0, ocean: 0.45, cscale: 2.0, cwarp: 0.5, coct: 4.0,
        mscale: 5.0, mstr: 0.35, mroct: 4.0,
        moisture: 2.5, ice: 3.0, lapse: 1.0, rain: 0.3,
        disp: 0.18, coloring: 0.0, rotspeed: 1.5,
        sunaz: 230.0, sunel: 15.0, ambient: 0.25,
        atmo_color: 4.0, atmo_strength: 0.50, atmo_rim_power: 2.8, atmo_haze_strength: 0.42, atmo_haze_power: 1.3
    }
}
fn preset_volcanic() {
    #{
        seed: 66.0, ocean: 0.3, cscale: 3.0, cwarp: 1.5, coct: 6.0,
        mscale: 10.0, mstr: 0.85, mroct: 7.0,
        moisture: 1.5, ice: 0.0, lapse: 0.5, rain: 0.15,
        disp: 0.55, coloring: 0.0, rotspeed: 5.0,
        sunaz: 180.0, sunel: 35.0, ambient: 0.1,
        atmo_color: 1.0, atmo_strength: 0.35, atmo_rim_power: 4.5, atmo_haze_strength: 0.22, atmo_haze_power: 2.3
    }
}
fn preset_archipelago() {
    #{
        seed: 27.0, ocean: 0.75, cscale: 1.0, cwarp: 1.8, coct: 7.0,
        mscale: 3.0, mstr: 0.2, mroct: 3.0,
        moisture: 4.0, ice: 0.5, lapse: 0.65, rain: 0.5,
        disp: 0.12, coloring: 0.0, rotspeed: 2.5,
        sunaz: 215.0, sunel: 45.0, ambient: 0.14,
        atmo_color: 2.0, atmo_strength: 0.58, atmo_rim_power: 3.0, atmo_haze_strength: 0.40, atmo_haze_power: 1.4
    }
}

// ── Normalize/denormalize helpers (pure functions, no external state) ─────────
fn seed_to_01(v)      { (v / 9999.0).max(0.0).min(1.0) }
fn seed_from_01(v)    { v * 9999.0 }
fn ocean_to_01(v)     { ((v - 0.01) / 0.98).max(0.0).min(1.0) }
fn ocean_from_01(v)   { 0.01 + v * 0.98 }
fn cscale_to_01(v)    { ((v - 0.5) / 9.5).max(0.0).min(1.0) }
fn cscale_from_01(v)  { 0.5 + v * 9.5 }
fn cwarp_to_01(v)     { (v / 2.0).max(0.0).min(1.0) }
fn cwarp_from_01(v)   { v * 2.0 }
fn coct_to_01(v)      { ((v - 1.0) / 7.0).max(0.0).min(1.0) }
fn coct_from_01(v)    { (v * 7.0 + 1.0).round().max(1.0).min(8.0) }
fn mscale_to_01(v)    { ((v - 1.0) / 14.0).max(0.0).min(1.0) }
fn mscale_from_01(v)  { 1.0 + v * 14.0 }
fn mstr_to_01(v)      { v.max(0.0).min(1.0) }
fn mstr_from_01(v)    { v }
fn mroct_to_01(v)     { ((v - 1.0) / 7.0).max(0.0).min(1.0) }
fn mroct_from_01(v)   { (v * 7.0 + 1.0).round().max(1.0).min(8.0) }
fn moisture_to_01(v)  { ((v - 0.5) / 7.5).max(0.0).min(1.0) }
fn moisture_from_01(v){ 0.5 + v * 7.5 }
fn ice_to_01(v)       { (v / 3.0).max(0.0).min(1.0) }
fn ice_from_01(v)     { v * 3.0 }
fn lapse_to_01(v)     { (v / 1.5).max(0.0).min(1.0) }
fn lapse_from_01(v)   { v * 1.5 }
fn rain_to_01(v)      { v.max(0.0).min(1.0) }
fn rain_from_01(v)    { v }
fn subdiv_to_01(v)    { if v <= 48.0 { 0.0 } else if v <= 96.0 { 0.25 } else if v <= 192.0 { 0.5 } else if v <= 384.0 { 0.75 } else { 1.0 } }
fn subdiv_from_01(v)  { if v < 0.125 { 32.0 } else if v < 0.375 { 64.0 } else if v < 0.625 { 128.0 } else if v < 0.875 { 256.0 } else { 512.0 } }
fn disp_to_01(v)      { (v / 0.6).max(0.0).min(1.0) }
fn disp_from_01(v)    { v * 0.6 }
fn coloring_to_01(v)  { (v / 2.0).max(0.0).min(1.0) }
fn coloring_from_01(v){ (v * 2.0).round().max(0.0).min(2.0) }
fn rotspeed_to_01(v)  { (v / 10.0).max(0.0).min(1.0) }
fn rotspeed_from_01(v){ v * 10.0 }
fn sunaz_to_01(v)     { (v / 360.0).max(0.0).min(1.0) }
fn sunaz_from_01(v)   { v * 360.0 }
fn sunel_to_01(v)     { ((v + 10.0) / 95.0).max(0.0).min(1.0) }
fn sunel_from_01(v)   { v * 95.0 - 10.0 }
fn ambient_to_01(v)   { (v / 0.5).max(0.0).min(1.0) }
fn ambient_from_01(v) { v * 0.5 }
fn atmo_color_to_01(v)      { (v / 5.0).max(0.0).min(1.0) }
fn atmo_color_from_01(v)    { (v * 5.0).round().max(0.0).min(5.0) }
fn atmo_color_name(v)       { if v < 0.5 { "AMBER" } else if v < 1.5 { "ROSE" } else if v < 2.5 { "CYAN" } else if v < 3.5 { "LIME" } else if v < 4.5 { "AZURE" } else { "VIOLET" } }
fn atmo_color_hex(v)        { if v < 0.5 { "#d9b36a" } else if v < 1.5 { "#cf7a8c" } else if v < 2.5 { "#7cc8ff" } else if v < 3.5 { "#9bcf86" } else if v < 4.5 { "#9ab8ff" } else { "#b79dff" } }
fn atmo_strength_to_01(v)   { v.max(0.0).min(1.0) }
fn atmo_strength_from_01(v) { v.max(0.0).min(1.0) }
fn atmo_rim_to_01(v)        { ((v - 1.0) / 7.0).max(0.0).min(1.0) }
fn atmo_rim_from_01(v)      { 1.0 + v * 7.0 }
fn atmo_haze_to_01(v)       { v.max(0.0).min(1.0) }
fn atmo_haze_from_01(v)     { v.max(0.0).min(1.0) }
fn atmo_haze_power_to_01(v) { ((v - 0.5) / 3.5).max(0.0).min(1.0) }
fn atmo_haze_power_from_01(v) { 0.5 + v * 3.5 }

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.tab = 0;
    let p = preset_earth();
    local.seed = p.seed; local.ocean = p.ocean; local.cscale = p.cscale; local.cwarp = p.cwarp;
    local.coct = p.coct; local.mscale = p.mscale; local.mstr = p.mstr; local.mroct = p.mroct;
    local.moisture = p.moisture; local.ice = p.ice; local.lapse = p.lapse; local.rain = p.rain;
    local.disp = p.disp; local.coloring = p.coloring; local.rotspeed = p.rotspeed;
    local.sunaz = p.sunaz; local.sunel = p.sunel; local.ambient = p.ambient;
    local.atmo_color = p.atmo_color; local.atmo_strength = p.atmo_strength; local.atmo_rim_power = p.atmo_rim_power;
    local.atmo_haze_strength = p.atmo_haze_strength; local.atmo_haze_power = p.atmo_haze_power;
    local.subdiv = 32.0;
    local.rotation_paused = false;
    local.tab_sync_needed = true;
}

let tab  = local.tab  ?? 0;
let seed = local.seed ?? 0.0;
let ocean    = local.ocean    ?? 0.55;
let cscale   = local.cscale   ?? 2.5;
let cwarp    = local.cwarp    ?? 0.65;
let coct     = local.coct     ?? 5.0;
let mscale   = local.mscale   ?? 6.0;
let mstr     = local.mstr     ?? 0.45;
let mroct    = local.mroct    ?? 5.0;
let moisture = local.moisture ?? 3.0;
let ice      = local.ice      ?? 1.0;
let lapse    = local.lapse    ?? 0.6;
let rain     = local.rain     ?? 0.4;
let disp     = local.disp     ?? 0.22;
let subdiv   = local.subdiv   ?? 32.0;
let coloring = local.coloring ?? 0.0;
let rotspeed = local.rotspeed ?? 3.0;
let sunaz    = local.sunaz    ?? 210.0;
let sunel    = local.sunel    ?? 40.0;
let ambient  = local.ambient  ?? 0.12;
let atmo_color = local.atmo_color ?? 2.0;
let atmo_strength = local.atmo_strength ?? 0.55;
let atmo_rim_power = local.atmo_rim_power ?? 3.2;
let atmo_haze_strength = local.atmo_haze_strength ?? 0.38;
let atmo_haze_power = local.atmo_haze_power ?? 1.5;
let rotation_paused   = local.rotation_paused   ?? false;

// ── Tab switching (keyboard: 1/2/3/4/5 — mouse: click tab button) ────────────
if input.just_pressed("1") || gui.button_clicked("tab-btn-continents") { tab = 0; local.tab = 0; local.tab_sync_needed = true; }
if input.just_pressed("2") || gui.button_clicked("tab-btn-mountains")  { tab = 1; local.tab = 1; local.tab_sync_needed = true; }
if input.just_pressed("3") || gui.button_clicked("tab-btn-climate")    { tab = 2; local.tab = 2; local.tab_sync_needed = true; }
if input.just_pressed("4") || gui.button_clicked("tab-btn-visual")     { tab = 3; local.tab = 3; local.tab_sync_needed = true; }
if input.just_pressed("5") || gui.button_clicked("tab-btn-atmo")       { tab = 4; local.tab = 4; local.tab_sync_needed = true; }

// Highlight active tab: bright green = active, hover green = hovered, dim = inactive
let tab_fg_0 = if tab == 0 { "#aaffcc" } else if gui.widget_hovered("tab-btn-continents") { "#66cc88" } else { "#2a9440" };
let tab_fg_1 = if tab == 1 { "#aaffcc" } else if gui.widget_hovered("tab-btn-mountains")  { "#66cc88" } else { "#2a9440" };
let tab_fg_2 = if tab == 2 { "#aaffcc" } else if gui.widget_hovered("tab-btn-climate")    { "#66cc88" } else { "#2a9440" };
let tab_fg_3 = if tab == 3 { "#aaffcc" } else if gui.widget_hovered("tab-btn-visual")     { "#66cc88" } else { "#2a9440" };
let tab_fg_4 = if tab == 4 { "#aaffcc" } else if gui.widget_hovered("tab-btn-atmo")       { "#66cc88" } else { "#2a9440" };
scene.set("tab-btn-continents", "text.fg", tab_fg_0);
scene.set("tab-btn-mountains",  "text.fg", tab_fg_1);
scene.set("tab-btn-climate",    "text.fg", tab_fg_2);
scene.set("tab-btn-visual",     "text.fg", tab_fg_3);
scene.set("tab-btn-atmo",       "text.fg", tab_fg_4);

// Action/preset hover colors
scene.set("btn-randomize", "text.fg", if gui.widget_hovered("btn-randomize") { "#ffe060" } else { "#ffc030" });
scene.set("btn-reset", "text.fg", if gui.widget_hovered("btn-reset") { "#80b0ff" } else { "#4080ff" });
scene.set("preset-earth", "text.fg", if gui.widget_hovered("preset-btn-earth") { "#8cc08c" } else { "#507050" });
scene.set("preset-mars", "text.fg", if gui.widget_hovered("preset-btn-mars") { "#8cc08c" } else { "#507050" });
scene.set("preset-ocean", "text.fg", if gui.widget_hovered("preset-btn-ocean") { "#8cc08c" } else { "#507050" });
scene.set("preset-desert", "text.fg", if gui.widget_hovered("preset-btn-desert") { "#8cc08c" } else { "#507050" });
scene.set("preset-ice", "text.fg", if gui.widget_hovered("preset-btn-ice") { "#8cc08c" } else { "#507050" });
scene.set("preset-volcanic", "text.fg", if gui.widget_hovered("preset-btn-volcanic") { "#8cc08c" } else { "#507050" });
scene.set("preset-archipelago", "text.fg", if gui.widget_hovered("preset-btn-archipelago") { "#8cc08c" } else { "#507050" });

// ── Rotation toggle (R) ───────────────────────────────────────────────────────
if input.just_pressed("r") {
    local.rotation_paused = !(local.rotation_paused ?? false);
}

// ── Preset keys (F1-F7) ───────────────────────────────────────────────────────
let preset_applied = false;

if input.just_pressed("F1") || gui.button_clicked("preset-btn-earth") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F2") || gui.button_clicked("preset-btn-mars") {
    let p = preset_mars();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F3") || gui.button_clicked("preset-btn-ocean") {
    let p = preset_ocean();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F4") || gui.button_clicked("preset-btn-desert") {
    let p = preset_desert();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F5") || gui.button_clicked("preset-btn-ice") {
    let p = preset_ice();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F6") || gui.button_clicked("preset-btn-volcanic") {
    let p = preset_volcanic();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}
if input.just_pressed("F7") || gui.button_clicked("preset-btn-archipelago") {
    let p = preset_archipelago();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    preset_applied = true;
}

// ── Randomize (GUI button only) ───────────────────────────────────────────────
if gui.button_clicked("btn-randomize") {
    local.seed     = (seed * 1234.5 + 6789.0) % 9999.0;
    local.ocean    = 0.2 + ((local.seed * 0.618) % 0.7);
    local.cscale   = 1.0 + ((local.seed * 0.382) % 9.0);
    local.cwarp    = (local.seed * 0.271) % 2.0;
    local.coct     = 3.0 + ((local.seed * 0.157) % 5.0).round();
    local.mscale   = 2.0 + ((local.seed * 0.493) % 10.0);
    local.mstr     = (local.seed * 0.311) % 1.0;
    local.mroct    = 3.0 + ((local.seed * 0.229) % 5.0).round();
    local.moisture = 1.0 + ((local.seed * 0.441) % 5.0);
    local.ice      = (local.seed * 0.173) % 2.0;
    local.lapse    = 0.2 + ((local.seed * 0.337) % 0.8);
    local.rain     = (local.seed * 0.251) % 0.8;
    local.disp     = (local.seed * 0.197) % 0.6;
    local.rotspeed = 1.0 + ((local.seed * 0.283) % 6.0);
    local.sunaz    = (local.seed * 0.431) % 360.0;
    local.sunel    = 10.0 + ((local.seed * 0.317) % 70.0);
    local.ambient  = 0.05 + ((local.seed * 0.211) % 0.25);
    local.atmo_color = ((local.seed * 0.173) % 6.0).floor();
    local.atmo_strength = (local.seed * 0.137) % 0.65;
    local.atmo_rim_power = 1.5 + ((local.seed * 0.191) % 5.5);
    local.atmo_haze_strength = (local.seed * 0.227) % 0.55;
    local.atmo_haze_power = 0.9 + ((local.seed * 0.149) % 2.4);
    preset_applied = true;
}

// ── Reset to Earth defaults (Delete) ─────────────────────────────────────────
if input.just_pressed("Delete") || gui.button_clicked("btn-reset") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.atmo_color=p.atmo_color; local.atmo_strength=p.atmo_strength; local.atmo_rim_power=p.atmo_rim_power; local.atmo_haze_strength=p.atmo_haze_strength; local.atmo_haze_power=p.atmo_haze_power;
    local.tab = 0; tab = 0;
    preset_applied = true;
}

// Re-read local after possible preset changes
seed     = local.seed     ?? seed;
ocean    = local.ocean    ?? ocean;
cscale   = local.cscale   ?? cscale;
cwarp    = local.cwarp    ?? cwarp;
coct     = local.coct     ?? coct;
mscale   = local.mscale   ?? mscale;
mstr     = local.mstr     ?? mstr;
mroct    = local.mroct    ?? mroct;
moisture = local.moisture ?? moisture;
ice      = local.ice      ?? ice;
lapse    = local.lapse    ?? lapse;
rain     = local.rain     ?? rain;
disp     = local.disp     ?? disp;
subdiv   = local.subdiv   ?? subdiv;
coloring = local.coloring ?? coloring;
rotspeed = local.rotspeed ?? rotspeed;
sunaz    = local.sunaz    ?? sunaz;
sunel    = local.sunel    ?? sunel;
ambient  = local.ambient  ?? ambient;
atmo_color = local.atmo_color ?? atmo_color;
atmo_strength = local.atmo_strength ?? atmo_strength;
atmo_rim_power = local.atmo_rim_power ?? atmo_rim_power;
atmo_haze_strength = local.atmo_haze_strength ?? atmo_haze_strength;
atmo_haze_power = local.atmo_haze_power ?? atmo_haze_power;

// ── Sync slider widgets → local state (tab switch or preset) ──────────────────
// When tab switches or preset fires, push current local values to gui sliders.
// This updates the visual handle positions and gui internal state.
let need_sync = preset_applied || (local.tab_sync_needed ?? false);
if need_sync {
    if tab == 0 {
        gui.set_widget_value("param-0", seed_to_01(seed));
        gui.set_widget_value("param-1", ocean_to_01(ocean));
        gui.set_widget_value("param-2", cscale_to_01(cscale));
        gui.set_widget_value("param-3", cwarp_to_01(cwarp));
        gui.set_widget_value("param-4", coct_to_01(coct));
    } else if tab == 1 {
        gui.set_widget_value("param-0", mscale_to_01(mscale));
        gui.set_widget_value("param-1", mstr_to_01(mstr));
        gui.set_widget_value("param-2", mroct_to_01(mroct));
    } else if tab == 2 {
        gui.set_widget_value("param-0", moisture_to_01(moisture));
        gui.set_widget_value("param-1", ice_to_01(ice));
        gui.set_widget_value("param-2", lapse_to_01(lapse));
        gui.set_widget_value("param-3", rain_to_01(rain));
    } else if tab == 3 {
        gui.set_widget_value("param-0", subdiv_to_01(subdiv));
        gui.set_widget_value("param-1", disp_to_01(disp));
        gui.set_widget_value("param-2", coloring_to_01(coloring));
        gui.set_widget_value("param-3", rotspeed_to_01(rotspeed));
        gui.set_widget_value("param-4", sunaz_to_01(sunaz));
        gui.set_widget_value("param-5", sunel_to_01(sunel));
        gui.set_widget_value("param-6", ambient_to_01(ambient));
    } else {
        gui.set_widget_value("param-0", atmo_color_to_01(atmo_color));
        gui.set_widget_value("param-1", atmo_strength_to_01(atmo_strength));
        gui.set_widget_value("param-2", atmo_rim_to_01(atmo_rim_power));
        gui.set_widget_value("param-3", atmo_haze_to_01(atmo_haze_strength));
        gui.set_widget_value("param-4", atmo_haze_power_to_01(atmo_haze_power));
    }
    local.tab_sync_needed = false;
}

// ── Read slider values → local state (normal mouse drag frames) ───────────────
// gui.slider_value() returns the value from this frame's gui state (after mouse events).
// Skip on sync frames (preset/tab-switch) — local.* is already correct.
if !need_sync {
    if tab == 0 {
        seed     = seed_from_01(gui.slider_value("param-0"));
        ocean    = ocean_from_01(gui.slider_value("param-1"));
        cscale   = cscale_from_01(gui.slider_value("param-2"));
        cwarp    = cwarp_from_01(gui.slider_value("param-3"));
        let raw_coct = gui.slider_value("param-4");
        coct     = coct_from_01(raw_coct);
        let snap_coct = coct_to_01(coct);
        if (raw_coct - snap_coct).abs() > 0.005 { gui.set_widget_value("param-4", snap_coct); }
        local.seed = seed; local.ocean = ocean; local.cscale = cscale;
        local.cwarp = cwarp; local.coct = coct;
    } else if tab == 1 {
        mscale   = mscale_from_01(gui.slider_value("param-0"));
        mstr     = mstr_from_01(gui.slider_value("param-1"));
        let raw_mroct = gui.slider_value("param-2");
        mroct    = mroct_from_01(raw_mroct);
        let snap_mroct = mroct_to_01(mroct);
        if (raw_mroct - snap_mroct).abs() > 0.005 { gui.set_widget_value("param-2", snap_mroct); }
        local.mscale = mscale; local.mstr = mstr; local.mroct = mroct;
    } else if tab == 2 {
        moisture = moisture_from_01(gui.slider_value("param-0"));
        ice      = ice_from_01(gui.slider_value("param-1"));
        lapse    = lapse_from_01(gui.slider_value("param-2"));
        rain     = rain_from_01(gui.slider_value("param-3"));
        local.moisture = moisture; local.ice = ice; local.lapse = lapse; local.rain = rain;
    } else if tab == 3 {
        let raw_subdiv = gui.slider_value("param-0");
        subdiv   = subdiv_from_01(raw_subdiv);
        let snap_subdiv = subdiv_to_01(subdiv);
        if (raw_subdiv - snap_subdiv).abs() > 0.005 { gui.set_widget_value("param-0", snap_subdiv); }
        disp     = disp_from_01(gui.slider_value("param-1"));
        let raw_coloring = gui.slider_value("param-2");
        coloring = coloring_from_01(raw_coloring);
        let snap_coloring = coloring_to_01(coloring);
        if (raw_coloring - snap_coloring).abs() > 0.005 { gui.set_widget_value("param-2", snap_coloring); }
        rotspeed = rotspeed_from_01(gui.slider_value("param-3"));
        sunaz    = sunaz_from_01(gui.slider_value("param-4"));
        sunel    = sunel_from_01(gui.slider_value("param-5"));
        ambient  = ambient_from_01(gui.slider_value("param-6"));
        local.subdiv = subdiv; local.disp = disp; local.coloring = coloring;
        local.rotspeed = rotspeed; local.sunaz = sunaz; local.sunel = sunel; local.ambient = ambient;
    } else {
        let raw_atmo_color = gui.slider_value("param-0");
        atmo_color = atmo_color_from_01(raw_atmo_color);
        let snap_atmo_color = atmo_color_to_01(atmo_color);
        if (raw_atmo_color - snap_atmo_color).abs() > 0.005 { gui.set_widget_value("param-0", snap_atmo_color); }
        atmo_strength = atmo_strength_from_01(gui.slider_value("param-1"));
        atmo_rim_power = atmo_rim_from_01(gui.slider_value("param-2"));
        atmo_haze_strength = atmo_haze_from_01(gui.slider_value("param-3"));
        atmo_haze_power   = atmo_haze_power_from_01(gui.slider_value("param-4"));
        local.atmo_color = atmo_color; local.atmo_strength = atmo_strength; local.atmo_rim_power = atmo_rim_power;
        local.atmo_haze_strength = atmo_haze_strength; local.atmo_haze_power = atmo_haze_power;
    }
}

// ── Slot visibility (hide unused slots per tab) ───────────────────────────────
let NUM_PARAMS = if tab == 0 { 5 } else if tab == 1 { 3 } else if tab == 2 { 4 } else if tab == 3 { 7 } else { 5 };
for i in 0..8 {
    let vis = i < NUM_PARAMS;
    scene.set(`param-label-${i}`,  "visible", vis);
    scene.set(`param-track-${i}`,  "visible", vis);
    scene.set(`param-handle-${i}`, "visible", vis);
}

// ── Update labels & title per active tab ──────────────────────────────────────
if tab == 0 {
    scene.set("param-title", "text.content", "CONTINENTS");
    scene.set("param-label-0", "text.content", `  SEED           ${seed.round()}`);
    scene.set("param-label-1", "text.content", `  OCEAN          ${(ocean * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", `  CONTINENT SZ   ${(cscale * 10.0).round() / 10.0}`);
    scene.set("param-label-3", "text.content", `  COAST CHAOS    ${(cwarp * 10.0).round() / 10.0}`);
    scene.set("param-label-4", "text.content", `  OCTAVES        ${coct.round()}`);
} else if tab == 1 {
    scene.set("param-title", "text.content", "MOUNTAINS");
    scene.set("param-label-0", "text.content", `  MTN SPACING    ${(mscale * 10.0).round() / 10.0}`);
    scene.set("param-label-1", "text.content", `  MTN HEIGHT     ${(mstr * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", `  RIDGE DETAIL   ${mroct.round()}`);
} else if tab == 2 {
    scene.set("param-title", "text.content", "CLIMATE");
    scene.set("param-label-0", "text.content", `  MOISTURE SZ    ${(moisture * 10.0).round() / 10.0}`);
    scene.set("param-label-1", "text.content", `  ICE CAPS       ${(ice * 10.0).round() / 10.0}`);
    scene.set("param-label-2", "text.content", `  ALT COOLING    ${(lapse * 10.0).round() / 10.0}`);
    scene.set("param-label-3", "text.content", `  RAIN SHADOW    ${(rain * 10.0).round() / 10.0}`);
} else if tab == 3 {
    let coloring_name = if coloring < 0.5 { "BIOME" } else if coloring < 1.5 { "ELEV" } else { "MOIST" };
    scene.set("param-title", "text.content", "VISUAL");
    scene.set("param-label-0", "text.content", `  RESOLUTION     ${subdiv.round()}`);
    scene.set("param-label-1", "text.content", `  DISPLACEMENT   ${(disp * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", `  COLORING  [${coloring_name}]`);
    scene.set("param-label-3", "text.content", `  ROTATION       ${(rotspeed * 10.0).round() / 10.0}`);
    scene.set("param-label-4", "text.content", `  SUN AZIMUTH    ${sunaz.round()}`);
    scene.set("param-label-5", "text.content", `  SUN ELEVATION   ${sunel.round()}`);
    scene.set("param-label-6", "text.content", `  AMBIENT        ${(ambient * 100.0).round()}%`);
} else {
    let atmo_color_label = atmo_color_name(atmo_color);
    scene.set("param-title", "text.content", "ATMOSPHERE");
    scene.set("param-label-0", "text.content", `  COLOR      [${atmo_color_label}]`);
    scene.set("param-label-1", "text.content", `  DENSITY        ${(atmo_strength * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", `  RIM POWER      ${(atmo_rim_power * 10.0).round() / 10.0}`);
    scene.set("param-label-3", "text.content", `  HAZE DENSITY   ${(atmo_haze_strength * 100.0).round()}%`);
    scene.set("param-label-4", "text.content", `  HAZE POWER     ${(atmo_haze_power * 10.0).round() / 10.0}`);
}

// ── Stats strip ───────────────────────────────────────────────────────────────
scene.set("stats-seed", "text.content", `SEED: ${seed.round()}`);
let ps = planet_last_stats();
if ps.len() > 0 {
    let ocean_pct  = (ps["ocean"]  * 100.0).round();
    let forest_pct = (ps["forest"] * 100.0).round();
    let desert_pct = (ps["desert"] * 100.0).round();
    let cold_pct   = (ps["cold"]   * 100.0).round();
    scene.set("stats-biomes", "text.content",
        `OCEAN ${ocean_pct}%  FOREST ${forest_pct}%  DESERT ${desert_pct}%  SNOW ${cold_pct}%`);
}

// ── Push world params to planet mesh (throttled to ≤ THROTTLE_MS between pushes) ──
// Push immediately on first frame, then only when dirty and enough time has passed.
let THROTTLE_MS = 150.0;

let prev_seed    = local.prev_seed    ?? seed;
let prev_ocean   = local.prev_ocean   ?? ocean;
let prev_cscale  = local.prev_cscale  ?? cscale;
let prev_cwarp   = local.prev_cwarp   ?? cwarp;
let prev_coct    = local.prev_coct    ?? coct;
let prev_mscale  = local.prev_mscale  ?? mscale;
let prev_mstr    = local.prev_mstr    ?? mstr;
let prev_mroct   = local.prev_mroct   ?? mroct;
let prev_moisture= local.prev_moisture?? moisture;
let prev_ice     = local.prev_ice     ?? ice;
let prev_lapse   = local.prev_lapse   ?? lapse;
let prev_rain    = local.prev_rain    ?? rain;
let prev_disp    = local.prev_disp    ?? disp;
let prev_subdiv  = local.prev_subdiv  ?? subdiv;
let prev_coloring= local.prev_coloring?? coloring;

let world_changed = (seed != prev_seed) || (ocean != prev_ocean) || (cscale != prev_cscale)
    || (cwarp != prev_cwarp) || (coct != prev_coct) || (mscale != prev_mscale)
    || (mstr != prev_mstr) || (mroct != prev_mroct) || (moisture != prev_moisture)
    || (ice != prev_ice) || (lapse != prev_lapse) || (rain != prev_rain)
    || (disp != prev_disp) || (subdiv != prev_subdiv) || (coloring != prev_coloring);

local.prev_seed = seed; local.prev_ocean = ocean; local.prev_cscale = cscale;
local.prev_cwarp = cwarp; local.prev_coct = coct; local.prev_mscale = mscale;
local.prev_mstr = mstr; local.prev_mroct = mroct; local.prev_moisture = moisture;
local.prev_ice = ice; local.prev_lapse = lapse; local.prev_rain = rain;
local.prev_disp = disp; local.prev_subdiv = subdiv; local.prev_coloring = coloring;

if !(local.push_ready ?? false) {
    local.push_ready = true;
    local.push_timer = THROTTLE_MS;  // Start ready to push on frame 1
    local.push_dirty = true;
}

local.push_timer = (local.push_timer ?? THROTTLE_MS) + frame_ms.to_float();

if world_changed {
    local.push_dirty = true;
}

let should_push = (local.push_dirty ?? false) && local.push_timer >= THROTTLE_MS;

if should_push {
    local.push_dirty = false;
    local.push_timer = 0.0;
    let coloring_str = if coloring < 0.5 { "biome" } else if coloring < 1.5 { "elevation" } else { "moisture" };
    scene.set("planet-mesh", "world.seed",               seed);
    scene.set("planet-mesh", "world.ocean_fraction",     ocean);
    scene.set("planet-mesh", "world.continent_scale",    cscale);
    scene.set("planet-mesh", "world.continent_warp",     cwarp);
    scene.set("planet-mesh", "world.continent_octaves",  coct);
    scene.set("planet-mesh", "world.mountain_scale",         mscale);
    scene.set("planet-mesh", "world.mountain_strength",      mstr);
    scene.set("planet-mesh", "world.mountain_ridge_octaves", mroct);
    scene.set("planet-mesh", "world.moisture_scale",         moisture);
    scene.set("planet-mesh", "world.ice_cap_strength",       ice);
    scene.set("planet-mesh", "world.lapse_rate",             lapse);
    scene.set("planet-mesh", "world.rain_shadow",            rain);
    scene.set("planet-mesh", "world.displacement_scale",     disp);
    scene.set("planet-mesh", "world.subdivisions",           subdiv);
    scene.set("planet-mesh", "world.shape",                  "sphere");
    scene.set("planet-mesh", "world.base",                   "cube");
    scene.set("planet-mesh", "world.coloring",               coloring_str);
}

// Visual-only params always update immediately (no mesh rebuild)
let effective_rotspeed = if local.rotation_paused ?? false { 0.0 } else { rotspeed };
scene.set("planet-mesh", "obj.rotation-speed", effective_rotspeed);
scene.set("planet-mesh", "obj.ambient",         ambient);
scene.set("planet-mesh", "obj.atmo.color",         atmo_color_hex(atmo_color));
scene.set("planet-mesh", "obj.atmo.strength",      atmo_strength);
scene.set("planet-mesh", "obj.atmo.rim_power",     atmo_rim_power);
scene.set("planet-mesh", "obj.atmo.haze_strength", atmo_haze_strength);
scene.set("planet-mesh", "obj.atmo.haze_power",    atmo_haze_power);

let az_rad = sunaz * 0.01745329;
let el_rad = sunel * 0.01745329;
scene.set("planet-mesh", "obj.light.x", -(az_rad.cos() * el_rad.cos()));
scene.set("planet-mesh", "obj.light.y", el_rad.sin());
scene.set("planet-mesh", "obj.light.z", -(az_rad.sin() * el_rad.cos()));

