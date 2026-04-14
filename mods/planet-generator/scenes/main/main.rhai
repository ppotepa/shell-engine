// ── Planet Generator ─────────────────────────────────────────────────────────
// 1/2/3/4  → switch tab (Continents / Mountains / Climate / Visual)
// F1-F7    → load preset (Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago)
// R        → randomize all params
// Delete   → reset to Earth defaults
// ↑ / ↓   → cycle selected parameter in current tab
// ← / →   → adjust selected parameter
// Ctrl+F   → toggle orbit / free-look camera (handled by engine)
//
// Pushes world.* params to "planet-mesh" via scene.set on every change.

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

// ── Persistent state ──────────────────────────────────────────────────────────
if !(local.initialized ?? false) {
    local.initialized = true;
    local.tab         = 0;
    local.sel         = 0;
    let p = preset_earth();
    local.seed = p.seed; local.ocean = p.ocean; local.cscale = p.cscale; local.cwarp = p.cwarp;
    local.coct = p.coct; local.mscale = p.mscale; local.mstr = p.mstr; local.mroct = p.mroct;
    local.moisture = p.moisture; local.ice = p.ice; local.lapse = p.lapse; local.rain = p.rain;
    local.disp = p.disp; local.coloring = p.coloring; local.rotspeed = p.rotspeed;
    local.sunaz = p.sunaz; local.sunel = p.sunel; local.ambient = p.ambient;
}

let tab      = local.tab      ?? 0;
let sel      = local.sel      ?? 0;
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
let coloring = local.coloring ?? 0.0;  // 0=biome, 1=elevation, 2=moisture
let rotspeed = local.rotspeed ?? 3.0;
let sunaz    = local.sunaz    ?? 210.0;
let sunel    = local.sunel    ?? 40.0;
let ambient  = local.ambient  ?? 0.12;

// ── Tab switching (1/2/3/4) ───────────────────────────────────────────────────
if input.just_pressed("1") { tab = 0; sel = 0; local.tab = 0; local.sel = 0; }
if input.just_pressed("2") { tab = 1; sel = 0; local.tab = 1; local.sel = 0; }
if input.just_pressed("3") { tab = 2; sel = 0; local.tab = 2; local.sel = 0; }
if input.just_pressed("4") { tab = 3; sel = 0; local.tab = 3; local.sel = 0; }

scene.set("panel-continents", "visible", tab == 0);
scene.set("panel-mountains",  "visible", tab == 1);
scene.set("panel-climate",    "visible", tab == 2);
scene.set("panel-visual",     "visible", tab == 3);

// Highlight active tab button
scene.set("tab-btn-continents", "text.fg", if tab == 0 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-mountains",  "text.fg", if tab == 1 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-climate",    "text.fg", if tab == 2 { "#aaffcc" } else { "#2a9440" });
scene.set("tab-btn-visual",     "text.fg", if tab == 3 { "#aaffcc" } else { "#2a9440" });

// ── Preset keys (F1–F7) ───────────────────────────────────────────────────────

