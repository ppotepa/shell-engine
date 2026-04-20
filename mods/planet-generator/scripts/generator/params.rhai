fn model_source(mode) {
    if mode == 1 {
        "world://32"
    } else if mode == 2 {
        "/assets/3d/sphere.obj"
    } else if mode == 3 {
        "/assets/3d/cube.obj"
    } else if mode == 4 {
        "/assets/3d/suzanne.obj"
    } else {
        "world://32"
    }
}

fn seed_to_01(v)      { (v / 9999.0).max(0.0).min(1.0) }
fn seed_from_01(v)    { v * 9999.0 }
fn toggle_to_01(v)    { if v { 1.0 } else { 0.0 } }
fn toggle_from_01(v)  { v >= 0.5 }
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
fn atmo_color_to_01(v)        { (v / 5.0).max(0.0).min(1.0) }
fn atmo_color_from_01(v)      { (v * 5.0).round().max(0.0).min(5.0) }
fn atmo_density_to_01(v)      { v.max(0.0).min(1.0) }
fn atmo_density_from_01(v)    { v.max(0.0).min(1.0) }
fn atmo_height_to_01(v)       { v.max(0.0).min(1.0) }
fn atmo_height_from_01(v)     { v.max(0.0).min(1.0) }
fn atmo_forward_to_01(v)      { v.max(0.0).min(1.0) }
fn atmo_forward_from_01(v)    { v.max(0.0).min(1.0) }
fn atmo_absorption_to_01(v)   { v.max(0.0).min(1.0) }
fn atmo_absorption_from_01(v) { v.max(0.0).min(1.0) }
fn atmo_ray_falloff_to_01(v)  { ((v - 0.05) / 0.85).max(0.0).min(1.0) }
fn atmo_ray_falloff_from_01(v){ 0.05 + v * 0.85 }
fn atmo_haze_amount_to_01(v)  { v.max(0.0).min(1.0) }
fn atmo_haze_amount_from_01(v){ v.max(0.0).min(1.0) }
fn atmo_haze_falloff_to_01(v) { ((v - 0.05) / 0.90).max(0.0).min(1.0) }
fn atmo_haze_falloff_from_01(v) { 0.05 + v * 0.90 }

fn atmo_color_name(v) {
    if v < 0.5 { "AMBER" }
    else if v < 1.5 { "ROSE" }
    else if v < 2.5 { "CYAN" }
    else if v < 3.5 { "LIME" }
    else if v < 4.5 { "AZURE" }
    else { "VIOLET" }
}

fn atmo_color_hex(v) {
    if v < 0.5 { "#d9b36a" }
    else if v < 1.5 { "#cf7a8c" }
    else if v < 2.5 { "#7cc8ff" }
    else if v < 3.5 { "#9bcf86" }
    else if v < 4.5 { "#9ab8ff" }
    else { "#b79dff" }
}

fn atmo_haze_hex(v) {
    if v < 0.5 { "#f6eddc" }
    else if v < 1.5 { "#f6ebef" }
    else if v < 2.5 { "#edf7ff" }
    else if v < 3.5 { "#eef7ea" }
    else if v < 4.5 { "#eef4ff" }
    else { "#f2eeff" }
}

fn atmo_absorption_hex(v) {
    if v < 0.5 { "#ffbf7b" }
    else if v < 1.5 { "#ffc0ad" }
    else if v < 2.5 { "#ffd59d" }
    else if v < 3.5 { "#ffd08f" }
    else if v < 4.5 { "#ffd9b4" }
    else { "#ffd0dc" }
}

fn coloring_label(v) {
    if v < 0.5 { "BIOME" }
    else if v < 1.5 { "ELEV" }
    else { "MOIST" }
}

fn coloring_mode(v) {
    if v < 0.5 { "biome" }
    else if v < 1.5 { "elevation" }
    else { "moisture" }
}

fn num_params(tab) {
    if tab == 0 { 5 }
    else if tab == 1 { 3 }
    else if tab == 2 { 4 }
    else if tab == 3 { 7 }
    else { 8 }
}

