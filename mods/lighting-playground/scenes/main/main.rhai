const TRACK_W = 168;

if local.active_view == () { local.active_view = "orbit-realistic"; }
if local.active_light == () { local.active_light = "space-hard-vacuum"; }
if local.active_env == () { local.active_env = "deep-space-sparse"; }
if local.active_model == () { local.active_model = "planet"; }

fn clamp01(v) {
    let out = v;
    if out < 0.0 { out = 0.0; }
    if out > 1.0 { out = 1.0; }
    out
}

fn to_fixed_2(v) {
    let scaled = (v * 100.0).round() / 100.0;
    `${scaled}`
}

fn exposure_to_01(v) { clamp01((v - 0.6) / 0.8) }
fn exposure_from_01(v) { 0.6 + clamp01(v) * 0.8 }
fn black_to_01(v) { clamp01(v / 0.22) }
fn black_from_01(v) { clamp01(v) * 0.22 }
fn gamma_to_01(v) { clamp01((v - 1.8) / 0.8) }
fn gamma_from_01(v) { 1.8 + clamp01(v) * 0.8 }
fn contrast_to_01(v) { clamp01((v - 0.5) / 1.0) }
fn contrast_from_01(v) { 0.5 + clamp01(v) * 1.0 }
fn stars_to_01(v) { clamp01(v / 1.2) }
fn stars_from_01(v) { clamp01(v) * 1.2 }
fn glare_to_01(v) { clamp01(v / 0.35) }
fn glare_from_01(v) { clamp01(v) * 0.35 }
fn sunaz_to_01(v) { clamp01(v / 360.0) }
fn sunaz_from_01(v) { clamp01(v) * 360.0 }
fn sunel_to_01(v) { clamp01((v + 10.0) / 95.0) }
fn sunel_from_01(v) { -10.0 + clamp01(v) * 95.0 }
fn sunint_to_01(v) { clamp01(v / 2.0) }
fn sunint_from_01(v) { clamp01(v) * 2.0 }

fn camera_preset_name(idx) {
    if idx == 1 {
        "terminator"
    } else if idx == 2 {
        "night-side"
    } else if idx == 3 {
        "backlit"
    } else {
        "far-orbit"
    }
}

fn view_index(name) {
    if name == "orbit-cinematic" { 1 } else if name == "backlit-harsh" { 2 } else { 0 }
}

fn light_index(name) {
    if name == "space-cinematic-soft" { 1 } else if name == "showcase-high-contrast" { 2 } else { 0 }
}

fn env_index(name) {
    if name == "deep-space-rich" { 1 } else if name == "orbit-bright-star" { 2 } else { 0 }
}

fn scene_set(scene, target, path, value) {
    scene.object(target).set(path, value);
}

fn scene_text(scene, target, value) {
    scene_set(scene, target, "text.content", value);
}

fn scene_fg(scene, target, value) {
    scene_set(scene, target, "text.fg", value);
}

fn scene_visible(scene, target, value) {
    scene_set(scene, target, "visible", value);
}

fn scene_vector(scene, target, points) {
    scene_set(scene, target, "vector.points", points);
}

if local.bootstrapped == () {
    local.bootstrapped = true;
    local.active_model = "planet";
    local.active_view = "orbit-realistic";
    local.active_light = "space-hard-vacuum";
    local.active_env = "deep-space-sparse";
    scene.set_view_profile("orbit-realistic");
    scene.set_lighting_profile("space-hard-vacuum");
    scene.set_space_environment_profile("deep-space-sparse");
    gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.92));
    gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
    gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
    gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.0));
    gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.68));
    gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.08));
    gui.set_widget_value("lighting-sun-azimuth-slider", sunaz_to_01(210.0));
    gui.set_widget_value("lighting-sun-elevation-slider", sunel_to_01(40.0));
    gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.0));
    gui.set_widget_value("lighting-view-dropdown", 0.0);
    gui.set_widget_value("lighting-profile-dropdown", 0.0);
    gui.set_widget_value("lighting-environment-dropdown", 0.0);
    gui.set_widget_value("lighting-model-selector", 0.0);
    local.sun_azimuth = 210.0;
    local.sun_elevation = 40.0;
    local.sun_intensity = 1.0;
    local.camera_preset_idx = 0;
    local.camera_preset_dirty = true;
}

