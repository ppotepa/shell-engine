import "std/math3" as math3;
import "std/runtime_scene" as scene_rt;
import "flight/spawn" as spawn_locator;

fn pi() { 3.1415926535 }

fn axis(negative, positive) {
    if positive && !negative { return 1.0; }
    if negative && !positive { return -1.0; }
    0.0
}

fn clamp_abs(value, limit) {
    if value > limit { return limit; }
    if value < -limit { return -limit; }
    value
}

fn km_per_render_unit(local, render_radius) {
    let real_radius_km = (local.planet_radius_real_km ?? 6371.0).max(1.0);
    let scale_divisor = (local.planet_radius_scale_divisor ?? 1.0).max(0.0001);
    (real_radius_km / scale_divisor) / render_radius.max(0.000001)
}

fn render_units_per_meter(km_per_render_unit) {
    1.0 / (km_per_render_unit.max(0.000001) * 1000.0)
}

fn meters_to_render(units_per_meter, meters) {
    if meters <= 0.0 {
        return 0.0;
    }
    meters * units_per_meter
}

fn cube_scale_for_side_m(units_per_meter, side_m) {
    meters_to_render(units_per_meter, side_m.max(0.1) * 0.5)
}

fn set_obj_world(runtime, target, x, y, z) {
    scene_rt::set(runtime, target, "obj.world.x", x);
    scene_rt::set(runtime, target, "obj.world.y", y);
    scene_rt::set(runtime, target, "obj.world.z", z);
}

fn set_obj_camera_world(runtime, target, x, y, z) {
    scene_rt::set(runtime, target, "obj.cam.world.x", x);
    scene_rt::set(runtime, target, "obj.cam.world.y", y);
    scene_rt::set(runtime, target, "obj.cam.world.z", z);
}