fn tab_title(tab) {
    if tab == 0 { "CONTINENTS" }
    else if tab == 1 { "MOUNTAINS" }
    else if tab == 2 { "CLIMATE" }
    else if tab == 3 { "VISUAL" }
    else { "ATMOSPHERE" }
}

fn tab_summary(tab) {
    if tab == 0 { "Shape large landmasses before erosion and climate." }
    else if tab == 1 { "Tune ridge spacing, height, and breakup." }
    else if tab == 2 { "Adjust moisture, cold zones, and rain shadow." }
    else if tab == 3 { "Swap density, palette source, and direct light." }
    else { "Drive halo thickness, limb glow, and haze." }
}

fn slot_label(tab, index) {
    if tab == 0 {
        if index == 0 { "SEED" }
        else if index == 1 { "OCEAN" }
        else if index == 2 { "CONTINENT SZ" }
        else if index == 3 { "COAST CHAOS" }
        else if index == 4 { "OCTAVES" }
        else { "" }
    } else if tab == 1 {
        if index == 0 { "MTN SPACING" }
        else if index == 1 { "MTN HEIGHT" }
        else if index == 2 { "RIDGE DETAIL" }
        else { "" }
    } else if tab == 2 {
        if index == 0 { "MOISTURE SZ" }
        else if index == 1 { "ICE CAPS" }
        else if index == 2 { "ALT COOLING" }
        else if index == 3 { "RAIN SHADOW" }
        else { "" }
    } else if tab == 3 {
        if index == 0 { "RESOLUTION" }
        else if index == 1 { "DISPLACEMENT" }
        else if index == 2 { "COLORING" }
        else if index == 3 { "ROTATION" }
        else if index == 4 { "SUN AZIMUTH" }
        else if index == 5 { "SUN ELEVATION" }
        else if index == 6 { "AMBIENT" }
        else { "" }
    } else {
        if index == 0 { "COLOR" }
        else if index == 1 { "ATMO DENSITY" }
        else if index == 2 { "ATMO HEIGHT" }
        else if index == 3 { "FWD SCATTER" }
        else if index == 4 { "ABSORPTION" }
        else if index == 5 { "RAY FALLOFF" }
        else if index == 6 { "HAZE AMOUNT" }
        else if index == 7 { "HAZE FALLOFF" }
        else { "" }
    }
}

