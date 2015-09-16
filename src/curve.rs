#[cfg(test)]
use cgmath::*;
use std::fmt::Debug;
use animatable::*;

pub trait Curve<T> : Debug {
    fn value(&self, time: f32) -> T;
}


#[derive(PartialEq, Debug)]
pub struct FixedValueCurve<T> {
    pub value: T
}
impl<T: Debug + Clone> Curve<T> for FixedValueCurve<T> {
    fn value(&self, _: f32) -> T {
        self.value.clone()
    }
}

#[derive(PartialEq, Debug)]
pub struct Key<T: Clone>(pub f32, pub T);

#[derive(PartialEq, Debug)]
pub struct LinearKeyFrameCurve<T: Clone> {
    pub keys: Vec<Key<T>>
}

impl<T: Interpolateable + Debug + Clone> LinearKeyFrameCurve<T> {
    pub fn to_discreet(&self, n_keys: usize, duration: f32) -> DiscreetKeyFrameCurve<T> {
        let mut keys = vec![];
        for i in 0..n_keys {
            let p = duration * (i as f32 / n_keys as f32);
            keys.push(Key(p, self.value(p)));
        }
        DiscreetKeyFrameCurve { keys: keys }
    }
}

impl<T: Interpolateable + Debug + Clone> Curve<T> for LinearKeyFrameCurve<T> {
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

#[derive(PartialEq, Debug)]
pub struct DiscreetKeyFrameCurve<T: Clone> {
    pub keys: Vec<Key<T>>
}

impl<T: Interpolateable + Debug + Clone> Curve<T> for DiscreetKeyFrameCurve<T> {
    fn value(&self, time: f32) -> T {
        let i = (time * self.keys.len() as f32) as usize;
        if i < 0 {
            return self.keys[0].1.clone()
        } else if (i >= self.keys.len()) {
            return self.keys[self.keys.len() - 1].1.clone()
        } else {
            return self.keys[i as usize].1.clone();
        }
    }
}

#[test]
fn test_key_frame_single() {
    let kf = LinearKeyFrameCurve {
        keys: vec![Key(0.0, 0.0), Key(1.0, 1.0)]
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
        keys: vec![Key(0.0, Vector2::new(0.0, 0.0)), Key(1.0, Vector2::new(1.0, 1.0))]
    };
    assert_eq!(kf.value(-0.1), Vector2::new(0.0, 0.0));
    assert_eq!(kf.value(0.0), Vector2::new(0.0, 0.0));
    assert_eq!(kf.value(0.5), Vector2::new(0.5, 0.5));
    assert_eq!(kf.value(1.0), Vector2::new(1.0, 1.0));
    assert_eq!(kf.value(1.1), Vector2::new(1.0, 1.0));
}

#[test]
fn test_key_frame_multi_keys() {
    let kf = LinearKeyFrameCurve {
        keys: vec![Key(0.0, 0.0), Key(10.0, 1.0), Key(20.0, 0.5), Key(21.0, 1.0), Key(22.0, 5.0)]
    };
    assert_eq!(kf.value(-0.1), 0.0);
    assert_eq!(kf.value(0.0), 0.0);
    assert_eq!(kf.value(0.5), 0.05);
    assert_eq!(kf.value(20.5), 0.75);
    assert_eq!(kf.value(21.5), 3.0);
    assert_eq!(kf.value(30.0), 5.0);
}
