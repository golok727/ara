use crate::path::Point;

pub struct QuadraticBezier {
    pub from: Point,
    pub to: Point,
    pub ctrl: Point,
}

impl QuadraticBezier {
    pub fn sample(&self, t: f32) -> Point {
        let one_minus_t = 1.0 - t;
        let p0 = self.from * (one_minus_t * one_minus_t);
        let p1 = self.ctrl * (2.0 * one_minus_t * t);
        let p2 = self.to * (t * t);

        p0 + p1 + p2
    }
}

pub struct CubicBezier {
    pub from: Point,
    pub to: Point,
    pub ctrl1: Point,
    pub ctrl2: Point,
}

impl CubicBezier {
    pub fn sample(&self, t: f32) -> Point {
        let one_minus_t = 1.0 - t;
        let t_squared = t * t;
        let one_minus_t_squared = one_minus_t * one_minus_t;
        let one_minus_t_cubed = one_minus_t_squared * one_minus_t;
        let t_cubed = t_squared * t;

        let p0 = self.from * one_minus_t_cubed;
        let p1 = self.ctrl1 * 3.0 * one_minus_t_squared * t;
        let p2 = self.ctrl2 * 3.0 * one_minus_t * t_squared;
        let p3 = self.to * t_cubed;

        p0 + p1 + p2 + p3
    }
}