if local.camera_preset_idx == () { local.camera_preset_idx = 0; }
if local.camera_preset_dirty == () { local.camera_preset_dirty = true; }
if local.compare_show_baseline == () { local.compare_show_baseline = false; }
if local.compare_saved_ready == () { local.compare_saved_ready = false; }

if input.just_pressed("1") { gui.set_widget_value("lighting-model-selector", 0.0); }
if input.just_pressed("2") { gui.set_widget_value("lighting-model-selector", 1.0); }
if input.just_pressed("3") { gui.set_widget_value("lighting-model-selector", 2.0); }
if input.just_pressed("4") { gui.set_widget_value("lighting-model-selector", 3.0); }
let model_idx = gui.selected_index("lighting-model-selector");
if model_idx == 0 { local.active_model = "planet"; }
if model_idx == 1 { local.active_model = "sphere"; }
if model_idx == 2 { local.active_model = "cube"; }
if model_idx == 3 { local.active_model = "suzanne"; }

if input.just_pressed("V") {
    local.camera_preset_idx = (local.camera_preset_idx + 3) % 4;
    local.camera_preset_dirty = true;
}
if input.just_pressed("F") {
    local.camera_preset_idx = (local.camera_preset_idx + 1) % 4;
    local.camera_preset_dirty = true;
}
if input.just_pressed("G") { local.camera_preset_idx = 0; local.camera_preset_dirty = true; }
if input.just_pressed("T") { local.camera_preset_idx = 1; local.camera_preset_dirty = true; }
if input.just_pressed("N") { local.camera_preset_idx = 2; local.camera_preset_dirty = true; }
if input.just_pressed("K") { local.camera_preset_idx = 3; local.camera_preset_dirty = true; }

if input.just_pressed("Z") { gui.set_widget_value("lighting-view-dropdown", 0.0); }
if input.just_pressed("X") { gui.set_widget_value("lighting-view-dropdown", 1.0); }
if input.just_pressed("C") { gui.set_widget_value("lighting-view-dropdown", 2.0); }
let gui_view_idx = gui.selected_index("lighting-view-dropdown");
let selected_view = if gui_view_idx == 1 { "orbit-cinematic" } else if gui_view_idx == 2 { "backlit-harsh" } else { "orbit-realistic" };
if selected_view != local.active_view {
    if selected_view == "orbit-cinematic" {
        local.active_view = "orbit-cinematic";
        local.active_light = "space-cinematic-soft";
        local.active_env = "deep-space-rich";
        gui.set_widget_value("lighting-profile-dropdown", 1.0);
        gui.set_widget_value("lighting-environment-dropdown", 1.0);
        scene.set_view_profile("orbit-cinematic");
        scene.set_lighting_profile("space-cinematic-soft");
        scene.set_space_environment_profile("deep-space-rich");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(1.08));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.025));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(0.82));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.95));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.24));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.15));
    } else if selected_view == "backlit-harsh" {
        local.active_view = "backlit-harsh";
        local.active_light = "showcase-high-contrast";
        local.active_env = "orbit-bright-star";
        gui.set_widget_value("lighting-profile-dropdown", 2.0);
        gui.set_widget_value("lighting-environment-dropdown", 2.0);
        scene.set_view_profile("backlit-harsh");
        scene.set_lighting_profile("showcase-high-contrast");
        scene.set_space_environment_profile("orbit-bright-star");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.86));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.18));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.35));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.72));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.34));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.35));
    } else {
        local.active_view = "orbit-realistic";
        local.active_light = "space-hard-vacuum";
        local.active_env = "deep-space-sparse";
        gui.set_widget_value("lighting-profile-dropdown", 0.0);
        gui.set_widget_value("lighting-environment-dropdown", 0.0);
        scene.set_view_profile("orbit-realistic");
        scene.set_lighting_profile("space-hard-vacuum");
        scene.set_space_environment_profile("deep-space-sparse");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.92));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.0));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.68));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.08));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.0));
    }
}

