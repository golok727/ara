mod builder;
pub mod geo;

pub use geo::*;
pub mod polygon;
pub use builder::*;
pub use polygon::*;

pub type Point = ara_math::Vec2<f32>;

use core::f32;

use ara_math::Zero;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub(crate) enum PathVerb {
    Begin,
    LineTo,
    QuadraticTo,
    CubicTo,
    Close,
    End,
}

/// Struct to store built paths
#[derive(Clone, Debug)]
pub struct Path {
    pub(crate) points: Box<[Point]>,
    pub(crate) verbs: Box<[PathVerb]>,
}

impl Path {
    pub fn builder() -> PathBuilder {
        PathBuilder::default()
    }
    pub fn events(&self) -> PathEventsIter {
        PathEventsIter::new(&self.points, &self.verbs)
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = PathEvent;

    type IntoIter = PathEventsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PathEventsIter::new(&self.points, &self.verbs)
    }
}

pub struct PathEventsIter<'a> {
    points: std::slice::Iter<'a, Point>,
    verbs: std::slice::Iter<'a, PathVerb>,
    p_index: usize,
    first: Point,
    current: Point,
}

impl From<PathBuilder> for Path {
    #[inline]
    fn from(value: PathBuilder) -> Self {
        value.build()
    }
}

impl<'a> PathEventsIter<'a> {
    pub(crate) fn new(points: &'a [Point], verbs: &'a [PathVerb]) -> Self {
        Self {
            points: points.iter(),
            verbs: verbs.iter(),
            current: Point::zero(),
            first: Point::zero(),
            p_index: 0,
        }
    }

    pub fn next_point(&mut self) -> Point {
        if let Some(point) = self.points.next().copied() {
            self.p_index += 1;
            point
        } else {
            Point::new(f32::NAN, f32::NAN)
        }
    }
}

impl Iterator for PathEventsIter<'_> {
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        match self.verbs.next() {
            Some(&PathVerb::Begin) => {
                self.current = self.next_point();
                self.first = self.current;

                Some(PathEvent::Begin { at: self.current })
            }
            Some(&PathVerb::LineTo) => {
                let from = self.current;
                self.current = self.next_point();
                Some(PathEvent::Line {
                    from,
                    to: self.current,
                })
            }
            Some(&PathVerb::QuadraticTo) => {
                let from = self.current;
                let ctrl = self.next_point();

                self.current = self.next_point();
                Some(PathEvent::Quadratic {
                    from,
                    ctrl,
                    to: self.current,
                })
            }
            Some(&PathVerb::CubicTo) => {
                let from = self.current;
                let ctrl1 = self.next_point();
                let ctrl2 = self.next_point();

                self.current = self.next_point();

                Some(PathEvent::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to: self.current,
                })
            }
            Some(&PathVerb::Close) => {
                let last = self.current;

                self.current = self.next_point();
                Some(PathEvent::End {
                    contour: Contour(self.p_index),
                    last,
                    first: self.first,
                    close: true,
                })
            }
            Some(&PathVerb::End) => {
                let last = self.current;
                self.current = self.first;
                Some(PathEvent::End {
                    contour: Contour(self.p_index),
                    last,
                    first: self.first,
                    close: false,
                })
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.verbs.len();
        (remaining, Some(remaining))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathEvent {
    Begin {
        at: Point,
    },

    Line {
        from: Point,
        to: Point,
    },

    Quadratic {
        from: Point,
        ctrl: Point,
        to: Point,
    },

    Cubic {
        from: Point,
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },

    End {
        contour: Contour,
        last: Point,
        close: bool,
        first: Point,
    },
}

#[cfg(test)]
mod tests {
    use ara_math::vec2;

    use super::*;

    #[test]
    fn path_contours() {
        let mut path = Path::builder();

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(-20.0, 100.0));
        let leg_l = path.end(false);

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(20.0, 100.0));
        let leg_r = path.end(false);

        let head = path.circle(vec2(0.0, 0.0), 10.0);

        assert_eq!(leg_l, Contour(2));
        assert_eq!(leg_r, Contour(2 + 2));
        assert_eq!(head, Contour(2 + 2 + 14));
    }

    #[test]
    fn path_events_iter_test() {
        // todo add tests for rest of the events
        let mut path = Path::builder();

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(-20.0, 100.0));
        path.end(false);

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(20.0, 100.0));
        path.end(false);

        let mut iter = path.path_events();
        // first
        assert_eq!(iter.next(), Some(PathEvent::Begin { at: vec2(0.0, 0.0) }));
        assert_eq!(
            iter.next(),
            Some(PathEvent::Line {
                from: vec2(0.0, 0.0),
                to: vec2(-20.0, 100.0),
            })
        );
        assert_eq!(
            iter.next(),
            Some(PathEvent::End {
                contour: Contour(2),
                last: vec2(-20.0, 100.0),
                close: false,
                first: vec2(0.0, 0.0),
            })
        );

        // second
        assert_eq!(iter.next(), Some(PathEvent::Begin { at: vec2(0.0, 0.0) }));
        assert_eq!(
            iter.next(),
            Some(PathEvent::Line {
                from: vec2(0.0, 0.0),
                to: vec2(20.0, 100.0),
            })
        );
        assert_eq!(
            iter.next(),
            Some(PathEvent::End {
                contour: Contour(4),
                last: vec2(20.0, 100.0),
                close: false,
                first: vec2(0.0, 0.0),
            })
        );

        assert_eq!(iter.next(), None);
    }
}
