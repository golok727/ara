use std::f32;
use std::ops::Range;

use crate::{paint::WHITE_UV, Vec2};

use super::{Color, Rgba, TextureId};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: Rgba,
}

impl Vertex {
    pub fn new(pos: Vec2<f32>, color: impl Into<Rgba>, uv: (f32, f32)) -> Self {
        Self {
            position: pos.into(),
            uv: uv.into(),
            color: color.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: TextureId,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.indices.clear();
        self.vertices = Default::default();
    }

    #[inline]
    pub fn add_vertex(&mut self, pos: Vec2<f32>, color: Color, uv: (f32, f32)) {
        self.vertices.push(Vertex::new(pos, color, uv));
    }

    pub fn map_range(&mut self, range: Range<usize>, f: impl Fn(&mut Vertex)) {
        for vertex in &mut self.vertices[range] {
            f(vertex);
        }
    }

    pub fn append(&mut self, other: &Self) {
        debug_assert!(other.is_valid());

        let index_offset = self.vertices.len() as u32;
        self.indices
            .extend(other.indices.iter().map(|index| index + index_offset));
        self.vertices.extend(other.vertices.iter());
    }

    pub fn is_valid(&self) -> bool {
        if let Ok(n) = u32::try_from(self.vertices.len()) {
            self.indices.iter().all(|&i| i < n)
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.vertices.is_empty()
    }

    #[inline(always)]
    pub fn reserve_prim(&mut self, vertex_count: usize, index_count: usize) {
        self.vertices.reserve(vertex_count);
        self.indices.reserve(index_count);
    }

    #[inline(always)]
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    #[inline(always)]
    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    #[inline(always)]
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn add_triangle_fan(
        &mut self,
        color: Color,
        connect_to: Vec2<f32>,
        origin: Vec2<f32>,
        start: Vec2<f32>,
        end: Vec2<f32>,
        clockwise: bool,
    ) {
        let from = start - origin;
        let to = end - origin;

        let mut from_angle = from.y.atan2(from.x);
        let mut to_angle = to.y.atan2(to.x);

        if clockwise {
            if to_angle > from_angle {
                to_angle -= f32::consts::TAU;
            }
        } else if from_angle > to_angle {
            from_angle -= f32::consts::TAU;
        }

        const ROUND_MIN_ANGLE: f32 = 0.174533; // ~10 deg

        let angle = to_angle - from_angle;
        let num_triangles = (angle / ROUND_MIN_ANGLE).abs().floor().max(1.0) as usize;
        let seg_angle = angle / num_triangles as f32;

        self.reserve_prim(2 + num_triangles, num_triangles * 3);

        let conn_vertex_index = self.vertex_count();
        let start_vertex_index = conn_vertex_index + 1;

        self.add_vertex(connect_to, color, WHITE_UV);
        self.add_vertex(start, color, WHITE_UV);

        let mut prev_vertex_index = start_vertex_index;

        for i in 0..num_triangles - 1 {
            let rotation = (i as f32 + 1.0) * seg_angle;
            let c = rotation.cos();
            let s = rotation.sin();
            let end_point = Vec2::new(c * from.x - s * from.y, s * from.x + c * from.y) + origin;

            let cur_vertex_idx = self.vertex_count();
            self.add_triangle(conn_vertex_index, prev_vertex_index, cur_vertex_idx);

            self.add_vertex(end_point, color, WHITE_UV);
            prev_vertex_index = cur_vertex_idx;
        }

        // add the end point
        self.add_triangle(conn_vertex_index, prev_vertex_index, self.vertex_count());

        self.add_vertex(end, color, WHITE_UV);
    }
}
