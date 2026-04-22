import "std/math3" as math3;
import "std/runtime_scene" as scene_rt;
import "std/flight_handoff" as flight_handoff;

fn clamp_abs(value, limit) {
    if value > limit { return limit; }
    if value < -limit { return -limit; }
    value
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

fn step(local, runtime, world, input, game, persist, dt, scene_elapsed_ms) {
    if input.just_pressed("Escape") || input.just_pressed("Esc") {
        game.jump("/scenes/main/scene.yml");
        return local;
    }
    if input.just_pressed("F10") {
        flight_handoff::store_launch_snapshot(persist, local);
        game.jump("/scenes/flight/scene.yml");
        return local;
    }

    let center_x = local.cockpit_center_x ?? 320.0;
    let center_y = local.cockpit_center_y ?? 180.0;
    let yaw_limit = (local.cockpit_mouse_yaw_limit_deg ?? 10.0).max(0.5);
    let pitch_limit = (local.cockpit_mouse_pitch_limit_deg ?? 7.0).max(0.5);
    let lag = (local.cockpit_mouse_lag_sec ?? 0.08).max(0.01);
    let look_alpha = (dt / (lag + dt)).max(0.0).min(1.0);

    let mouse_yaw_deg = clamp_abs(
        ((input.mouse_x - center_x) / center_x.max(1.0)) * yaw_limit,
        yaw_limit
    );
    let mouse_pitch_deg = clamp_abs(
        (-(input.mouse_y - center_y) / center_y.max(1.0)) * pitch_limit,
        pitch_limit
    );

    let next_yaw_deg =
        (local.cockpit_look_yaw_deg ?? 0.0)
        + (mouse_yaw_deg - (local.cockpit_look_yaw_deg ?? 0.0)) * look_alpha;
    let next_pitch_deg =
        (local.cockpit_look_pitch_deg ?? 0.0)
        + (mouse_pitch_deg - (local.cockpit_look_pitch_deg ?? 0.0)) * look_alpha;

    let sway_phase = scene_elapsed_ms * 0.00075;
    let eye_x = (local.cockpit_eye_x ?? 0.0) + sin(sway_phase * 1.2) * 0.018;
    let eye_y = (local.cockpit_eye_y ?? 0.08) + sin(sway_phase * 0.7) * 0.014;
    let eye_z = (local.cockpit_eye_z ?? 3.75) + cos(sway_phase * 0.35) * 0.022;

    let world_up = #{ x: 0.0, y: 1.0, z: 0.0 };
    let base_forward = math3::normalize_or_zero(0.0, -0.05, -1.0);
    let yawed_forward = math3::rotate_axis(
        base_forward["x"] ?? 0.0,
        base_forward["y"] ?? 0.0,
        base_forward["z"] ?? -1.0,
        0.0,
        1.0,
        0.0,
        math3::deg_to_rad(next_yaw_deg)
    );
    let yaw_basis = basis_from_forward_up(
        yawed_forward["x"] ?? 0.0,
        yawed_forward["y"] ?? 0.0,
        yawed_forward["z"] ?? -1.0,
        0.0,
        1.0,
        0.0,
        0.0,
        1.0,
        0.0
    );
    let yaw_forward = yaw_basis["forward"] ?? base_forward;
    let yaw_right = yaw_basis["right"] ?? #{ x: 1.0, y: 0.0, z: 0.0 };
    let pitched_forward = math3::rotate_axis(
        yaw_forward["x"] ?? 0.0,
        yaw_forward["y"] ?? 0.0,
        yaw_forward["z"] ?? -1.0,
        yaw_right["x"] ?? 1.0,
        yaw_right["y"] ?? 0.0,
        yaw_right["z"] ?? 0.0,
        math3::deg_to_rad(next_pitch_deg)
    );
    let basis = basis_from_forward_up(
        pitched_forward["x"] ?? 0.0,
        pitched_forward["y"] ?? 0.0,
        pitched_forward["z"] ?? -1.0,
        0.0,
        1.0,
        0.0,
        0.0,
        1.0,
        0.0
    );
    let forward = basis["forward"] ?? base_forward;
    let right = basis["right"] ?? #{ x: 1.0, y: 0.0, z: 0.0 };
    let up = basis["up"] ?? world_up;

    let look_distance = (local.cockpit_look_distance ?? 12.0).max(1.0);
    world.set_camera_3d_look_at(
        eye_x,
        eye_y,
        eye_z,
        eye_x + (forward["x"] ?? 0.0) * look_distance,
        eye_y + (forward["y"] ?? 0.0) * look_distance,
        eye_z + (forward["z"] ?? -1.0) * look_distance
    );
    world.set_camera_3d_up(up["x"] ?? 0.0, up["y"] ?? 1.0, up["z"] ?? 0.0);

    scene_rt::visible(runtime, "cockpit-sim-mesh", true);
    scene_rt::set(runtime, "cockpit-sim-mesh", "obj.scale", local.cockpit_mesh_scale ?? 0.22);
    set_obj_world(runtime, "cockpit-sim-mesh", eye_x, eye_y, eye_z);
    set_obj_camera_world(runtime, "cockpit-sim-mesh", eye_x, eye_y, eye_z);
    set_obj_view(runtime, "cockpit-sim-mesh", right, up, forward);

    local.cockpit_look_yaw_deg = next_yaw_deg;
    local.cockpit_look_pitch_deg = next_pitch_deg;
    local
}
