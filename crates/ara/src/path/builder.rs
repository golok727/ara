use ara_math::{vec2, Corners, Rect};

use super::{Path, PathEvent, PathEventsIter, PathVerb, Point, Polygon};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, PartialOrd, Eq)]
pub struct Contour(pub(crate) usize);

impl Contour {
    pub const INVALID: Contour = Contour(0);
}

#[derive(Default, Clone)]
pub struct PathBuilder {
    pub(crate) points: Vec<Point>,
    pub(crate) verbs: Vec<PathVerb>,
    pub(crate) validator: DebugPathValidator,

    first: Point,
}

impl PathBuilder {
    pub fn with_capacity(points: usize, edges: usize) -> Self {
        Self {
            points: Vec::with_capacity(points),
            verbs: Vec::with_capacity(edges),
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.verbs.clear();
        self.validator = DebugPathValidator::default();
    }

    pub fn extend<Events>(&mut self, other: Events)
    where
        Events: IntoIterator<Item = PathEvent>,
    {
        for event in other.into_iter() {
            match event {
                PathEvent::Begin { at } => {
                    self.begin(at);
                }
                PathEvent::Line { from: _, to } => {
                    self.line_to(to);
                }
                PathEvent::Quadratic { from: _, ctrl, to } => {
                    self.quadratic_to(ctrl, to);
                }
                PathEvent::Cubic {
                    from: _,
                    ctrl1,
                    ctrl2,
                    to,
                } => {
                    self.cubic_to(ctrl1, ctrl2, to);
                }
                PathEvent::End {
                    contour: _,
                    last: _,
                    close,
                    first: _,
                } => {
                    self.end(close);
                }
            }
        }
    }

    pub fn begin(&mut self, at: Point) {
        self.validator.begin();
        check_is_nan(at);

        self.first = at;
        self.points.push(at);
        self.verbs.push(PathVerb::Begin);
    }

    pub fn end(&mut self, close: bool) -> Contour {
        self.validator.end();

        if close {
            self.points.push(self.first);
        }

        self.verbs.push(if close {
            PathVerb::Close
        } else {
            PathVerb::End
        });

        Contour(self.points.len())
    }

    /// alias for self.end(true)
    #[inline]
    pub fn close(&mut self) -> Contour {
        self.end(true)
    }

    #[inline]
    pub fn path_events(&self) -> PathEventsIter {
        self.validator.build();
        PathEventsIter::new(&self.points, &self.verbs)
    }

    pub fn line_to(&mut self, to: Point) {
        self.validator.edge();
        check_is_nan(to);

        self.points.push(to);
        self.verbs.push(PathVerb::LineTo)
    }

    pub fn quadratic_to(&mut self, ctrl: Point, to: Point) {
        self.validator.edge();
        check_is_nan(ctrl);
        check_is_nan(to);

        self.points.push(ctrl);
        self.points.push(to);
        self.verbs.push(PathVerb::QuadraticTo);
    }

    pub fn cubic_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.validator.edge();
        check_is_nan(ctrl1);
        check_is_nan(ctrl2);
        check_is_nan(to);

        self.points.push(ctrl1);
        self.points.push(ctrl2);
        self.points.push(to);
        self.verbs.push(PathVerb::CubicTo);
    }

    pub fn add_point(&mut self, at: Point) {
        self.begin(at);
        self.end(false);
    }

    pub fn polygon(&mut self, polygon: Polygon<Point>) -> Contour {
        if polygon.points.is_empty() {
            return Contour::INVALID;
        }

        self.begin(polygon.points[0]);

        self.reserve(polygon.points.len(), 0);

        for p in &polygon.points[1..] {
            self.line_to(*p);
        }

        self.end(polygon.closed)
    }

    pub fn triangle(&mut self, a: Point, b: Point, c: Point) -> Contour {
        self.reserve(3, 0);
        self.begin(a);
        self.line_to(b);
        self.line_to(c);
        self.end(true)
    }

    pub fn rect(&mut self, rect: &Rect<f32>) -> Contour {
        self.polygon(Polygon {
            points: &[
                rect.top_left(),
                rect.top_right(),
                rect.bottom_right(),
                rect.bottom_left(),
            ],
            closed: true,
        })
    }

