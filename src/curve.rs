use cgmath::*;

#[derive(PartialEq, Debug)]
pub struct LinearKeyFrameCurve<T> {
    pub keys: Vec<(f32, T)>
}

pub trait Curve<T> {
    fn value(&self, time: f32) -> T;
}

pub trait Interpolateable {
    fn interpolate(a: &Self, b: &Self, p: &f32) -> Self;
}

impl Interpolateable for f32 {
    fn interpolate(a: &f32, b: &f32, p: &f32) -> f32 {
        a * (1.0 - p) + b * p
    }
}
impl Interpolateable for Vector3<f32> {
    fn interpolate(a: &Vector3<f32>, b: &Vector3<f32>, p: &f32) -> Vector3<f32> {
        a.mul_s(1.0 - p) + b.mul_s(*p)
    }
}
impl Interpolateable for Vector2<f32> {
    fn interpolate(a: &Vector2<f32>, b: &Vector2<f32>, p: &f32) -> Vector2<f32> {
        a.mul_s(1.0 - p) + b.mul_s(*p)
    }
}

impl<T: Interpolateable> Curve<T> for LinearKeyFrameCurve<T> {
    fn value(&self, time: f32) -> T {
        let mut key_before = None;
        let mut key_after = None;
        for i in 0..self.keys.len() {
            if self.keys[i].0 > time { break; }
            else { key_before = Some(&self.keys[i]); }
        }
        for i in 0..self.keys.len() {
            if self.keys[i].0 > time {
                key_after = Some(&self.keys[i]);
                break;
            }
        };
        let key_before = match key_before {
            Some(k) => k,
            None => return Interpolateable::interpolate(&self.keys[0].1, &self.keys[0].1, &0.0)
        };
        let key_after = match key_after {
            Some(k) => k,
            None => {
                let k = &self.keys[self.keys.len() - 1].1;
                return Interpolateable::interpolate(k, k, &0.0)
            }
        };
        let d = key_after.0 - key_before.0;
        let p = (time - key_before.0) / d;
        return Interpolateable::interpolate(&key_before.1, &key_after.1, &p);
    }
}

#[test]
fn test_key_frame_single() {
    let kf = LinearKeyFrameCurve {
        keys: vec![(0.0, 0.0), (1.0, 1.0)]
    };
    assert_eq!(kf.value(-0.1), 0.0);
    assert_eq!(kf.value(0.0), 0.0);
    assert_eq!(kf.value(0.5), 0.5);
    assert_eq!(kf.value(1.0), 1.0);
    assert_eq!(kf.value(1.1), 1.0);
}

#[test]
fn test_key_frame_vector() {
    let kf = LinearKeyFrameCurve {
        keys: vec![(0.0, Vector2::new(0.0, 0.0)), (1.0, Vector2::new(1.0, 1.0))]
    };
    assert_eq!(kf.value(-0.1), Vector2::new(0.0, 0.0));
    assert_eq!(kf.value(0.0), Vector2::new(0.0, 0.0));
    assert_eq!(kf.value(0.5), Vector2::new(0.5, 0.5));
    assert_eq!(kf.value(1.0), Vector2::new(1.0, 1.0));
    assert_eq!(kf.value(1.1), Vector2::new(1.0, 1.0));
}
