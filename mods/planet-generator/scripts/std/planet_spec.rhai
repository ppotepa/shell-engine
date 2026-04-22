fn defaults() {
    #{
        planet_type: "earth_like",
        seed: 0.0,
        has_ocean: true,
        ocean: 0.55,
        cscale: 2.5,
        cwarp: 0.65,
        coct: 5.0,
        mscale: 6.0,
        mstr: 0.45,
        mroct: 5.0,
        moisture: 3.0,
        ice: 1.0,
        lapse: 0.6,
        rain: 0.4,
        disp: 0.22,
        subdiv: 32.0,
        coloring: 0.0,
        rotspeed: 3.0,
        ambient: 0.12,
        sunaz: 210.0,
        sunel: 40.0,
        atmo_color: 2.0,
        atmo_density: 0.48,
        atmo_height: 0.11,
        atmo_forward_scatter: 0.78,
        atmo_absorption: 0.12,
        atmo_rayleigh_falloff: 0.22,
        atmo_haze_amount: 0.22,
        atmo_haze_falloff: 0.46,
        planet_radius_real_km: 6371.0,
        planet_radius_scale_divisor: 1.0,
        surface_gravity_mps2: 9.81,
        atmo_top_km: 80.0,
        atmo_dense_start_km: 12.0,
        atmo_drag_max: 2.0
    }
}

fn normalize(spec) {
    let d = defaults();
    #{
        planet_type: spec["planet_type"] ?? d["planet_type"],
        seed: spec["seed"] ?? d["seed"],
        has_ocean: spec["has_ocean"] ?? d["has_ocean"],
        ocean: spec["ocean"] ?? d["ocean"],
        cscale: spec["cscale"] ?? d["cscale"],
        cwarp: spec["cwarp"] ?? d["cwarp"],
        coct: spec["coct"] ?? d["coct"],
        mscale: spec["mscale"] ?? d["mscale"],
        mstr: spec["mstr"] ?? d["mstr"],
        mroct: spec["mroct"] ?? d["mroct"],
        moisture: spec["moisture"] ?? d["moisture"],
        ice: spec["ice"] ?? d["ice"],
        lapse: spec["lapse"] ?? d["lapse"],
        rain: spec["rain"] ?? d["rain"],
        disp: spec["disp"] ?? d["disp"],
        subdiv: spec["subdiv"] ?? d["subdiv"],
        coloring: spec["coloring"] ?? d["coloring"],
        rotspeed: spec["rotspeed"] ?? d["rotspeed"],
        ambient: spec["ambient"] ?? d["ambient"],
        sunaz: spec["sunaz"] ?? d["sunaz"],
        sunel: spec["sunel"] ?? d["sunel"],
        atmo_color: spec["atmo_color"] ?? d["atmo_color"],
        atmo_density: spec["atmo_density"] ?? d["atmo_density"],
        atmo_height: spec["atmo_height"] ?? d["atmo_height"],
        atmo_forward_scatter: spec["atmo_forward_scatter"] ?? d["atmo_forward_scatter"],
        atmo_absorption: spec["atmo_absorption"] ?? d["atmo_absorption"],
        atmo_rayleigh_falloff: spec["atmo_rayleigh_falloff"] ?? d["atmo_rayleigh_falloff"],
        atmo_haze_amount: spec["atmo_haze_amount"] ?? d["atmo_haze_amount"],
        atmo_haze_falloff: spec["atmo_haze_falloff"] ?? d["atmo_haze_falloff"],
        planet_radius_real_km: spec["planet_radius_real_km"] ?? d["planet_radius_real_km"],
        planet_radius_scale_divisor: spec["planet_radius_scale_divisor"] ?? d["planet_radius_scale_divisor"],
        surface_gravity_mps2: spec["surface_gravity_mps2"] ?? d["surface_gravity_mps2"],
        atmo_top_km: spec["atmo_top_km"] ?? d["atmo_top_km"],
        atmo_dense_start_km: spec["atmo_dense_start_km"] ?? d["atmo_dense_start_km"],
        atmo_drag_max: spec["atmo_drag_max"] ?? d["atmo_drag_max"]
    }
}

