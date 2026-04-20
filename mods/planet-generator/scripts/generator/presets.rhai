fn earth() {
    #{
        planet_type: "earth_like",
        seed: 0.0, has_ocean: true, ocean: 0.55, cscale: 2.5, cwarp: 0.65, coct: 5.0,
        mscale: 6.0, mstr: 0.45, mroct: 5.0,
        moisture: 3.0, ice: 1.0, lapse: 0.6, rain: 0.4,
        disp: 0.22, coloring: 0.0, rotspeed: 3.0,
        sunaz: 210.0, sunel: 40.0, ambient: 0.12,
        atmo_color: 2.0, atmo_density: 0.48, atmo_height: 0.11,
        atmo_forward_scatter: 0.78, atmo_absorption: 0.12,
        atmo_rayleigh_falloff: 0.22, atmo_haze_amount: 0.22, atmo_haze_falloff: 0.46
    }
}

fn mars() {
    #{
        planet_type: "barren",
        seed: 42.0, has_ocean: false, ocean: 0.05, cscale: 3.5, cwarp: 0.3, coct: 4.0,
        mscale: 8.0, mstr: 0.65, mroct: 6.0,
        moisture: 1.0, ice: 0.1, lapse: 0.3, rain: 0.05,
        disp: 0.38, coloring: 1.0, rotspeed: 3.0,
        sunaz: 190.0, sunel: 30.0, ambient: 0.08,
        atmo_color: 0.0, atmo_density: 0.20, atmo_height: 0.08,
        atmo_forward_scatter: 0.18, atmo_absorption: 0.06,
        atmo_rayleigh_falloff: 0.17, atmo_haze_amount: 0.14, atmo_haze_falloff: 0.38
    }
}

fn ocean() {
    #{
        planet_type: "ocean",
        seed: 7.0, has_ocean: true, ocean: 0.88, cscale: 1.5, cwarp: 1.2, coct: 3.0,
        mscale: 4.0, mstr: 0.15, mroct: 3.0,
        moisture: 5.0, ice: 0.8, lapse: 0.7, rain: 0.6,
        disp: 0.08, coloring: 0.0, rotspeed: 2.0,
        sunaz: 220.0, sunel: 50.0, ambient: 0.15,
        atmo_color: 2.0, atmo_density: 0.65, atmo_height: 0.20,
        atmo_forward_scatter: 0.72, atmo_absorption: 0.28,
        atmo_rayleigh_falloff: 0.25, atmo_haze_amount: 0.45, atmo_haze_falloff: 0.56
    }
}

fn desert() {
    #{
        planet_type: "desert",
        seed: 13.0, has_ocean: false, ocean: 0.15, cscale: 4.0, cwarp: 0.2, coct: 3.0,
        mscale: 5.0, mstr: 0.3, mroct: 4.0,
        moisture: 1.2, ice: 0.0, lapse: 0.4, rain: 0.05,
        disp: 0.3, coloring: 0.0, rotspeed: 4.0,
        sunaz: 200.0, sunel: 25.0, ambient: 0.18,
        atmo_color: 0.0, atmo_density: 0.42, atmo_height: 0.11,
        atmo_forward_scatter: 0.36, atmo_absorption: 0.14,
        atmo_rayleigh_falloff: 0.21, atmo_haze_amount: 0.28, atmo_haze_falloff: 0.43
    }
}

fn ice() {
    #{
        planet_type: "ice",
        seed: 99.0, has_ocean: true, ocean: 0.45, cscale: 2.0, cwarp: 0.5, coct: 4.0,
        mscale: 5.0, mstr: 0.35, mroct: 4.0,
        moisture: 2.5, ice: 3.0, lapse: 1.0, rain: 0.3,
        disp: 0.18, coloring: 0.0, rotspeed: 1.5,
        sunaz: 230.0, sunel: 15.0, ambient: 0.25,
        atmo_color: 4.0, atmo_density: 0.50, atmo_height: 0.18,
        atmo_forward_scatter: 0.46, atmo_absorption: 0.24,
        atmo_rayleigh_falloff: 0.26, atmo_haze_amount: 0.42, atmo_haze_falloff: 0.59
    }
}

fn volcanic() {
    #{
        planet_type: "volcanic",
        seed: 66.0, has_ocean: false, ocean: 0.3, cscale: 3.0, cwarp: 1.5, coct: 6.0,
        mscale: 10.0, mstr: 0.85, mroct: 7.0,
        moisture: 1.5, ice: 0.0, lapse: 0.5, rain: 0.15,
        disp: 0.55, coloring: 0.0, rotspeed: 5.0,
        sunaz: 180.0, sunel: 35.0, ambient: 0.1,
        atmo_color: 1.0, atmo_density: 0.35, atmo_height: 0.10,
        atmo_forward_scatter: 0.32, atmo_absorption: 0.12,
        atmo_rayleigh_falloff: 0.18, atmo_haze_amount: 0.22, atmo_haze_falloff: 0.37
    }
}