fn slot_value(local, tab, index) {
    let seed = local.seed ?? 0.0;
    let has_ocean = local.has_ocean ?? true;
    let cscale = local.cscale ?? 2.5;
    let cwarp = local.cwarp ?? 0.65;
    let coct = local.coct ?? 5.0;
    let mscale = local.mscale ?? 6.0;
    let mstr = local.mstr ?? 0.45;
    let mroct = local.mroct ?? 5.0;
    let moisture = local.moisture ?? 3.0;
    let ice = local.ice ?? 1.0;
    let lapse = local.lapse ?? 0.6;
    let rain = local.rain ?? 0.4;
    let subdiv = local.subdiv ?? 32.0;
    let disp = local.disp ?? 0.22;
    let coloring = local.coloring ?? 0.0;
    let rotspeed = local.rotspeed ?? 3.0;
    let sunaz = local.sunaz ?? 210.0;
    let sunel = local.sunel ?? 40.0;
    let ambient = local.ambient ?? 0.12;
    let atmo_color = local.atmo_color ?? 2.0;
    let atmo_density = local.atmo_density ?? 0.48;
    let atmo_height = local.atmo_height ?? 0.11;
    let atmo_forward_scatter = local.atmo_forward_scatter ?? 0.78;
    let atmo_absorption = local.atmo_absorption ?? 0.12;
    let atmo_rayleigh_falloff = local.atmo_rayleigh_falloff ?? 0.22;
    let atmo_haze_amount = local.atmo_haze_amount ?? 0.22;
    let atmo_haze_falloff = local.atmo_haze_falloff ?? 0.46;

    if tab == 0 {
        if index == 0 { `${seed.round()}` }
        else if index == 1 { if has_ocean { "[ON]" } else { "[OFF]" } }
        else if index == 2 { `${(cscale * 10.0).round() / 10.0}` }
        else if index == 3 { `${(cwarp * 10.0).round() / 10.0}` }
        else if index == 4 { `${coct.round()}` }
        else { "" }
    } else if tab == 1 {
        if index == 0 { `${(mscale * 10.0).round() / 10.0}` }
        else if index == 1 { `${(mstr * 100.0).round()}%` }
        else if index == 2 { `${mroct.round()}` }
        else { "" }
    } else if tab == 2 {
        if index == 0 { `${(moisture * 10.0).round() / 10.0}` }
        else if index == 1 { `${(ice * 10.0).round() / 10.0}` }
        else if index == 2 { `${(lapse * 10.0).round() / 10.0}` }
        else if index == 3 { `${(rain * 10.0).round() / 10.0}` }
        else { "" }
    } else if tab == 3 {
        if index == 0 { `${subdiv.round()}` }
        else if index == 1 { `${(disp * 100.0).round()}%` }
        else if index == 2 { `[${coloring_label(coloring)}]` }
        else if index == 3 { `${(rotspeed * 10.0).round() / 10.0}` }
        else if index == 4 { `${sunaz.round()}` }
        else if index == 5 { `${sunel.round()}` }
        else if index == 6 { `${(ambient * 100.0).round()}%` }
        else { "" }
    } else {
        if index == 0 { `[${atmo_color_name(atmo_color)}]` }
        else if index == 1 { `${(atmo_density * 100.0).round()}%` }
        else if index == 2 { `${(atmo_height * 100.0).round()}%` }
        else if index == 3 { `${(atmo_forward_scatter * 100.0).round()}%` }
        else if index == 4 { `${(atmo_absorption * 100.0).round()}%` }
        else if index == 5 { `${(atmo_rayleigh_falloff * 100.0).round()}%` }
        else if index == 6 { `${(atmo_haze_amount * 100.0).round()}%` }
        else if index == 7 { `${(atmo_haze_falloff * 100.0).round()}%` }
        else { "" }
    }
}

fn sync_widgets_from_state(local, gui, tab) {
    let seed = local.seed ?? 0.0;
    let has_ocean = local.has_ocean ?? true;
    let cscale = local.cscale ?? 2.5;
    let cwarp = local.cwarp ?? 0.65;
    let coct = local.coct ?? 5.0;
    let mscale = local.mscale ?? 6.0;
    let mstr = local.mstr ?? 0.45;
    let mroct = local.mroct ?? 5.0;
    let moisture = local.moisture ?? 3.0;
    let ice = local.ice ?? 1.0;
    let lapse = local.lapse ?? 0.6;
    let rain = local.rain ?? 0.4;
    let subdiv = local.subdiv ?? 32.0;
    let disp = local.disp ?? 0.22;
    let coloring = local.coloring ?? 0.0;
    let rotspeed = local.rotspeed ?? 3.0;
    let sunaz = local.sunaz ?? 210.0;
    let sunel = local.sunel ?? 40.0;
    let ambient = local.ambient ?? 0.12;
    let atmo_color = local.atmo_color ?? 2.0;
    let atmo_density = local.atmo_density ?? 0.48;
    let atmo_height = local.atmo_height ?? 0.11;
    let atmo_forward_scatter = local.atmo_forward_scatter ?? 0.78;
    let atmo_absorption = local.atmo_absorption ?? 0.12;
    let atmo_rayleigh_falloff = local.atmo_rayleigh_falloff ?? 0.22;
    let atmo_haze_amount = local.atmo_haze_amount ?? 0.22;
    let atmo_haze_falloff = local.atmo_haze_falloff ?? 0.46;

    if tab == 0 {
        gui.set_widget_value("param-0", seed_to_01(seed));
        gui.set_widget_value("param-1", toggle_to_01(has_ocean));
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
        gui.set_widget_value("param-1", atmo_density_to_01(atmo_density));
        gui.set_widget_value("param-2", atmo_height_to_01(atmo_height));
        gui.set_widget_value("param-3", atmo_forward_to_01(atmo_forward_scatter));
        gui.set_widget_value("param-4", atmo_absorption_to_01(atmo_absorption));
        gui.set_widget_value("param-5", atmo_ray_falloff_to_01(atmo_rayleigh_falloff));
        gui.set_widget_value("param-6", atmo_haze_amount_to_01(atmo_haze_amount));
        gui.set_widget_value("param-7", atmo_haze_falloff_to_01(atmo_haze_falloff));
    }
    local
}