fn from_legacy(local) {
    let d = defaults();
    #{
        planet_type: local["planet_type"] ?? d["planet_type"],
        seed: local["seed"] ?? d["seed"],
        has_ocean: local["has_ocean"] ?? d["has_ocean"],
        ocean: local["ocean"] ?? d["ocean"],
        cscale: local["cscale"] ?? d["cscale"],
        cwarp: local["cwarp"] ?? d["cwarp"],
        coct: local["coct"] ?? d["coct"],
        mscale: local["mscale"] ?? d["mscale"],
        mstr: local["mstr"] ?? d["mstr"],
        mroct: local["mroct"] ?? d["mroct"],
        moisture: local["moisture"] ?? d["moisture"],
        ice: local["ice"] ?? d["ice"],
        lapse: local["lapse"] ?? d["lapse"],
        rain: local["rain"] ?? d["rain"],
        disp: local["disp"] ?? d["disp"],
        subdiv: local["subdiv"] ?? d["subdiv"],
        coloring: local["coloring"] ?? d["coloring"],
        rotspeed: local["rotspeed"] ?? d["rotspeed"],
        ambient: local["ambient"] ?? d["ambient"],
        sunaz: local["sunaz"] ?? d["sunaz"],
        sunel: local["sunel"] ?? d["sunel"],
        atmo_color: local["atmo_color"] ?? d["atmo_color"],
        atmo_density: local["atmo_density"] ?? d["atmo_density"],
        atmo_height: local["atmo_height"] ?? d["atmo_height"],
        atmo_forward_scatter: local["atmo_forward_scatter"] ?? d["atmo_forward_scatter"],
        atmo_absorption: local["atmo_absorption"] ?? d["atmo_absorption"],
        atmo_rayleigh_falloff: local["atmo_rayleigh_falloff"] ?? d["atmo_rayleigh_falloff"],
        atmo_haze_amount: local["atmo_haze_amount"] ?? d["atmo_haze_amount"],
        atmo_haze_falloff: local["atmo_haze_falloff"] ?? d["atmo_haze_falloff"],
        planet_radius_real_km: local["planet_radius_real_km"] ?? d["planet_radius_real_km"],
        planet_radius_scale_divisor: local["planet_radius_scale_divisor"] ?? d["planet_radius_scale_divisor"],
        surface_gravity_mps2: local["surface_gravity_mps2"] ?? d["surface_gravity_mps2"],
        atmo_top_km: local["atmo_top_km"] ?? d["atmo_top_km"],
        atmo_dense_start_km: local["atmo_dense_start_km"] ?? d["atmo_dense_start_km"],
        atmo_drag_max: local["atmo_drag_max"] ?? d["atmo_drag_max"]
    }
}

fn ensure_spec_only(local) {
    let existing = local["planet_spec"] ?? ();
    local["planet_spec"] = if existing == () {
        from_legacy(local)
    } else {
        normalize(existing)
    };
    local
}

fn sync_legacy_from_spec(local) {
    local = ensure_spec_only(local);
    let spec = local["planet_spec"] ?? defaults();
    local.planet_type = spec["planet_type"] ?? "earth_like";
    local.seed = spec["seed"] ?? 0.0;
    local.has_ocean = spec["has_ocean"] ?? true;
    local.ocean = spec["ocean"] ?? 0.55;
    local.cscale = spec["cscale"] ?? 2.5;
    local.cwarp = spec["cwarp"] ?? 0.65;
    local.coct = spec["coct"] ?? 5.0;
    local.mscale = spec["mscale"] ?? 6.0;
    local.mstr = spec["mstr"] ?? 0.45;
    local.mroct = spec["mroct"] ?? 5.0;
    local.moisture = spec["moisture"] ?? 3.0;
    local.ice = spec["ice"] ?? 1.0;
    local.lapse = spec["lapse"] ?? 0.6;
    local.rain = spec["rain"] ?? 0.4;
    local.disp = spec["disp"] ?? 0.22;
    local.subdiv = spec["subdiv"] ?? 32.0;
    local.coloring = spec["coloring"] ?? 0.0;
    local.rotspeed = spec["rotspeed"] ?? 3.0;
    local.ambient = spec["ambient"] ?? 0.12;
    local.sunaz = spec["sunaz"] ?? 210.0;
    local.sunel = spec["sunel"] ?? 40.0;
    local.atmo_color = spec["atmo_color"] ?? 2.0;
    local.atmo_density = spec["atmo_density"] ?? 0.48;
    local.atmo_height = spec["atmo_height"] ?? 0.11;
    local.atmo_forward_scatter = spec["atmo_forward_scatter"] ?? 0.78;
    local.atmo_absorption = spec["atmo_absorption"] ?? 0.12;
    local.atmo_rayleigh_falloff = spec["atmo_rayleigh_falloff"] ?? 0.22;
    local.atmo_haze_amount = spec["atmo_haze_amount"] ?? 0.22;
    local.atmo_haze_falloff = spec["atmo_haze_falloff"] ?? 0.46;
    local.planet_radius_real_km = spec["planet_radius_real_km"] ?? 6371.0;
    local.planet_radius_scale_divisor = spec["planet_radius_scale_divisor"] ?? 1.0;
    local.surface_gravity_mps2 = spec["surface_gravity_mps2"] ?? 9.81;
    local.atmo_top_km = spec["atmo_top_km"] ?? 80.0;
    local.atmo_dense_start_km = spec["atmo_dense_start_km"] ?? 12.0;
    local.atmo_drag_max = spec["atmo_drag_max"] ?? 2.0;
    local
}

fn sync_spec_from_legacy(local) {
    local = ensure_spec_only(local);
    local["planet_spec"] = from_legacy(local);
    sync_legacy_from_spec(local)
}

fn ensure(local) {
    local = ensure_spec_only(local);
    sync_legacy_from_spec(local)
}

fn snapshot(local) {
    let spec = local["planet_spec"] ?? from_legacy(local);
    normalize(spec)
}

fn apply_snapshot(local, snapshot) {
    local["planet_spec"] = normalize(snapshot ?? defaults());
    sync_legacy_from_spec(local)
}
