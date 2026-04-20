import "std/math3" as math3;
import "std/runtime_scene" as scene_rt;
import "vehicle/control" as control;
import "vehicle/environment" as environment;
import "vehicle/handoff" as handoff;
import "vehicle/state" as vehicle_state_mod;

// vehicle_phase:
// - "pad": grounded on the surface, no local drift, yaw around the local normal only
// - "hover": near-surface local-horizon flight with strong radial authority
// - "flight": detached/free-flight regime
// - "orbit": high-altitude presentation state layered on top of flight

fn current_ship_runtime_state(state, vehicle) {
    let runtime_state = state.vehicle_ship_runtime ?? ();
    if type_of(runtime_state) == "()" {
        return vehicle.ship_runtime_state();
    }
    runtime_state.normalized()
}

fn current_vehicle_surface_mode(state, vehicle) {
    current_ship_runtime_state(state, vehicle).surface_mode ?? "detached"
}

fn current_vehicle_phase(state, vehicle) {
    let phase = state.vehicle_phase ?? ();
    if type_of(phase) == "string" && phase != "" {
        return phase;
    }
    let surface_mode = current_vehicle_surface_mode(state, vehicle);
    if surface_mode == "grounded" {
        return "pad";
    }
    if surface_mode == "surface_locked" {
        return "hover";
    }
    let altitude_km = state.vehicle_altitude_km ?? 0.0;
    if altitude_km > 25.0 {
        return "orbit";
    }
    "flight"
}

fn current_vehicle_model(state) {
    vehicle_state_mod::current_vehicle_model(state)
}

fn vehicle_model_source(model) {
    if model == "rover" { "/assets/3d/rover.obj" } else { "/assets/3d/lm.obj" }
}

fn vehicle_model_scale(model) {
    if model == "rover" { 0.000024 } else { 0.0000075 }
}

fn vehicle_model_visual_lift(surface_clearance_wu, model) {
    if model == "rover" { surface_clearance_wu * 0.28 } else { surface_clearance_wu * 1.15 }
}

fn reset_vehicle_for_model(state, vehicle, body_ctx, model) {
    let surface_radius = body_ctx["surface_radius"] ?? 1.0;
    let km_per_wu = body_ctx["km_per_wu"] ?? 1.0;
    let ship_tuning = control::current_ship_profile_tuning(state, vehicle);
    let surface_clearance_wu = ship_tuning.surface_clearance_wu(km_per_wu.max(0.0001));
    state.vehicle_model = if model == "rover" { "rover" } else { "lm" };
    state = reset_vehicle_to_surface_state(state, vehicle, surface_radius, surface_clearance_wu);
    state
}

fn build_vehicle_runtime_telemetry_snapshot(state, vehicle, ship_x, ship_y, ship_vx, ship_vy) {
    vehicle.packet_telemetry_from(#{
        x: ship_x,
        y: ship_y,
        vx: ship_vx,
        vy: ship_vy,
        heading_deg: math3::rad_to_deg(state.ship_heading ?? state.vehicle_heading ?? 0.0),
        altitude_km: state.vehicle_altitude_km ?? 0.0,
        tangent_speed_kms: state.vehicle_speed_kms ?? 0.0,
        radial_speed_kms: state.vehicle_vrad_kms ?? 0.0,
        spawn_angle_deg: state.vehicle_spawn_angle_deg ?? 0.0,
        camera_sway: state.cam_sway ?? state.vehicle_camera_sway ?? 0.0,
        radius_wu: state.radius ?? 0.0,
        vfwd_wu_s: state.vfwd ?? 0.0,
        vright_wu_s: state.vright ?? 0.0,
        vrad_wu_s: state.vrad ?? 0.0,
        yaw_rate_rad_s: state.yaw_rate ?? 0.0,
        basis: handoff::build_vehicle_packet_basis_map(state),
        grounded: state.vehicle_grounded ?? false
    }).to_snapshot()
}

fn sync_vehicle_ship_runtime(
    state,
    vehicle,
    body_snapshot,
    control_state,
    ship_x,
    ship_y,
    ship_vx,
    ship_vy,
    surface_contact,
    prefer_grounded_on_contact,
    request_surface_lock,
    request_detach
) {
    let telemetry = build_vehicle_runtime_telemetry_snapshot(
        state,
        vehicle,
        ship_x,
        ship_y,
        ship_vx,
        ship_vy
    );
    let environment_binding =
        environment::build_vehicle_runtime_environment_binding(state, body_snapshot, vehicle);
    let runtime_input = vehicle
        .ship_runtime_input_from_parts(control_state, telemetry, environment_binding)
        .with_surface_contact(surface_contact)
        .with_prefer_grounded_on_contact(prefer_grounded_on_contact)
        .with_surface_lock_request(request_surface_lock)
        .with_detach_request(request_detach);
    let step = vehicle.ship_runtime_step(
        control::current_vehicle_profile_name(state, vehicle),
        current_ship_runtime_state(state, vehicle),
        runtime_input
    );
    state.vehicle_ship_runtime = step.state;
    state.vehicle_grounded = step.telemetry.grounded ?? state.vehicle_grounded ?? false;
    state
}