if input.just_pressed("Q") { gui.set_widget_value("lighting-profile-dropdown", 0.0); }
if input.just_pressed("W") { gui.set_widget_value("lighting-profile-dropdown", 1.0); }
if input.just_pressed("E") { gui.set_widget_value("lighting-profile-dropdown", 2.0); }
let gui_light_idx = gui.selected_index("lighting-profile-dropdown");
let selected_light = if gui_light_idx == 1 { "space-cinematic-soft" } else if gui_light_idx == 2 { "showcase-high-contrast" } else { "space-hard-vacuum" };
if selected_light != local.active_light {
    let current_stars = stars_from_01(gui.slider_value("lighting-starfield-slider"));
    let current_glare = glare_from_01(gui.slider_value("lighting-star-glare-slider"));
    if selected_light == "space-cinematic-soft" {
        local.active_light = "space-cinematic-soft";
        scene.set_lighting_profile("space-cinematic-soft");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(1.05));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.03));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(0.82));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(current_stars));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(current_glare));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.15));
    } else if selected_light == "showcase-high-contrast" {
        local.active_light = "showcase-high-contrast";
        scene.set_lighting_profile("showcase-high-contrast");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.88));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.18));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.35));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(current_stars));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(current_glare));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.35));
    } else {
        local.active_light = "space-hard-vacuum";
        scene.set_lighting_profile("space-hard-vacuum");
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.94));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.0));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(current_stars));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(current_glare));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.0));
    }
}

if input.just_pressed("A") { gui.set_widget_value("lighting-environment-dropdown", 0.0); }
if input.just_pressed("S") { gui.set_widget_value("lighting-environment-dropdown", 1.0); }
if input.just_pressed("D") { gui.set_widget_value("lighting-environment-dropdown", 2.0); }
let gui_env_idx = gui.selected_index("lighting-environment-dropdown");
let selected_env = if gui_env_idx == 1 { "deep-space-rich" } else if gui_env_idx == 2 { "orbit-bright-star" } else { "deep-space-sparse" };
if selected_env != local.active_env {
    if selected_env == "deep-space-rich" {
        local.active_env = "deep-space-rich";
        scene.set_space_environment_profile("deep-space-rich");
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.92));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.20));
    } else if selected_env == "orbit-bright-star" {
        local.active_env = "orbit-bright-star";
        scene.set_space_environment_profile("orbit-bright-star");
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.78));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.32));
    } else {
        local.active_env = "deep-space-sparse";
        scene.set_space_environment_profile("deep-space-sparse");
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.72));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.10));
    }
}

if input.just_pressed("R") {
    local.active_view = "orbit-realistic";
    local.active_light = "space-hard-vacuum";
    local.active_env = "deep-space-sparse";
    local.camera_preset_idx = 0;
    local.camera_preset_dirty = true;
    scene.set_view_profile("orbit-realistic");
    scene.set_lighting_profile("space-hard-vacuum");
    scene.set_space_environment_profile("deep-space-sparse");
    gui.set_widget_value("lighting-view-dropdown", 0.0);
    gui.set_widget_value("lighting-profile-dropdown", 0.0);
    gui.set_widget_value("lighting-environment-dropdown", 0.0);
    gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.92));
    gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
    gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
    gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.0));
    gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.68));
    gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.08));
    gui.set_widget_value("lighting-sun-azimuth-slider", sunaz_to_01(210.0));
    gui.set_widget_value("lighting-sun-elevation-slider", sunel_to_01(40.0));
    gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.0));
}

