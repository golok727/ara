use core::f32;
use std::cell::RefCell;
use std::ops::Range;

use ara_math::{IsZero, Mat3};

use super::{
    Brush, Circle, Color, FillStyle, Mesh, PathBrush, Primitive, Quad, StrokeTessellator, Vertex,
};

use crate::earcut::Earcut;
use crate::math::{Rect, Vec2};
use crate::paint::WHITE_UV;
use crate::{get_path_bounds, Contour, PathEventsIter, PathGeometryBuilder};

use std::ops::{Deref, DerefMut};

use crate::path::{Path, PathBuilder, Point};

#[derive(Default)]
struct ScratchPathBuilder {
    builder: PathBuilder,
    temp_path_data: Vec<Point>,
    earcut: Earcut<f32>,
}

#[derive(Debug, Clone, Copy)]
enum ShapeType {
    Concave,
    Convex,
}

#[derive(Debug, Clone, Copy)]
enum PathBuildMode {
    Single,
    Full,
}

enum AnyBrush<'a> {
    Brush(&'a Brush),
    Path(&'a PathBrush),
}

struct FillAndStrokeOptions<'a> {
    brush: AnyBrush<'a>,
    mesh: &'a mut Mesh,
    feathering: f32,
    shape_type: ShapeType,
    textured: bool,
    build_mode: PathBuildMode,
}

impl ScratchPathBuilder {
    fn _fill(
        mesh: &mut Mesh,
        path: &[Point],
        earcut: &mut Earcut<f32>,
        brush: &Brush,
        feathering: f32,
        textured: bool,
        shape_type: ShapeType,
    ) {
        if brush.fill_style.color.is_transparent() {
            return;
        }

        let fill_style = &brush.fill_style;
        let stroke_color = brush.stroke_style.color;

        match shape_type {
            ShapeType::Convex => {
                fill_path_convex(
                    mesh,
                    path,
                    fill_style.color,
                    textured,
                    feathering,
                    (!stroke_color.is_transparent()).then_some(stroke_color),
                    |_| {},
                );
            }
            ShapeType::Concave => {
                fill_path_concave(mesh, path, earcut, fill_style, feathering, |_| {});
            }
        }
    }

    fn fill_and_stroke(
        &mut self,
        options: FillAndStrokeOptions,
        map_points: Option<impl Fn(&mut [Point])>,
    ) {
        let FillAndStrokeOptions {
            brush,
            feathering,
            shape_type,
            build_mode,
            mesh,
            textured,
        } = options;

        let geometry: PathGeometryBuilder<_> =
            create_geometry_builder_for_path(self.builder.path_events(), &mut self.temp_path_data)
                .with_auto_segments();

        // Different handling based on build mode
        match build_mode {
            PathBuildMode::Single => {
                let brush = match brush {
                    AnyBrush::Brush(brush) => brush,
                    AnyBrush::Path(path_brush) => &path_brush.get_or_default(&Contour::default()),
                };

                let feathering = if brush.antialias { feathering } else { 0.0 };

                let range = expect_one_contour(geometry).1;

                if let Some(map_points) = map_points {
                    map_points(&mut self.temp_path_data[range.clone()]);
                }

                let path = &self.temp_path_data[range];

                Self::_fill(
                    mesh,
                    path,
                    &mut self.earcut,
                    brush,
                    feathering,
                    textured,
                    shape_type,
                );

                StrokeTessellator::add_to_mesh(mesh, path, &brush.stroke_style);
            }
            PathBuildMode::Full => {
                let geo_build = geometry.collect::<Vec<_>>();

                for (contour, range) in geo_build {
                    let brush = match brush {
                        AnyBrush::Brush(brush) => brush,
                        AnyBrush::Path(path_brush) => &path_brush.get_or_default(&contour),
                    };

                    let feathering = if brush.antialias { feathering } else { 0.0 };

                    let points = &mut self.temp_path_data[range];

                    if let Some(ref map_points) = map_points {
                        map_points(points);
                    }

                    Self::_fill(
                        mesh,
                        points,
                        &mut self.earcut,
                        brush,
                        feathering,
                        textured,
                        shape_type,
                    );
                    StrokeTessellator::add_to_mesh(mesh, points, &brush.stroke_style);
                }
            }
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.points.clear();
        self.verbs.clear();
        self.temp_path_data.clear();
    }
}

impl Deref for ScratchPathBuilder {
    type Target = PathBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for ScratchPathBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

#[derive(Default)]
pub struct DrawList {
    pub(crate) feathering_px: f32,
    pub(crate) mesh: Mesh,
    path: ScratchPathBuilder,
}

impl DrawList {
    pub fn feathering(&mut self, value: f32) {
        self.feathering_px = value;
    }

