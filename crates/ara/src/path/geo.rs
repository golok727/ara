use std::ops::Range;

use ara_math::Rect;

use crate::paint::{ CubicBezier, QuadraticBezier };

use super::{ Contour, PathEvent, Point };

pub struct PathGeometryBuilder<'a, PathIter> where PathIter: Iterator<Item = PathEvent> {
    output: &'a mut Vec<Point>,
    offset: usize,
    num_segments: u32,
    path_iter: PathIter,
}

// fn calc_cubic_segments(from: Point, ctrl1: Point, ctrl2: Point, to: Point, pixels_per_unit: f32) -> u32 {
//     let chord = (to - from).magnitude();
//     let control_polygon =
//         (ctrl1 - from).magnitude() + (ctrl2 - ctrl1).magnitude() + (to - ctrl2).magnitude();

//     let flatness = control_polygon / chord;
//     let pixel_tolerance = 0.5; // Target error in pixels
//     let world_tolerance = pixel_tolerance / pixels_per_unit;
//     let segments = ((flatness * chord) / world_tolerance).ceil() as u32;
//     segments.clamp(Self::MIN_SEGMENTS, Self::MAX_SEGMENTS)
// }

impl<'a, PathIter> PathGeometryBuilder<'a, PathIter> where PathIter: Iterator<Item = PathEvent> {
    const MIN_SEGMENTS: u32 = 4;
    const MAX_SEGMENTS: u32 = 64;
    const TOLERANCE: f32 = 7.0; // Lower = more quality

    pub fn new(path_iter: impl Into<PathIter>, output: &'a mut Vec<Point>) -> Self {
        let offset = output.len();

        Self {
            output,
            offset,
            // auto calculate by default
            num_segments: 0,
            path_iter: path_iter.into(),
        }
    }

    // Set the number of segments to use for cubic and quadratic bezier curves.
    // If set to 0, the number of segments will be calculated automatically.
    pub fn with_segments(mut self, num_segments: u32) -> Self {
        self.num_segments = num_segments;
        self
    }

    pub fn with_auto_segments(mut self) -> Self {
        self.num_segments = 0;
        self
    }

    // todo adaptive
    fn calc_cubic_segments(from: Point, ctrl1: Point, ctrl2: Point, to: Point) -> u32 {
        let chord = (to - from).magnitude();
        let control_polygon =
            (ctrl1 - from).magnitude() + (ctrl2 - ctrl1).magnitude() + (to - ctrl2).magnitude();

        // More segments when control points are far from the chord
        let flatness = control_polygon / chord;
        let segments = ((flatness * chord) / Self::TOLERANCE).ceil() as u32;
        segments.clamp(Self::MIN_SEGMENTS, Self::MAX_SEGMENTS)
    }

    fn calc_quadratic_segments(from: Point, ctrl: Point, to: Point) -> u32 {
        let chord = (to - from).magnitude();
        let control_polygon = (ctrl - from).magnitude() + (to - ctrl).magnitude();

        let flatness = control_polygon / chord;
        let segments = ((flatness * chord) / Self::TOLERANCE).ceil() as u32;
        segments.clamp(Self::MIN_SEGMENTS, Self::MAX_SEGMENTS)
    }

    fn push_point(&mut self, point: Point) {
        // Only push if the point is different from the last point
        if let Some(last) = self.output.last() {
            // Use small epsilon for float comparison
            const EPSILON: f32 = 1e-6;
            if (point.x - last.x).abs() > EPSILON || (point.y - last.y).abs() > EPSILON {
                self.output.push(point);
            }
        } else {
            self.output.push(point);
        }
    }