if input.just_pressed("B") {
    if local.compare_show_baseline {
        if local.compare_saved_ready {
            local.active_view = local.saved_active_view;
            local.active_light = local.saved_active_light;
            local.active_env = local.saved_active_env;
            scene.set_view_profile(local.active_view);
            scene.set_lighting_profile(local.active_light);
            scene.set_space_environment_profile(local.active_env);
            gui.set_widget_value("lighting-view-dropdown", view_index(local.active_view).to_float());
            gui.set_widget_value("lighting-profile-dropdown", light_index(local.active_light).to_float());
            gui.set_widget_value("lighting-environment-dropdown", env_index(local.active_env).to_float());
            gui.set_widget_value("lighting-exposure-slider", local.saved_exposure);
            gui.set_widget_value("lighting-black-level-slider", local.saved_black);
            gui.set_widget_value("lighting-gamma-slider", local.saved_gamma);
            gui.set_widget_value("lighting-contrast-slider", local.saved_contrast);
            gui.set_widget_value("lighting-starfield-slider", local.saved_stars);
            gui.set_widget_value("lighting-star-glare-slider", local.saved_glare);
            gui.set_widget_value("lighting-sun-azimuth-slider", local.saved_sun_az);
            gui.set_widget_value("lighting-sun-elevation-slider", local.saved_sun_el);
            gui.set_widget_value("lighting-sun-intensity-slider", local.saved_sun_int);
            local.camera_preset_idx = local.saved_camera_preset_idx;
            local.camera_preset_dirty = true;
        }
        local.compare_show_baseline = false;
    } else {
        local.compare_saved_ready = true;
        local.saved_active_view = local.active_view;
        local.saved_active_light = local.active_light;
        local.saved_active_env = local.active_env;
        local.saved_exposure = gui.slider_value("lighting-exposure-slider");
        local.saved_black = gui.slider_value("lighting-black-level-slider");
        local.saved_gamma = gui.slider_value("lighting-gamma-slider");
        local.saved_contrast = gui.slider_value("lighting-contrast-slider");
        local.saved_stars = gui.slider_value("lighting-starfield-slider");
        local.saved_glare = gui.slider_value("lighting-star-glare-slider");
        local.saved_sun_az = gui.slider_value("lighting-sun-azimuth-slider");
        local.saved_sun_el = gui.slider_value("lighting-sun-elevation-slider");
        local.saved_sun_int = gui.slider_value("lighting-sun-intensity-slider");
        local.saved_camera_preset_idx = local.camera_preset_idx;

        local.active_view = "orbit-realistic";
        local.active_light = "space-hard-vacuum";
        local.active_env = "deep-space-sparse";
        scene.set_view_profile("orbit-realistic");
        scene.set_lighting_profile("space-hard-vacuum");
        scene.set_space_environment_profile("deep-space-sparse");
        gui.set_widget_value("lighting-view-dropdown", 0.0);
        gui.set_widget_value("lighting-profile-dropdown", 0.0);
        gui.set_widget_value("lighting-environment-dropdown", 0.0);
        gui.set_widget_value("lighting-exposure-slider", exposure_to_01(0.92));
        gui.set_widget_value("lighting-black-level-slider", black_to_01(0.0));
        gui.set_widget_value("lighting-gamma-slider", gamma_to_01(2.2));
        gui.set_widget_value("lighting-contrast-slider", contrast_to_01(1.0));
        gui.set_widget_value("lighting-starfield-slider", stars_to_01(0.68));
        gui.set_widget_value("lighting-star-glare-slider", glare_to_01(0.08));
        gui.set_widget_value("lighting-sun-azimuth-slider", sunaz_to_01(210.0));
        gui.set_widget_value("lighting-sun-elevation-slider", sunel_to_01(40.0));
        gui.set_widget_value("lighting-sun-intensity-slider", sunint_to_01(1.0));
        local.camera_preset_idx = 0;
        local.camera_preset_dirty = true;
        local.compare_show_baseline = true;
    }
}

let exposure = exposure_from_01(gui.slider_value("lighting-exposure-slider"));
let black = black_from_01(gui.slider_value("lighting-black-level-slider"));
let gamma = gamma_from_01(gui.slider_value("lighting-gamma-slider"));
let contrast = contrast_from_01(gui.slider_value("lighting-contrast-slider"));
let stars = stars_from_01(gui.slider_value("lighting-starfield-slider"));
let glare = glare_from_01(gui.slider_value("lighting-star-glare-slider"));
let sun_azimuth = sunaz_from_01(gui.slider_value("lighting-sun-azimuth-slider"));
let sun_elevation = sunel_from_01(gui.slider_value("lighting-sun-elevation-slider"));
let sun_intensity = sunint_from_01(gui.slider_value("lighting-sun-intensity-slider"));
local.sun_azimuth = sun_azimuth;
local.sun_elevation = sun_elevation;
local.sun_intensity = sun_intensity;

scene.set_lighting_param("exposure", exposure);
scene.set_lighting_param("black_level", black);
scene.set_lighting_param("gamma", gamma);
scene.set_lighting_param("shadow_contrast", contrast);
scene.set_space_environment_param("starfield_brightness", stars);
scene.set_space_environment_param("primary_star_glare_strength", glare);

// Keep object ambient tied to black level so slider can reach near-true black
// on the night side when black = 0.0 and still allow artistic shadow lift.
let ambient_world = (black * 1.0).min(0.22);
let ambient_ref = (black * 0.8).min(0.18);
scene_set(scene, "world-atmo", "obj.ambient", ambient_world);
scene_set(scene, "sphere-ref", "obj.ambient", ambient_ref);
scene_set(scene, "cube-ref", "obj.ambient", ambient_ref);
scene_set(scene, "suzanne-ref", "obj.ambient", ambient_ref);

