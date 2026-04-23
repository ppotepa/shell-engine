local = local ?? #{};

const MOVE_SPEED = 85.0;
const CAMERA_HEIGHT = 8.0;
const CAMERA_BACK = 14.0;
const CAMERA_LOOK_AHEAD = 4.5;

fn clamp_len2(x, y) {
    let len = sqrtf(x * x + y * y);
    if len <= 0.0001 {
        return #{ x: 0.0, y: 0.0 };
    }
    #{ x: x / len, y: y / len }
}

fn one_decimal(v) {
    let av = absf(v);
    let scaled = (av * 10.0).to_int();
    let whole = scaled / 10;
    let frac = scaled - whole * 10;
    let sign = if v < 0.0 { "-" } else { "" };
    sign + whole + "." + frac
}

if !(local.initialized ?? false) {
    input.load_profile("default");
    local.initialized = true;
    local.player_id = world.spawn_prefab("player_capsule", #{ x: 0.0, y: 0.0, vx: 0.0, vy: 0.0 });
    local.x = 0.0;
    local.y = 0.0;
    local.look_x = 0.0;
    local.look_y = 1.0;

    if local.player_id > 0 {
        let e = world.entity(local.player_id);
        let capsule_poly = [
            [-4.0, -12.0],
            [4.0, -12.0],
            [7.0, -8.0],
            [7.0, 8.0],
            [4.0, 12.0],
            [-4.0, 12.0],
            [-7.0, 8.0],
            [-7.0, -8.0],
        ];
        e.physics.set_collider_polygon(capsule_poly, 1, 65535);
    }
}

let dt = frame_ms.to_float() / 1000.0;
let move_x = 0.0;
let move_y = 0.0;

if input.action_down("move_forward") { move_y += 1.0; }
if input.action_down("move_back") { move_y -= 1.0; }
if input.action_down("move_right") { move_x += 1.0; }
if input.action_down("move_left") { move_x -= 1.0; }

let n = clamp_len2(move_x, move_y);
if n.x != 0.0 || n.y != 0.0 {
    local.look_x = n.x;
    local.look_y = n.y;
}

let px = (local.x ?? 0.0) + n.x * MOVE_SPEED * dt;
let py = (local.y ?? 0.0) + n.y * MOVE_SPEED * dt;
local.x = px;
local.y = py;

let player_id = local.player_id ?? 0;
if player_id > 0 {
    let e = world.entity(player_id);
    if e.exists() {
        e.set_position(px, py);
        if n.x != 0.0 || n.y != 0.0 {
            e.set_heading(atan2(n.y, n.x));
        }
    }
}

let lx = local.look_x ?? 0.0;
let ly = local.look_y ?? 1.0;

let eye_x = px - lx * CAMERA_BACK;
let eye_y = CAMERA_HEIGHT;
let eye_z = py - ly * CAMERA_BACK;
let target_x = px + lx * CAMERA_LOOK_AHEAD;
let target_y = 1.5;
let target_z = py + ly * CAMERA_LOOK_AHEAD;

world.set_camera_3d_look_at(eye_x, eye_y, eye_z, target_x, target_y, target_z);
world.set_camera_3d_up(0.0, 1.0, 0.0);

scene.object("player-pos").set("text.content", "POS " + one_decimal(px) + "," + one_decimal(py));
