//! GPU mesh management for OBJ rendering.
//! Handles vertex/index buffer creation and mesh metadata.

use std::sync::Arc;
use wgpu::util::DeviceExt;
use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// GPU-resident mesh data.
pub struct GpuMesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub bounding_sphere: [f32; 4],
}

impl GpuMesh {
    /// Upload mesh vertices and indices to GPU.
    pub fn new(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let vertex_buffer = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex-buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        let index_buffer = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index-buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        }));

        // Calculate bounding sphere (simplified: center + max distance)
        let center = if vertices.is_empty() {
            Vec3::ZERO
        } else {
            let sum = vertices.iter().fold(Vec3::ZERO, |acc, v| {
                acc + Vec3::from_slice(&v.position)
            });
            sum / vertices.len() as f32
        };

        let radius = vertices
            .iter()
            .map(|v| {
                let pos = Vec3::from_slice(&v.position);
                (pos - center).length()
            })
            .fold(0.0f32, f32::max);

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            bounding_sphere: [center.x, center.y, center.z, radius],
        }
    }
}

/// Render parameters for OBJ rendering.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderParams {
    pub model: [f32; 16],
    pub view: [f32; 16],
    pub projection: [f32; 16],
    pub light_pos: [f32; 4],
    pub camera_pos: [f32; 4],
}

impl RenderParams {
    pub fn zero() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array(),
            view: glam::Mat4::IDENTITY.to_cols_array(),
            projection: glam::Mat4::IDENTITY.to_cols_array(),
            light_pos: [0.0; 4],
            camera_pos: [0.0; 4],
        }
    }
}
