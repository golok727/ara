// Adapted from
// https://github.com/CrushedPixel/Polyline2D
// https://artgrammer.blogspot.com/2011/07/drawing-polylines-by-tessellation.html?m=1
use std::{ f32, ops::{ Deref, DerefMut } };

const MITER_MIN_ANGLE: f32 = 0.349066; // ~20 degrees

use crate::{ LineJoin, Vec2 };

use super::{ LineCap, Mesh, StrokeStyle, WHITE_UV };

/*
TODO

- [] support antialiasing
- [] support line dash

*/

#[derive(Debug)]
pub struct StrokeTessellator<'a> {
    mesh: StrokeTessellatorMesh<'a>,
}

impl<'a> StrokeTessellator<'a> {
    pub fn add_to_mesh(mesh: &'a mut Mesh, points: &[Vec2<f32>], stroke_style: &StrokeStyle) {
        let mut polyline = Self {
            mesh: StrokeTessellatorMesh::Borrowed(mesh),
        };

        polyline.add_polyline(points, stroke_style);
    }

    pub fn create(points: &[Vec2<f32>], stroke_style: &StrokeStyle) -> Mesh {
        let mut polyline = Self {
            mesh: StrokeTessellatorMesh::Owned(Default::default()),
        };

        polyline.add_polyline(points, stroke_style);

        match polyline.mesh {
            StrokeTessellatorMesh::Owned(mesh) => mesh,
            StrokeTessellatorMesh::Borrowed(_) => unreachable!(),
        }
    }

    fn add_polyline(&mut self, points: &[Vec2<f32>], stroke_style: &StrokeStyle) {
        if points.len() < 2 || stroke_style.line_width == 0 || stroke_style.color.is_transparent() {
            return;
        }

        let h_linewidth = (stroke_style.line_width.max(1) as f32) / 2.0;

        let segments: Vec<PolySegment> = points
            .windows(2)
            .filter(|p| p[0] != p[1])
            .map(|p| PolySegment::new(LineSegment::new(p[0], p[1]), h_linewidth))
            .collect();

        let mut join = false;

        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            if first == last {
                join = true;
            }
        }

        if segments.is_empty() {
            return;
        }

        let first_segment = segments.first().unwrap();
        let last_segment = segments.last().unwrap();

        // Path start and end edge vertices
        let mut path_start_1 = first_segment.edge1.a;
        let mut path_start_2 = first_segment.edge2.a;

        let mut path_end_1 = last_segment.edge1.b;
        let mut path_end_2 = last_segment.edge2.b;

        if join {
            self.polyline_create_joint(
                stroke_style,
                last_segment,
                first_segment,
                &mut path_end_1,
                &mut path_end_2,
                &mut path_start_1,
                &mut path_start_2
            );
        } else {
            match stroke_style.line_cap {
                LineCap::Butt => {
                    // NOOP
                }
                LineCap::Round => {
                    // add the start and end round caps
                    self.mesh.add_triangle_fan(
                        stroke_style.color,
                        first_segment.center.a,
                        first_segment.center.a,
                        path_start_1,
                        path_start_2,
                        false
                    );

                    self.mesh.add_triangle_fan(
                        stroke_style.color,
                        last_segment.center.b,
                        last_segment.center.b,
                        path_end_1,
                        path_end_2,
                        true
                    );
                }
                LineCap::Square => {
                    // offset the start and end with the half line width
                    path_start_1 += first_segment.edge1.direction() * h_linewidth;
                    path_start_2 += first_segment.edge2.direction() * h_linewidth;
                    path_end_1 -= last_segment.edge1.direction() * h_linewidth;
                    path_end_2 -= last_segment.edge2.direction() * h_linewidth;
                }
            }
        }

        let mut start_1: Vec2<f32> = Vec2::default();
        let mut start_2: Vec2<f32> = Vec2::default();

        let mut next_start_1: Vec2<f32> = Vec2::default();
        let mut next_start_2: Vec2<f32> = Vec2::default();