    pub fn clear(&mut self) {
        self.mesh.clear();
        self.path.clear();
    }

    #[inline]
    fn _get_feathering(&self, brush: &Brush) -> f32 {
        if brush.antialias {
            self.feathering_px
        } else {
            0.0
        }
    }

    /// captures any drawlist operations done inside the function `f` and returns a
    /// `DrawListCapture` allowing to modify the added vertex data
    pub fn capture(&mut self, f: impl FnOnce(&mut Self)) -> DrawListCapture<'_> {
        let start = self.mesh.vertices.len();
        f(self);
        let end = self.mesh.vertices.len();

        DrawListCapture {
            list: self,
            range: start..end,
        }
    }

    #[inline]
    pub fn capture_range(&mut self, f: impl FnOnce(&mut Self)) -> Range<usize> {
        let start = self.mesh.vertices.len();
        f(self);
        let end = self.mesh.vertices.len();
        start..end
    }

    #[inline]
    pub fn map_range(&mut self, range: Range<usize>, f: impl Fn(&mut Vertex)) {
        for vertex in &mut self.mesh.vertices[range] {
            f(vertex);
        }
    }

    pub fn add_quad(
        &mut self,
        quad: &Quad,
        brush: &Brush,
        textured: bool,
        transform: Option<Mat3>,
    ) {
        let has_no_corner_radius = quad.corners.is_zero();

        self.path.clear();

        if has_no_corner_radius {
            self.path.rect(&quad.bounds);
        } else {
            self.path.round_rect(&quad.bounds, &quad.corners);
        }

        self.path.fill_and_stroke(
            FillAndStrokeOptions {
                brush: AnyBrush::Brush(brush),
                mesh: &mut self.mesh,
                feathering: self.feathering_px,
                shape_type: ShapeType::Convex,
                textured,
                build_mode: PathBuildMode::Single,
            },
            Some(|path: &mut [Point]| {
                if let Some(transform) = transform {
                    if !transform.is_identity() {
                        for point in path.iter_mut() {
                            *point = transform * *point;
                        }
                    }
                }
            }),
        );
    }

    pub fn add_circle(
        &mut self,
        circle: &Circle,
        brush: &Brush,
        textured: bool,
        transform: Option<Mat3>,
    ) {
        self.path.clear();

        self.path.circle(circle.center, circle.radius);

        self.path.fill_and_stroke(
            FillAndStrokeOptions {
                brush: AnyBrush::Brush(brush),
                mesh: &mut self.mesh,
                feathering: self.feathering_px,
                shape_type: ShapeType::Convex,
                textured,
                build_mode: PathBuildMode::Single,
            },
            Some(|path: &mut [Point]| {
                if let Some(transform) = transform {
                    if !transform.is_identity() {
                        for point in path.iter_mut() {
                            *point = transform * *point;
                        }
                    }
                }
            }),
        );
    }

    pub fn add_path(&mut self, path: &Path, brush: &PathBrush, transform: Option<Mat3>) {
        self.path.clear();
        self.path.extend(path);

        self.path.fill_and_stroke(
            FillAndStrokeOptions {
                brush: AnyBrush::Path(brush),
                mesh: &mut self.mesh,
                feathering: self.feathering_px,
                shape_type: ShapeType::Concave,
                textured: false,
                build_mode: PathBuildMode::Full,
            },
            Some(|path: &mut [Point]| {
                if let Some(transform) = transform {
                    if !transform.is_identity() {
                        for point in path.iter_mut() {
                            *point = transform * *point;
                        }
                    }
                }
            }),
        );
    }

    pub fn add_primitive(
        &mut self,
        primitive: &Primitive,
        brush: &Brush,
        textured: bool,
        transform: Option<Mat3>,
    ) {
        match primitive {
            Primitive::Circle(circle) => self.add_circle(circle, brush, textured, transform),

            Primitive::Quad(quad) => self.add_quad(quad, brush, textured, transform),

            Primitive::Path { path, brush } => self.add_path(path, brush, transform),
        }
    }

    #[allow(unused)]
    fn fill_rect(&mut self, rect: &Rect<f32>, color: Color) {
        if color.is_transparent() {
            return;
        }

        let v_index_offset = self.mesh.vertex_count();
        self.mesh.reserve_prim(4, 6);

        self.mesh.add_vertex(rect.top_left(), color, (0.0, 0.0)); // Top-left
        self.mesh.add_vertex(rect.top_right(), color, (1.0, 0.0)); // Top-right
        self.mesh.add_vertex(rect.bottom_left(), color, (0.0, 1.0)); // Bottom-left
        self.mesh.add_vertex(rect.bottom_right(), color, (1.0, 1.0)); // Bottom-right

        self.mesh
            .add_triangle(v_index_offset, v_index_offset + 1, v_index_offset + 2);

        self.mesh
            .add_triangle(v_index_offset + 2, v_index_offset + 1, v_index_offset + 3);
    }