    fn build_geometry_till_end(&mut self, start: Point) -> Contour {
        self.push_point(start);

        loop {
            match self.path_iter.next() {
                Some(PathEvent::Begin { .. }) => unreachable!("invalid geometry"),
                Some(PathEvent::Cubic { from, ctrl1, ctrl2, to }) => {
                    let bezier = CubicBezier { from, ctrl1, ctrl2, to };
                    let num_segments = if self.num_segments == 0 {
                        Self::calc_cubic_segments(from, ctrl1, ctrl2, to)
                    } else {
                        self.num_segments
                    };

                    let t_step = 1.0 / (num_segments as f32);
                    self.output.reserve(num_segments as usize);

                    for i in 1..=num_segments {
                        self.push_point(bezier.sample(t_step * (i as f32)));
                    }
                }
                Some(PathEvent::Quadratic { from, ctrl, to }) => {
                    let bezier = QuadraticBezier { from, ctrl, to };
                    let num_segments = if self.num_segments == 0 {
                        Self::calc_quadratic_segments(from, ctrl, to)
                    } else {
                        self.num_segments
                    };

                    let t_step = 1.0 / (num_segments as f32);
                    self.output.reserve(num_segments as usize);

                    for i in 1..=num_segments {
                        self.push_point(bezier.sample(t_step * (i as f32)));
                    }
                }
                Some(PathEvent::Line { to, .. }) => self.push_point(to),
                Some(PathEvent::End { close, first, contour, .. }) => {
                    if close {
                        // Only add closing point if it's different from the last point
                        if let Some(last) = self.output.last() {
                            const EPSILON: f32 = 1e-6;
                            if
                                (first.x - last.x).abs() > EPSILON ||
                                (first.y - last.y).abs() > EPSILON
                            {
                                self.push_point(first);
                            }
                        }
                    }
                    return contour;
                }
                None => {
                    return Contour::INVALID;
                }
            }
        }
    }
}

impl<'a, PathIter> Iterator
    for PathGeometryBuilder<'a, PathIter>
    where PathIter: Iterator<Item = PathEvent>
{
    type Item = (Contour, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.path_iter.next() {
            Some(PathEvent::Begin { at }) => {
                let start = self.offset;
                let contour = self.build_geometry_till_end(at);
                let end = self.output.len();
                self.offset = end;
                Some((contour, start..end))
            }

            None => None,
            _ => {
                // this should not happen
                unreachable!("invalid path path operation")
            }
        }
    }
}

pub fn get_path_bounds(path: &[Point]) -> Rect<f32> {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;

    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in path {
        let x = point.x;
        let y = point.y;
        min_x = if x < min_x { x } else { min_x };
        max_x = if x > max_x { x } else { max_x };

        min_y = if y < min_y { y } else { min_y };
        max_y = if y > max_y { y } else { max_y };
    }

    Rect::from_corners((min_x, min_y).into(), (max_x, max_y).into())
}

#[cfg(test)]
mod tests {
    use crate::path::{ PathBuilder, PathEventsIter, Point };
    use ara_math::{ vec2, Corners, Rect, Vec2 };

    use super::PathGeometryBuilder;

    #[test]
    fn path_geometry_build_basic() {
        let mut output = <Vec<Point>>::new();

        let mut path = PathBuilder::default();
        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(0.0, 10.0));
        path.line_to(vec2(0.0, 20.0));
        path.line_to(vec2(0.0, 30.0));
        path.end(false);

        path.begin(vec2(100.0, 100.0));
        path.line_to(vec2(200.0, 300.0));
        path.close();

        let geo_build = <PathGeometryBuilder<PathEventsIter>>::new(path.path_events(), &mut output);

        let contours = geo_build.map(|v| v.1).collect::<Vec<_>>();

        assert_eq!(output.len(), 7);
        assert_eq!(contours.len(), 2);

        {
            let start = contours[0].start;
            let end = contours[0].end;
            let points = &output[start..end];

            assert_eq!(
                points,
                &[vec2(0.0, 0.0), vec2(0.0, 10.0), vec2(0.0, 20.0), vec2(0.0, 30.0)]
            );
        }

