import "std/math3" as math3;

fn default_biomes() {
    ["beach", "grassland", "desert"]
}

fn planet_config(local) {
    #{
        seed: local.seed ?? 0.0,
        has_ocean: local.has_ocean ?? true,
        ocean: local.ocean ?? 0.55,
        cscale: local.cscale ?? 2.5,
        cwarp: local.cwarp ?? 0.65,
        coct: local.coct ?? 5.0,
        mscale: local.mscale ?? 6.0,
        mstr: local.mstr ?? 0.45,
        mroct: local.mroct ?? 5.0,
        moisture: local.moisture ?? 3.0,
        ice: local.ice ?? 1.0,
        lapse: local.lapse ?? 0.6,
        rain: local.rain ?? 0.4,
        disp: local.disp ?? 0.22
    }
}

fn fallback_spawn(local) {
    let angle_deg = local.ship_spawn_angle_deg ?? 0.0;
    let angle = math3::deg_to_rad(angle_deg);
    #{
        angle_deg: angle_deg,
        longitude_deg: angle_deg,
        latitude_deg: 0.0,
        normal_x: angle.cos(),
        normal_y: angle.sin(),
        normal_z: 0.0,
        surface_radius_scale: 1.0,
        surface_offset: 0.0,
        biome: "fallback"
    }
}

fn resolve_spawn(local, world) {
    if !(local.ship_spawn_use_finder ?? true) {
        return fallback_spawn(local);
    }

    let spawn_info = world.find_planet_spawn(
        planet_config(local),
        local.ship_spawn_biomes ?? default_biomes()
    );

    if (spawn_info["surface_radius_scale"] ?? 0.0) <= 0.0 {
        return fallback_spawn(local);
    }

    spawn_info
}

fn tangent_basis(nx, ny, nz, heading) {
    let normal = math3::normalize_or_zero(nx, ny, nz);
    let ref_x = 0.0;
    let ref_y = if absf(normal["y"] ?? 0.0) > 0.98 { 0.0 } else { 1.0 };
    let ref_z = if absf(normal["y"] ?? 0.0) > 0.98 { 1.0 } else { 0.0 };

    let right_seed = math3::cross(
        ref_x,
        ref_y,
        ref_z,
        normal["x"] ?? 0.0,
        normal["y"] ?? 1.0,
        normal["z"] ?? 0.0
    );
    let right = math3::normalize_or_zero(
        right_seed["x"] ?? 0.0,
        right_seed["y"] ?? 0.0,
        right_seed["z"] ?? 1.0
    );
    let forward_seed = math3::cross(
        normal["x"] ?? 0.0,
        normal["y"] ?? 1.0,
        normal["z"] ?? 0.0,
        right["x"] ?? 0.0,
        right["y"] ?? 0.0,
        right["z"] ?? 1.0
    );
    let forward = math3::normalize_or_zero(
        forward_seed["x"] ?? 0.0,
        forward_seed["y"] ?? 1.0,
        forward_seed["z"] ?? 0.0
    );

    let rotated_forward = math3::rotate_axis(
        forward["x"] ?? 0.0,
        forward["y"] ?? 1.0,
        forward["z"] ?? 0.0,
        normal["x"] ?? 0.0,
        normal["y"] ?? 1.0,
        normal["z"] ?? 0.0,
        heading
    );
    let rotated_right = math3::rotate_axis(
        right["x"] ?? 0.0,
        right["y"] ?? 0.0,
        right["z"] ?? 1.0,
        normal["x"] ?? 0.0,
        normal["y"] ?? 1.0,
        normal["z"] ?? 0.0,
        heading
    );

    #{
        snx: normal["x"] ?? 0.0,
        sny: normal["y"] ?? 1.0,
        snz: normal["z"] ?? 0.0,
        sfx: rotated_forward["x"] ?? (forward["x"] ?? 0.0),
        sfy: rotated_forward["y"] ?? (forward["y"] ?? 1.0),
        sfz: rotated_forward["z"] ?? (forward["z"] ?? 0.0),
        srx: rotated_right["x"] ?? (right["x"] ?? 0.0),
        sry: rotated_right["y"] ?? (right["y"] ?? 0.0),
        srz: rotated_right["z"] ?? (right["z"] ?? 1.0)
    }
}

fn seed_ship_state(local, world, base_render_radius) {
    if local.ship_seeded ?? false {
        return local;
    }

    let spawn_info = resolve_spawn(local, world);
    let heading = math3::deg_to_rad(local.ship_heading_deg ?? 0.0);
    let basis = tangent_basis(
        spawn_info["normal_x"] ?? 1.0,
        spawn_info["normal_y"] ?? 0.0,
        spawn_info["normal_z"] ?? 0.0,
        heading
    );
    let spawn_altitude = (local.ship_surface_altitude ?? 0.00004).max(0.00000005);
    let ground_radius =
        base_render_radius.max(0.000001) * (spawn_info["surface_radius_scale"] ?? 1.0).max(0.000001);

    local.snx = basis["snx"] ?? 1.0;
    local.sny = basis["sny"] ?? 0.0;
    local.snz = basis["snz"] ?? 0.0;
    local.sfx = basis["sfx"] ?? 0.0;
    local.sfy = basis["sfy"] ?? 1.0;
    local.sfz = basis["sfz"] ?? 0.0;
    local.srx = basis["srx"] ?? 0.0;
    local.sry = basis["sry"] ?? 0.0;
    local.srz = basis["srz"] ?? 1.0;
    local.ship_ground_radius = ground_radius;
    local.radius = ground_radius + spawn_altitude;
    local.pos_x = (basis["snx"] ?? 1.0) * (ground_radius + spawn_altitude);
    local.pos_y = (basis["sny"] ?? 0.0) * (ground_radius + spawn_altitude);
    local.pos_z = (basis["snz"] ?? 0.0) * (ground_radius + spawn_altitude);
    local.vel_x = 0.0;
    local.vel_y = 0.0;
    local.vel_z = 0.0;
    local.vfwd = 0.0;
    local.vright = 0.0;
    local.vrad = 0.0;
    local.yaw_rate = 0.0;
    local.pitch_rate = 0.0;
    local.ship_heading = heading;
    local.ship_vis_pitch = 3.1415926535;
    local.ship_vis_yaw = -1.57079632675;
    local.ship_vis_roll = 0.0;
    local.cam_fx = basis["sfx"] ?? 0.0;
    local.cam_fy = basis["sfy"] ?? 1.0;
    local.cam_fz = basis["sfz"] ?? 0.0;
    local.cam_ux = basis["snx"] ?? 1.0;
    local.cam_uy = basis["sny"] ?? 0.0;
    local.cam_uz = basis["snz"] ?? 0.0;
    local.cam_roll = 0.0;
    local.cam_side_sway_m = 0.0;
    local.cam_lift_sway_m = 0.0;
    local.prev_mouse_x = 0.0;
    local.prev_mouse_y = 0.0;
    local.mouse_look_seeded = false;
    local.ship_spawn_angle_deg = spawn_info["angle_deg"] ?? (local.ship_spawn_angle_deg ?? 0.0);
    local.ship_spawn_latitude_deg = spawn_info["latitude_deg"] ?? 0.0;
    local.ship_spawn_biome = spawn_info["biome"] ?? "unknown";
    local.ship_spawn_surface_offset = spawn_info["surface_offset"] ?? 0.0;
    local.ship_seeded = true;
    local
}
