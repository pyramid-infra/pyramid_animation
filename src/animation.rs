
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
        let v = self.value(time);
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "vec3".to_string(),
            data: Pon::Object(hashmap!(
                "x".to_string() => Pon::Float(v.x),
                "y".to_string() => Pon::Float(v.y),
                "z".to_string() => Pon::Float(v.z)
                ))
        }))
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

impl<'a, 'b, C> Translatable<'a, 'b, Loop, C> for Pon {
    fn translate(&'a self, context: &'b C) -> Result<Loop, PonTranslateErr> {
        let s: &str = from_pon!(self, self, context);
        match s {
            "forever" => Ok(Loop::Forever),
            "once" => Ok(Loop::Once),
            _ => Err(PonTranslateErr::InvalidValue { value: s.to_string() })
        }
    }
}

impl<'a, 'b, T, C> Translatable<'a, 'b, Key<T>, C> for Pon where Pon: Translatable<'a, 'b, T, C> + Translatable<'a, 'b, f32, C> {
    fn translate(&'a self, context: &'b C) -> Result<Key<T>, PonTranslateErr> {
        match self {
            &Pon::Object(..) => {
                let time: f32 = try!(self.field_as(context, "time"));
                let value = try!(self.field_as(context, "value"));
                Ok(Key(time, value))
            },
            &Pon::Array(ref arr) => {
                let time: f32 = from_pon!(self, arr[0], context);
                let value = from_pon!(self, arr[1], context);
                Ok(Key(time, value))
            },
            _ => {
                Err(PonTranslateErr::MismatchType { expected: "Object or Array".to_string(), found: format!("{:?}", self) })
            }
        }
    }
}

impl<'a, 'b, C> Translatable<'a, 'b, Animation, C> for Pon {
    fn translate(&'a self, context: &'b C) -> Result<Animation, PonTranslateErr> {
        let &TypedPon { ref type_name, ref data } = try!(self.translate(context));
        match type_name.as_str() {
            "key_framed" => {
                let property: &NamedPropRef = from_pon!(self, try!(data.field("property")), &ReferenceContext);
                let duration: f32 = try!(data.field_as_or(context, "duration", 1.0));
                let loop_type = try!(data.field_as_or(context, "loop", Loop::Once));
                let keys_array: &Vec<Pon> = try!(data.field_as(context, "keys"));
                let first_key = &keys_array[0];
                let curve: Box<PonCurve> = {
                    let as_vec3: Result<Key<Vector3<f32>>, PonTranslateErr> = first_key.translate(context);
                    if let Ok(..) = as_vec3 {
                        let keys: Vec<Key<Vector3<f32>>> = try!(data.field_as(context, "keys"));
                        Box::new(LinearKeyFrameCurve {
                            keys: keys
                        })
                    } else {
                        let as_float: Result<Key<f32>, PonTranslateErr> = first_key.translate(context);
                        if let Ok(..) = as_float {
                            let keys: Vec<Key<f32>> = try!(data.field_as(context, "keys"));
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
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: 0.0 }, { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap().translate(&()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}

#[test]
fn test_animation_from_pon_vec3() {
    let mut kf: Animation = Pon::from_string(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: vec3 { x: 0.0, y: 0.0, z: 1.0 } }, { time: 1.0, value: vec3 { x: 1.0, y: 1.0, z: 1.0 } }], loop: 'forever' }").unwrap().translate(&()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::from_string("vec3 { x: 0.1, y: 0.1, z: 1.0 }").unwrap());
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::from_string("vec3 { x: 0.6, y: 0.6, z: 1.0 }").unwrap());
}

#[test]
fn test_animation_from_pon_alternative_syntax() {
    let mut kf: Animation = Pon::from_string(
        "key_framed { property: this.pos_y, keys: [[0.0, 0.0], { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap().translate(&()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}