fn archipelago() {
    #{
        planet_type: "ocean",
        seed: 27.0, has_ocean: true, ocean: 0.75, cscale: 1.0, cwarp: 1.8, coct: 7.0,
        mscale: 3.0, mstr: 0.2, mroct: 3.0,
        moisture: 4.0, ice: 0.5, lapse: 0.65, rain: 0.5,
        disp: 0.12, coloring: 0.0, rotspeed: 2.5,
        sunaz: 215.0, sunel: 45.0, ambient: 0.14,
        atmo_color: 2.0, atmo_density: 0.58, atmo_height: 0.17,
        atmo_forward_scatter: 0.60, atmo_absorption: 0.20,
        atmo_rayleigh_falloff: 0.25, atmo_haze_amount: 0.40, atmo_haze_falloff: 0.56
    }
}

fn infer_planet_type(local) {
    let has_ocean = local.has_ocean ?? true;
    let ocean = local.ocean ?? 0.55;
    let ice = local.ice ?? 0.0;
    let moisture = local.moisture ?? 3.0;
    let mountain = local.mstr ?? 0.45;
    let displacement = local.disp ?? 0.22;

    if has_ocean {
        if ice > 1.8 {
            return "ice";
        }
        if ocean > 0.72 {
            return "ocean";
        }
        return "earth_like";
    }

    if mountain > 0.72 && displacement > 0.40 {
        return "volcanic";
    }
    if moisture < 1.4 {
        return "desert";
    }
    "barren"
}

fn by_choice(choice) {
    if choice == 1 {
        mars()
    } else if choice == 2 {
        ocean()
    } else if choice == 3 {
        desert()
    } else if choice == 4 {
        ice()
    } else if choice == 5 {
        volcanic()
    } else if choice == 6 {
        archipelago()
    } else {
        earth()
    }
}

fn apply_values(local, preset) {
    local.planet_type = preset.planet_type ?? local.planet_type ?? "earth_like";
    local.seed = preset.seed;
    local.has_ocean = preset.has_ocean ?? (preset.ocean ?? 0.55) >= 0.5;
    local.ocean = preset.ocean;
    local.cscale = preset.cscale;
    local.cwarp = preset.cwarp;
    local.coct = preset.coct;
    local.mscale = preset.mscale;
    local.mstr = preset.mstr;
    local.mroct = preset.mroct;
    local.moisture = preset.moisture;
    local.ice = preset.ice;
    local.lapse = preset.lapse;
    local.rain = preset.rain;
    local.disp = preset.disp;
    local.coloring = preset.coloring;
    local.rotspeed = preset.rotspeed;
    local.sunaz = preset.sunaz;
    local.sunel = preset.sunel;
    local.ambient = preset.ambient;
    local.atmo_color = preset.atmo_color;
    local.atmo_density = preset.atmo_density;
    local.atmo_height = preset.atmo_height;
    local.atmo_forward_scatter = preset.atmo_forward_scatter;
    local.atmo_absorption = preset.atmo_absorption;
    local.atmo_rayleigh_falloff = preset.atmo_rayleigh_falloff;
    local.atmo_haze_amount = preset.atmo_haze_amount;
    local.atmo_haze_falloff = preset.atmo_haze_falloff;
    local.tab_sync_needed = true;
    local.push_dirty = true;
    local.push_timer = 150.0;
    local
}

fn apply_choice(local, choice) {
    local = apply_values(local, by_choice(choice));
    local.preset_choice = choice;
    local
}

fn reset_to_earth(local) {
    apply_values(local, earth())
}

fn randomize(local) {
    let seed = local.seed ?? 0.0;
    local.seed = (seed * 1234.5 + 6789.0) % 9999.0;
    local.has_ocean = ((local.seed * 0.137) % 1.0) > 0.32;
    local.ocean = 0.2 + ((local.seed * 0.618) % 0.7);
    local.cscale = 1.0 + ((local.seed * 0.382) % 9.0);
    local.cwarp = (local.seed * 0.271) % 2.0;
    local.coct = 3.0 + ((local.seed * 0.157) % 5.0).round();
    local.mscale = 2.0 + ((local.seed * 0.493) % 10.0);
    local.mstr = (local.seed * 0.311) % 1.0;
    local.mroct = 3.0 + ((local.seed * 0.229) % 5.0).round();
    local.moisture = 1.0 + ((local.seed * 0.441) % 5.0);
    local.ice = (local.seed * 0.173) % 2.0;
    local.lapse = 0.2 + ((local.seed * 0.337) % 0.8);
    local.rain = (local.seed * 0.251) % 0.8;
    local.disp = (local.seed * 0.197) % 0.6;
    local.rotspeed = 1.0 + ((local.seed * 0.283) % 6.0);
    local.sunaz = (local.seed * 0.431) % 360.0;
    local.sunel = 10.0 + ((local.seed * 0.317) % 70.0);
    local.ambient = 0.05 + ((local.seed * 0.211) % 0.25);
    local.atmo_color = ((local.seed * 0.173) % 6.0).floor();
    local.atmo_density = (local.seed * 0.137) % 0.75;
    local.atmo_height = 0.06 + ((local.seed * 0.071) % 0.26);
    local.atmo_forward_scatter = (local.seed * 0.167) % 0.9;
    local.atmo_absorption = (local.seed * 0.119) % 0.55;
    local.atmo_rayleigh_falloff = 0.08 + ((local.seed * 0.191) % 0.52);
    local.atmo_haze_amount = (local.seed * 0.227) % 0.75;
    local.atmo_haze_falloff = 0.08 + ((local.seed * 0.149) % 0.68);
    local.planet_type = infer_planet_type(local);
    local.tab_sync_needed = true;
    local.push_dirty = true;
    local.push_timer = 150.0;
    local
}
