fn set(runtime, target, path, value) {
    runtime.scene.objects.find(target).set(path, value);
}

fn text(runtime, target, value) {
    let object = runtime.scene.objects.find(target);
    object.text.content = value;
}

fn fg(runtime, target, value) {
    let object = runtime.scene.objects.find(target);
    object.style.fg = value;
}

fn visible(runtime, target, value) {
    let object = runtime.scene.objects.find(target);
    object.frame.visible = value;
}

fn render_surface_mode(runtime, target, mode) {
    let object = runtime.scene.objects.find(target);
    object.render.surface_mode = mode;
}

fn render_generator(runtime, target, name, value) {
    let object = runtime.scene.objects.find(target);
    object.render.generator.set(name, value);
}

fn render_atmosphere(runtime, target, name, value) {
    let object = runtime.scene.objects.find(target);
    object.render.atmosphere.set(name, value);
}

fn render_view(runtime, target, name, value) {
    let object = runtime.scene.objects.find(target);
    object.render.view.set(name, value);
}

fn render_visual(runtime, target, rotation_speed, ambient) {
    set(runtime, target, "obj.rotation-speed", rotation_speed);
    set(runtime, target, "obj.ambient", ambient);
}

fn render_light(runtime, target, x, y, z) {
    set(runtime, target, "obj.light.x", x);
    set(runtime, target, "obj.light.y", y);
    set(runtime, target, "obj.light.z", z);
}
