pub fn look_at_basis(
    eye: [f32; 3],
    target: [f32; 3],
    world_up: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let fwd = {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        [d[0] / len, d[1] / len, d[2] / len]
    };
    // right = normalize(cross(fwd, world_up))
    let right = {
        let d = [
            fwd[1] * world_up[2] - fwd[2] * world_up[1],
            fwd[2] * world_up[0] - fwd[0] * world_up[2],
            fwd[0] * world_up[1] - fwd[1] * world_up[0],
        ];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        [d[0] / len, d[1] / len, d[2] / len]
    };
    // up = cross(right, fwd)
    let up = [
        right[1] * fwd[2] - right[2] * fwd[1],
        right[2] * fwd[0] - right[0] * fwd[2],
        right[0] * fwd[1] - right[1] * fwd[0],
    ];
    (right, up, fwd)
}