    #[allow(unused)]
    fn add_triangle_fan(
        &mut self,
        color: Color,
        connect_to: Vec2<f32>,
        origin: Vec2<f32>,
        start: Vec2<f32>,
        end: Vec2<f32>,
        clockwise: bool,
    ) {
        self.mesh
            .add_triangle_fan(color, connect_to, origin, start, end, clockwise);
    }

    pub fn build(&mut self) -> Mesh {
        std::mem::take(&mut self.mesh)
    }
}

pub struct DrawListCapture<'a> {
    list: &'a mut DrawList,
    range: Range<usize>,
}

impl DrawListCapture<'_> {
    pub fn map(self, f: impl Fn(&mut Vertex)) {
        self.list.map_range(self.range, f)
    }
}

#[inline]
pub fn expect_one_contour<I>(mut iter: I) -> (Contour, Range<usize>)
where
    I: Iterator<Item = (Contour, Range<usize>)>,
{
    iter.next()
        .expect("create_single_contour_path called with path with no contour!")
}

fn create_geometry_builder_for_path<'a>(
    iter: PathEventsIter<'a>,
    out: &'a mut Vec<Point>,
) -> PathGeometryBuilder<'a, PathEventsIter<'a>> {
    PathGeometryBuilder::new(iter, out)
}

thread_local! {
    static TEMP_BUFFER: RefCell<Vec<Vec2<f32>>> = Default::default();
}

fn is_path_closed(path: &[Vec2<f32>]) -> bool {
    if let (Some(first), Some(last)) = (path.first(), path.last()) {
        first == last
    } else {
        false
    }
}

pub fn fill_path_concave(
    mesh: &mut Mesh,
    path: &[Vec2<f32>],
    earcut: &mut Earcut<f32>,
    fill_style: &FillStyle,
    feathering: f32,
    mut on_add: impl FnMut(Point),
) {
    let points_count = {
        let n = path.len() as u32;

        if is_path_closed(path) {
            n - 1
        } else {
            n
        }
    };

    let fill = fill_style.color;
    if points_count < 3 || fill.is_transparent() {
        return;
    }

    let path = &path[..points_count as usize];

    if feathering > 0.0 {
        let out_color = {
            let mut c = fill_style.color;
            c.a = 0;
            c
        };

        let idx_inner = mesh.vertices.len() as u32;
        let idx_outer = idx_inner + 1;

        mesh.reserve_prim(
            2 * (points_count as usize), // 2 vertices per point (inner + outer)
            ((points_count - 2) * 3 + points_count * 6) as usize, // Fill triangles + 6 indices per edge for feathering
        );

        let mut temp_indices = <Vec<u32>>::new();
        earcut.earcut(
            path.iter().map(|p| [p.x, p.y]),
            &[],
            &mut temp_indices,
            false,
        );

        for triangle in temp_indices.chunks_exact(3) {
            let [i0, i1, i2] = [triangle[0], triangle[1], triangle[2]];

            let v0 = idx_inner + ((points_count - 1 - i0) % points_count) * 2;
            let v1 = idx_inner + ((points_count - 1 - i1) % points_count) * 2;
            let v2 = idx_inner + ((points_count - 1 - i2) % points_count) * 2;

            mesh.add_triangle(v0, v1, v2);
        }

        TEMP_BUFFER.with_borrow_mut(|normals| {
            normals.clear();
            normals.reserve(points_count as usize);

            // todo account for sharp angles

            let mut i0 = points_count - 1;
            for i1 in 0..points_count {
                let p0 = path[i0 as usize];
                let p1 = path[i1 as usize];
                let edge = (p1 - p0).normalize().rot90();
                normals.push(edge);
                i0 = i1;
            }

            // The feathering:
            let mut i0 = points_count - 1;
            for i1 in 0..points_count {
                let n0 = normals[i0 as usize];
                let n1 = normals[i1 as usize];
                let dm = (n0 + n1).normalize() * feathering * 0.5;
                let p = path[i0 as usize];

                let pos_inner = p - dm;
                let pos_outer = p + dm;

                on_add(pos_inner);
                on_add(pos_outer);
                mesh.add_vertex(pos_inner, fill, WHITE_UV);
                mesh.add_vertex(pos_outer, out_color, WHITE_UV);

                mesh.add_triangle(idx_inner + i1 * 2, idx_inner + i0 * 2, idx_outer + 2 * i0);
                mesh.add_triangle(idx_outer + i0 * 2, idx_outer + i1 * 2, idx_inner + 2 * i1);
                i0 = i1;
            }
        });
    } else {
        // Non-AA fill
        let vertex_offset = mesh.vertices.len() as u32;
        let index_offset = mesh.indices.len();
        // No antialiasing: simple fill
        mesh.reserve_prim(points_count as usize, ((points_count as usize) - 2) * 3);

        // Add vertices for the fill
        mesh.vertices
            .extend(path.iter().map(|p| Vertex::new(*p, fill, WHITE_UV)));

        // Perform earcut triangulation
        earcut.earcut(
            path.iter().map(|p| [p.x, p.y]),
            &[],
            &mut mesh.indices,
            false,
        );

        // Adjust indices to account for vertex offset
        for i in mesh.indices.iter_mut().skip(index_offset) {
            *i += vertex_offset;
        }
    }
}

