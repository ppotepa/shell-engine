import "std/runtime_scene" as scene_ui;

fn render(runtime, local) {
    scene_ui::text(runtime, "cockpitview-title", local.cockpit_title ?? "COCKPIT SIMULATOR");
    scene_ui::text(runtime, "cockpitview-status", "PROCEDURAL COCKPIT MESH");
    scene_ui::text(
        runtime,
        "cockpitview-controls-1",
        "Mouse head-look  subtle camera sway  generated planet outside"
    );
    scene_ui::text(
        runtime,
        "cockpitview-controls-2",
        "[F10] FLIGHT  [ESC] BACK"
    );
}
