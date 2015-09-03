
use pyramid::pon_parser as pon_parser;

use time::*;

use curve::*;
use pyramid::pon::*;
use cgmath::*;

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

#[derive(PartialEq, Debug, Clone)]
pub enum AnimationLoadError {
    PonTranslateErr(PonTranslateErr),
    UnkownAnimationType(String),
    MissingArgument(String),
    BadArgumentValue(String),
}

impl From<PonTranslateErr> for AnimationLoadError {
    fn from(err: PonTranslateErr) -> AnimationLoadError {
        AnimationLoadError::PonTranslateErr(err)
    }
}

impl Animation {
    fn normalized_pon_keys<'a>(keys_nodes: &'a Vec<Pon>) -> Result<Vec<(f32, &'a Pon)>, PonTranslateErr> {
        let mut keys = vec![];
        for key in keys_nodes {
            match key {
                key @ &Pon::Object(_) => {
                    let time = try!(try!(key.get_object_field("time")).as_float());
                    let value = try!(key.get_object_field("value"));
                    keys.push((*time, value));
                },
                &Pon::Array(ref key) => {
                    let time = try!(key[0].as_float());
                    keys.push((*time, &key[1]));
                },
                _ => panic!("Unkown key type")
            }
        }
        Ok(keys)
    }
    fn pon_keys_to_curve(keys_nodes: &Vec<Pon>) -> Result<Box<PonCurve>, AnimationLoadError> {
        let normalized_keys = try!(Animation::normalized_pon_keys(keys_nodes));
        match normalized_keys[0].1 {
            &Pon::TypedPon(box TypedPon { ref type_name, .. }) => {
                match type_name.as_str() {
                    "vec3" => {
                        let mut keys = vec![];
                        for (time, value) in normalized_keys {
                            let value = &try!(value.as_transform()).data;
                            let vec3 = Vector3::new(
                                *try!(try!(value.get_object_field("x")).as_float()),
                                *try!(try!(value.get_object_field("y")).as_float()),
                                *try!(try!(value.get_object_field("z")).as_float()));
                            keys.push((time, vec3));
                        }
                        Ok(Box::new(LinearKeyFrameCurve {
                            keys: keys
                        }))
                    },
                    _ => panic!("Unknown key type {}", type_name)
                }

            },
            &Pon::Float(_) => {
                let mut keys = vec![];
                for (time, value) in normalized_keys {
                    keys.push((time, *try!(value.as_float())));
                }
                Ok(Box::new(LinearKeyFrameCurve {
                    keys: keys
                }))
            },
            _ => panic!("Unknown key type")
        }

    }
    pub fn from_prop_node(node: &Pon) -> Result<Animation, AnimationLoadError> {
        let &TypedPon { ref type_name, ref data } = try!(node.as_transform());
        let data = try!(data.as_object());
        match type_name.as_str() {
            "key_framed" => {
                let property = match data.get("property") {
                    Some(&ref v) => try!(v.as_reference()),
                    _ => return Err(AnimationLoadError::MissingArgument("property".to_string()))
                };
                let duration = match data.get("duration") {
                    Some(&ref v) => *try!(v.as_float()),
                    _ => 1.0
                };
                let loop_type = match data.get("loop") {
                    Some(&ref v) => match try!(v.as_string()).as_str() {
                        "forever" => Loop::Forever,
                        "once" => Loop::Once,
                        _ => return Err(AnimationLoadError::BadArgumentValue("loop".to_string()))
                    },
                    None => Loop::Once
                };
                let property = match data.get("property") {
                    Some(&ref v) => try!(v.as_reference()),
                    _ => return Err(AnimationLoadError::MissingArgument("property".to_string()))
                };
                let keys_nodes = match data.get("keys") {
                    Some(&ref n) => try!(n.as_array()),
                    _ => return Err(AnimationLoadError::MissingArgument("keys".to_string()))
                };
                let curve = try!(Animation::pon_keys_to_curve(keys_nodes));
                return Ok(Animation {
                    curve: curve,
                    time: Duration::zero(),
                    property: property.clone(),
                    loop_type: loop_type,
                    duration: Duration::milliseconds((duration*1000.0) as i64)
                });
            },
            s @ _ => Err(AnimationLoadError::UnkownAnimationType(s.to_string()))
        }
    }

    pub fn update(&mut self, delta_time: Duration) -> Pon {
        {
            self.time = self.time + delta_time;
            if (self.loop_type == Loop::Forever && self.time > self.duration) {
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
            keys: vec![(0.0, 0.0), (1.0, 1.0)]
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
    let mut kf = Animation::from_prop_node(&pon_parser::parse(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: 0.0 }, { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}

#[test]
fn test_animation_from_pon_vec3() {
    let mut kf = Animation::from_prop_node(&pon_parser::parse(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: vec3 { x: 0.0, y: 0.0, z: 1.0 } }, { time: 1.0, value: vec2 { x: 1.0, y: 1.0, z: 1.0 } }], loop: 'forever' }").unwrap()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), pon_parser::parse("vec3 { x: 0.1, y: 0.1, z: 1.0 }").unwrap());
    assert_eq!(kf.update(Duration::milliseconds(500)), pon_parser::parse("vec3 { x: 0.6, y: 0.6, z: 1.0 }").unwrap());
}

#[test]
fn test_animation_from_pon_alternative_syntax() {
    let mut kf = Animation::from_prop_node(&pon_parser::parse(
        "key_framed { property: this.pos_y, keys: [[0.0, 0.0], { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), Pon::Float(0.1));
    assert_eq!(kf.update(Duration::milliseconds(500)), Pon::Float(0.6));
}
