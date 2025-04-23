pub mod geometry;
pub mod mat3;
pub mod rect;
pub mod size;
pub mod traits;
pub mod vec2;

pub use geometry::*;
pub use mat3::*;
pub use rect::*;
pub use size::*;
pub use traits::*;
pub use vec2::*;

#[cfg(test)]
mod tests {
    use super::*;

    mod mat3 {
        use super::*;

        #[test]
        fn should_multiply() {
            let m1 = mat3();
            let m2 = mat3();

            let c = m1 * m2;
            assert_eq!(c, mat3());
        }

        #[test]
        fn compose_matrices() {
            let scale = Mat3::from_scale(10.0, 10.0);
            let translate = Mat3::from_translation(100.0, 100.0);

            let res = scale * translate * vec2(10.0, 10.0);

            assert_eq!(res, vec2(200.0, 200.0));
        }

        #[test]
        fn matrix_transform() {
            let mut transform = mat3();
            transform
                .scale(10.0, 10.0)
                .translate(100.0, 100.0)
                .scale(10.0, 10.0);

            let res = transform * vec2(10.0, 10.0);

            assert_eq!(res, vec2(2000.0, 2000.0));
        }

        #[test]
        fn orthographic_projection() {
            let m = Mat3::ortho(0.0, 0.0, 100.0, 100.0);

            assert_eq!(m * vec2(50.0, 50.0), vec2(0.0, 0.0)); // center
            assert_eq!(m * vec2(0.0, 0.0), vec2(-1.0, 1.0)); // top left
            assert_eq!(m * vec2(100.0, 0.0), vec2(1.0, 1.0)); // top right
            assert_eq!(m * vec2(0.0, 100.0), vec2(-1.0, -1.0)); // bottom left
            assert_eq!(m * vec2(100.0, 100.0), vec2(1.0, -1.0)); // bottom right
        }

        #[test]
        fn triangle_proj_test() {
            let width: u32 = 1875;
            let height: u32 = 1023;

            let aspect: f32 = width as f32 / height as f32;
            let proj = Mat3::ortho(1.0, aspect, -1.0, -aspect);

            let positions = [vec2(-0.5, -0.5), vec2(0.0, 0.5), vec2(0.5, -0.5)];
            let transformed = positions.map(|v| proj * v);

            assert_eq!(
                [vec2(0.2728, -0.5), vec2(0.0, 0.5), vec2(-0.2728, -0.5)],
                transformed
            );
        }

        #[test]
        fn is_identity() {
            assert!(mat3().is_identity())
        }
    }
    mod vec2 {
        use crate::traits::{One, Zero};

        use super::*;

        #[test]
        fn zero_and_one() {
            assert_eq!(Vec2::<f64>::zero(), vec2(0.0, 0.0));
            assert_eq!(Vec2::<f64>::one(), vec2(1.0, 1.0));
        }
        #[test]
        fn vec_add() {
            let a = vec2(10.0, 10.0);
            let b = vec2(10.0, 10.0);

            assert_eq!(a + b, vec2(20.0, 20.0));
        }

        #[test]
        fn vec_add_assign() {
            let mut a = vec2(10.0, 10.0);
            a += vec2(10.0, 10.0);

            assert_eq!(a, vec2(20.0, 20.0));
        }

        #[test]
        fn vec_sub() {
            let a = vec2(10.0, 10.0);
            let b = vec2(10.0, 10.0);

            assert_eq!(a - b, vec2(0.0, 0.0));
        }

        #[test]
        fn vec_sub_assign() {
            let mut a = vec2(10.0, 10.0);
            a -= vec2(10.0, 10.0);

            assert_eq!(a, vec2(0.0, 0.0));
        }

        #[test]
        fn vec_mul() {
            let a = vec2(10.0, 10.0);
            let b = vec2(10.0, 10.0);

            assert_eq!(a * b, vec2(100.0, 100.0));
        }

        #[test]
        fn vec_mul_assign() {
            let mut a = vec2(10.0, 10.0);
            a *= vec2(10.0, 10.0);

            assert_eq!(a, vec2(100.0, 100.0));
        }

        #[test]
        fn vec_div() {
            let a = vec2(10.0, 10.0);
            let b = vec2(10.0, 10.0);

            assert_eq!(a / b, vec2(1.0, 1.0));
        }

        #[test]
        fn vec_div_assign() {
            let mut a = vec2(10.0, 10.0);
            a /= vec2(10.0, 10.0);

            assert_eq!(a, vec2(1.0, 1.0));
        }

        #[test]
        fn should_transform_with_matrix() {
            let mut m = mat3();
            m.translate(10.0, 100.0);
            m.translate(20.0, 100.0);

            let a = vec2(10.0, 0.0);
            assert_eq!(m * a, vec2(40.0, 200.0));
        }

        #[test]
        fn should_scale_with_matrix() {
            let mut m = mat3();
            m.scale(2.0, 2.0).scale(2.0, 2.0);

            let a = vec2(10.0, 50.0);

            assert_eq!(m * a, vec2(40.0, 200.0));
        }
    }
}