    pub fn round_rect(&mut self, rect: &Rect<f32>, corners: &Corners<f32>) -> Contour {
        add_rounded_rectangle(self, rect, corners)
    }

    pub fn circle(&mut self, center: Point, radius: f32) -> Contour {
        add_circle(self, center, radius)
    }

    pub fn reserve(&mut self, endpoints: usize, ctrl_points: usize) {
        self.points.reserve(endpoints + ctrl_points);
        self.verbs.reserve(endpoints);
    }

    #[must_use]
    pub fn build(self) -> Path {
        self.validator.build();

        Path {
            points: self.points.into_boxed_slice(),
            verbs: self.verbs.into_boxed_slice(),
        }
    }
}

// Adapted from
// https://github.com/nical/lyon/blob/main/crates/path/src/builder.rs
fn add_circle(builder: &mut PathBuilder, center: Point, radius: f32) -> Contour {
    let radius = radius.abs();
    // need this ?  we always go for positive winding
    // let dir = match winding {
    //     Winding::Positive => 1.0,
    //     Winding::Negative => -1.0,
    // };

    let dir = 1.0;

    // https://spencermortensen.com/articles/bezier-circle/
    const CONSTANT_FACTOR: f32 = 0.55191505;
    let d = radius * CONSTANT_FACTOR;

    builder.begin(center + vec2(-radius, 0.0));

    let ctrl_0 = center + vec2(-radius, -d * dir);
    let ctrl_1 = center + vec2(-d, -radius * dir);
    let mid = center + vec2(0.0, -radius * dir);
    builder.cubic_to(ctrl_0, ctrl_1, mid);

    let ctrl_0 = center + vec2(d, -radius * dir);
    let ctrl_1 = center + vec2(radius, -d * dir);
    let mid = center + vec2(radius, 0.0);
    builder.cubic_to(ctrl_0, ctrl_1, mid);

    let ctrl_0 = center + vec2(radius, d * dir);
    let ctrl_1 = center + vec2(d, radius * dir);
    let mid = center + vec2(0.0, radius * dir);
    builder.cubic_to(ctrl_0, ctrl_1, mid);

    let ctrl_0 = center + vec2(-d, radius * dir);
    let ctrl_1 = center + vec2(-radius, d * dir);
    let mid = center + vec2(-radius, 0.0);
    builder.cubic_to(ctrl_0, ctrl_1, mid);

    builder.close()
}

fn add_rounded_rectangle(
    builder: &mut PathBuilder,
    rect: &Rect<f32>,
    corners: &Corners<f32>,
) -> Contour {
    let w = rect.size.width;
    let h = rect.size.height;
    let min = rect.min();
    let max = rect.max();

    let x_min = min.x;
    let y_min = min.y;
    let x_max = max.x;
    let y_max = max.y;
    let min_wh = w.min(h);
    let mut tl = corners.top_left.abs().min(min_wh);
    let mut tr = corners.top_right.abs().min(min_wh);
    let mut bl = corners.bottom_left.abs().min(min_wh);
    let mut br = corners.bottom_right.abs().min(min_wh);

    // clamp border radii if they don't fit in the rectangle.
    if tl + tr > w {
        let x = (tl + tr - w) * 0.5;
        tl -= x;
        tr -= x;
    }
    if bl + br > w {
        let x = (bl + br - w) * 0.5;
        bl -= x;
        br -= x;
    }
    if tr + br > h {
        let x = (tr + br - h) * 0.5;
        tr -= x;
        br -= x;
    }
    if tl + bl > h {
        let x = (tl + bl - h) * 0.5;
        tl -= x;
        bl -= x;
    }

    // https://spencermortensen.com/articles/bezier-circle/
    const CONSTANT_FACTOR: f32 = 0.55191505;

    let tl_d = tl * CONSTANT_FACTOR;
    let tl_corner = vec2(x_min, y_min);

    let tr_d = tr * CONSTANT_FACTOR;
    let tr_corner = vec2(x_max, y_min);

    let br_d = br * CONSTANT_FACTOR;
    let br_corner = vec2(x_max, y_max);

    let bl_d = bl * CONSTANT_FACTOR;
    let bl_corner = vec2(x_min, y_max);

    let points = [
        vec2(x_min, y_min + tl),          // begin
        tl_corner + vec2(0.0, tl - tl_d), // control
        tl_corner + vec2(tl - tl_d, 0.0), // control
        tl_corner + vec2(tl, 0.0),        // end
        vec2(x_max - tr, y_min),
        tr_corner + vec2(-tr + tr_d, 0.0),
        tr_corner + vec2(0.0, tr - tr_d),
        tr_corner + vec2(0.0, tr),
        vec2(x_max, y_max - br),
        br_corner + vec2(0.0, -br + br_d),
        br_corner + vec2(-br + br_d, 0.0),
        br_corner + vec2(-br, 0.0),
        vec2(x_min + bl, y_max),
        bl_corner + vec2(bl - bl_d, 0.0),
        bl_corner + vec2(0.0, -bl + bl_d),
        bl_corner + vec2(0.0, -bl),
    ];

    builder.begin(points[0]);
    if tl > 0.0 {
        builder.cubic_to(points[1], points[2], points[3]);
    }
    builder.line_to(points[4]);

    if tl > 0.0 {
        builder.cubic_to(points[5], points[6], points[7]);
    }

    builder.line_to(points[8]);
    if br > 0.0 {
        builder.cubic_to(points[9], points[10], points[11]);
    }
    builder.line_to(points[12]);
    if bl > 0.0 {
        builder.cubic_to(points[13], points[14], points[15]);
    }
    builder.end(true)
}