fn clear_surface_kinematics(state) {
    state.radius = ();
    state.vfwd = ();
    state.vright = ();
    state.vrad = ();
    state.yaw_rate = ();
    state.snx = ();
    state.sny = ();
    state.snz = ();
    state.sfx = ();
    state.sfy = ();
    state.sfz = ();
    state.srx = ();
    state.sry = ();
    state.srz = ();
    state.cam_nx = ();
    state.cam_ny = ();
    state.cam_nz = ();
    state.cam_fx = ();
    state.cam_fy = ();
    state.cam_fz = ();
    state.cam_sway = ();
    state.vehicle_orbit_seeded = false;
    state
}

fn seed_vehicle_surface_state(state, surface_radius, surface_clearance_wu) {
    let spawn_angle_deg = state.vehicle_spawn_angle_deg ?? 0.0;
    let spawn_angle = math3::deg_to_rad(spawn_angle_deg);
    let heading = state.vehicle_heading ?? 0.0;
    let radius = surface_radius + surface_clearance_wu;

    let snx = spawn_angle.cos();
    let sny = spawn_angle.sin();
    let snz = 0.0;
    let tangent_fx = -spawn_angle.sin();
    let tangent_fy = spawn_angle.cos();
    let tangent_fz = 0.0;
    let tangent_rx = 0.0;
    let tangent_ry = 0.0;
    let tangent_rz = 1.0;

    let rotated_forward = math3::rotate_axis(tangent_fx, tangent_fy, tangent_fz, snx, sny, snz, heading);
    let rotated_right = math3::rotate_axis(tangent_rx, tangent_ry, tangent_rz, snx, sny, snz, heading);

    state.snx = snx;
    state.sny = sny;
    state.snz = snz;
    state.sfx = rotated_forward["x"] ?? tangent_fx;
    state.sfy = rotated_forward["y"] ?? tangent_fy;
    state.sfz = rotated_forward["z"] ?? tangent_fz;
    state.srx = rotated_right["x"] ?? tangent_rx;
    state.sry = rotated_right["y"] ?? tangent_ry;
    state.srz = rotated_right["z"] ?? tangent_rz;
    state.radius = radius;
    state.vfwd = 0.0;
    state.vright = 0.0;
    state.vrad = 0.0;
    state.yaw_rate = 0.0;
    state.ship_heading = heading;
    state.ship_vis_pitch = 3.1415926535;
    state.ship_vis_yaw = -1.57079632675;
    state.ship_vis_roll = 0.0;
    state.cam_nx = snx;
    state.cam_ny = sny;
    state.cam_nz = snz;
    state.cam_fx = rotated_forward["x"] ?? tangent_fx;
    state.cam_fy = rotated_forward["y"] ?? tangent_fy;
    state.cam_fz = rotated_forward["z"] ?? tangent_fz;
    state.cam_sway = 0.0;
    state.vehicle_camera_sway = 0.0;
    state.vehicle_altitude_km = 0.0;
    state.vehicle_speed_kms = 0.0;
    state.vehicle_vrad_kms = 0.0;
    state.vehicle_grounded = true;
    state.vehicle_phase = "pad";
    state.vehicle_orbit_seeded = true;
    state
}

fn phase_grounded_speed_kms(ship_tuning, km_per_wu) {
    (ship_tuning.grounded_speed_threshold_wu_s * km_per_wu).max(0.03)
}

fn phase_hover_exit_altitude_km(ship_tuning) {
    (ship_tuning.surface_contact_altitude_threshold_km * 8.0).max(0.45)
}

fn phase_hover_exit_speed_kms(ship_tuning, km_per_wu) {
    (phase_grounded_speed_kms(ship_tuning, km_per_wu) * 2.5).max(0.12)
}

fn phase_hover_reentry_altitude_km(ship_tuning) {
    (ship_tuning.surface_contact_altitude_threshold_km * 5.0).max(0.25)
}

fn phase_orbit_altitude_km(body_snapshot) {
    if body_snapshot.exists {
        body_snapshot.atmosphere_top_km.max(25.0) * 0.45
    } else {
        25.0
    }
}

fn camera_phase_profile(phase, altitude_px_frac, zoom_scale) {
    if phase == "pad" {
        return #{
            cam_lift: (0.003 + altitude_px_frac * 0.010) * (0.96 + zoom_scale * 0.04),
            cam_back: (0.014 + altitude_px_frac * 0.08) * zoom_scale,
            normal_alpha: 0.22,
            forward_alpha: 0.18,
            sway_scale: 0.22
        };
    }
    if phase == "hover" {
        return #{
            cam_lift: (0.004 + altitude_px_frac * 0.020) * (0.94 + zoom_scale * 0.06),
            cam_back: (0.018 + altitude_px_frac * 0.20) * zoom_scale,
            normal_alpha: 0.14,
            forward_alpha: 0.12,
            sway_scale: 0.45
        };
    }
    if phase == "orbit" {
        return #{
            cam_lift: (0.006 + altitude_px_frac * 0.040) * (0.90 + zoom_scale * 0.10),
            cam_back: (0.040 + altitude_px_frac * 0.78) * zoom_scale,
            normal_alpha: 0.08,
            forward_alpha: 0.08,
            sway_scale: 1.00
        };
    }
    #{
        cam_lift: (0.004 + altitude_px_frac * 0.032) * (0.92 + zoom_scale * 0.08),
        cam_back: (0.024 + altitude_px_frac * 0.50) * zoom_scale,
        normal_alpha: 0.10,
        forward_alpha: 0.10,
        sway_scale: 0.70
    }
}

