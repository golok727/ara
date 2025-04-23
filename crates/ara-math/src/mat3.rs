use super::Vec2;
use std::ops::Mul;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    data: [f32; 9],
}

impl Mat3 {
    pub const IDENTITY: Self = Self::identity();
    /**
    constructs a identity matrix
    */
    #[inline]
    pub const fn new() -> Self {
        Self {
            #[rustfmt::skip]
            data: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0
            ],
        }
    }

    #[inline]
    pub const fn identity() -> Self {
        Self {
            #[rustfmt::skip]
            data: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0
            ],
        }
    }

    #[inline]
    pub fn from_scale(sx: f32, sy: f32) -> Self {
        Self {
            #[rustfmt::skip]
            data: [
                sx, 0.0, 0.0,
                0.0, sy, 0.0,
                0.0, 0.0, 1.0
            ],
        }
    }

    #[inline]
    pub fn from_rotation(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            #[rustfmt::skip]
            data: [
                    cos, -sin, 0.0,
                    sin, cos, 0.0,
                    0.0, 0.0, 1.0
            ],
        }
    }

    #[inline]
    pub fn from_translation(dx: f32, dy: f32) -> Self {
        Self {
            #[rustfmt::skip]
            data: [
                1.0, 0.0, dx,
                0.0, 1.0, dy,
                0.0, 0.0, 1.0
            ],
        }
    }

    #[inline]
    pub fn translate(&mut self, dx: f32, dy: f32) -> &mut Self {
        *self = Self::from_translation(dx, dy) * *self;
        self
    }

    #[inline]
    pub fn translate_x(&mut self, dx: f32) -> &mut Self {
        self.translate(dx, 0.)
    }

    #[inline]
    pub fn translate_y(&mut self, dy: f32) -> &mut Self {
        self.translate(0., dy)
    }

    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        *self = Self::from_rotation(angle) * *self;
        self
    }

    #[inline]
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        *self = Self::from_scale(sx, sy) * *self;

        self
    }

    #[inline]
    pub fn scale_x(&mut self, sx: f32) -> &mut Self {
        self.scale(sx, 1.)
    }

    #[inline]
    pub fn scale_y(&mut self, sy: f32) -> &mut Self {
        self.scale(1., sy)
    }

    pub fn transpose(&mut self) -> &mut Self {
        self.data.swap(1, 3);
        self.data.swap(2, 6);
        self.data.swap(5, 7);
        self
    }

    pub fn inverse(&self) -> Self {
        let m = &self.data;

        let det = self.det();

        if det == 0.0 {
            return *self;
        }

        let inv_det = 1.0 / det;

        Self {
            data: [
                (m[4] * m[8] - m[5] * m[7]) * inv_det,
                (m[2] * m[7] - m[1] * m[8]) * inv_det,
                (m[1] * m[5] - m[2] * m[4]) * inv_det,
                (m[5] * m[6] - m[3] * m[8]) * inv_det,
                (m[0] * m[8] - m[2] * m[6]) * inv_det,
                (m[2] * m[3] - m[0] * m[5]) * inv_det,
                (m[3] * m[7] - m[4] * m[6]) * inv_det,
                (m[1] * m[6] - m[0] * m[7]) * inv_det,
                (m[0] * m[4] - m[1] * m[3]) * inv_det,
            ],
        }
    }

    pub fn det(&self) -> f32 {
        let m = &self.data;

        // Determinant formula for a 3x3 matrix:
        // | a b c |
        // | d e f |
        // | g h i |
        // det = a(ei - fh) - b(di - fg) + c(dh - eg)

        m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6])
    }

    pub fn is_identity(&self) -> bool {
        self == &Self::IDENTITY
    }

    /// Constructs an orthographic projection matrix
    pub fn ortho(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        let scale_x = 2.0 / (right - left);
        let scale_y = 2.0 / (top - bottom);
        let translate_x = -(right + left) / (right - left);
        let translate_y = -(top + bottom) / (top - bottom);

        Self {
            #[rustfmt::skip]
            data: [
                scale_x, 0.0, translate_x,
                0.0, scale_y, translate_y,
                0.0, 0.0, 1.0,
            ],
        }
    }
}

impl From<Mat3> for [[f32; 4]; 4] {
    #[rustfmt::skip]
    fn from(mat: Mat3) -> Self {
        let m = mat.data;
        [
            [m[0], m[1], m[2], 0.0], // Row 0
            [m[3], m[4], m[5], 0.0], // Row 1
            [m[6], m[7], m[8], 0.0], // Row 2
            [0.0, 0.0, 0.0, 1.0],    // Row 3
        ]
    }
}

#[inline]
pub fn mat3() -> Mat3 {
    Mat3::new()
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mul for Mat3 {
    type Output = Self;

    /*
    a b tx   m11 m12 m13
    c d ty   m21 m22 m23
    0 0 1    m31 m32 m33
    */
    fn mul(self, rhs: Self) -> Self::Output {
        let a = &rhs.data;
        let b = &self.data;

        let m11 = a[0] * b[0] + a[1] * b[3] + a[2] * b[6];
        let m12 = a[0] * b[1] + a[1] * b[4] + a[2] * b[7];
        let m13 = a[0] * b[2] + a[1] * b[5] + a[2] * b[8];

        let m21 = a[3] * b[0] + a[4] * b[3] + a[5] * b[6];
        let m22 = a[3] * b[1] + a[4] * b[4] + a[5] * b[7];
        let m23 = a[3] * b[2] + a[4] * b[5] + a[5] * b[8];

        let m31 = a[6] * b[0] + a[7] * b[3] + a[8] * b[6];
        let m32 = a[6] * b[1] + a[7] * b[4] + a[8] * b[7];
        let m33 = a[6] * b[2] + a[7] * b[5] + a[8] * b[8];

        Mat3 {
            data: [m11, m12, m13, m21, m22, m23, m31, m32, m33],
        }
    }
}

// :) we just need a vec2 for or needs!
impl Mul<Vec2<f32>> for Mat3 {
    type Output = Vec2<f32>;

    fn mul(self, v: Vec2<f32>) -> Self::Output {
        let m = &self.data;
        let x = m[0] * v.x + m[1] * v.y + m[2] * 1.0;
        let y = m[3] * v.x + m[4] * v.y + m[5] * 1.0;

        Self::Output { x, y }
    }
}