#[inline]
fn check_is_nan(p: Point) {
    debug_assert!(p.x.is_finite());
    debug_assert!(p.y.is_finite());
}

#[derive(Default, Clone)]
pub(crate) struct DebugPathValidator {
    #[cfg(debug_assertions)]
    in_subpath: bool,
}

impl DebugPathValidator {
    #[inline(always)]
    pub fn begin(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(
                !self.in_subpath,
                "Please end the current subpath with `end(<close>)` or `close()` before starting a new one"
            );
            self.in_subpath = true;
        }
    }

    #[inline(always)]
    pub fn end(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(self.in_subpath, "Please start a new subpath with `begin()`");
            self.in_subpath = false;
        }
    }

    #[inline(always)]
    pub fn edge(&self) {
        #[cfg(debug_assertions)]
        assert!(
            self.in_subpath,
            "Please begin a new subpath with begin() to continue this operation"
        )
    }

    #[inline(always)]
    pub fn build(&self) {
        #[cfg(debug_assertions)]
        assert!(
            !self.in_subpath,
            "Please end the current subpath with `end(<close>)` or `close()` before building"
        )
    }
}

#[cfg(test)]
mod tests {
    use ara_math::{vec2, Corners, Rect};

    use super::super::*;
    #[test]
    fn path_builder_basic_test() {
        // closed
        {
            let mut path = Path::builder();
            path.begin((0.0, 0.0).into());
            path.line_to((5.0, 5.0).into());
            path.line_to((10.0, 10.0).into());
            path.line_to((2.0, 10.0).into());
            path.close();

            assert_eq!(
                &path.points,
                &[
                    vec2(0.0, 0.0),
                    vec2(5.0, 5.0),
                    vec2(10.0, 10.0),
                    vec2(2.0, 10.0),
                    vec2(0.0, 0.0)
                ]
            );

            assert_eq!(
                &path.verbs,
                &[
                    PathVerb::Begin,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::Close,
                ]
            );
        }

        // open
        {
            let mut path = Path::builder();
            path.begin((0.0, 0.0).into());
            path.line_to((5.0, 5.0).into());
            path.line_to((10.0, 10.0).into());
            path.line_to((2.0, 10.0).into());
            path.end(false);

            assert_eq!(
                &path.points,
                &[
                    vec2(0.0, 0.0),
                    vec2(5.0, 5.0),
                    vec2(10.0, 10.0),
                    vec2(2.0, 10.0)
                ]
            );

            assert_eq!(
                &path.verbs,
                &[
                    PathVerb::Begin,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::End,
                ]
            );
        }
    }

