import "std/runtime_scene" as scene_ui;

fn format_altitude(meters) {
    if meters < 1000.0 {
        return `${meters.round()}m`;
    }
    let km = ((meters / 1000.0) * 10.0).round() / 10.0;
    `${km}km`
}

fn format_speed(mps) {
    if mps < 1000.0 {
        return `${mps.round()}m/s`;
    }
    let kms = ((mps / 1000.0) * 100.0).round() / 100.0;
    `${kms}km/s`
}

fn render(runtime, local) {
    let free = (local.camera_mode ?? "cockpit") == "free";
    let altitude_m = (local.ship_display_altitude_m ?? 0.0).max(0.0);
    let speed_mps = (local.ship_display_speed_mps ?? 0.0).max(0.0);

    scene_ui::text(
        runtime,
        "flight-mode",
        if free { "VIEW: FREE FLIGHT [F]" } else { "VIEW: COCKPIT [F]" }
    );
    scene_ui::fg(runtime, "flight-mode", if free { "#ffd36b" } else { "#8ef0b3" });
    scene_ui::text(
        runtime,
        "flight-status",
        `ALT ${format_altitude(altitude_m)}  SPD ${format_speed(speed_mps)}`
    );
    scene_ui::text(runtime, "flight-controls-1", "Mouse steer  W/S thrust  A/D strafe  Q/E vertical");
    scene_ui::text(runtime, "flight-controls-2", "F cockpit toggle  inertial flight with gravity + drag");
}