        let mut end_1: Vec2<f32> = Vec2::default();
        let mut end_2: Vec2<f32> = Vec2::default();

        for (i, segment) in segments.iter().enumerate() {
            if i == 0 {
                start_1 = path_start_1;
                start_2 = path_start_2;
            }

            if i + 1 == segments.len() {
                end_1 = path_end_1;
                end_2 = path_end_2;
            } else {
                // join the two segments
                self.polyline_create_joint(
                    stroke_style,
                    segment,
                    &segments[i + 1],
                    &mut end_1,
                    &mut end_2,
                    &mut next_start_1,
                    &mut next_start_2
                );
            }

            let cur_vertex_idx = self.mesh.vertex_count();
            // emit vertices
            self.mesh.reserve_prim(4, 6);
            self.mesh.add_vertex(start_1, stroke_style.color, WHITE_UV);
            self.mesh.add_vertex(start_2, stroke_style.color, WHITE_UV);
            self.mesh.add_vertex(end_1, stroke_style.color, WHITE_UV);
            self.mesh.add_vertex(end_2, stroke_style.color, WHITE_UV);

            self.mesh.add_triangle(cur_vertex_idx, cur_vertex_idx + 1, cur_vertex_idx + 2);

            self.mesh.add_triangle(cur_vertex_idx + 2, cur_vertex_idx + 1, cur_vertex_idx + 3);

            start_1 = next_start_1;
            start_2 = next_start_2;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn polyline_create_joint(
        &mut self,

        style: &StrokeStyle,
        segment1: &PolySegment,
        segment2: &PolySegment,

        end1: &mut Vec2<f32>,
        end2: &mut Vec2<f32>,

        next_start1: &mut Vec2<f32>,
        next_start2: &mut Vec2<f32>
    ) {
        let dir1 = segment1.center.direction();
        let dir2 = segment2.center.direction();

        let angle = dir1.angle(&dir2);

        let mut wrapped_angle = angle;
        if wrapped_angle > f32::consts::FRAC_PI_2 {
            wrapped_angle = f32::consts::PI - wrapped_angle;
        }

        let mut joint_style = style.line_join;

        if joint_style == LineJoin::Miter && wrapped_angle < MITER_MIN_ANGLE {
            joint_style = LineJoin::Bevel;
        }

        if joint_style == LineJoin::Miter {
            // calculate each edge's intersection point
            // with the next segment's central line
            let sec1 = segment1.edge1.intersection(&segment2.edge1, true);
            let sec2 = segment1.edge2.intersection(&segment2.edge2, true);

            *end1 = sec1.unwrap_or(segment1.edge1.b);
            *end2 = sec2.unwrap_or(segment1.edge2.b);

            *next_start1 = *end1;
            *next_start2 = *end2;
        } else {
            let x1 = dir1.x;
            let x2 = dir2.x;
            let y1 = dir1.y;
            let y2 = dir2.y;

            let clockwise = x1 * y2 - x2 * y1 < 0.0;

            let inner1: &LineSegment;
            let inner2: &LineSegment;
            let outer1: &LineSegment;
            let outer2: &LineSegment;

            if clockwise {
                outer1 = &segment1.edge1;
                outer2 = &segment2.edge1;
                inner1 = &segment1.edge2;
                inner2 = &segment2.edge2;
            } else {
                outer1 = &segment1.edge2;
                outer2 = &segment2.edge2;
                inner1 = &segment1.edge1;
                inner2 = &segment2.edge1;
            }

            let inner_sec_maybe = inner1.intersection(inner2, style.allow_overlap);
            let inner_sec = inner_sec_maybe.unwrap_or(inner1.b);

            let inner_start = if inner_sec_maybe.is_some() {
                inner_sec
            } else if angle > f32::consts::FRAC_PI_2 {
                outer1.b
            } else {
                inner1.b
            };

            if clockwise {
                *end1 = outer1.b;
                *end2 = inner_sec;

                *next_start1 = outer2.a;
                *next_start2 = inner_start;
            } else {
                *end1 = inner_sec;
                *end2 = outer1.b;

                *next_start1 = inner_start;
                *next_start2 = outer2.a;
            }

            if joint_style == LineJoin::Bevel {
                // simply connect the intersection points
                self.mesh.reserve_prim(3, 3);

                let cur_vertex_idx = self.mesh.vertex_count();

                self.mesh.add_vertex(outer1.b, style.color, WHITE_UV);
                self.mesh.add_vertex(outer2.a, style.color, WHITE_UV);
                self.mesh.add_vertex(inner_sec, style.color, WHITE_UV);

                self.mesh.add_triangle(
                    cur_vertex_idx, //
                    cur_vertex_idx + 1, //
                    cur_vertex_idx + 2 //
                );
            } else if joint_style == LineJoin::Round {
                self.mesh.add_triangle_fan(
                    style.color,
                    inner_sec,
                    segment1.center.b,
                    outer1.b,
                    outer2.a,
                    clockwise
                );
            }
        }
    }
}

#[derive(Debug)]
enum StrokeTessellatorMesh<'a> {
    Borrowed(&'a mut Mesh),
    Owned(Mesh),
}

impl<'a> Deref for StrokeTessellatorMesh<'a> {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        match &self {
            StrokeTessellatorMesh::Borrowed(mesh) => mesh,
            StrokeTessellatorMesh::Owned(mesh) => mesh,
        }
    }
}