    #[test]
    fn path_builder_quadratic_to() {
        let mut path = Path::builder();

        path.begin(vec2(0.0, 0.0));
        path.quadratic_to(vec2(5.0, 5.0), vec2(10.0, 0.0));
        assert_eq!(
            &path.points,
            &[vec2(0.0, 0.0), vec2(5.0, 5.0), vec2(10.0, 0.0)]
        );
        path.end(false);

        assert_eq!(
            &path.verbs,
            &[PathVerb::Begin, PathVerb::QuadraticTo, PathVerb::End]
        );
    }

    #[test]
    fn path_builder_cubic_to() {
        let mut path = Path::builder();

        path.begin(vec2(0.0, 0.0));
        path.cubic_to(vec2(0.0, 5.0), vec2(10.0, 5.0), vec2(10.0, 0.0));
        assert_eq!(
            &path.points,
            &[
                vec2(0.0, 0.0),
                vec2(0.0, 5.0),
                vec2(10.0, 5.0),
                vec2(10.0, 0.0)
            ]
        );
        path.end(false);

        assert_eq!(
            &path.verbs,
            &[PathVerb::Begin, PathVerb::CubicTo, PathVerb::End]
        );
    }

    #[test]
    fn path_builder_round_rect() {
        let mut path = Path::builder();
        path.round_rect(&Rect::xywh(0.0, 0.0, 10.0, 10.0), &Corners::with_all(3.0));

        assert_eq!(
            &path.points,
            &[
                vec2(0.0, 3.0),
                vec2(0.0, 1.3442549),
                vec2(1.3442549, 0.0),
                vec2(3.0, 0.0),
                vec2(7.0, 0.0),
                vec2(8.6557455, 0.0),
                vec2(10.0, 1.3442549),
                vec2(10.0, 3.0),
                vec2(10.0, 7.0),
                vec2(10.0, 8.6557455),
                vec2(8.6557455, 10.0),
                vec2(7.0, 10.0),
                vec2(3.0, 10.0),
                vec2(1.3442549, 10.0),
                vec2(0.0, 8.6557455),
                vec2(0.0, 7.0),
                vec2(0.0, 3.0),
            ]
        );

        assert_eq!(
            &path.verbs,
            &[
                PathVerb::Begin,
                PathVerb::CubicTo,
                PathVerb::LineTo,
                PathVerb::CubicTo,
                PathVerb::LineTo,
                PathVerb::CubicTo,
                PathVerb::LineTo,
                PathVerb::CubicTo,
                PathVerb::Close,
            ]
        );
    }

    #[test]
    fn path_builder_extend() {
        // Create the first path
        let mut path1 = Path::builder();
        path1.begin(vec2(0.0, 0.0));
        path1.line_to(vec2(10.0, 0.0));
        path1.line_to(vec2(10.0, 10.0));
        path1.end(true);

        // Create a second path to be used as source for events
        let mut path2 = Path::builder();
        path2.begin(vec2(20.0, 20.0));
        path2.line_to(vec2(30.0, 20.0));
        path2.quadratic_to(vec2(35.0, 25.0), vec2(30.0, 30.0));
        path2.cubic_to(vec2(25.0, 35.0), vec2(20.0, 35.0), vec2(20.0, 30.0));
        path2.end(true);

        // Create a third path and extend it with events from the first two
        let mut combined = Path::builder();
        combined.extend(path1.path_events());
        combined.extend(path2.path_events());

        let result = combined.build();

        // Expected points based on the operations
        let expected_points = vec![
            // First path
            vec2(0.0, 0.0),   // begin
            vec2(10.0, 0.0),  // line_to
            vec2(10.0, 10.0), // line_to
            vec2(0.0, 0.0),   // close
            // Second path
            vec2(20.0, 20.0), // begin
            vec2(30.0, 20.0), // line_to
            vec2(35.0, 25.0), // quadratic control point
            vec2(30.0, 30.0), // quadratic end point
            vec2(25.0, 35.0), // cubic control point 1
            vec2(20.0, 35.0), // cubic control point 2
            vec2(20.0, 30.0), // cubic end point
            vec2(20.0, 20.0), // close
        ];

        // Expected verbs
        let expected_verbs = vec![
            PathVerb::Begin,
            PathVerb::LineTo,
            PathVerb::LineTo,
            PathVerb::Close,
            PathVerb::Begin,
            PathVerb::LineTo,
            PathVerb::QuadraticTo,
            PathVerb::CubicTo,
            PathVerb::Close,
        ];

        assert_eq!(result.points.as_ref(), expected_points.as_slice());
        assert_eq!(result.verbs.as_ref(), expected_verbs.as_slice());
    }

