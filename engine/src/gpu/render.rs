//! GPU rendering for OBJ meshes.
//! Executes render passes and reads back pixels to CPU.

use super::{GpuContext, GpuMesh, RenderParams};
use wgpu::util::DeviceExt;

/// Render OBJ mesh to GPU texture.
/// Returns RGBA pixels (width * height * 4 bytes).
/// Note: Blocking readback is the bottleneck; async double-buffering in Phase 3.
pub fn render_obj_gpu(
    gpu_ctx: &GpuContext,
    mesh: &GpuMesh,
    _params: &RenderParams,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    let pipeline = gpu_ctx.render_pipeline()?;

    // Create render target and depth texture
    let color_texture = gpu_ctx.create_render_target(width, height);
    let depth_texture = gpu_ctx.create_depth_texture(width, height);

    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create staging buffer for readback
    let bytes_per_pixel = 4;
    let buffer_width = (width * bytes_per_pixel).next_multiple_of(256);
    let buffer_size = (buffer_width as u64) * (height as u64);
    let staging_buffer = gpu_ctx.create_staging_buffer(buffer_size);

    // Record render commands
    let mut encoder = gpu_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render-encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("obj-render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    // Copy render target to staging buffer
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &color_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(buffer_width as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    gpu_ctx.queue.submit(std::iter::once(encoder.finish()));

    // Poll device and read back
    gpu_ctx.device.poll(wgpu::Maintain::Wait);

    let slice = staging_buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu_ctx.device.poll(wgpu::Maintain::Wait);

    let data = slice.get_mapped_range();
    let pixels = data.to_vec();
    drop(data);
    staging_buffer.unmap();

    Some(pixels)
}

/// Convert raw RGBA pixels to optional RGB samples.
pub fn convert_rgba_to_rgb_samples(rgba: &[u8], width: u32, height: u32) -> Vec<Option<[u8; 3]>> {
    let mut result = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if idx + 3 < rgba.len() {
                let r = rgba[idx];
                let g = rgba[idx + 1];
                let b = rgba[idx + 2];
                let a = rgba[idx + 3];

                // Transparent pixels become None
                if a < 128 {
                    result.push(None);
                } else {
                    result.push(Some([r, g, b]));
                }
            } else {
                result.push(None);
            }
        }
    }

    result
}
