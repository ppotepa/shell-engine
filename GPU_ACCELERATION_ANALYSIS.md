# GPU ACCELERATION ANALYSIS

## CURRENT IMPLEMENTATION

### Rendering Pipeline (CPU-Based)

1. **Load OBJ Mesh** (`obj_loader.rs`)
   - Parse OBJ file format
   - Load vertices, faces, edges into memory
   - Calculate mesh bounding sphere/center
   - Store mesh in CPU memory

2. **Project Vertices** (`obj_render.rs:222-254`)
   - For each vertex:
     - Center on mesh origin
     - Apply model scale
     - Rotate (pitch, yaw, roll)
     - Apply camera pan
     - Apply camera distance
     - Project to NDC (normalized device coordinates)
     - Map to virtual screen space

3. **Rasterization** (CPU Rasterizer)
   - **Wireframe mode**:
     - For each edge, draw line using Bresenham's algorithm
     - Apply depth buffer for brightness variation

   - **Solid mode**:
     - Sort faces back-to-front (painter's algorithm)
     - For each face:
       - Clip to viewport
       - Draw filled triangle to canvas
       - Apply lighting calculations per pixel
       - Perform shading (cel shading with multiple levels)

4. **Color Conversion**
   - RGB [0-255] to terminal Color
   - Dithering/quantization to terminal colors

5. **Terminal Output**
   - Write canvas to terminal buffer
   - Batch writes for efficiency

## GPU ACCELERATION APPROACH

### Architecture Overview

```
OBJ Mesh (CPU)
    ↓ [Transfer]
GPU Buffer (Vertex/Index)
    ↓
Vertex Shader (Transform: rotate, project, pan)
    ↓
Fragment Shader (Lighting: diffuse, normal, cel shading)
    ↓
Render Target (RGBA texture)
    ↓ [Readback]
Pixel Buffer (CPU)
    ↓ [Convert]
Terminal Colors
    ↓
Terminal Output
```

### Key Changes Required

#### 1. Graphics API Integration
**Add GPU Graphics Library** (choose one):
- **WGPU** (modern, cross-platform, WebGPU compatible) ✅ **RECOMMENDED**
  - Best choice: WebGPU standard, future-proof
  - Dependencies: wgpu, pollster, bytemuck

- **glium** (OpenGL wrapper)
  - Legacy option, simpler API
  - Dependencies: glium, glutin

- **glow** (minimal OpenGL)
  - Direct WebGL/OpenGL, lower-level

#### 2. Mesh Management

**CPU Changes** (obj_loader.rs):
```rust
// Current: Store in Vec<[f32; 3]>
pub vertices: Vec<[f32; 3]>,
pub faces: Vec<ObjFace>,

// GPU: Add GPU buffers
pub vertices: Vec<[f32; 3]>,
pub gpu_vertex_buffer: wgpu::Buffer,
pub gpu_index_buffer: wgpu::Buffer,
pub gpu_mesh: GpuMesh,
```

**New Structures** (gpu_mesh.rs):
```rust
pub struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_count: u32,
    bounding_sphere: [f32; 4],
}

pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: wgpu::Texture,
    color_texture: wgpu::Texture,
}
```

#### 3. Shader Implementation

**Vertex Shader** (shaders/obj.vert - WGSL):
```glsl
struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let world_pos = uniforms.model * vec4<f32>(vertex.position, 1.0);
    let view_pos = uniforms.view * world_pos;
    let proj_pos = uniforms.projection * view_pos;

    return VertexOutput(
        proj_pos,
        vertex.position,  // For per-vertex lighting in fragment
    );
}
```

**Fragment Shader** (shaders/obj.frag - WGSL):
```glsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let light_dir = normalize(light_position - in.world_pos);

    // Cel shading with discrete levels
    let diffuse = dot(normal, light_dir);
    let diffuse_level = floor(diffuse * cel_levels) / cel_levels;

    let color = base_color * diffuse_level;
    return vec4<f32>(color, 1.0);
}
```

#### 4. Render Pass

**GPU Rendering** (gpu_render.rs):
```rust
pub fn render_obj_gpu(
    gpu_ctx: &GpuContext,
    mesh: &GpuMesh,
    params: &ObjRenderParams,
    target_w: u32,
    target_h: u32,
) -> Vec<[u8; 3]> {
    // Create render pass
    let mut encoder = gpu_ctx.device.create_command_encoder(...);

    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &gpu_ctx.color_texture_view,
                load_op: wgpu::LoadOp::Clear(Color::BLACK),
                store_op: wgpu::StoreOp::Store,
            })],
            depth_stencil_attachment: Some(...),
        });

        pass.set_pipeline(&gpu_ctx.render_pipeline);
        pass.set_bind_group(0, &uniforms_bind_group, &[]);
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    // Readback to CPU
    gpu_ctx.queue.submit(std::iter::once(encoder.finish()));
    let pixels = readback_texture(&gpu_ctx.color_texture);

    // Convert RGBA to RGB and quantize to terminal colors
    convert_to_terminal_colors(&pixels, target_w, target_h)
}
```

#### 5. Integration Points

**In compositor system**:
```rust
// Current
pub fn render_obj_to_canvas(...) -> Vec<Option<[u8; 3]>> { ... }

// GPU-accelerated
pub fn render_obj_gpu(...) -> Vec<[u8; 3]> { ... }

// Or with feature flag
#[cfg(feature = "gpu")]
pub fn render_obj_to_canvas(...) -> Vec<Option<[u8; 3]>> {
    let pixels = render_obj_gpu(...)?;
    // Convert back to Option<[u8; 3]>
}

#[cfg(not(feature = "gpu"))]
pub fn render_obj_to_canvas(...) -> Vec<Option<[u8; 3]>> {
    // Current CPU implementation
}
```

## IMPLEMENTATION ROADMAP

### Phase 1: Foundation (1-2 weeks)
- [ ] Add WGPU dependency
- [ ] Create GPU context management
- [ ] Implement basic mesh upload
- [ ] Write basic vertex/fragment shaders
- [ ] Set up render pipeline

### Phase 2: Feature Parity (2-3 weeks)
- [ ] Implement all lighting models (directional, point)
- [ ] Implement cel shading levels
- [ ] Wireframe rendering
- [ ] Depth buffering
- [ ] Backface culling

### Phase 3: Optimization (1 week)
- [ ] Mesh caching (don't re-upload every frame)
- [ ] Batch rendering (render multiple meshes in one pass)
- [ ] Mipmapping for better quality
- [ ] Profile and optimize shaders

### Phase 4: Integration (1 week)
- [ ] Feature flag for GPU/CPU switching
- [ ] Fallback to CPU if GPU unavailable
- [ ] Benchmarking against CPU version
- [ ] Documentation

## PERFORMANCE EXPECTATIONS

### Current CPU Performance
- Vertex projection: O(V) per frame
- Face sorting: O(F log F) per frame
- Rasterization: O(F × pixels_per_face) per frame
- Total: ~500-1000 μs per 30K face mesh at 120×40

### GPU Performance
- Mesh upload: O(V + F) once per load
- Vertex transformation: GPU parallel, ~10 μs
- Fragment shading: GPU parallel, ~50-100 μs
- Readback: ~50-100 μs (bottleneck)
- Total: ~100-200 μs per frame after warm-up

**Speedup**: 3-10x depending on mesh complexity

### Bottleneck: Readback
GPU → CPU texture readback is the limiting factor:
- Can't avoid it (need pixels for terminal)
- ~1-2 ms per 120×40 texture
- Optimizable with async readback (double-buffering)

## CHALLENGES

1. **Terminal Output Constraint**
   - GPU renders to offscreen texture
   - Must read pixels back to CPU
   - No direct GPU → terminal output
   - Limits practical speedup

2. **Terminal Color Quantization**
   - GPU renders full RGB
   - Terminal needs 256-color palette or true color
   - Color space conversion needed
   - Affects quality/performance tradeoff

3. **Headless Rendering**
   - Game engine may run without display
   - GPU rendering needs surface/context
   - Requires special headless mode
   - Offscreen rendering to texture only

4. **Cross-Platform Compatibility**
   - WGPU good but still emerging
   - Testing on multiple GPU vendors needed
   - Shader language compatibility (WGSL vs GLSL)

## ESTIMATED EFFORT

| Task | Time | Difficulty |
|------|------|------------|
| Setup WGPU context | 2-3 days | Medium |
| Mesh upload pipeline | 2-3 days | Medium |
| Basic shaders | 3-4 days | Medium |
| Lighting/shading | 4-5 days | Hard |
| Performance optimization | 3-4 days | Hard |
| Testing/fallback | 2-3 days | Medium |
| **Total** | **3-4 weeks** | **Hard** |

## FILES TO CREATE/MODIFY

### New Files
- `engine/src/gpu/context.rs` - GPU initialization and management
- `engine/src/gpu/mesh.rs` - GPU mesh structures
- `engine/src/gpu/render.rs` - GPU rendering functions
- `engine/src/shaders/obj.wgsl` - OBJ shader (WGSL format)
- `engine/src/systems/compositor/gpu_obj_render.rs` - GPU OBJ renderer

### Modified Files
- `engine/src/systems/compositor/obj_render.rs` - Add GPU path
- `engine/src/systems/compositor/obj_loader.rs` - Add GPU mesh upload
- `Cargo.toml` - Add WGPU dependency (with feature flag)
- `engine/src/lib.rs` - Add GPU module

## RECOMMENDATION

**Worth pursuing if**:
- Scenes with heavy 3D content (100K+ faces)
- Performance bottleneck confirmed via profiling
- Terminal environment supports GPU (not SSH/headless)

**Not worth pursuing if**:
- Scenes are simple (< 10K faces)
- Current CPU perf is acceptable (>30fps)
- Running in constrained environment (headless, SSH)

**Alternative**: Profile CPU implementation first. The ~1ms CPU cost might be acceptable given terminal rendering overhead.

---

**Status**: Analysis complete. Ready for implementation if needed.
