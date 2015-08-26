

#[derive(PartialEq, Debug)]
pub struct LinearKeyFrameCurve {
    pub keys: Vec<(f32, f32)>
}

pub trait Curve {
    fn value(&self, time: f32) -> f32;
}

impl Curve for LinearKeyFrameCurve {
    fn value(&self, time: f32) -> f32 {
        let mut key_before = None;
        let mut key_after = None;
        for i in 0..self.keys.len() {
            if self.keys[i].0 > time { break; }
            else { key_before = Some(self.keys[i]); }
        }
        for i in 0..self.keys.len() {
            if self.keys[i].0 > time {
                key_after = Some(self.keys[i]);
                break;
            }
        };
        let key_before = match key_before {
            Some(k) => k,
            None => return self.keys[0].1
        };
        let key_after = match key_after {
            Some(k) => k,
            None => return self.keys[self.keys.len() - 1].1
        };
        let d = key_after.0 - key_before.0;
        let p = (time - key_before.0) / d;
        return key_before.1 * (1.0 - p) + key_after.1 * p;
    }
}

#[test]
fn test_key_frame() {
    let kf = LinearKeyFrameCurve {
        keys: vec![(0.0, 0.0), (1.0, 1.0)]
    };
    assert_eq!(kf.value(-0.1), 0.0);
    assert_eq!(kf.value(0.0), 0.0);
    assert_eq!(kf.value(0.5), 0.5);
    assert_eq!(kf.value(1.0), 1.0);
    assert_eq!(kf.value(1.1), 1.0);
}