    #[test]
    fn path_builder_circle() {
        let mut path = Path::builder();
        path.circle((0.0, 0.0).into(), 10.0);

        assert_eq!(
            &path.points,
            &[
                vec2(-10.0, 0.0),
                vec2(-10.0, -5.5191507),
                vec2(-5.5191507, -10.0),
                vec2(0.0, -10.0),
                vec2(5.5191507, -10.0),
                vec2(10.0, -5.5191507),
                vec2(10.0, 0.0),
                vec2(10.0, 5.5191507),
                vec2(5.5191507, 10.0),
                vec2(0.0, 10.0),
                vec2(-5.5191507, 10.0),
                vec2(-10.0, 5.5191507),
                vec2(-10.0, 0.0),
                vec2(-10.0, 0.0),
            ]
        );

        assert_eq!(
            &path.verbs,
            &[
                PathVerb::Begin,
                PathVerb::CubicTo,
                PathVerb::CubicTo,
                PathVerb::CubicTo,
                PathVerb::CubicTo,
                PathVerb::Close,
            ]
        );
    }

    #[test]
    fn path_builder_rect() {
        let mut path = Path::builder();
        path.rect(&Rect::xywh(10.0, 10.0, 100.0, 100.0));

        assert_eq!(
            &path.points,
            &[
                vec2(10.0, 10.0),
                vec2(110.0, 10.0),
                vec2(110.0, 110.0),
                vec2(10.0, 110.0),
                vec2(10.0, 10.0),
            ]
        );

        assert_eq!(
            &path.verbs,
            &[
                PathVerb::Begin,
                PathVerb::LineTo,
                PathVerb::LineTo,
                PathVerb::LineTo,
                PathVerb::Close,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Please start a new subpath with `begin()`")]
    fn path_builder_validator_end_without_a_subpath() {
        let mut path = Path::builder();
        path.close();
    }

    #[test]
    #[should_panic(
        expected = "Please end the current subpath with `end(<close>)` or `close()` before starting a new one"
    )]
    fn path_builder_validator_begin_without_end() {
        let mut path = Path::builder();
        path.begin(vec2(0.0, 0.0));
        path.begin(vec2(10.0, 10.0));
    }

    #[test]
    fn path_builder_polygon() {
        // closed
        {
            let mut path = Path::builder();
            path.polygon(Polygon {
                points: &[
                    vec2(0.0, 0.0),
                    vec2(10.0, 100.0),
                    vec2(200.0, 300.0),
                    vec2(500.0, 600.0),
                ],
                closed: true,
            });

            assert_eq!(
                &path.points,
                &[
                    vec2(0.0, 0.0),
                    vec2(10.0, 100.0),
                    vec2(200.0, 300.0),
                    vec2(500.0, 600.0),
                    vec2(0.0, 0.0),
                ]
            );

            assert_eq!(
                &path.verbs,
                &[
                    PathVerb::Begin,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::Close,
                ]
            );
        }
        // open
        {
            let mut path = Path::builder();
            path.polygon(Polygon {
                points: &[
                    vec2(0.0, 0.0),
                    vec2(10.0, 100.0),
                    vec2(200.0, 300.0),
                    vec2(500.0, 600.0),
                ],
                closed: false,
            });
            assert_eq!(
                &path.points,
                &[
                    vec2(0.0, 0.0),
                    vec2(10.0, 100.0),
                    vec2(200.0, 300.0),
                    vec2(500.0, 600.0)
                ]
            );
            assert_eq!(
                &path.verbs,
                &[
                    PathVerb::Begin,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::LineTo,
                    PathVerb::End,
                ]
            );
        }
    }
}
