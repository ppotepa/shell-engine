// ── Planet Generator ─────────────────────────────────────────────────────────
// 1/2/3/4  → switch tab (Continents / Mountains / Climate / Visual)
// F1-F7    → load preset (Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago)
// R        → randomize all params
// Delete   → reset to Earth defaults
// Mouse    → drag slider handles to adjust parameters
// Ctrl+F   → toggle orbit / free-look camera (handled by engine)
//
// 7 multiplexed slider slots: content remapped per active tab.
// All gui widgets use min=0, max=1 (normalized). Rhai maps to/from param domains.

// ── Preset definitions ────────────────────────────────────────────────────────
fn preset_earth() {
    #{
        seed: 0.0, ocean: 0.55, cscale: 2.5, cwarp: 0.65, coct: 5.0,
        mscale: 6.0, mstr: 0.45, mroct: 5.0,
        moisture: 3.0, ice: 1.0, lapse: 0.6, rain: 0.4,
        disp: 0.22, coloring: 0.0, rotspeed: 3.0,
        sunaz: 210.0, sunel: 40.0, ambient: 0.12
    }
}
fn preset_mars() {
    #{
        seed: 42.0, ocean: 0.05, cscale: 3.5, cwarp: 0.3, coct: 4.0,
        mscale: 8.0, mstr: 0.65, mroct: 6.0,
        moisture: 1.0, ice: 0.1, lapse: 0.3, rain: 0.05,
        disp: 0.38, coloring: 1.0, rotspeed: 3.0,
        sunaz: 190.0, sunel: 30.0, ambient: 0.08
    }
}
fn preset_ocean() {
    #{
        seed: 7.0, ocean: 0.88, cscale: 1.5, cwarp: 1.2, coct: 3.0,
        mscale: 4.0, mstr: 0.15, mroct: 3.0,
        moisture: 5.0, ice: 0.8, lapse: 0.7, rain: 0.6,
        disp: 0.08, coloring: 0.0, rotspeed: 2.0,
        sunaz: 220.0, sunel: 50.0, ambient: 0.15
    }
}
fn preset_desert() {
    #{
        seed: 13.0, ocean: 0.15, cscale: 4.0, cwarp: 0.2, coct: 3.0,
        mscale: 5.0, mstr: 0.3, mroct: 4.0,
        moisture: 1.2, ice: 0.0, lapse: 0.4, rain: 0.05,
        disp: 0.3, coloring: 0.0, rotspeed: 4.0,
        sunaz: 200.0, sunel: 25.0, ambient: 0.18
    }
}
fn preset_ice() {
    #{
        seed: 99.0, ocean: 0.45, cscale: 2.0, cwarp: 0.5, coct: 4.0,
        mscale: 5.0, mstr: 0.35, mroct: 4.0,
        moisture: 2.5, ice: 3.0, lapse: 1.0, rain: 0.3,
        disp: 0.18, coloring: 0.0, rotspeed: 1.5,
        sunaz: 230.0, sunel: 15.0, ambient: 0.25
    }
}
fn preset_volcanic() {
    #{
        seed: 66.0, ocean: 0.3, cscale: 3.0, cwarp: 1.5, coct: 6.0,
        mscale: 10.0, mstr: 0.85, mroct: 7.0,
        moisture: 1.5, ice: 0.0, lapse: 0.5, rain: 0.15,
        disp: 0.55, coloring: 0.0, rotspeed: 5.0,
        sunaz: 180.0, sunel: 35.0, ambient: 0.1
    }
}
fn preset_archipelago() {
    #{
        seed: 27.0, ocean: 0.75, cscale: 1.0, cwarp: 1.8, coct: 7.0,
        mscale: 3.0, mstr: 0.2, mroct: 3.0,
        moisture: 4.0, ice: 0.5, lapse: 0.65, rain: 0.5,
        disp: 0.12, coloring: 0.0, rotspeed: 2.5,
        sunaz: 215.0, sunel: 45.0, ambient: 0.14
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
fn subdiv_to_01(v)    { if v < 24.0 { 0.0 } else if v < 48.0 { 0.333 } else if v < 96.0 { 0.667 } else { 1.0 } }
fn subdiv_from_01(v)  { if v < 0.166 { 16.0 } else if v < 0.5 { 32.0 } else if v < 0.833 { 64.0 } else { 128.0 } }
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
    local.subdiv = 32.0;
    local.tab_sync_needed = true;
}

let tab      = local.tab      ?? 0;
let dirty_ms = local.dirty_ms ?? -1.0;
let seed     = local.seed     ?? 0.0;
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

// ── Tab switching (1/2/3/4) ───────────────────────────────────────────────────
if input.just_pressed("1") { tab = 0; local.tab = 0; local.tab_sync_needed = true; }
if input.just_pressed("2") { tab = 1; local.tab = 1; local.tab_sync_needed = true; }
if input.just_pressed("3") { tab = 2; local.tab = 2; local.tab_sync_needed = true; }
if input.just_pressed("4") { tab = 3; local.tab = 3; local.tab_sync_needed = true; }

// Highlight active tab button
scene.set("tab-btn-continents", "text.fg", if tab == 0 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-mountains",  "text.fg", if tab == 1 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-climate",    "text.fg", if tab == 2 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-visual",     "text.fg", if tab == 3 { "#aaffcc" } else { "#2a9440" });

// ── Preset keys (F1-F7) ───────────────────────────────────────────────────────
let preset_applied = false;

if input.just_pressed("F1") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F2") {
    let p = preset_mars();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F3") {
    let p = preset_ocean();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F4") {
    let p = preset_desert();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F5") {
    let p = preset_ice();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F6") {
    let p = preset_volcanic();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}
