fn deg_to_rad(deg) { deg * 0.01745329252 }
fn rad_to_deg(rad) { rad * 57.29577951 }

fn clamp_unit(v) {
    if v < -1.0 { return -1.0; }
    if v > 1.0 { return 1.0; }
    v
}

fn clamp01(v) {
    if v < 0.0 { return 0.0; }
    if v > 1.0 { return 1.0; }
    v
}

fn normalize_or_zero(x, y, z) {
    let len = sqrtf(x * x + y * y + z * z);
    if len <= 0.000001 {
        return #{ x: 0.0, y: 0.0, z: 0.0 };
    }
    #{ x: x / len, y: y / len, z: z / len }
}

fn cross(ax, ay, az, bx, by, bz) {
    #{
        x: ay * bz - az * by,
        y: az * bx - ax * bz,
        z: ax * by - ay * bx
    }
}

fn nearest_angle(angle, reference) {
    let best = angle;
    let best_delta = absf(best - reference);
    let plus = angle + 6.2831853070;
    if absf(plus - reference) < best_delta {
        best = plus;
        best_delta = absf(plus - reference);
    }
    let minus = angle - 6.2831853070;
    if absf(minus - reference) < best_delta {
        best = minus;
    }
    while best > 3.1415926535 { best -= 6.2831853070; }
    while best < -3.1415926535 { best += 6.2831853070; }
    best
}

fn rotate_axis(vx, vy, vz, ax, ay, az, angle) {
    let c = cos(angle);
    let s = sin(angle);
    let d = vx * ax + vy * ay + vz * az;
    let cx = ay * vz - az * vy;
    let cy = az * vx - ax * vz;
    let cz = ax * vy - ay * vx;
    #{
        x: vx * c + cx * s + ax * d * (1.0 - c),
        y: vy * c + cy * s + ay * d * (1.0 - c),
        z: vz * c + cz * s + az * d * (1.0 - c)
    }
}

fn angle_from_cos_sin(c, s) {
    let a = acos(clamp_unit(c));
    if s < 0.0 { return -a; }
    a
}

fn heading_from_forward(fx, fy, fallback) {
    let fl = sqrtf(fx * fx + fy * fy);
    if fl < 0.0001 { return fallback; }
    let nfx = fx / fl;
    let nfy = fy / fl;
    let h = acos(clamp_unit(-nfy));
    if nfx < 0.0 { return -h; }
    h
}

fn ship_basis_euler(srx, sry, srz, sfx, sfy, sfz, snx, sny, snz, prev_pitch, prev_yaw, prev_roll) {
    let cy = sqrtf(srx * srx + sry * sry);
    let yaw = angle_from_cos_sin(cy, -srz);
    let pitch = prev_pitch;
    let roll = prev_roll;
    if cy > 0.0005 {
        pitch = angle_from_cos_sin(snz / cy, sfz / cy);
        roll = angle_from_cos_sin(srx / cy, sry / cy);
    } else {
        let combo = if srz >= 0.0 {
            angle_from_cos_sin(-snx, -sny)
        } else {
            angle_from_cos_sin(snx, -sny)
        };
        pitch = if srz >= 0.0 { combo - roll } else { combo + roll };
    }
    #{
        pitch: nearest_angle(pitch, prev_pitch),
        yaw: nearest_angle(yaw, prev_yaw),
        roll: nearest_angle(roll, prev_roll),
    }
}
