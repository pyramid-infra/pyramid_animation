
use cgmath::*;

pub trait Interpolateable {
    fn interpolate(a: &Self, b: &Self, p: &f32) -> Self;
}

impl Interpolateable for f32 {
    fn interpolate(a: &f32, b: &f32, p: &f32) -> f32 {
        a * (1.0 - p) + b * p
    }
}
impl Interpolateable for Vector2<f32> {
    fn interpolate(a: &Vector2<f32>, b: &Vector2<f32>, p: &f32) -> Vector2<f32> {
        a.mul_s(1.0 - p) + b.mul_s(*p)
    }
}
impl Interpolateable for Vector3<f32> {
    fn interpolate(a: &Vector3<f32>, b: &Vector3<f32>, p: &f32) -> Vector3<f32> {
        a.mul_s(1.0 - p) + b.mul_s(*p)
    }
}
impl Interpolateable for Vector4<f32> {
    fn interpolate(a: &Vector4<f32>, b: &Vector4<f32>, p: &f32) -> Vector4<f32> {
        a.mul_s(1.0 - p) + b.mul_s(*p)
    }
}