let az_rad = sun_azimuth * 0.01745329;
let el_rad = sun_elevation * 0.01745329;
let sun_ux = -(az_rad.cos() * el_rad.cos());
let sun_uy = el_rad.sin();
let sun_uz = -(az_rad.sin() * el_rad.cos());
let sun_x = sun_ux * sun_intensity;
let sun_y = sun_uy * sun_intensity;
let sun_z = sun_uz * sun_intensity;

if local.camera_preset_dirty {
    if local.camera_preset_idx == 1 {
        let tx = -sun_uz;
        let ty = 0.18;
        let tz = sun_ux;
        scene.set_camera3d_look_at(tx * 3.7, ty * 3.7, tz * 3.7, 0.0, 0.0, 0.0);
        scene.set_camera3d_up(0.0, 1.0, 0.0);
    } else if local.camera_preset_idx == 2 {
        scene.set_camera3d_look_at(-sun_ux * 3.5, (-sun_uy + 0.08) * 3.5, -sun_uz * 3.5, 0.0, 0.0, 0.0);
        scene.set_camera3d_up(0.0, 1.0, 0.0);
    } else if local.camera_preset_idx == 3 {
        let bx = (-sun_ux * 0.82) + (-sun_uz * 0.42);
        let by = (-sun_uy * 0.82) + 0.12;
        let bz = (-sun_uz * 0.82) + (sun_ux * 0.42);
        scene.set_camera3d_look_at(bx * 3.8, by * 3.8, bz * 3.8, 0.0, 0.0, 0.0);
        scene.set_camera3d_up(0.0, 1.0, 0.0);
    } else {
        scene.set_camera3d_look_at(2.45, 1.2, 3.75, 0.0, 0.0, 0.0);
        scene.set_camera3d_up(0.0, 1.0, 0.0);
    }
    local.camera_preset_dirty = false;
}
scene_set(scene, "world-atmo", "obj.light.x", sun_x);
scene_set(scene, "world-atmo", "obj.light.y", sun_y);
scene_set(scene, "world-atmo", "obj.light.z", sun_z);
scene_set(scene, "sphere-ref", "obj.light.x", sun_x);
scene_set(scene, "sphere-ref", "obj.light.y", sun_y);
scene_set(scene, "sphere-ref", "obj.light.z", sun_z);
scene_set(scene, "cube-ref", "obj.light.x", sun_x);
scene_set(scene, "cube-ref", "obj.light.y", sun_y);
scene_set(scene, "cube-ref", "obj.light.z", sun_z);
scene_set(scene, "suzanne-ref", "obj.light.x", sun_x);
scene_set(scene, "suzanne-ref", "obj.light.y", sun_y);
scene_set(scene, "suzanne-ref", "obj.light.z", sun_z);

scene_visible(scene, "world-atmo", local.active_model == "planet");
scene_visible(scene, "sphere-ref", local.active_model == "sphere");
scene_visible(scene, "cube-ref", local.active_model == "cube");
scene_visible(scene, "suzanne-ref", local.active_model == "suzanne");

scene_fg(scene, "model-label-planet", if local.active_model == "planet" { "#dbe4f6" } else { "#71819a" });
scene_fg(scene, "model-label-sphere", if local.active_model == "sphere" { "#dbe4f6" } else { "#71819a" });
scene_fg(scene, "model-label-cube", if local.active_model == "cube" { "#dbe4f6" } else { "#71819a" });
scene_fg(scene, "model-label-suzanne", if local.active_model == "suzanne" { "#dbe4f6" } else { "#71819a" });

let view_open = gui.widget_open("lighting-view-dropdown");
let light_open = gui.widget_open("lighting-profile-dropdown");
let env_open = gui.widget_open("lighting-environment-dropdown");
scene_text(scene, "lighting-view-dropdown-arrow", if view_open { "^" } else { "v" });
scene_fg(scene, "lighting-view-dropdown-arrow", if view_open { "#cfe6ff" } else { "#8aa2c4" });
scene_text(scene, "lighting-profile-dropdown-arrow", if light_open { "^" } else { "v" });
scene_fg(scene, "lighting-profile-dropdown-arrow", if light_open { "#cfe6ff" } else { "#8aa2c4" });
scene_text(scene, "lighting-environment-dropdown-arrow", if env_open { "^" } else { "v" });
scene_fg(scene, "lighting-environment-dropdown-arrow", if env_open { "#cfe6ff" } else { "#8aa2c4" });

