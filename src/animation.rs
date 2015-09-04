
use time::*;

use curve::*;
use pyramid::pon::*;
use cgmath::*;
use std::fmt;

pub trait PonCurve {
    fn value_as_pon(&self, time: f32) -> Pon;
}

impl PonCurve for LinearKeyFrameCurve<f32> {
    fn value_as_pon(&self, time: f32) -> Pon {
        Pon::Float(self.value(time))
    }
}
impl PonCurve for LinearKeyFrameCurve<Vector3<f32>> {
    fn value_as_pon(&self, time: f32) -> Pon {
        Pon::Vector3(self.value(time))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Loop {
    Forever,
    Once
}

pub struct Animation {
    pub curve: Box<PonCurve>,
    pub time: Duration,
    pub property: NamedPropRef,
    pub loop_type: Loop,
    pub duration: Duration
}

impl fmt::Debug for Animation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Animation")
    }
}

impl<'a> Translatable<'a, Loop> for Pon {
    fn inner_translate(&'a self) -> Result<Loop, PonTranslateErr> {
        match try!(self.translate()) {
            "forever" => Ok(Loop::Forever),
            "once" => Ok(Loop::Once),
            _ => Err(PonTranslateErr::InvalidValue { value: format!("{:?}", self) })
        }
    }
}

impl<'a, T> Translatable<'a, Key<T>> for Pon where Pon: Translatable<'a, T> {
    fn inner_translate(&'a self) -> Result<Key<T>, PonTranslateErr> {
        match self {
            &Pon::Object(..) => {
                let time: f32 = try!(self.field_as::<f32>("time"));
                let value = try!(self.field_as("value"));
                Ok(Key(time, value))
            },
            &Pon::Array(ref arr) => {
                let time: f32 = try!(arr[0].translate::<f32>());
                let value = try!(arr[1].translate());
                Ok(Key(time, value))
            },
            _ => {
                Err(PonTranslateErr::MismatchType { expected: "Object or Array".to_string(), found: format!("{:?}", self) })
            }
        }
    }
}

impl<'a> Translatable<'a, Animation> for Pon {
    fn inner_translate(&'a self) -> Result<Animation, PonTranslateErr> {
        let &TypedPon { ref type_name, ref data } = try!(self.translate());
        match type_name.as_str() {
            "key_framed" => {
                let property: &NamedPropRef = try!(try!(data.field("property")).as_reference());
                let duration: f32 = try!(data.field_as_or("duration", 1.0));
                let loop_type = try!(data.field_as_or("loop", Loop::Once));
                let keys_array: &Vec<Pon> = try!(data.field_as("keys"));
                let first_key = &keys_array[0];
                let curve: Box<PonCurve> = {
                    let as_vec3: Result<Key<Vector3<f32>>, PonTranslateErr> = first_key.translate();
                    if let Ok(..) = as_vec3 {
                        let keys: Vec<Key<Vector3<f32>>> = try!(data.field_as("keys"));
                        Box::new(LinearKeyFrameCurve {
                            keys: keys
                        })
                    } else {
                        let as_float: Result<Key<f32>, PonTranslateErr> = first_key.translate();
                        if let Ok(..) = as_float {
                            let keys: Vec<Key<f32>> = try!(data.field_as("keys"));
                            Box::new(LinearKeyFrameCurve {
                                keys: keys
                            })
                        } else {
                            return Err(PonTranslateErr::Generic("Unrecognized keys".to_string()))
                        }
                    }
                };
                return Ok(Animation {
                    curve: curve,
                    time: Duration::zero(),
                    property: property.clone(),
                    loop_type: loop_type,
                    duration: Duration::milliseconds((duration*1000.0) as i64)
                });
            },
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}

impl Animation {

    pub fn update(&mut self, delta_time: Duration) -> Pon {
        {
            self.time = self.time + delta_time;
            if self.loop_type == Loop::Forever && self.time > self.duration {
                self.time = Duration::zero();
            }
        }
        return self.curve.value_as_pon(self.time.num_milliseconds() as f32/self.duration.num_milliseconds() as f32);
    }
}


#[test]
fn test_animation() {
    let mut kf = Animation {
        curve: Box::new(LinearKeyFrameCurve {
            keys: vec![Key(0.0, 0.0), Key(1.0, 1.0)]
        }),
        time: Duration::zero(),
        property: NamedPropRef { entity_name: "this".to_string(), property_key: "x".to_string() },
        loop_type: Loop::Once,
        duration: Duration::seconds(1)
    };
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}

#[test]
fn test_animation_from_pon() {
    let mut kf: Animation = Pon::from_string(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: 0.0 }, { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap().translate().unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}

#[test]
fn test_animation_from_pon_vec3() {
    let mut kf: Animation = Pon::from_string(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: vec3 { x: 0.0, y: 0.0, z: 1.0 } }, { time: 1.0, value: vec3 { x: 1.0, y: 1.0, z: 1.0 } }], loop: 'forever' }").unwrap().translate().unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Vector3(Vector3::new(0.1, 0.1, 1.0)));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Vector3(Vector3::new(0.6, 0.6, 1.0)));
}

#[test]
fn test_animation_from_pon_alternative_syntax() {
    let mut kf: Animation = Pon::from_string(
        "key_framed { property: this.pos_y, keys: [[0.0, 0.0], { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap().translate().unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}