if input.just_pressed("F7") {
    let p = preset_archipelago();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    preset_applied = true;
}

// ── Randomize (R) ─────────────────────────────────────────────────────────────
if !orbit_active && input.just_pressed("r") {
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
    preset_applied = true;
}

// ── Reset to Earth defaults (Delete) ─────────────────────────────────────────
if !orbit_active && input.just_pressed("Delete") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct;
    local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
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
    } else {
        gui.set_widget_value("param-0", subdiv_to_01(subdiv));
        gui.set_widget_value("param-1", disp_to_01(disp));
        gui.set_widget_value("param-2", coloring_to_01(coloring));
        gui.set_widget_value("param-3", rotspeed_to_01(rotspeed));
        gui.set_widget_value("param-4", sunaz_to_01(sunaz));
        gui.set_widget_value("param-5", sunel_to_01(sunel));
        gui.set_widget_value("param-6", ambient_to_01(ambient));
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
        coct     = coct_from_01(gui.slider_value("param-4"));
        local.seed = seed; local.ocean = ocean; local.cscale = cscale;
        local.cwarp = cwarp; local.coct = coct;
    } else if tab == 1 {
        mscale   = mscale_from_01(gui.slider_value("param-0"));
        mstr     = mstr_from_01(gui.slider_value("param-1"));
        mroct    = mroct_from_01(gui.slider_value("param-2"));
        local.mscale = mscale; local.mstr = mstr; local.mroct = mroct;
    } else if tab == 2 {
        moisture = moisture_from_01(gui.slider_value("param-0"));
        ice      = ice_from_01(gui.slider_value("param-1"));
        lapse    = lapse_from_01(gui.slider_value("param-2"));
        rain     = rain_from_01(gui.slider_value("param-3"));
        local.moisture = moisture; local.ice = ice; local.lapse = lapse; local.rain = rain;
    } else {
        subdiv   = subdiv_from_01(gui.slider_value("param-0"));
        disp     = disp_from_01(gui.slider_value("param-1"));
        coloring = coloring_from_01(gui.slider_value("param-2"));
        rotspeed = rotspeed_from_01(gui.slider_value("param-3"));
        sunaz    = sunaz_from_01(gui.slider_value("param-4"));
        sunel    = sunel_from_01(gui.slider_value("param-5"));
        ambient  = ambient_from_01(gui.slider_value("param-6"));
        local.subdiv = subdiv; local.disp = disp; local.coloring = coloring;
        local.rotspeed = rotspeed; local.sunaz = sunaz; local.sunel = sunel; local.ambient = ambient;
    }
}

// ── Slot visibility (hide unused slots per tab) ───────────────────────────────
let NUM_PARAMS = if tab == 0 { 5 } else if tab == 1 { 3 } else if tab == 2 { 4 } else { 7 };
for i in 0..7 {
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
} else {
    let coloring_name = if coloring < 0.5 { "BIOME" } else if coloring < 1.5 { "ELEV" } else { "MOIST" };
    scene.set("param-title", "text.content", "VISUAL");
    scene.set("param-label-0", "text.content", `  RESOLUTION     ${subdiv.round()}`);
    scene.set("param-label-1", "text.content", `  DISPLACEMENT   ${(disp * 100.0).round()}%`);
    scene.set("param-label-2", "text.content", `  COLORING  [${coloring_name}]`);
    scene.set("param-label-3", "text.content", `  ROTATION       ${(rotspeed * 10.0).round() / 10.0}`);
    scene.set("param-label-4", "text.content", `  SUN AZIMUTH    ${sunaz.round()}`);
    scene.set("param-label-5", "text.content", `  SUN ELEVATION   ${sunel.round()}`);
    scene.set("param-label-6", "text.content", `  AMBIENT        ${(ambient * 100.0).round()}%`);
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

// ── Push world params to planet mesh (throttled) ─────────────────────────────
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

if world_changed {
    dirty_ms = dirty_ms + frame_ms.to_float();
    local.dirty_ms = dirty_ms;
} else if dirty_ms > 0.0 {
    dirty_ms = dirty_ms + frame_ms.to_float();
    local.dirty_ms = dirty_ms;
}

let THROTTLE_MS = 500.0;
let should_push = (dirty_ms < 0.0)
    || (world_changed && dirty_ms >= THROTTLE_MS)
    || (!world_changed && dirty_ms > 0.0);

if should_push {
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
    scene.set("planet-mesh", "world.coloring",               coloring_str);
    dirty_ms = 0.0;
    local.dirty_ms = 0.0;
    if !world_changed { local.dirty_ms = -1.0; }
}

// Visual-only params always update immediately (no mesh rebuild)
scene.set("planet-mesh", "obj.rotation-speed", rotspeed);
scene.set("planet-mesh", "obj.ambient",         ambient);

let az_rad = sunaz * 0.01745329;
let el_rad = sunel * 0.01745329;
scene.set("planet-mesh", "obj.light.x", -(az_rad.cos() * el_rad.cos()));
scene.set("planet-mesh", "obj.light.y", el_rad.sin());
scene.set("planet-mesh", "obj.light.z", -(az_rad.sin() * el_rad.cos()));