pub fn fill_path_convex(
    mesh: &mut Mesh,
    path: &[Point],
    fill: Color,
    textured: bool,
    feathering: f32,
    fade_to: Option<Color>,
    mut on_add: impl FnMut(Point),
) {
    let points_count = {
        let n = path.len() as u32;

        if is_path_closed(path) {
            n - 1
        } else {
            n
        }
    };

    if points_count < 3 || fill.is_transparent() {
        return;
    }
    let path = &path[..points_count as usize];

    debug_assert!(cw_signed_area(path) > 0.0, "Path should be clockwise");

    let bounds = if textured {
        get_path_bounds(path)
    } else {
        Default::default()
    };
    let b_min = bounds.min();
    let b_max = bounds.max();

    let get_uv = |point: &Point| {
        let uv_x = if b_max.x != b_min.x {
            (point.x - b_min.x) / (b_max.x - b_min.x)
        } else {
            0.0
        };
        let uv_y = if b_max.y != b_min.y {
            (point.y - b_min.y) / (b_max.y - b_min.y)
        } else {
            0.0
        };

        (uv_x, uv_y)
    };

    if feathering > 0.0 {
        // AA fill
        let out_color = fade_to.unwrap_or_else(|| {
            let mut c = fill;
            c.a = 0;
            c
        });

        mesh.reserve_prim(2 * (points_count as usize), 3 * (points_count as usize));

        let idx_inner = mesh.vertices.len() as u32;
        let idx_outer = idx_inner + 1;

        // The fill:
        for i in 2..points_count {
            mesh.add_triangle(idx_inner + 2 * (i - 1), idx_inner, idx_inner + 2 * i);
        }

        // TODO: precompute normals on building path
        TEMP_BUFFER.with_borrow_mut(|normals| {
            normals.clear();
            normals.reserve(points_count as usize);

            let mut i0 = points_count - 1;
            for i1 in 0..points_count {
                let p0 = path[i0 as usize];
                let p1 = path[i1 as usize];
                let edge = (p1 - p0).normalize().rot90();
                normals.push(edge);
                i0 = i1;
            }

            // The feathering:
            let mut i0 = points_count - 1;
            for i1 in 0..points_count {
                let n0 = normals[i0 as usize];
                let n1 = normals[i1 as usize];
                let dm = (n0 + n1).normalize() * feathering * 0.5;
                let p = path[i0 as usize];

                let pos_inner = p - dm;
                let pos_outer = p + dm;

                on_add(pos_inner);
                on_add(pos_outer);
                mesh.add_vertex(pos_inner, fill, get_uv(&pos_inner));
                mesh.add_vertex(pos_outer, out_color, get_uv(&pos_outer));
                mesh.add_triangle(idx_inner + i1 * 2, idx_inner + i0 * 2, idx_outer + 2 * i0);
                mesh.add_triangle(idx_outer + i0 * 2, idx_outer + i1 * 2, idx_inner + 2 * i1);
                i0 = i1;
            }
        });
    } else {
        let index_count = (points_count - 2) * 3;
        let vtx_count = points_count;

        mesh.reserve_prim(vtx_count as usize, index_count as usize);
        let idx = mesh.vertex_count();

        for point in path {
            let uv = get_uv(point);
            mesh.add_vertex(*point, fill, uv);
        }

        for i in 2..points_count {
            mesh.add_triangle(idx, idx + (i - 1), idx + i);
        }
    }
}

fn cw_signed_area(path: &[Point]) -> f64 {
    if let Some(last) = path.last() {
        let mut previous = *last;
        let mut area = 0.0;
        for p in path {
            area += (previous.x * p.y - p.x * previous.y) as f64;
            previous = *p;
        }
        area
    } else {
        0.0
    }
}