if input.just_pressed("F1") { let p = preset_earth();       local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F2") { let p = preset_mars();        local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F3") { let p = preset_ocean();       local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F4") { let p = preset_desert();      local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F5") { let p = preset_ice();         local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F6") { let p = preset_volcanic();    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }
if input.just_pressed("F7") { let p = preset_archipelago(); local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp; local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct; local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain; local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed; local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient; local.sel=0; }

// Highlight active preset strip (F1=Earth, F2=Mars, ... F7=Archipelago)

// ── Randomize (R) ─────────────────────────────────────────────────────────────
if !orbit_active && input.just_pressed("r") {
    local.seed     = (seed * 1234.5 + 6789.0) % 9999.0;  // deterministic shuffle
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
}

// ── Reset to Earth defaults (Delete) ─────────────────────────────────────────
if !orbit_active && input.just_pressed("Delete") {
    let p = preset_earth();
    local.seed=p.seed; local.ocean=p.ocean; local.cscale=p.cscale; local.cwarp=p.cwarp;
    local.coct=p.coct; local.mscale=p.mscale; local.mstr=p.mstr; local.mroct=p.mroct;
    local.moisture=p.moisture; local.ice=p.ice; local.lapse=p.lapse; local.rain=p.rain;
    local.disp=p.disp; local.coloring=p.coloring; local.rotspeed=p.rotspeed;
    local.sunaz=p.sunaz; local.sunel=p.sunel; local.ambient=p.ambient;
    local.tab = 0; local.sel = 0;
}

// Re-read after possible preset/randomize changes
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
coloring = local.coloring ?? coloring;
rotspeed = local.rotspeed ?? rotspeed;
sunaz    = local.sunaz    ?? sunaz;
sunel    = local.sunel    ?? sunel;
ambient  = local.ambient  ?? ambient;

// ── Parameter adjustment (↑↓ ←→) ─────────────────────────────────────────────
if !orbit_active {
    let dt = frame_ms.to_float() / 1000.0;

    // Tab-local param counts
    let NUM_PARAMS = if tab == 0 { 5 } else if tab == 1 { 3 } else if tab == 2 { 4 } else { 6 };

    if input.just_pressed("Up")   { sel = (sel + NUM_PARAMS - 1) % NUM_PARAMS; local.sel = sel; }
    if input.just_pressed("Down") { sel = (sel + 1) % NUM_PARAMS; local.sel = sel; }

    if tab == 0 {
        // Continents
        if sel == 0 {
            if input.just_pressed("Left")  { seed = (seed - 1.0).max(0.0);    local.seed = seed; }
            if input.just_pressed("Right") { seed = (seed + 1.0).min(9999.0); local.seed = seed; }
        } else if sel == 1 {
            if input.down("Left")  { ocean = (ocean - 0.3 * dt).max(0.01); local.ocean = ocean; }
            if input.down("Right") { ocean = (ocean + 0.3 * dt).min(0.99); local.ocean = ocean; }
        } else if sel == 2 {
            if input.down("Left")  { cscale = (cscale - 1.5 * dt).max(0.5);  local.cscale = cscale; }
            if input.down("Right") { cscale = (cscale + 1.5 * dt).min(10.0); local.cscale = cscale; }
        } else if sel == 3 {
            if input.down("Left")  { cwarp = (cwarp - 0.8 * dt).max(0.0); local.cwarp = cwarp; }
            if input.down("Right") { cwarp = (cwarp + 0.8 * dt).min(2.0); local.cwarp = cwarp; }
        } else {
            if input.just_pressed("Left")  { coct = (coct - 1.0).max(1.0); local.coct = coct; }
            if input.just_pressed("Right") { coct = (coct + 1.0).min(8.0); local.coct = coct; }
        }
    } else if tab == 1 {
        // Mountains
        if sel == 0 {
            if input.down("Left")  { mscale = (mscale - 3.0 * dt).max(1.0);  local.mscale = mscale; }
            if input.down("Right") { mscale = (mscale + 3.0 * dt).min(15.0); local.mscale = mscale; }
        } else if sel == 1 {
            if input.down("Left")  { mstr = (mstr - 0.5 * dt).max(0.0); local.mstr = mstr; }
            if input.down("Right") { mstr = (mstr + 0.5 * dt).min(1.0); local.mstr = mstr; }
        } else {
            if input.just_pressed("Left")  { mroct = (mroct - 1.0).max(1.0); local.mroct = mroct; }
            if input.just_pressed("Right") { mroct = (mroct + 1.0).min(8.0); local.mroct = mroct; }
        }
    } else if tab == 2 {
        // Climate
        if sel == 0 {
            if input.down("Left")  { moisture = (moisture - 1.5 * dt).max(0.5); local.moisture = moisture; }
            if input.down("Right") { moisture = (moisture + 1.5 * dt).min(8.0); local.moisture = moisture; }
        } else if sel == 1 {
            if input.down("Left")  { ice = (ice - 0.8 * dt).max(0.0); local.ice = ice; }
            if input.down("Right") { ice = (ice + 0.8 * dt).min(3.0); local.ice = ice; }
        } else if sel == 2 {
            if input.down("Left")  { lapse = (lapse - 0.4 * dt).max(0.0); local.lapse = lapse; }
            if input.down("Right") { lapse = (lapse + 0.4 * dt).min(1.5); local.lapse = lapse; }
        } else {
            if input.down("Left")  { rain = (rain - 0.4 * dt).max(0.0); local.rain = rain; }
            if input.down("Right") { rain = (rain + 0.4 * dt).min(1.0); local.rain = rain; }
        }
    } else {
        // Visual
        if sel == 0 {
            if input.down("Left")  { disp = (disp - 0.2 * dt).max(0.0); local.disp = disp; }
            if input.down("Right") { disp = (disp + 0.2 * dt).min(0.6); local.disp = disp; }
        } else if sel == 1 {
            // cycle coloring modes: 0=biome, 1=elevation, 2=moisture
            if input.just_pressed("Left")  { coloring = (coloring - 1.0 + 3.0) % 3.0; local.coloring = coloring; }
            if input.just_pressed("Right") { coloring = (coloring + 1.0) % 3.0;        local.coloring = coloring; }
        } else if sel == 2 {
            if input.down("Left")  { rotspeed = (rotspeed - 2.0 * dt).max(0.0);  local.rotspeed = rotspeed; }
            if input.down("Right") { rotspeed = (rotspeed + 2.0 * dt).min(10.0); local.rotspeed = rotspeed; }
        } else if sel == 3 {
            if input.down("Left")  { sunaz = (sunaz - 60.0 * dt + 360.0) % 360.0; local.sunaz = sunaz; }
            if input.down("Right") { sunaz = (sunaz + 60.0 * dt) % 360.0;         local.sunaz = sunaz; }
        } else if sel == 4 {
            if input.down("Left")  { sunel = (sunel - 30.0 * dt).max(-10.0); local.sunel = sunel; }
            if input.down("Right") { sunel = (sunel + 30.0 * dt).min(85.0);  local.sunel = sunel; }
        } else {
            if input.down("Left")  { ambient = (ambient - 0.1 * dt).max(0.0);  local.ambient = ambient; }
            if input.down("Right") { ambient = (ambient + 0.1 * dt).min(0.5);  local.ambient = ambient; }
        }
    }
}

// ── Update labels & handles ───────────────────────────────────────────────────
let TRACK_W = 170.0;
let TRACK_X = 4.0;

// Selector arrows per tab
let a = [" "," "," "," "," "," "];
if !orbit_active { a[sel] = ">"; }

if tab == 0 {
    scene.set("ep-seed-label",    "text.content", `${a[0]} SEED           ${seed.round()}`);
    scene.set("ep-ocean-label",   "text.content", `${a[1]} OCEAN          ${(ocean * 100.0).round()}%`);
    scene.set("ep-cscale-label",  "text.content", `${a[2]} CONTINENT SZ   ${(cscale * 10.0).round() / 10.0}`);
    scene.set("ep-cwarp-label",   "text.content", `${a[3]} COAST CHAOS    ${(cwarp * 10.0).round() / 10.0}`);
    scene.set("ep-coct-label",    "text.content", `${a[4]} OCTAVES        ${coct.round()}`);

    scene.set("ep-seed-handle",   "position.x", (TRACK_X + seed / 9999.0 * TRACK_W).round());
    scene.set("ep-ocean-handle",  "position.x", (TRACK_X + (ocean - 0.01) / 0.98 * TRACK_W).round());
    scene.set("ep-cscale-handle", "position.x", (TRACK_X + (cscale - 0.5) / 9.5 * TRACK_W).round());
    scene.set("ep-cwarp-handle",  "position.x", (TRACK_X + cwarp / 2.0 * TRACK_W).round());
    scene.set("ep-coct-handle",   "position.x", (TRACK_X + (coct - 1.0) / 7.0 * TRACK_W).round());
} else if tab == 1 {
    scene.set("ep-mscale-label",  "text.content", `${a[0]} MTN SPACING    ${(mscale * 10.0).round() / 10.0}`);
    scene.set("ep-mstr-label",    "text.content", `${a[1]} MTN HEIGHT     ${(mstr * 100.0).round()}%`);
    scene.set("ep-mroct-label",   "text.content", `${a[2]} RIDGE DETAIL   ${mroct.round()}`);

    scene.set("ep-mscale-handle", "position.x", (TRACK_X + (mscale - 1.0) / 14.0 * TRACK_W).round());
    scene.set("ep-mstr-handle",   "position.x", (TRACK_X + mstr * TRACK_W).round());
    scene.set("ep-mroct-handle",  "position.x", (TRACK_X + (mroct - 1.0) / 7.0 * TRACK_W).round());
} else if tab == 2 {
    scene.set("ep-moistscale-label",  "text.content", `${a[0]} MOISTURE SZ    ${(moisture * 10.0).round() / 10.0}`);
    scene.set("ep-ice-label",         "text.content", `${a[1]} ICE CAPS       ${(ice * 10.0).round() / 10.0}`);
    scene.set("ep-lapse-label",       "text.content", `${a[2]} ALT COOLING    ${(lapse * 10.0).round() / 10.0}`);
    scene.set("ep-rain-label",        "text.content", `${a[3]} RAIN SHADOW    ${(rain * 10.0).round() / 10.0}`);

    scene.set("ep-moistscale-handle", "position.x", (TRACK_X + (moisture - 0.5) / 7.5 * TRACK_W).round());
    scene.set("ep-ice-handle",        "position.x", (TRACK_X + ice / 3.0 * TRACK_W).round());
    scene.set("ep-lapse-handle",      "position.x", (TRACK_X + lapse / 1.5 * TRACK_W).round());
    scene.set("ep-rain-handle",       "position.x", (TRACK_X + rain * TRACK_W).round());
} else {
    let coloring_name = if coloring < 0.5 { "BIOME" } else if coloring < 1.5 { "ELEV" } else { "MOIST" };
    scene.set("ep-disp-label",     "text.content", `${a[0]} DISPLACEMENT   ${(disp * 100.0).round()}%`);
    scene.set("ep-coloring-label", "text.content", `${a[1]} COLORING  [${coloring_name}]`);
    scene.set("ep-rotspeed-label", "text.content", `${a[2]} ROTATION       ${(rotspeed * 10.0).round() / 10.0}`);
    scene.set("ep-sunaz-label",    "text.content", `${a[3]} SUN AZIMUTH    ${sunaz.round()}`);
    scene.set("ep-sunel-label",    "text.content", `${a[4]} SUN ELEVATION   ${sunel.round()}`);
    scene.set("ep-ambient-label",  "text.content", `${a[5]} AMBIENT        ${(ambient * 100.0).round()}%`);

    scene.set("ep-disp-handle",     "position.x", (TRACK_X + disp / 0.6 * TRACK_W).round());
    scene.set("ep-coloring-handle", "position.x", (TRACK_X + coloring / 2.0 * TRACK_W).round());
    scene.set("ep-rotspeed-handle", "position.x", (TRACK_X + rotspeed / 10.0 * TRACK_W).round());
    scene.set("ep-sunaz-handle",    "position.x", (TRACK_X + sunaz / 360.0 * TRACK_W).round());
    scene.set("ep-sunel-handle",    "position.x", (TRACK_X + (sunel + 10.0) / 95.0 * TRACK_W).round());
    scene.set("ep-ambient-handle",  "position.x", (TRACK_X + ambient / 0.5 * TRACK_W).round());
}

// ── Stats strip ───────────────────────────────────────────────────────────────
scene.set("stats-seed", "text.content", `SEED: ${seed.round()}`);

let ps = planet_last_stats();
if ps.len() > 0 {
    let ocean_pct  = (ps["ocean"]     * 100.0).round();
    let forest_pct = (ps["forest"]    * 100.0).round();
    let desert_pct = (ps["desert"]    * 100.0).round();
    let cold_pct   = (ps["cold"]      * 100.0).round();
    scene.set("stats-biomes", "text.content",
        `OCEAN ${ocean_pct}%  FOREST ${forest_pct}%  DESERT ${desert_pct}%  SNOW ${cold_pct}%`);
}

// ── Push world params to planet mesh ─────────────────────────────────────────
let coloring_str = if coloring < 0.5 { "biome" } else if coloring < 1.5 { "elevation" } else { "moisture" };
scene.set("planet-mesh", "world.seed",               seed);
scene.set("planet-mesh", "world.ocean_fraction",     ocean);
scene.set("planet-mesh", "world.continent_scale",    cscale);
scene.set("planet-mesh", "world.continent_warp",     cwarp);
scene.set("planet-mesh", "world.continent_octaves",  coct);
scene.set("planet-mesh", "world.mountain_scale",          mscale);
scene.set("planet-mesh", "world.mountain_strength",       mstr);
scene.set("planet-mesh", "world.mountain_ridge_octaves",  mroct);
scene.set("planet-mesh", "world.moisture_scale",          moisture);
scene.set("planet-mesh", "world.ice_cap_strength",        ice);
scene.set("planet-mesh", "world.lapse_rate",              lapse);
scene.set("planet-mesh", "world.rain_shadow",             rain);
scene.set("planet-mesh", "world.displacement_scale",      disp);
scene.set("planet-mesh", "world.coloring",                coloring_str);
scene.set("planet-mesh", "rotate_y_deg_per_sec",     rotspeed);
scene.set("planet-mesh", "ambient",                  ambient);

// Recompute light direction from azimuth + elevation angles
let az_rad = sunaz * 0.01745329;
let el_rad = sunel * 0.01745329;
scene.set("planet-mesh", "light_direction_x", -(az_rad.cos() * el_rad.cos()));
scene.set("planet-mesh", "light_direction_y", el_rad.sin());
scene.set("planet-mesh", "light_direction_z", -(az_rad.sin() * el_rad.cos()));