let fill_exposure = (clamp01(gui.slider_value("lighting-exposure-slider")) * TRACK_W.to_float()).round().to_int();
let fill_black = (clamp01(gui.slider_value("lighting-black-level-slider")) * TRACK_W.to_float()).round().to_int();
let fill_gamma = (clamp01(gui.slider_value("lighting-gamma-slider")) * TRACK_W.to_float()).round().to_int();
let fill_contrast = (clamp01(gui.slider_value("lighting-contrast-slider")) * TRACK_W.to_float()).round().to_int();
let fill_stars = (clamp01(gui.slider_value("lighting-starfield-slider")) * TRACK_W.to_float()).round().to_int();
let fill_glare = (clamp01(gui.slider_value("lighting-star-glare-slider")) * TRACK_W.to_float()).round().to_int();
let fill_sun_az = (clamp01(gui.slider_value("lighting-sun-azimuth-slider")) * TRACK_W.to_float()).round().to_int();
let fill_sun_el = (clamp01(gui.slider_value("lighting-sun-elevation-slider")) * TRACK_W.to_float()).round().to_int();
let fill_sun_int = (clamp01(gui.slider_value("lighting-sun-intensity-slider")) * TRACK_W.to_float()).round().to_int();

scene_vector(scene, "lighting-exposure-slider-fill", [[0, 0], [if fill_exposure < 1 { 1 } else { fill_exposure }, 0]]);
scene_vector(scene, "lighting-black-level-slider-fill", [[0, 0], [if fill_black < 1 { 1 } else { fill_black }, 0]]);
scene_vector(scene, "lighting-gamma-slider-fill", [[0, 0], [if fill_gamma < 1 { 1 } else { fill_gamma }, 0]]);
scene_vector(scene, "lighting-contrast-slider-fill", [[0, 0], [if fill_contrast < 1 { 1 } else { fill_contrast }, 0]]);
scene_vector(scene, "lighting-starfield-slider-fill", [[0, 0], [if fill_stars < 1 { 1 } else { fill_stars }, 0]]);
scene_vector(scene, "lighting-star-glare-slider-fill", [[0, 0], [if fill_glare < 1 { 1 } else { fill_glare }, 0]]);
scene_vector(scene, "lighting-sun-azimuth-slider-fill", [[0, 0], [if fill_sun_az < 1 { 1 } else { fill_sun_az }, 0]]);
scene_vector(scene, "lighting-sun-elevation-slider-fill", [[0, 0], [if fill_sun_el < 1 { 1 } else { fill_sun_el }, 0]]);
scene_vector(scene, "lighting-sun-intensity-slider-fill", [[0, 0], [if fill_sun_int < 1 { 1 } else { fill_sun_int }, 0]]);

scene_text(scene, "lighting-exposure-slider-value", to_fixed_2(exposure));
scene_text(scene, "lighting-black-level-slider-value", to_fixed_2(black));
scene_text(scene, "lighting-gamma-slider-value", to_fixed_2(gamma));
scene_text(scene, "lighting-contrast-slider-value", to_fixed_2(contrast));
scene_text(scene, "lighting-starfield-slider-value", to_fixed_2(stars));
scene_text(scene, "lighting-star-glare-slider-value", to_fixed_2(glare));
scene_text(scene, "lighting-sun-azimuth-slider-value", `${sun_azimuth.round()}`);
scene_text(scene, "lighting-sun-elevation-slider-value", `${sun_elevation.round()}`);
scene_text(scene, "lighting-sun-intensity-slider-value", to_fixed_2(sun_intensity));

scene_text(scene, "subtitle", `model=${local.active_model} | view=${local.active_view} | light=${local.active_light} | env=${local.active_env} | cam=${camera_preset_name(local.camera_preset_idx)} | sun=${sun_azimuth.round()}/${sun_elevation.round()} i=${to_fixed_2(sun_intensity)}`);
scene_text(scene, "hint-1", "keys: 1/2/3/4 model | Z/X/C view | Q/W/E light | A/S/D env");
scene_text(scene, "hint-2", `camera: F/V cycle | G/T/N/K direct | B compare=${if local.compare_show_baseline { "baseline" } else { "current" }} | R reset`);