impl<'a> DerefMut for StrokeTessellatorMesh<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            StrokeTessellatorMesh::Borrowed(mesh) => mesh,
            StrokeTessellatorMesh::Owned(mesh) => mesh,
        }
    }
}

#[derive(Debug, Clone)]
struct PolySegment {
    pub edge1: LineSegment,
    pub center: LineSegment,
    pub edge2: LineSegment,
}

impl PolySegment {
    #[inline]
    pub fn new(center: LineSegment, line_width: f32) -> Self {
        let normal = center.normal();
        let edge1 = center.clone() + normal * line_width;
        let edge2 = center.clone() - normal * line_width;

        Self {
            center,
            edge1,
            edge2,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LineSegment {
    pub a: Vec2<f32>,
    pub b: Vec2<f32>,
}

impl LineSegment {
    #[inline]
    pub fn new(a: Vec2<f32>, b: Vec2<f32>) -> Self {
        Self { a, b }
    }
}

impl std::ops::Add<Vec2<f32>> for LineSegment {
    type Output = Self;
    fn add(self, other: Vec2<f32>) -> Self::Output {
        Self {
            a: self.a + other,
            b: self.b + other,
        }
    }
}

impl std::ops::Mul<f32> for LineSegment {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self::Output {
        Self {
            a: self.a * scalar,
            b: self.b * scalar,
        }
    }
}

impl std::ops::Sub<Vec2<f32>> for LineSegment {
    type Output = Self;
    fn sub(self, other: Vec2<f32>) -> Self::Output {
        Self {
            a: self.a - other,
            b: self.b - other,
        }
    }
}

impl LineSegment {
    pub fn direction(&self) -> Vec2<f32> {
        self.a.direction(self.b)
    }

    pub fn normal(&self) -> Vec2<f32> {
        Vec2::new(-(self.b.y - self.a.y), self.b.x - self.a.x).direction(Vec2::new(0.0, 0.0))
    }

    // https://www.desmos.com/calculator/ujamclid3g
    pub fn intersection(
        &self,
        other: &LineSegment,
        allow_infinite_lines: bool
    ) -> Option<Vec2<f32>> {
        const EPSILON: f32 = 0.0001;

        let dir_self = self.b - self.a;
        let dir_other = other.b - other.a;

        let origin_dist = other.a - self.a;
        let numerator = origin_dist.cross(&dir_self);
        let denominator = dir_self.cross(&dir_other);

        // parallel
        if denominator.abs() < EPSILON {
            return None;
        }
        let u = numerator / denominator;
        let t = origin_dist.cross(&dir_other) / denominator;

        if !allow_infinite_lines && (!(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u)) {
            return None;
        }

        Some(self.a + dir_self * t)
    }
}
