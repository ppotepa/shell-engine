// OBJ mesh rendering shader (WGSL format)
// Handles vertex transformation, normal calculation, and cel shading

struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    light_pos: vec4<f32>,
    camera_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) view_dir: vec3<f32>,
};

// Vertex shader: transform vertices to clip space
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = (uniforms.model * vec4<f32>(in.position, 1.0)).xyz;
    let world_normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;

    let view_pos = uniforms.view * vec4<f32>(world_pos, 1.0);
    let proj_pos = uniforms.projection * view_pos;

    let view_dir = normalize(uniforms.camera_pos.xyz - world_pos);

    return VertexOutput(
        proj_pos,
        world_pos,
        normalize(world_normal),
        view_dir,
    );
}

// Fragment shader: apply cel shading with discrete lighting levels
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(uniforms.light_pos.xyz - in.world_pos);
    let normal = normalize(in.normal);

    // Diffuse lighting
    let diffuse = max(dot(normal, light_dir), 0.0);

    // Cel shading: discretize diffuse into levels
    let cel_levels = 4.0;
    let diffuse_level = floor(diffuse * cel_levels) / cel_levels;

    // Base color (white, apply per-pixel in compositor)
    let base_color = vec3<f32>(1.0, 1.0, 1.0);
    let lit_color = base_color * (0.3 + 0.7 * diffuse_level);

    return vec4<f32>(lit_color, 1.0);
}