        {
            let start = contours[1].start;
            let end = contours[1].end;
            let points = &output[start..end];
            assert_eq!(points, &[vec2(100.0, 100.0), vec2(200.0, 300.0), vec2(100.0, 100.0)]);
        }
    }

    #[test]
    fn path_geometry_contours() {
        let mut output = <Vec<Point>>::new();
        let mut path = PathBuilder::default();
        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(0.0, 10.0));
        path.line_to(vec2(0.0, 20.0));
        path.line_to(vec2(0.0, 30.0));
        path.end(false);

        path.begin(vec2(100.0, 100.0));
        path.line_to(vec2(200.0, 300.0));
        path.close();

        path.circle(vec2(0.0, 0.0), 5.0);
        path.rect(&Rect::xywh(10.0, 10.0, 100.0, 100.0));
        path.round_rect(&Rect::xywh(100.0, 100.0, 100.0, 100.0), &Corners::with_all(20.0));

        let geo_build = <PathGeometryBuilder<PathEventsIter>>::new(path.path_events(), &mut output);

        let contours = geo_build.collect::<Vec<_>>();
        assert_eq!(contours.len(), 5);
    }

    #[test]
    fn path_geometry_quadratic_bezier() {
        let mut output = <Vec<Point>>::new();

        let mut path = PathBuilder::default();
        path.begin(vec2(0.0, 0.0));
        path.quadratic_to(vec2(5.0, 5.0), vec2(10.0, 0.0));
        path.end(false);

        let mut geo_build = <PathGeometryBuilder<PathEventsIter>>
            ::new(path.path_events(), &mut output)
            .with_segments(16)
            .map(|v| v.1);
        let range = geo_build.next().expect("no contours found");
        assert!(geo_build.next().is_none());

        let points = &output[range];
        assert_eq!(
            points,
            &[
                vec2(0.0, 0.0),
                vec2(0.625, 0.5859375),
                vec2(1.25, 1.09375),
                vec2(1.875, 1.5234375),
                vec2(2.5, 1.875),
                vec2(3.125, 2.1484375),
                vec2(3.75, 2.34375),
                vec2(4.375, 2.4609375),
                vec2(5.0, 2.5),
                vec2(5.625, 2.4609375),
                vec2(6.25, 2.34375),
                vec2(6.875, 2.1484375),
                vec2(7.5, 1.875),
                vec2(8.125, 1.5234375),
                vec2(8.75, 1.09375),
                vec2(9.375, 0.5859375),
                vec2(10.0, 0.0),
            ]
        );
    }

    #[test]
    fn path_geometry_cubic_bezier() {
        let mut output = <Vec<Point>>::new();

        let mut path = PathBuilder::default();
        path.begin(vec2(0.0, 0.0));
        path.cubic_to(vec2(0.0, 4.0), vec2(6.0, 0.0), vec2(6.0, 8.0));
        path.end(false);

        let mut geo_build = <PathGeometryBuilder<PathEventsIter>>
            ::new(path.path_events(), &mut output)
            .with_segments(16)
            .map(|v| v.1);

        let range = geo_build.next().expect("no contours found");
        let points = &output[range];

        let expected_points = [
            vec2(0.0, 0.0),
            vec2(0.06738281, 0.6611328),
            vec2(0.2578125, 1.1640625),
            vec2(0.55371094, 1.5380859),
            vec2(0.9375, 1.8125),
            vec2(1.3916016, 2.0166016),
            vec2(1.8984375, 2.1796875),
            vec2(2.4404297, 2.3310547),
            vec2(3.0, 2.5),
            vec2(3.5595703, 2.7158203),
            vec2(4.1015625, 3.0078125),
            vec2(4.6083984, 3.4052734),
            vec2(5.0625, 3.9375),
            vec2(5.446289, 4.633789),
            vec2(5.7421875, 5.5234375),
            vec2(5.932617, 6.635742),
            vec2(6.0, 8.0),
        ];

        assert_eq!(points, &expected_points);
    }

    #[test]
    fn path_geometry_circle() {
        let mut output = <Vec<Point>>::new();

        let mut path = PathBuilder::default();
        path.circle(vec2(0.0, 0.0), 5.0);

        let mut geo_build = <PathGeometryBuilder<PathEventsIter>>
            ::new(path.path_events(), &mut output)
            .with_segments(16)
            .map(|v| v.1);

        let range = geo_build.next().expect("no contours found");
        let points = &output[range];

        assert_eq!(
            points,
            &[
                vec2(-5.0, 0.0),
                vec2(-4.9741654, -0.51091635),
                vec2(-4.898342, -1.0071437),
                vec2(-4.7750516, -1.4861608),
                vec2(-4.6068153, -1.9454459),
                vec2(-4.396155, -2.3824778),
                vec2(-4.1455913, -2.7947354),
                vec2(-3.857646, -3.1796968),
                vec2(-3.5348408, -3.5348408),
                vec2(-3.1796968, -3.857646),
                vec2(-2.7947354, -4.1455913),
                vec2(-2.3824778, -4.396155),
                vec2(-1.9454459, -4.6068153),
                vec2(-1.4861608, -4.7750516),
                vec2(-1.0071437, -4.898342),
                vec2(-0.51091635, -4.9741654),
                vec2(0.0, -5.0),
                vec2(0.51091635, -4.9741654),
                vec2(1.0071437, -4.898342),
                vec2(1.4861608, -4.7750516),
                vec2(1.9454459, -4.6068153),
                vec2(2.3824778, -4.396155),
                vec2(2.7947354, -4.1455913),
                vec2(3.1796968, -3.857646),
                vec2(3.5348408, -3.5348408),
                vec2(3.857646, -3.1796968),
                vec2(4.1455913, -2.7947354),
                vec2(4.396155, -2.3824778),
                vec2(4.6068153, -1.9454459),
                vec2(4.7750516, -1.4861608),
                vec2(4.898342, -1.0071437),
                vec2(4.9741654, -0.51091635),
                vec2(5.0, 0.0),
                vec2(4.9741654, 0.51091635),
                vec2(4.898342, 1.0071437),
                vec2(4.7750516, 1.4861608),
                vec2(4.6068153, 1.9454459),
                vec2(4.396155, 2.3824778),
                vec2(4.1455913, 2.7947354),
                vec2(3.857646, 3.1796968),
                vec2(3.5348408, 3.5348408),
                vec2(3.1796968, 3.857646),
                vec2(2.7947354, 4.1455913),
                vec2(2.3824778, 4.396155),
                vec2(1.9454459, 4.6068153),
                vec2(1.4861608, 4.7750516),
                vec2(1.0071437, 4.898342),
                vec2(0.51091635, 4.9741654),
                vec2(0.0, 5.0),
                vec2(-0.51091635, 4.9741654),
                vec2(-1.0071437, 4.898342),
                vec2(-1.4861608, 4.7750516),
                vec2(-1.9454459, 4.6068153),
                vec2(-2.3824778, 4.396155),
                vec2(-2.7947354, 4.1455913),
                vec2(-3.1796968, 3.857646),
                vec2(-3.5348408, 3.5348408),
                vec2(-3.857646, 3.1796968),
                vec2(-4.1455913, 2.7947354),
                vec2(-4.396155, 2.3824778),
                vec2(-4.6068153, 1.9454459),
                vec2(-4.7750516, 1.4861608),
                vec2(-4.898342, 1.0071437),
                vec2(-4.9741654, 0.51091635),
                vec2(-5.0, 0.0),
            ]
        );
    }
    #[test]
    fn path_geometry_round_rect() {
        let mut output = <Vec<Point>>::new();

        let mut path = PathBuilder::default();

        path.round_rect(&Rect::xywh(10.0, 10.0, 100.0, 100.0), &Corners::with_all(20.0));

        let mut geo_build = <PathGeometryBuilder<PathEventsIter>>
            ::new(path.path_events(), &mut output)
            .with_segments(16)
            .map(|v| v.1);

        let range = geo_build.next().expect("no contours found");
        let points = &output[range];

        assert_eq!(
            &points,
            &[
                Vec2 { x: 10.0, y: 30.0 },
                Vec2 { x: 10.103339, y: 27.956335 },
                Vec2 { x: 10.406632, y: 25.971424 },
                Vec2 { x: 10.899794, y: 24.055357 },
                Vec2 { x: 11.572739, y: 22.218216 },
                Vec2 { x: 12.4153805, y: 20.470089 },
                Vec2 { x: 13.417635, y: 18.821058 },
                Vec2 { x: 14.569416, y: 17.281212 },
                Vec2 { x: 15.860637, y: 15.860637 },
                Vec2 { x: 17.281212, y: 14.569416 },
                Vec2 { x: 18.821058, y: 13.417635 },
                Vec2 { x: 20.470089, y: 12.4153805 },
                Vec2 { x: 22.218216, y: 11.572739 },
                Vec2 { x: 24.055357, y: 10.899794 },
                Vec2 { x: 25.971424, y: 10.406632 },
                Vec2 { x: 27.956335, y: 10.103339 },
                Vec2 { x: 30.0, y: 10.0 },
                Vec2 { x: 90.0, y: 10.0 },
                Vec2 { x: 92.04366, y: 10.103339 },
                Vec2 { x: 94.02857, y: 10.406632 },
                Vec2 { x: 95.94464, y: 10.899794 },
                Vec2 { x: 97.781784, y: 11.572739 },
                Vec2 { x: 99.52991, y: 12.4153805 },
                Vec2 { x: 101.17894, y: 13.417635 },
                Vec2 { x: 102.71878, y: 14.569416 },
                Vec2 { x: 104.13936, y: 15.860637 },
                Vec2 { x: 105.43059, y: 17.281212 },
                Vec2 { x: 106.58237, y: 18.821058 },
                Vec2 { x: 107.58462, y: 20.470089 },
                Vec2 { x: 108.42726, y: 22.218216 },
                Vec2 { x: 109.100204, y: 24.055357 },
                Vec2 { x: 109.59337, y: 25.971424 },
                Vec2 { x: 109.89666, y: 27.956335 },
                Vec2 { x: 110.0, y: 30.0 },
                Vec2 { x: 110.0, y: 90.0 },
                Vec2 { x: 109.89666, y: 92.04366 },
                Vec2 { x: 109.59337, y: 94.02857 },
                Vec2 { x: 109.100204, y: 95.94464 },
                Vec2 { x: 108.42726, y: 97.781784 },
                Vec2 { x: 107.58462, y: 99.52991 },
                Vec2 { x: 106.58237, y: 101.17894 },
                Vec2 { x: 105.43059, y: 102.71878 },
                Vec2 { x: 104.13936, y: 104.13936 },
                Vec2 { x: 102.71878, y: 105.43059 },
                Vec2 { x: 101.17894, y: 106.58237 },
                Vec2 { x: 99.52991, y: 107.58462 },
                Vec2 { x: 97.781784, y: 108.42726 },
                Vec2 { x: 95.94464, y: 109.100204 },
                Vec2 { x: 94.02857, y: 109.59337 },
                Vec2 { x: 92.04366, y: 109.89666 },
                Vec2 { x: 90.0, y: 110.0 },
                Vec2 { x: 30.0, y: 110.0 },
                Vec2 { x: 27.956335, y: 109.89666 },
                Vec2 { x: 25.971424, y: 109.59337 },
                Vec2 { x: 24.055357, y: 109.100204 },
                Vec2 { x: 22.218216, y: 108.42726 },
                Vec2 { x: 20.470089, y: 107.58462 },
                Vec2 { x: 18.821058, y: 106.58237 },
                Vec2 { x: 17.281212, y: 105.43059 },
                Vec2 { x: 15.860637, y: 104.13936 },
                Vec2 { x: 14.569416, y: 102.71878 },
                Vec2 { x: 13.417635, y: 101.17894 },
                Vec2 { x: 12.4153805, y: 99.52991 },
                Vec2 { x: 11.572739, y: 97.781784 },
                Vec2 { x: 10.899794, y: 95.94464 },
                Vec2 { x: 10.406632, y: 94.02857 },
                Vec2 { x: 10.103339, y: 92.04366 },
                Vec2 { x: 10.0, y: 90.0 },
                Vec2 { x: 10.0, y: 30.0 },
            ]
        );
    }
}