fn read_widgets_into_state(local, gui, tab) {
    if tab == 0 {
        local.seed = seed_from_01(gui.slider_value("param-0"));
        local.has_ocean = toggle_from_01(gui.slider_value("param-1"));
        local.cscale = cscale_from_01(gui.slider_value("param-2"));
        local.cwarp = cwarp_from_01(gui.slider_value("param-3"));
        let raw_coct = gui.slider_value("param-4");
        local.coct = coct_from_01(raw_coct);
        let snap_coct = coct_to_01(local.coct);
        if (raw_coct - snap_coct).abs() > 0.005 {
            gui.set_widget_value("param-4", snap_coct);
        }
    } else if tab == 1 {
        local.mscale = mscale_from_01(gui.slider_value("param-0"));
        local.mstr = mstr_from_01(gui.slider_value("param-1"));
        let raw_mroct = gui.slider_value("param-2");
        local.mroct = mroct_from_01(raw_mroct);
        let snap_mroct = mroct_to_01(local.mroct);
        if (raw_mroct - snap_mroct).abs() > 0.005 {
            gui.set_widget_value("param-2", snap_mroct);
        }
    } else if tab == 2 {
        local.moisture = moisture_from_01(gui.slider_value("param-0"));
        local.ice = ice_from_01(gui.slider_value("param-1"));
        local.lapse = lapse_from_01(gui.slider_value("param-2"));
        local.rain = rain_from_01(gui.slider_value("param-3"));
    } else if tab == 3 {
        let raw_subdiv = gui.slider_value("param-0");
        local.subdiv = subdiv_from_01(raw_subdiv);
        let snap_subdiv = subdiv_to_01(local.subdiv);
        if (raw_subdiv - snap_subdiv).abs() > 0.005 {
            gui.set_widget_value("param-0", snap_subdiv);
        }
        local.disp = disp_from_01(gui.slider_value("param-1"));
        let raw_coloring = gui.slider_value("param-2");
        local.coloring = coloring_from_01(raw_coloring);
        let snap_coloring = coloring_to_01(local.coloring);
        if (raw_coloring - snap_coloring).abs() > 0.005 {
            gui.set_widget_value("param-2", snap_coloring);
        }
        local.rotspeed = rotspeed_from_01(gui.slider_value("param-3"));
        local.sunaz = sunaz_from_01(gui.slider_value("param-4"));
        local.sunel = sunel_from_01(gui.slider_value("param-5"));
        local.ambient = ambient_from_01(gui.slider_value("param-6"));
    } else {
        let raw_atmo_color = gui.slider_value("param-0");
        local.atmo_color = atmo_color_from_01(raw_atmo_color);
        let snap_atmo_color = atmo_color_to_01(local.atmo_color);
        if (raw_atmo_color - snap_atmo_color).abs() > 0.005 {
            gui.set_widget_value("param-0", snap_atmo_color);
        }
        local.atmo_density = atmo_density_from_01(gui.slider_value("param-1"));
        local.atmo_height = atmo_height_from_01(gui.slider_value("param-2"));
        local.atmo_forward_scatter = atmo_forward_from_01(gui.slider_value("param-3"));
        local.atmo_absorption = atmo_absorption_from_01(gui.slider_value("param-4"));
        local.atmo_rayleigh_falloff = atmo_ray_falloff_from_01(gui.slider_value("param-5"));
        local.atmo_haze_amount = atmo_haze_amount_from_01(gui.slider_value("param-6"));
        local.atmo_haze_falloff = atmo_haze_falloff_from_01(gui.slider_value("param-7"));
    }
    local
}