fn set_obj_view(runtime, target, right, up, forward) {
    scene_rt::set(runtime, target, "obj.view.right.x", right["x"] ?? 1.0);
    scene_rt::set(runtime, target, "obj.view.right.y", right["y"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.right.z", right["z"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.up.x", up["x"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.up.y", up["y"] ?? 1.0);
    scene_rt::set(runtime, target, "obj.view.up.z", up["z"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.fwd.x", forward["x"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.fwd.y", forward["y"] ?? 0.0);
    scene_rt::set(runtime, target, "obj.view.fwd.z", forward["z"] ?? 1.0);
}

fn vec_len(x, y, z) {
    sqrtf(x * x + y * y + z * z)
}

fn dot(ax, ay, az, bx, by, bz) {
    ax * bx + ay * by + az * bz
}

fn basis_from_forward_up(fx, fy, fz, ux, uy, uz, fallback_ux, fallback_uy, fallback_uz) {
    let forward = math3::normalize_or_zero(fx, fy, fz);
    let right_seed = math3::cross(
        forward["x"] ?? 0.0,
        forward["y"] ?? 0.0,
        forward["z"] ?? 1.0,
        ux,
        uy,
        uz
    );
    let right = math3::normalize_or_zero(
        right_seed["x"] ?? 0.0,
        right_seed["y"] ?? 0.0,
        right_seed["z"] ?? 0.0
    );

    if absf(right["x"] ?? 0.0) + absf(right["y"] ?? 0.0) + absf(right["z"] ?? 0.0) <= 0.000001 {
        let fallback_right_seed = math3::cross(
            forward["x"] ?? 0.0,
            forward["y"] ?? 0.0,
            forward["z"] ?? 1.0,
            fallback_ux,
            fallback_uy,
            fallback_uz
        );
        right = math3::normalize_or_zero(
            fallback_right_seed["x"] ?? 0.0,
            fallback_right_seed["y"] ?? 0.0,
            fallback_right_seed["z"] ?? 0.0
        );
    }

    if absf(right["x"] ?? 0.0) + absf(right["y"] ?? 0.0) + absf(right["z"] ?? 0.0) <= 0.000001 {
        let world_right_seed = math3::cross(
            forward["x"] ?? 0.0,
            forward["y"] ?? 0.0,
            forward["z"] ?? 1.0,
            0.0,
            1.0,
            0.0
        );
        right = math3::normalize_or_zero(
            world_right_seed["x"] ?? 0.0,
            world_right_seed["y"] ?? 0.0,
            world_right_seed["z"] ?? 0.0
        );
    }

    let up_seed = math3::cross(
        right["x"] ?? 1.0,
        right["y"] ?? 0.0,
        right["z"] ?? 0.0,
        forward["x"] ?? 0.0,
        forward["y"] ?? 0.0,
        forward["z"] ?? 1.0
    );
    let up = math3::normalize_or_zero(
        up_seed["x"] ?? fallback_ux,
        up_seed["y"] ?? fallback_uy,
        up_seed["z"] ?? fallback_uz
    );

    #{
        forward: forward,
        right: right,
        up: up
    }
}

fn apply_ship_proxy(local, runtime, pos_x, pos_y, pos_z, right, up, forward) {
    let ship_scale = local.ship_visual_scale ?? 0.0000004;
    let ship_euler = math3::ship_basis_euler(
        right["x"] ?? 1.0,
        right["y"] ?? 0.0,
        right["z"] ?? 0.0,
        forward["x"] ?? 0.0,
        forward["y"] ?? 1.0,
        forward["z"] ?? 0.0,
        up["x"] ?? 0.0,
        up["y"] ?? 1.0,
        up["z"] ?? 0.0,
        local.ship_vis_pitch ?? pi(),
        local.ship_vis_yaw ?? -(pi() * 0.5),
        local.ship_vis_roll ?? 0.0
    );

    scene_rt::visible(runtime, "flight-player", false);
    scene_rt::set(runtime, "flight-player", "obj.scale", ship_scale);
    scene_rt::set(runtime, "flight-player", "obj.pitch", math3::rad_to_deg(ship_euler["pitch"] ?? pi()));
    scene_rt::set(runtime, "flight-player", "obj.yaw", math3::rad_to_deg(ship_euler["yaw"] ?? -(pi() * 0.5)));
    scene_rt::set(runtime, "flight-player", "obj.roll", math3::rad_to_deg(ship_euler["roll"] ?? 0.0));
    set_obj_world(runtime, "flight-player", pos_x, pos_y, pos_z);

    ship_euler
}

fn hide_cockpit(runtime) {
    scene_rt::visible(runtime, "flight-cockpit", false);
}

fn apply_cockpit_visual(local, runtime, units_per_meter, eye_x, eye_y, eye_z, right, up, forward) {
    let cockpit_scale = meters_to_render(
        units_per_meter,
        (local.cockpit_model_scale_m ?? 1.0).max(0.1)
    );

    scene_rt::visible(runtime, "flight-cockpit", true);
    scene_rt::set(runtime, "flight-cockpit", "obj.scale", cockpit_scale);
    set_obj_world(runtime, "flight-cockpit", eye_x, eye_y, eye_z);
    set_obj_camera_world(runtime, "flight-cockpit", eye_x, eye_y, eye_z);
    set_obj_view(runtime, "flight-cockpit", right, up, forward);
}

fn apply_flight_camera(
    local,
    runtime,
    world,
    dt,
    units_per_meter,
    ship_x,
    ship_y,
    ship_z,
    right,
    up,
    forward,
    yaw_rate,
    pitch_rate,
    lateral_thrust,
    vertical_thrust,
    cockpit_visible
) {
    let target_forward = math3::normalize_or_zero(
        forward["x"] ?? 0.0,
        forward["y"] ?? 1.0,
        forward["z"] ?? 0.0
    );
    let target_up = math3::normalize_or_zero(
        up["x"] ?? 0.0,
        up["y"] ?? 1.0,
        up["z"] ?? 0.0
    );
    let target_right = math3::normalize_or_zero(
        right["x"] ?? 1.0,
        right["y"] ?? 0.0,
        right["z"] ?? 0.0
    );

    let forward_lag = (local.cockpit_forward_lag_sec ?? 0.06).max(0.01);
    let up_lag = (local.cockpit_up_lag_sec ?? 0.08).max(0.01);
    let sway_lag = (local.cockpit_sway_lag_sec ?? 0.16).max(0.01);
    let roll_lag = (local.cockpit_roll_lag_sec ?? 0.14).max(0.01);

    let forward_alpha = (dt / (forward_lag + dt)).max(0.0).min(1.0);
    let up_alpha = (dt / (up_lag + dt)).max(0.0).min(1.0);
    let sway_alpha = (dt / (sway_lag + dt)).max(0.0).min(1.0);
    let roll_alpha = (dt / (roll_lag + dt)).max(0.0).min(1.0);

    let raw_forward_x = (local.cam_fx ?? (target_forward["x"] ?? 0.0))
        + ((target_forward["x"] ?? 0.0) - (local.cam_fx ?? (target_forward["x"] ?? 0.0))) * forward_alpha;
    let raw_forward_y = (local.cam_fy ?? (target_forward["y"] ?? 1.0))
        + ((target_forward["y"] ?? 1.0) - (local.cam_fy ?? (target_forward["y"] ?? 1.0))) * forward_alpha;
    let raw_forward_z = (local.cam_fz ?? (target_forward["z"] ?? 0.0))
        + ((target_forward["z"] ?? 0.0) - (local.cam_fz ?? (target_forward["z"] ?? 0.0))) * forward_alpha;

    let raw_up_x = (local.cam_ux ?? (target_up["x"] ?? 0.0))
        + ((target_up["x"] ?? 0.0) - (local.cam_ux ?? (target_up["x"] ?? 0.0))) * up_alpha;
    let raw_up_y = (local.cam_uy ?? (target_up["y"] ?? 1.0))
        + ((target_up["y"] ?? 1.0) - (local.cam_uy ?? (target_up["y"] ?? 1.0))) * up_alpha;
    let raw_up_z = (local.cam_uz ?? (target_up["z"] ?? 0.0))
        + ((target_up["z"] ?? 0.0) - (local.cam_uz ?? (target_up["z"] ?? 0.0))) * up_alpha;

    let cam_basis = basis_from_forward_up(
        raw_forward_x,
        raw_forward_y,
        raw_forward_z,
        raw_up_x,
        raw_up_y,
        raw_up_z,
        target_up["x"] ?? 0.0,
        target_up["y"] ?? 1.0,
        target_up["z"] ?? 0.0
    );
    let cam_forward = cam_basis["forward"] ?? target_forward;
    let cam_right = cam_basis["right"] ?? target_right;
    let cam_up = cam_basis["up"] ?? target_up;

    let side_sway_limit_m = (local.cockpit_side_sway_m ?? 0.18).max(0.0);
    let lift_sway_limit_m = (local.cockpit_lift_sway_m ?? 0.05).max(0.0);
    let side_sway_target_m = clamp_abs(
        (-yaw_rate * 0.18) + lateral_thrust * 0.06,
        side_sway_limit_m
    );
    let lift_sway_target_m = clamp_abs(
        (-pitch_rate * 0.12) + vertical_thrust * 0.03,
        lift_sway_limit_m
    );
    let next_side_sway_m =
        (local.cam_side_sway_m ?? 0.0)
        + (side_sway_target_m - (local.cam_side_sway_m ?? 0.0)) * sway_alpha;
    let next_lift_sway_m =
        (local.cam_lift_sway_m ?? 0.0)
        + (lift_sway_target_m - (local.cam_lift_sway_m ?? 0.0)) * sway_alpha;

    let roll_target = clamp_abs(
        (-yaw_rate * (local.cockpit_roll_gain ?? 0.08)) + lateral_thrust * 0.04,
        math3::deg_to_rad((local.cockpit_roll_max_deg ?? 3.8).max(0.0))
    );
    let next_roll =
        (local.cam_roll ?? 0.0)
        + (roll_target - (local.cam_roll ?? 0.0)) * roll_alpha;
    let rolled_up = math3::rotate_axis(
        cam_up["x"] ?? 0.0,
        cam_up["y"] ?? 1.0,
        cam_up["z"] ?? 0.0,
        cam_forward["x"] ?? 0.0,
        cam_forward["y"] ?? 0.0,
        cam_forward["z"] ?? 1.0,
        next_roll
    );
    let rolled_right = math3::rotate_axis(
        cam_right["x"] ?? 1.0,
        cam_right["y"] ?? 0.0,
        cam_right["z"] ?? 0.0,
        cam_forward["x"] ?? 0.0,
        cam_forward["y"] ?? 0.0,
        cam_forward["z"] ?? 1.0,
        next_roll
    );

    let eye_forward = meters_to_render(
        units_per_meter,
        (local.cockpit_eye_forward_m ?? 0.92).max(0.0)
    );
    let eye_lift = meters_to_render(
        units_per_meter,
        (local.cockpit_eye_lift_m ?? 0.36).max(0.0)
    );
    let look_distance = meters_to_render(
        units_per_meter,
        (local.cockpit_look_distance_m ?? 4000.0).max(50.0)
    );
    let look_up_bias = meters_to_render(
        units_per_meter,
        (local.cockpit_look_up_bias_m ?? 14.0).max(0.0)
    );
    let side_sway = meters_to_render(units_per_meter, next_side_sway_m);
    let lift_sway = meters_to_render(units_per_meter, next_lift_sway_m);

    let eye_x =
        ship_x
        + (cam_forward["x"] ?? 0.0) * eye_forward
        + (rolled_up["x"] ?? 0.0) * (eye_lift + lift_sway)
        + (rolled_right["x"] ?? 0.0) * side_sway;
    let eye_y =
        ship_y
        + (cam_forward["y"] ?? 0.0) * eye_forward
        + (rolled_up["y"] ?? 1.0) * (eye_lift + lift_sway)
        + (rolled_right["y"] ?? 0.0) * side_sway;
    let eye_z =
        ship_z
        + (cam_forward["z"] ?? 0.0) * eye_forward
        + (rolled_up["z"] ?? 0.0) * (eye_lift + lift_sway)
        + (rolled_right["z"] ?? 0.0) * side_sway;

    let look_x = eye_x + (cam_forward["x"] ?? 0.0) * look_distance + (rolled_up["x"] ?? 0.0) * look_up_bias;
    let look_y = eye_y + (cam_forward["y"] ?? 1.0) * look_distance + (rolled_up["y"] ?? 1.0) * look_up_bias;
    let look_z = eye_z + (cam_forward["z"] ?? 0.0) * look_distance + (rolled_up["z"] ?? 0.0) * look_up_bias;

    world.set_camera_3d_look_at(eye_x, eye_y, eye_z, look_x, look_y, look_z);
    world.set_camera_3d_up(
        rolled_up["x"] ?? 0.0,
        rolled_up["y"] ?? 1.0,
        rolled_up["z"] ?? 0.0
    );

    if cockpit_visible {
        apply_cockpit_visual(local, runtime, units_per_meter, eye_x, eye_y, eye_z, rolled_right, rolled_up, cam_forward);
    } else {
        hide_cockpit(runtime);
    }

    local.cam_fx = cam_forward["x"] ?? 0.0;
    local.cam_fy = cam_forward["y"] ?? 1.0;
    local.cam_fz = cam_forward["z"] ?? 0.0;
    local.cam_ux = cam_up["x"] ?? 0.0;
    local.cam_uy = cam_up["y"] ?? 1.0;
    local.cam_uz = cam_up["z"] ?? 0.0;
    local.cam_roll = next_roll;
    local.cam_side_sway_m = next_side_sway_m;
    local.cam_lift_sway_m = next_lift_sway_m;
    local
}

fn step(local, runtime, world, input, dt) {
    let body = world.body("generated-planet");
    let base_render_radius =
        if body.exists {
            body.radius_px.max(0.000001)
        } else {
            1.0
        };
    let km_per_wu = km_per_render_unit(local, base_render_radius);
    let units_per_meter = render_units_per_meter(km_per_wu);
    let ship_size_m = (local.ship_size_m ?? 5.0).max(0.5);
    let ship_clearance_m = (local.ship_ground_clearance_m ?? 1.5).max(0.0);
    let ship_min_altitude_m = ship_size_m * 0.5 + ship_clearance_m;
    let ship_spawn_altitude_m = (local.ship_spawn_altitude_m ?? ship_min_altitude_m).max(ship_min_altitude_m);
    let ship_scale = cube_scale_for_side_m(units_per_meter, ship_size_m);
    let ship_min_altitude = meters_to_render(units_per_meter, ship_min_altitude_m).max(0.00000005);
    let ship_spawn_altitude = meters_to_render(units_per_meter, ship_spawn_altitude_m).max(ship_min_altitude);
    let atmo_top_wu = (local.atmo_top_km ?? 80.0) / km_per_wu.max(0.000001);

    local.ship_surface_altitude = ship_spawn_altitude;
    local.ship_visual_scale = ship_scale;
    local = spawn_locator::seed_ship_state(local, world, base_render_radius);

    if input.just_pressed("f") {
        local.camera_mode =
            if (local.camera_mode ?? "cockpit") == "free" {
                "cockpit"
            } else {
                "free"
            };
    }

    let thrust_axis = axis(
        input.down("s") || input.down("S") || input.down("Down"),
        input.down("w") || input.down("W") || input.down("Up")
    );
    let strafe_axis = axis(
        input.down("a") || input.down("A") || input.down("Left"),
        input.down("d") || input.down("D") || input.down("Right")
    );
    let vertical_axis = axis(
        input.down("q") || input.down("Q"),
        input.down("e") || input.down("E") || input.down("Space") || input.down(" ")
    );

    let ground_radius = (local.ship_ground_radius ?? base_render_radius).max(0.000001);
    let pos_x = local.pos_x ?? ((local.snx ?? 1.0) * (local.radius ?? (ground_radius + ship_spawn_altitude)));
    let pos_y = local.pos_y ?? ((local.sny ?? 0.0) * (local.radius ?? (ground_radius + ship_spawn_altitude)));
    let pos_z = local.pos_z ?? ((local.snz ?? 0.0) * (local.radius ?? (ground_radius + ship_spawn_altitude)));
    let vel_x = local.vel_x ?? 0.0;
    let vel_y = local.vel_y ?? 0.0;
    let vel_z = local.vel_z ?? 0.0;

    let radius = vec_len(pos_x, pos_y, pos_z).max(ground_radius + ship_min_altitude);
    let radial_up = math3::normalize_or_zero(pos_x, pos_y, pos_z);
    let altitude_wu = (radius - ground_radius).max(0.0);
    let atmo_alpha =
        if atmo_top_wu > 0.0 {
            math3::clamp01((atmo_top_wu - altitude_wu) / atmo_top_wu.max(0.000001))
        } else {
            0.0
        };

    let current_basis = basis_from_forward_up(
        local.sfx ?? 0.0,
        local.sfy ?? 1.0,
        local.sfz ?? 0.0,
        local.snx ?? (radial_up["x"] ?? 0.0),
        local.sny ?? (radial_up["y"] ?? 1.0),
        local.snz ?? (radial_up["z"] ?? 0.0),
        radial_up["x"] ?? 0.0,
        radial_up["y"] ?? 1.0,
        radial_up["z"] ?? 0.0
    );
    let current_forward = current_basis["forward"] ?? #{ x: 0.0, y: 1.0, z: 0.0 };
    let current_right = current_basis["right"] ?? #{ x: 1.0, y: 0.0, z: 0.0 };
    let current_up = current_basis["up"] ?? #{
        x: radial_up["x"] ?? 0.0,
        y: radial_up["y"] ?? 1.0,
        z: radial_up["z"] ?? 0.0
    };

    let mouse_x = input.mouse_x;
    let mouse_y = input.mouse_y;
    let mouse_dx =
        if local.mouse_look_seeded ?? false {
            mouse_x - (local.prev_mouse_x ?? mouse_x)
        } else {
            0.0
        };
    let mouse_dy =
        if local.mouse_look_seeded ?? false {
            mouse_y - (local.prev_mouse_y ?? mouse_y)
        } else {
            0.0
        };

    let mouse_deg_per_px = (local.flight_mouse_deg_per_px ?? 0.12).max(0.001);
    let mouse_lag = (local.flight_mouse_lag_sec ?? 0.05).max(0.001);
    let mouse_damp = (local.flight_mouse_damp ?? 7.5).max(0.0);
    let max_turn_rate = math3::deg_to_rad((local.flight_mouse_max_rate_deg ?? 180.0).max(1.0));
    let mouse_alpha = (dt / (mouse_lag + dt)).max(0.0).min(1.0);
    let target_yaw_rate = clamp_abs(
        math3::deg_to_rad(-mouse_dx * mouse_deg_per_px) / dt.max(0.0001),
        max_turn_rate
    );
    let target_pitch_rate = clamp_abs(
        math3::deg_to_rad(-mouse_dy * mouse_deg_per_px) / dt.max(0.0001),
        max_turn_rate
    );

    let next_yaw_rate =
        (local.yaw_rate ?? 0.0)
        + (target_yaw_rate - (local.yaw_rate ?? 0.0)) * mouse_alpha;
    let next_pitch_rate =
        (local.pitch_rate ?? 0.0)
        + (target_pitch_rate - (local.pitch_rate ?? 0.0)) * mouse_alpha;

    next_yaw_rate =
        if absf(mouse_dx) <= 0.001 {
            next_yaw_rate * (1.0 / (1.0 + mouse_damp * dt))
        } else {
            next_yaw_rate
        };
    next_pitch_rate =
        if absf(mouse_dy) <= 0.001 {
            next_pitch_rate * (1.0 / (1.0 + mouse_damp * dt))
        } else {
            next_pitch_rate
        };

    let yawed_forward = math3::rotate_axis(
        current_forward["x"] ?? 0.0,
        current_forward["y"] ?? 1.0,
        current_forward["z"] ?? 0.0,
        current_up["x"] ?? 0.0,
        current_up["y"] ?? 1.0,
        current_up["z"] ?? 0.0,
        next_yaw_rate * dt
    );
    let yawed_right = math3::rotate_axis(
        current_right["x"] ?? 1.0,
        current_right["y"] ?? 0.0,
        current_right["z"] ?? 0.0,
        current_up["x"] ?? 0.0,
        current_up["y"] ?? 1.0,
        current_up["z"] ?? 0.0,
        next_yaw_rate * dt
    );
    let pitched_forward = math3::rotate_axis(
        yawed_forward["x"] ?? (current_forward["x"] ?? 0.0),
        yawed_forward["y"] ?? (current_forward["y"] ?? 1.0),
        yawed_forward["z"] ?? (current_forward["z"] ?? 0.0),
        yawed_right["x"] ?? (current_right["x"] ?? 1.0),
        yawed_right["y"] ?? (current_right["y"] ?? 0.0),
        yawed_right["z"] ?? (current_right["z"] ?? 0.0),
        next_pitch_rate * dt
    );
    let pitched_up = math3::rotate_axis(
        current_up["x"] ?? 0.0,
        current_up["y"] ?? 1.0,
        current_up["z"] ?? 0.0,
        yawed_right["x"] ?? (current_right["x"] ?? 1.0),
        yawed_right["y"] ?? (current_right["y"] ?? 0.0),
        yawed_right["z"] ?? (current_right["z"] ?? 0.0),
        next_pitch_rate * dt
    );

    let horizon_assist = (local.flight_horizon_assist ?? 0.28).max(0.0) * atmo_alpha;
    let assisted_up_x =
        (pitched_up["x"] ?? (current_up["x"] ?? 0.0))
        + ((radial_up["x"] ?? 0.0) - (pitched_up["x"] ?? (current_up["x"] ?? 0.0)))
            * (horizon_assist * dt).min(1.0);
    let assisted_up_y =
        (pitched_up["y"] ?? (current_up["y"] ?? 1.0))
        + ((radial_up["y"] ?? 1.0) - (pitched_up["y"] ?? (current_up["y"] ?? 1.0)))
            * (horizon_assist * dt).min(1.0);
    let assisted_up_z =
        (pitched_up["z"] ?? (current_up["z"] ?? 0.0))
        + ((radial_up["z"] ?? 0.0) - (pitched_up["z"] ?? (current_up["z"] ?? 0.0)))
            * (horizon_assist * dt).min(1.0);

    let flight_basis = basis_from_forward_up(
        pitched_forward["x"] ?? (current_forward["x"] ?? 0.0),
        pitched_forward["y"] ?? (current_forward["y"] ?? 1.0),
        pitched_forward["z"] ?? (current_forward["z"] ?? 0.0),
        assisted_up_x,
        assisted_up_y,
        assisted_up_z,
        radial_up["x"] ?? 0.0,
        radial_up["y"] ?? 1.0,
        radial_up["z"] ?? 0.0
    );
    let forward = flight_basis["forward"] ?? current_forward;
    let right = flight_basis["right"] ?? current_right;
    let up = flight_basis["up"] ?? current_up;

    let accel_fwd = meters_to_render(
        units_per_meter,
        (local.flight_thrust_forward_mps2 ?? 42.0).max(0.0)
    );
    let accel_side = meters_to_render(
        units_per_meter,
        (local.flight_thrust_strafe_mps2 ?? 26.0).max(0.0)
    );
    let accel_vertical = meters_to_render(
        units_per_meter,
        (local.flight_lift_thrust_mps2 ?? 24.0).max(0.0)
    );
    let gravity_surface = meters_to_render(
        units_per_meter,
        (local.surface_gravity_mps2 ?? 9.81).max(0.0)
    );
    let gravity_ratio = (ground_radius / radius.max(ground_radius + 0.000001));
    let gravity_accel = gravity_surface * gravity_ratio * gravity_ratio;

    let accel_x =
        (forward["x"] ?? 0.0) * (thrust_axis * accel_fwd)
        + (right["x"] ?? 0.0) * (strafe_axis * accel_side)
        + (up["x"] ?? 0.0) * (vertical_axis * accel_vertical)
        - (radial_up["x"] ?? 0.0) * gravity_accel;
    let accel_y =
        (forward["y"] ?? 1.0) * (thrust_axis * accel_fwd)
        + (right["y"] ?? 0.0) * (strafe_axis * accel_side)
        + (up["y"] ?? 0.0) * (vertical_axis * accel_vertical)
        - (radial_up["y"] ?? 0.0) * gravity_accel;
    let accel_z =
        (forward["z"] ?? 0.0) * (thrust_axis * accel_fwd)
        + (right["z"] ?? 0.0) * (strafe_axis * accel_side)
        + (up["z"] ?? 1.0) * (vertical_axis * accel_vertical)
        - (radial_up["z"] ?? 0.0) * gravity_accel;

    let next_vel_x = vel_x + accel_x * dt;
    let next_vel_y = vel_y + accel_y * dt;
    let next_vel_z = vel_z + accel_z * dt;

    let drag =
        (local.flight_vacuum_drag ?? 0.02)
        + (local.flight_atmo_drag ?? 0.70) * atmo_alpha * atmo_alpha
        + (
            if absf(thrust_axis) + absf(strafe_axis) + absf(vertical_axis) <= 0.001 {
                local.flight_idle_drag ?? 0.05
            } else {
                0.0
            }
        );
    next_vel_x = next_vel_x * (1.0 / (1.0 + drag * dt));
    next_vel_y = next_vel_y * (1.0 / (1.0 + drag * dt));
    next_vel_z = next_vel_z * (1.0 / (1.0 + drag * dt));

    let max_speed = meters_to_render(
        units_per_meter,
        ((local.flight_max_speed_atmo_mps ?? 950.0) * atmo_alpha)
            + ((local.flight_max_speed_vacuum_mps ?? 3600.0) * (1.0 - atmo_alpha))
    )
        .max(0.000001);
    let speed = vec_len(next_vel_x, next_vel_y, next_vel_z);
    if speed > max_speed {
        let speed_scale = max_speed / speed.max(0.000001);
        next_vel_x = next_vel_x * speed_scale;
        next_vel_y = next_vel_y * speed_scale;
        next_vel_z = next_vel_z * speed_scale;
    }

    let next_pos_x = pos_x + next_vel_x * dt;
    let next_pos_y = pos_y + next_vel_y * dt;
    let next_pos_z = pos_z + next_vel_z * dt;
    let next_radius = vec_len(next_pos_x, next_pos_y, next_pos_z);
    let min_radius = ground_radius + ship_min_altitude;
    let clamp_normal =
        if next_radius > 0.000001 {
            math3::normalize_or_zero(next_pos_x, next_pos_y, next_pos_z)
        } else {
            radial_up
        };

    if next_radius < min_radius {
        next_pos_x = (clamp_normal["x"] ?? 0.0) * min_radius;
        next_pos_y = (clamp_normal["y"] ?? 1.0) * min_radius;
        next_pos_z = (clamp_normal["z"] ?? 0.0) * min_radius;
        let inward_speed = dot(
            next_vel_x,
            next_vel_y,
            next_vel_z,
            clamp_normal["x"] ?? 0.0,
            clamp_normal["y"] ?? 1.0,
            clamp_normal["z"] ?? 0.0
        );
        if inward_speed < 0.0 {
            next_vel_x -= (clamp_normal["x"] ?? 0.0) * inward_speed;
            next_vel_y -= (clamp_normal["y"] ?? 1.0) * inward_speed;
            next_vel_z -= (clamp_normal["z"] ?? 0.0) * inward_speed;
        }
    }

    let final_radius = vec_len(next_pos_x, next_pos_y, next_pos_z).max(min_radius);
    let final_up = math3::normalize_or_zero(next_pos_x, next_pos_y, next_pos_z);
    let final_basis = basis_from_forward_up(
        forward["x"] ?? 0.0,
        forward["y"] ?? 1.0,
        forward["z"] ?? 0.0,
        up["x"] ?? (final_up["x"] ?? 0.0),
        up["y"] ?? (final_up["y"] ?? 1.0),
        up["z"] ?? (final_up["z"] ?? 0.0),
        final_up["x"] ?? 0.0,
        final_up["y"] ?? 1.0,
        final_up["z"] ?? 0.0
    );
    let final_forward = final_basis["forward"] ?? forward;
    let final_right = final_basis["right"] ?? right;
    let final_up_basis = final_basis["up"] ?? up;

    let ship_euler = apply_ship_proxy(
        local,
        runtime,
        next_pos_x,
        next_pos_y,
        next_pos_z,
        final_right,
        final_up_basis,
        final_forward
    );

    local = apply_flight_camera(
        local,
        runtime,
        world,
        dt,
        units_per_meter,
        next_pos_x,
        next_pos_y,
        next_pos_z,
        final_right,
        final_up_basis,
        final_forward,
        next_yaw_rate,
        next_pitch_rate,
        strafe_axis,
        vertical_axis,
        (local.camera_mode ?? "cockpit") != "free"
    );

    let display_altitude_km = ((final_radius - ground_radius).max(0.0)) * km_per_wu;
    let display_speed_kms = vec_len(next_vel_x, next_vel_y, next_vel_z) * km_per_wu;
    let radial_speed = dot(
        next_vel_x,
        next_vel_y,
        next_vel_z,
        final_up["x"] ?? 0.0,
        final_up["y"] ?? 1.0,
        final_up["z"] ?? 0.0
    );
    let vfwd = dot(
        next_vel_x,
        next_vel_y,
        next_vel_z,
        final_forward["x"] ?? 0.0,
        final_forward["y"] ?? 1.0,
        final_forward["z"] ?? 0.0
    );
    let vright = dot(
        next_vel_x,
        next_vel_y,
        next_vel_z,
        final_right["x"] ?? 1.0,
        final_right["y"] ?? 0.0,
        final_right["z"] ?? 0.0
    );

    local.snx = final_up["x"] ?? 0.0;
    local.sny = final_up["y"] ?? 1.0;
    local.snz = final_up["z"] ?? 0.0;
    local.sfx = final_forward["x"] ?? 0.0;
    local.sfy = final_forward["y"] ?? 1.0;
    local.sfz = final_forward["z"] ?? 0.0;
    local.srx = final_right["x"] ?? 1.0;
    local.sry = final_right["y"] ?? 0.0;
    local.srz = final_right["z"] ?? 0.0;
    local.radius = final_radius;
    local.pos_x = next_pos_x;
    local.pos_y = next_pos_y;
    local.pos_z = next_pos_z;
    local.vel_x = next_vel_x;
    local.vel_y = next_vel_y;
    local.vel_z = next_vel_z;
    local.vfwd = vfwd;
    local.vright = vright;
    local.vrad = radial_speed;
    local.yaw_rate = next_yaw_rate;
    local.pitch_rate = next_pitch_rate;
    local.ship_vis_pitch = ship_euler["pitch"] ?? (local.ship_vis_pitch ?? pi());
    local.ship_vis_yaw = ship_euler["yaw"] ?? (local.ship_vis_yaw ?? -(pi() * 0.5));
    local.ship_vis_roll = ship_euler["roll"] ?? (local.ship_vis_roll ?? 0.0);
    local.ship_display_altitude_km = display_altitude_km;
    local.ship_display_speed_kms = display_speed_kms;
    local.ship_display_altitude_m = display_altitude_km * 1000.0;
    local.ship_display_speed_mps = display_speed_kms * 1000.0;
    local.prev_mouse_x = mouse_x;
    local.prev_mouse_y = mouse_y;
    local.mouse_look_seeded = true;
    local
}
