use engine_asset::ObjMesh;

use crate::geom::clip::{clip_line_to_viewport, clipped_depths, Viewport};
use crate::geom::types::ProjectedVertex;
use crate::raster::{draw_line_depth, draw_line_flat};

pub(crate) fn draw_wireframe_edges_with_depth(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
    virtual_w: u16,
    virtual_h: u16,
    clipped_viewport: Viewport,
    line_color: [u8; 3],
    depth_near: f32,
    depth_far: f32,
    max_edges: usize,
) {
    let mut drawn_edges = 0usize;
    for (a, b) in &mesh.edges {
        if drawn_edges > max_edges {
            break;
        }
        let Some(pa) = projected.get(*a).and_then(|p| *p) else {
            continue;
        };
        let Some(pb) = projected.get(*b).and_then(|p| *p) else {
            continue;
        };
        let x0 = pa.x.round() as i32;
        let y0 = pa.y.round() as i32;
        let x1 = pb.x.round() as i32;
        let y1 = pb.y.round() as i32;
        if let Some((cx0, cy0, cx1, cy1)) =
            clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
        {
            let (cz0, cz1) =
                clipped_depths(x0, y0, x1, y1, cx0, cy0, cx1, cy1, pa.depth, pb.depth);
            draw_line_depth(
                canvas, depth_buf, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color, cz0,
                cz1, depth_near, depth_far,
            );
            drawn_edges += 1;
        }
    }
}

pub(crate) fn draw_outline_edges_flat(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    clipped_viewport: Viewport,
    line_color: [u8; 3],
) {
    for (a, b) in &mesh.edges {
        let Some(pa) = projected.get(*a).and_then(|p| *p) else {
            continue;
        };
        let Some(pb) = projected.get(*b).and_then(|p| *p) else {
            continue;
        };
        let x0 = pa.x.round() as i32;
        let y0 = pa.y.round() as i32;
        let x1 = pb.x.round() as i32;
        let y1 = pb.y.round() as i32;
        if let Some((cx0, cy0, cx1, cy1)) =
            clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
        {
            draw_line_flat(canvas, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color);
        }
    }
}