fn reset_vehicle_to_surface_state(state, vehicle, surface_radius, surface_clearance_wu) {
    state = clear_surface_kinematics(state);
    state.vehicle_heading = 0.0;
    state.vehicle_speed_kms = 0.0;
    state.vehicle_vrad_kms = 0.0;
    state.vehicle_camera_sway = 0.0;
    state.vehicle_camera_zoom = 0.0;
    state.vehicle_grounded = true;
    state.vehicle_phase = "pad";
    state.vehicle_ship_runtime = vehicle.ship_runtime_state();
    state = control::set_vehicle_target_heading(state, vehicle, 0.0);
    state = control::set_vehicle_target_altitude(state, vehicle, 0.0);
    state = seed_vehicle_surface_state(state, surface_radius, surface_clearance_wu);
    state
}

fn step(state, runtime, world, input, vehicle, body_ctx, dt) {
    let body_snapshot = body_ctx["body_snapshot"];
    let cx = body_ctx["cx"] ?? 0.0;
    let cy = body_ctx["cy"] ?? 0.0;
    let surface_radius = body_ctx["surface_radius"] ?? 1.0;
    let km_per_wu = body_ctx["km_per_wu"] ?? 1.0;
    let gravity_mu = body_snapshot.gravity_mu.max(0.0);
    let surface_gravity_wu = if gravity_mu > 0.0 {
        gravity_mu / (surface_radius.max(0.1) * surface_radius.max(0.1))
    } else {
        0.01
    };
    let surface_circular_speed = if gravity_mu > 0.0 {
        sqrtf(gravity_mu / surface_radius.max(0.1))
    } else {
        1.0
    };
    let ship_tuning = control::current_ship_profile_tuning(state, vehicle);
    let acc_fwd = ship_tuning.forward_accel_wu_s2(surface_gravity_wu);
    let acc_side = ship_tuning.side_accel_wu_s2(surface_gravity_wu);
    let acc_lift = ship_tuning.lift_accel_wu_s2(surface_gravity_wu);
    let acc_main = ship_tuning.main_engine_accel_wu_s2(surface_gravity_wu);
    let max_spd = ship_tuning.max_speed_wu_s(surface_circular_speed).max(0.5);
    let max_vrad = ship_tuning.max_radial_speed_wu_s(surface_circular_speed).max(0.25);
    let lin_damp = ship_tuning.linear_damp;
    let side_trim = ship_tuning.side_trim;
    let side_thrust_trim = ship_tuning.side_thrust_trim;
    let yaw_damp = ship_tuning.yaw_damp;
    let yaw_max = ship_tuning.yaw_max;
    let surface_contact_altitude_km = ship_tuning.surface_contact_altitude_threshold_km;
    let takeoff_lift_threshold = ship_tuning.takeoff_lift_threshold;
    let grounded_speed_threshold_kms = phase_grounded_speed_kms(ship_tuning, km_per_wu);
    let hover_exit_altitude_km = phase_hover_exit_altitude_km(ship_tuning);
    let hover_exit_speed_kms = phase_hover_exit_speed_kms(ship_tuning, km_per_wu);
    let hover_reentry_altitude_km = phase_hover_reentry_altitude_km(ship_tuning);
    let orbit_altitude_km = phase_orbit_altitude_km(body_snapshot);
    let atmo_top_alt_wu =
        if body_snapshot.exists {
            (body_snapshot.atmosphere_top_km / km_per_wu.max(0.0001)).max(0.0)
        } else {
            0.0
        };
    let atmo_drag_max =
        if body_snapshot.exists {
            body_snapshot.atmosphere_drag_max
        } else {
            state.atmo_drag_max ?? 0.0
        };
    let surface_clearance_wu =
        ship_tuning.surface_clearance_wu(km_per_wu.max(0.0001));
    let surface_spin_omega = math3::deg_to_rad(state.rotspeed ?? 0.0);
    let ship_id = if (state.ship_id ?? 0) > 0 { state.ship_id ?? 0 } else { vehicle.active() };

    if ship_id > 0 {
        state.ship_id = ship_id;
    }
    if !(state.vehicle_orbit_seeded ?? false) {
        state = seed_vehicle_surface_state(state, surface_radius, surface_clearance_wu);
    }

    let snx = state.snx ?? 1.0;
    let sny = state.sny ?? 0.0;
    let snz = state.snz ?? 0.0;
    let sfx = state.sfx ?? 0.0;
    let sfy = state.sfy ?? 1.0;
    let sfz = state.sfz ?? 0.0;
    let srx = state.srx ?? 0.0;
    let sry = state.sry ?? 0.0;
    let srz = state.srz ?? 1.0;
    let radius = (state.radius ?? (surface_radius + surface_clearance_wu)).max(surface_radius + surface_clearance_wu);
    let vrad = state.vrad ?? 0.0;
    let vfwd = state.vfwd ?? 0.0;
    let vright = state.vright ?? 0.0;
    let yaw_rate = state.yaw_rate ?? 0.0;
    let ship_heading = state.ship_heading ?? (state.vehicle_heading ?? 0.0);
    let ship_vis_pitch = state.ship_vis_pitch ?? 3.1415926535;
    let ship_vis_yaw = state.ship_vis_yaw ?? -1.57079632675;
    let ship_vis_roll = state.ship_vis_roll ?? 0.0;
    let cam_nx = state.cam_nx ?? snx;
    let cam_ny = state.cam_ny ?? sny;
    let cam_nz = state.cam_nz ?? snz;
    let cam_fx = state.cam_fx ?? sfx;
    let cam_fy = state.cam_fy ?? sfy;
    let cam_fz = state.cam_fz ?? sfz;
    let cam_sway = state.cam_sway ?? state.vehicle_camera_sway ?? 0.0;
    let phase = current_vehicle_phase(state, vehicle);
    let vehicle_model = current_vehicle_model(state);
    let active_ship_id = ship_id;

    if active_ship_id <= 0 {
        let ship_spawn_x = cx + snx * radius;
        let ship_spawn_y = cy + sny * radius;
        active_ship_id = environment::ensure_ship(world, ship_spawn_x, ship_spawn_y, ship_heading);
        state.ship_id = active_ship_id;
        if active_ship_id > 0 {
            vehicle.set_active(active_ship_id);
            world.set_transform(active_ship_id, ship_spawn_x, ship_spawn_y, ship_heading);
            world.set_physics(active_ship_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
    } else if vehicle.active() <= 0 {
        vehicle.set_active(active_ship_id);
    }

    let ship_x = cx + snx * radius;
    let ship_y = cy + sny * radius;
    let thrust_axis = 0.0;
    let strafe_axis = 0.0;
    let yaw_axis = 0.0;
    let main_engine_on = false;
    let lift_on = false;
    let runtime_surface_contact = false;
    let runtime_prefer_grounded = true;
    let runtime_request_surface_lock = false;
    let runtime_request_detach = false;

    if active_ship_id > 0 {
        let xf = world.transform(active_ship_id) ?? #{};
        ship_x = xf["x"] ?? ship_x;
        ship_y = xf["y"] ?? ship_y;

        let frame_control = control::current_vehicle_frame_control(state, vehicle, input);
        let frame_control_assists = frame_control.assists;
        let vehicle_input = frame_control.intent();
        thrust_axis = vehicle_input.throttle ?? 0.0;
        strafe_axis = vehicle_input.strafe ?? 0.0;
        yaw_axis = vehicle_input.yaw ?? 0.0;
        main_engine_on = vehicle_input.main_engine ?? false;
        lift_on = (vehicle_input.lift ?? 0.0) > takeoff_lift_threshold;

        let altitude_wu = (radius - surface_radius).max(0.0);
        let altitude_km = altitude_wu * km_per_wu;
        let tangent_speed_kms = sqrtf(vfwd * vfwd + vright * vright) * km_per_wu;
        let radial_speed_kms = vrad * km_per_wu;

        let yaw_target =
            if absf(yaw_axis) > 0.001 {
                state = control::set_vehicle_target_heading(state, vehicle, ship_heading);
                yaw_axis * yaw_max
            } else if frame_control_assists.heading_hold ?? false {
                let target_heading = math3::nearest_angle(frame_control.target_heading_rad(ship_heading), ship_heading);
                let heading_error = target_heading - ship_heading;
                math3::clamp_unit(
                    heading_error * ship_tuning.heading_hold_kp
                ) * yaw_max
            } else {
                0.0
            };
        let yaw_blend = ship_tuning.yaw_blend(dt);
        yaw_rate += (yaw_target - yaw_rate) * yaw_blend;
        if absf(yaw_axis) <= 0.001 && !(frame_control_assists.heading_hold ?? false) {
            yaw_rate *= 1.0 / (1.0 + yaw_damp * dt);
        }
        if absf(yaw_rate) > yaw_max {
            yaw_rate = if yaw_rate > 0.0 { yaw_max } else { -yaw_max };
        }

        let dyaw = yaw_rate * dt;
        if absf(dyaw) > 0.00001 {
            let rotated_forward = math3::rotate_axis(sfx, sfy, sfz, snx, sny, snz, dyaw);
            sfx = rotated_forward["x"] ?? sfx;
            sfy = rotated_forward["y"] ?? sfy;
            sfz = rotated_forward["z"] ?? sfz;
            let rotated_right = math3::rotate_axis(srx, sry, srz, snx, sny, snz, dyaw);
            srx = rotated_right["x"] ?? srx;
            sry = rotated_right["y"] ?? sry;
            srz = rotated_right["z"] ?? srz;
        }

        if vehicle_model == "rover" {
            let drive_drag = 1.0 / (1.0 + (lin_damp + 0.8) * dt);
            let max_rover_spd = (max_spd * 0.35).max(0.12);
            vfwd += thrust_axis * acc_fwd * dt * 0.55;
            vfwd *= drive_drag;
            vright = 0.0;
            vrad = 0.0;
            radius = surface_radius + surface_clearance_wu;
            if vfwd > max_rover_spd { vfwd = max_rover_spd; }
            if vfwd < -max_rover_spd * 0.55 { vfwd = -max_rover_spd * 0.55; }
            state.vehicle_grounded = true;
            state.vehicle_phase = "pad";
            state = control::set_vehicle_target_altitude(state, vehicle, 0.0);

            runtime_surface_contact = true;
            runtime_prefer_grounded = true;
            runtime_request_surface_lock = false;
            runtime_request_detach = false;
        } else if phase == "pad" {
            vfwd = 0.0;
            vright = 0.0;
            vrad = 0.0;
            radius = surface_radius + surface_clearance_wu;
            state.vehicle_grounded = true;
            state.vehicle_phase = "pad";
            state = control::set_vehicle_target_altitude(state, vehicle, 0.0);

            runtime_surface_contact = true;
            runtime_prefer_grounded = true;
            runtime_request_surface_lock = false;
            runtime_request_detach = false;

            if lift_on {
                state.vehicle_grounded = false;
                state.vehicle_phase = "hover";
                runtime_surface_contact = true;
                runtime_prefer_grounded = false;
                runtime_request_surface_lock = true;
            }
        } else {
            vfwd += thrust_axis * acc_fwd * dt;
            vright += strafe_axis * acc_side * dt;
            if main_engine_on { vfwd += acc_main * dt; }

            let trim = side_trim + thrust_axis.max(0.0) * side_thrust_trim;
            let hover_stability =
                if phase == "hover" {
                    1.0 / (1.0 + 0.9 * dt)
                } else {
                    1.0
                };
            vright *= 1.0 / (1.0 + trim * dt);
            vfwd *= 1.0 / (1.0 + lin_damp * dt);
            vright *= 1.0 / (1.0 + lin_damp * dt);
            vfwd *= hover_stability;
            vright *= hover_stability;

            if atmo_top_alt_wu > 0.0 {
                let atmo_alpha = math3::clamp01((atmo_top_alt_wu - altitude_wu) / atmo_top_alt_wu.max(0.1));
                let atmo_drag = atmo_alpha * atmo_alpha * atmo_drag_max.max(0.0);
                let radial_drag = if phase == "hover" { 1.35 } else { 1.2 };
                vrad *= 1.0 / (1.0 + atmo_drag * dt * radial_drag);
                vfwd *= 1.0 / (1.0 + atmo_drag * dt);
                vright *= 1.0 / (1.0 + atmo_drag * dt);
            }

            if vfwd > max_spd { vfwd = max_spd; }
            if vfwd < -max_spd * 0.5 { vfwd = -max_spd * 0.5; }
            if vright > max_spd { vright = max_spd; }
            if vright < -max_spd { vright = -max_spd; }

            let ship_world_x = cx + snx * radius;
            let ship_world_y = cy + sny * radius;
            let vt2 = vfwd * vfwd + vright * vright;
            let ship_gravity = world.body_gravity("generated-planet", ship_world_x, ship_world_y, snz * radius);
            let grav_radial = (ship_gravity["ax"] ?? 0.0) * snx
                + (ship_gravity["ay"] ?? 0.0) * sny
                + (ship_gravity["az"] ?? 0.0) * snz;
            let centrifugal = vt2 / radius.max(surface_radius + surface_clearance_wu);

            let assist_radial_acc =
                if (frame_control_assists.alt_hold ?? false) && !lift_on {
                    let target_altitude_km = frame_control.target_altitude_km(altitude_km);
                    let altitude_error_km = target_altitude_km - altitude_km;
                    (altitude_error_km * ship_tuning.alt_hold_kp
                        - (vrad * km_per_wu) * ship_tuning.alt_hold_kd)
                        .max(-acc_lift * 0.75)
                        .min(acc_lift * 0.75)
                } else {
                    0.0
                };
            let hover_hold_acc =
                if phase == "hover" && !lift_on && !(frame_control_assists.alt_hold ?? false) {
                    (-vrad * 1.5).max(-acc_lift * 0.30).min(acc_lift * 0.30)
                } else {
                    0.0
                };
            let lift_acc = if lift_on { acc_lift } else { 0.0 };

            vrad += (centrifugal + grav_radial + lift_acc + assist_radial_acc + hover_hold_acc) * dt;
            if vrad > max_vrad { vrad = max_vrad; }
            if vrad < -max_vrad { vrad = -max_vrad; }
            radius += vrad * dt;

            let altitude_after_km = ((radius - surface_radius).max(0.0)) * km_per_wu;
            let tangent_speed_after_kms = sqrtf(vfwd * vfwd + vright * vright) * km_per_wu;
            let radial_speed_after_kms = vrad.abs() * km_per_wu;
            let landing_ready =
                altitude_after_km <= surface_contact_altitude_km
                && tangent_speed_after_kms <= grounded_speed_threshold_kms
                && radial_speed_after_kms <= grounded_speed_threshold_kms
                && !lift_on;

            if radius < surface_radius + surface_clearance_wu {
                radius = surface_radius + surface_clearance_wu;
                if landing_ready {
                    vrad = 0.0;
                    vfwd = 0.0;
                    vright = 0.0;
                    state.vehicle_grounded = true;
                    state.vehicle_phase = "pad";
                    state = control::set_vehicle_target_altitude(state, vehicle, 0.0);
                } else if vrad < 0.0 {
                    vrad = -vrad * 0.18;
                }
            }

            if state.vehicle_phase != "pad" {
                let altitude_now_km = ((radius - surface_radius).max(0.0)) * km_per_wu;
                let tangent_now_kms = sqrtf(vfwd * vfwd + vright * vright) * km_per_wu;
                let radial_now_kms = vrad.abs() * km_per_wu;
                let near_surface = altitude_now_km <= hover_reentry_altitude_km;
                let hover_entry_speed = hover_exit_speed_kms * 0.85;
                let should_reenter_hover =
                    near_surface
                    && tangent_now_kms <= hover_entry_speed
                    && radial_now_kms <= hover_entry_speed;
                let should_leave_hover =
                    altitude_now_km > hover_exit_altitude_km
                    || tangent_now_kms > hover_exit_speed_kms
                    || radial_now_kms > hover_exit_speed_kms;

                if phase == "hover" {
                    if should_leave_hover {
                        state.vehicle_phase = "flight";
                    } else {
                        state.vehicle_phase = "hover";
                    }
                } else {
                    if should_reenter_hover {
                        state.vehicle_phase = "hover";
                    } else {
                        state.vehicle_phase = "flight";
                    }
                }
            }

            let post_phase = state.vehicle_phase ?? "flight";
            runtime_surface_contact =
                post_phase == "hover"
                && ((radius - surface_radius).max(0.0) * km_per_wu) <= hover_exit_altitude_km;
            runtime_prefer_grounded = post_phase == "pad";
            runtime_request_surface_lock = post_phase == "hover";
            runtime_request_detach = post_phase == "flight" || post_phase == "orbit";
            state.vehicle_grounded = post_phase == "pad";
        }

        let spin_tang_x_raw = -sny;
        let spin_tang_y_raw = snx;
        let spin_tang_z_raw = 0.0;
        let spin_tang_len = sqrtf(
            spin_tang_x_raw * spin_tang_x_raw
            + spin_tang_y_raw * spin_tang_y_raw
            + spin_tang_z_raw * spin_tang_z_raw
        );
        let spin_tang_x = if spin_tang_len > 0.0001 { spin_tang_x_raw / spin_tang_len } else { sfx };
        let spin_tang_y = if spin_tang_len > 0.0001 { spin_tang_y_raw / spin_tang_len } else { sfy };
        let spin_tang_z = if spin_tang_len > 0.0001 { spin_tang_z_raw / spin_tang_len } else { sfz };
        let carrier_speed = surface_spin_omega * radius * spin_tang_len;
        let carrier_x = spin_tang_x * carrier_speed;
        let carrier_y = spin_tang_y * carrier_speed;
        let carrier_z = spin_tang_z * carrier_speed;
        let total_tang_x = carrier_x + sfx * vfwd + srx * vright;
        let total_tang_y = carrier_y + sfy * vfwd + sry * vright;
        let total_tang_z = carrier_z + sfz * vfwd + srz * vright;
        let tangential_speed = sqrtf(total_tang_x * total_tang_x + total_tang_y * total_tang_y + total_tang_z * total_tang_z);
        if tangential_speed > 0.001 {
            let delta = tangential_speed * dt / radius.max(surface_radius + surface_clearance_wu);
            let tang_x = total_tang_x / tangential_speed;
            let tang_y = total_tang_y / tangential_speed;
            let tang_z = total_tang_z / tangential_speed;

            let rotated_normal = math3::rotate_axis(snx, sny, snz, tang_x, tang_y, tang_z, delta);
            snx = rotated_normal["x"] ?? snx;
            sny = rotated_normal["y"] ?? sny;
            snz = rotated_normal["z"] ?? snz;
            let rotated_forward = math3::rotate_axis(sfx, sfy, sfz, tang_x, tang_y, tang_z, delta);
            sfx = rotated_forward["x"] ?? sfx;
            sfy = rotated_forward["y"] ?? sfy;
            sfz = rotated_forward["z"] ?? sfz;
            let rotated_right = math3::rotate_axis(srx, sry, srz, tang_x, tang_y, tang_z, delta);
            srx = rotated_right["x"] ?? srx;
            sry = rotated_right["y"] ?? sry;
            srz = rotated_right["z"] ?? srz;
        }

        let normal_len = sqrtf(snx * snx + sny * sny + snz * snz).max(0.0001);
        snx /= normal_len;
        sny /= normal_len;
        snz /= normal_len;
        let forward_dot = sfx * snx + sfy * sny + sfz * snz;
        sfx -= snx * forward_dot;
        sfy -= sny * forward_dot;
        sfz -= snz * forward_dot;
        let forward_len = sqrtf(sfx * sfx + sfy * sfy + sfz * sfz).max(0.0001);
        sfx /= forward_len;
        sfy /= forward_len;
        sfz /= forward_len;
        srx = sfy * snz - sfz * sny;
        sry = sfz * snx - sfx * snz;
        srz = sfx * sny - sfy * snx;

        ship_heading = math3::heading_from_forward(sfx, sfy, ship_heading);
        let ship_euler = math3::ship_basis_euler(
            srx, sry, srz,
            sfx, sfy, sfz,
            snx, sny, snz,
            ship_vis_pitch, ship_vis_yaw, ship_vis_roll
        );
        ship_vis_pitch = ship_euler["pitch"] ?? ship_vis_pitch;
        ship_vis_yaw = ship_euler["yaw"] ?? ship_vis_yaw;
        ship_vis_roll = ship_euler["roll"] ?? ship_vis_roll;

        let altitude_now_km = ((radius - surface_radius).max(0.0)) * km_per_wu;
        let tangent_now_kms = sqrtf(vfwd * vfwd + vright * vright) * km_per_wu;
        if (state.vehicle_phase ?? "flight") == "flight" && altitude_now_km >= orbit_altitude_km {
            state.vehicle_phase = "orbit";
        } else if (state.vehicle_phase ?? "flight") == "orbit" && altitude_now_km < orbit_altitude_km * 0.75 {
            state.vehicle_phase = "flight";
        } else if (state.vehicle_phase ?? "pad") == "hover"
            && altitude_now_km > hover_exit_altitude_km
            && tangent_now_kms > hover_exit_speed_kms {
            state.vehicle_phase = "flight";
        }

        scene_rt::set(runtime, "vehicle-slot", "obj.source", vehicle_model_source(vehicle_model));
        scene_rt::set(runtime, "vehicle-slot", "obj.scale", vehicle_model_scale(vehicle_model));
        let ship_visual_lift_wu = vehicle_model_visual_lift(surface_clearance_wu, vehicle_model);
        let ship_scene_r = (radius + ship_visual_lift_wu) / surface_radius.max(0.0001);
        scene_rt::visible(runtime, "vehicle-slot", true);
        scene_rt::set(runtime, "vehicle-slot", "obj.pitch", math3::rad_to_deg(ship_vis_pitch));
        scene_rt::set(runtime, "vehicle-slot", "obj.yaw", math3::rad_to_deg(ship_vis_yaw));
        scene_rt::set(runtime, "vehicle-slot", "obj.roll", math3::rad_to_deg(ship_vis_roll));
        scene_rt::set(runtime, "vehicle-slot", "obj.world.x", snx * ship_scene_r);
        scene_rt::set(runtime, "vehicle-slot", "obj.world.y", sny * ship_scene_r);
        scene_rt::set(runtime, "vehicle-slot", "obj.world.z", snz * ship_scene_r);
        scene_rt::set(runtime, `ship-${active_ship_id}`, "visible", false);
        world.entity(active_ship_id).set_heading(ship_heading);

        state.snx = snx;
        state.sny = sny;
        state.snz = snz;
        state.sfx = sfx;
        state.sfy = sfy;
        state.sfz = sfz;
        state.srx = srx;
        state.sry = sry;
        state.srz = srz;
        state.radius = radius;
        state.vrad = vrad;
        state.vfwd = vfwd;
        state.vright = vright;
        state.yaw_rate = yaw_rate;
        state.ship_heading = ship_heading;
        state.vehicle_heading = ship_heading;
        state.ship_vis_pitch = ship_vis_pitch;
        state.ship_vis_yaw = ship_vis_yaw;
        state.ship_vis_roll = ship_vis_roll;

        let next_ship_x = cx + snx * radius;
        let next_ship_y = cy + sny * radius;
        world.set_physics(active_ship_id, (next_ship_x - ship_x) / dt, (next_ship_y - ship_y) / dt, 0.0, 0.0, 0.0, 0.0);
        ship_x = next_ship_x;
        ship_y = next_ship_y;

        let ctrl_scroll_zoom = input.ctrl_scroll_y ?? 0.0;
        if absf(ctrl_scroll_zoom) > 0.001 {
            state.vehicle_camera_zoom =
                ((state.vehicle_camera_zoom ?? 0.0) + ctrl_scroll_zoom * 0.10)
                    .max(-0.70)
                    .min(0.55);
        }
        let altitude_px_frac = math3::clamp01((radius - surface_radius) / surface_radius.max(0.1));
        let zoom_scale = (1.0 - (state.vehicle_camera_zoom ?? 0.0)).max(0.45).min(1.70);
        let camera_phase = state.vehicle_phase ?? "pad";
        let profile = camera_phase_profile(camera_phase, altitude_px_frac, zoom_scale);
        let cam_lift = profile.cam_lift;
        let cam_back = profile.cam_back;
        let ship_center_x = snx * ship_scene_r;
        let ship_center_y = sny * ship_scene_r;
        let ship_center_z = snz * ship_scene_r;

        cam_nx += (snx - cam_nx) * (profile.normal_alpha ?? 0.10);
        cam_ny += (sny - cam_ny) * (profile.normal_alpha ?? 0.10);
        cam_nz += (snz - cam_nz) * (profile.normal_alpha ?? 0.10);
        let cam_n_len = sqrtf(cam_nx * cam_nx + cam_ny * cam_ny + cam_nz * cam_nz).max(0.0001);
        cam_nx /= cam_n_len;
        cam_ny /= cam_n_len;
        cam_nz /= cam_n_len;

        let flight_forward_alpha = ship_tuning.camera_sway_alpha(dt);
        let forward_alpha =
            if camera_phase == "flight" || camera_phase == "orbit" {
                flight_forward_alpha
            } else {
                profile.forward_alpha ?? 0.10
            };
        cam_fx += (sfx - cam_fx) * forward_alpha;
        cam_fy += (sfy - cam_fy) * forward_alpha;
        cam_fz += (sfz - cam_fz) * forward_alpha;
        let cam_forward_dot = cam_fx * cam_nx + cam_fy * cam_ny + cam_fz * cam_nz;
        cam_fx -= cam_nx * cam_forward_dot;
        cam_fy -= cam_ny * cam_forward_dot;
        cam_fz -= cam_nz * cam_forward_dot;
        let cam_f_len = sqrtf(cam_fx * cam_fx + cam_fy * cam_fy + cam_fz * cam_fz).max(0.0001);
        cam_fx /= cam_f_len;
        cam_fy /= cam_f_len;
        cam_fz /= cam_f_len;

        let eye_x = ship_center_x + cam_nx * cam_lift - cam_fx * cam_back;
        let eye_y = ship_center_y + cam_ny * cam_lift - cam_fy * cam_back;
        let eye_z = ship_center_z + cam_nz * cam_lift - cam_fz * cam_back;
        let look_x = ship_center_x;
        let look_y = ship_center_y;
        let look_z = ship_center_z;

        let sway_target = ship_tuning.camera_sway_target(yaw_rate) * (profile.sway_scale ?? 1.0);
        let alpha_s = ship_tuning.camera_sway_alpha(dt);
        cam_sway += (sway_target - cam_sway) * alpha_s;
        let sway_up = math3::rotate_axis(cam_nx, cam_ny, cam_nz, cam_fx, cam_fy, cam_fz, cam_sway);

        world.set_camera_3d_look_at(eye_x, eye_y, eye_z, look_x, look_y, look_z);
        world.set_camera_3d_up(sway_up["x"] ?? cam_nx, sway_up["y"] ?? cam_ny, sway_up["z"] ?? cam_nz);

        state.cam_nx = cam_nx;
        state.cam_ny = cam_ny;
        state.cam_nz = cam_nz;
        state.cam_fx = cam_fx;
        state.cam_fy = cam_fy;
        state.cam_fz = cam_fz;
        state.cam_sway = cam_sway;
        state.vehicle_camera_sway = cam_sway;
        state.vehicle_altitude_km = altitude_now_km;
        state.vehicle_speed_kms = tangent_now_kms;
        state.vehicle_vrad_kms = vrad * km_per_wu;
        state.vehicle_grounded = (state.vehicle_phase ?? "pad") == "pad";

        let orbit_xy_len = sqrtf(snx * snx + sny * sny).max(0.0001);
        let spawn_angle_rad = math3::angle_from_cos_sin(math3::clamp_unit(snx / orbit_xy_len), sny / orbit_xy_len);
        state.vehicle_spawn_angle_deg = math3::rad_to_deg(math3::nearest_angle(spawn_angle_rad, 0.0));

        if input.just_pressed("Escape") && !(handoff::resolve_return_scene_id(state) != "" && (state.handoff_source ?? "defaults") == "packet") {
            state = reset_vehicle_to_surface_state(state, vehicle, surface_radius, surface_clearance_wu);
            world.set_transform(
                active_ship_id,
                cx + (state.snx ?? 1.0) * (state.radius ?? surface_radius),
                cy + (state.sny ?? 0.0) * (state.radius ?? surface_radius),
                state.vehicle_heading ?? 0.0
            );
            world.set_physics(active_ship_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
    }

    if active_ship_id > 0 {
        let frame_control = control::current_vehicle_frame_control(state, vehicle, input);
        let ship_transform = world.transform(active_ship_id) ?? #{};
        let ship_physics = world.physics(active_ship_id) ?? #{};
        state = sync_vehicle_ship_runtime(
            state,
            vehicle,
            body_snapshot,
            frame_control,
            ship_transform["x"] ?? ship_x,
            ship_transform["y"] ?? ship_y,
            ship_physics["vx"] ?? 0.0,
            ship_physics["vy"] ?? 0.0,
            runtime_surface_contact,
            runtime_prefer_grounded,
            runtime_request_surface_lock,
            runtime_request_detach
        );
    }

    #{
        state: state,
        ship_id: active_ship_id,
        ship_x: ship_x,
        ship_y: ship_y
    }
}
