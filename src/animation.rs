
use pyramid::pon_parser as pon_parser;

use time::*;

use curve::*;
use pyramid::pon::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Loop {
    Forever,
    Once
}

pub struct Animation {
    pub curve: Box<Curve>,
    pub time: Duration,
    pub property: NamedPropRef,
    pub loop_type: Loop,
    pub duration: Duration
}

#[derive(PartialEq, Debug, Clone)]
pub enum AnimationLoadError {
    PropTranslateErr(PropTranslateErr),
    UnkownAnimationType(String),
    MissingArgument(String),
    BadArgumentValue(String),
}

impl From<PropTranslateErr> for AnimationLoadError {
    fn from(err: PropTranslateErr) -> AnimationLoadError {
        AnimationLoadError::PropTranslateErr(err)
    }
}

impl Animation {
    pub fn from_prop_node(node: &Pon) -> Result<Animation, AnimationLoadError> {
        let &PropTransform { ref name, ref arg } = try!(node.as_transform());
        let arg = try!(arg.as_object());
        match name.as_str() {
            "key_framed" => {
                let property = match arg.get("property") {
                    Some(&ref v) => try!(v.as_reference()),
                    _ => return Err(AnimationLoadError::MissingArgument("property".to_string()))
                };
                let duration = match arg.get("duration") {
                    Some(&ref v) => *try!(v.as_float()),
                    _ => 1.0
                };
                let loop_type = match arg.get("loop") {
                    Some(&ref v) => match try!(v.as_string()).as_str() {
                        "forever" => Loop::Forever,
                        "once" => Loop::Once,
                        _ => return Err(AnimationLoadError::BadArgumentValue("loop".to_string()))
                    },
                    None => Loop::Once
                };
                let property = match arg.get("property") {
                    Some(&ref v) => try!(v.as_reference()),
                    _ => return Err(AnimationLoadError::MissingArgument("property".to_string()))
                };
                let keys_nodes = match arg.get("keys") {
                    Some(&ref n) => try!(n.as_array()),
                    _ => return Err(AnimationLoadError::MissingArgument("keys".to_string()))
                };
                let mut keys = vec![];
                for key in keys_nodes {
                    let key = try!(key.as_object());
                    let time = match key.get("time") {
                        Some(&ref time) => *try!(time.as_float()),
                        _ => return Err(AnimationLoadError::MissingArgument("time".to_string()))
                    };
                    let value = match key.get("value") {
                        Some(&ref value) => *try!(value.as_float()),
                        _ => return Err(AnimationLoadError::MissingArgument("value".to_string()))
                    };
                    keys.push((time, value));
                }
                return Ok(Animation {
                    curve: Box::new(LinearKeyFrameCurve {
                        keys: keys
                    }),
                    time: Duration::zero(),
                    property: property.clone(),
                    loop_type: loop_type,
                    duration: Duration::milliseconds((duration*1000.0) as i64)
                });
            },
            s @ _ => Err(AnimationLoadError::UnkownAnimationType(s.to_string()))
        }
    }

    pub fn update(&mut self, delta_time: Duration) -> f32 {
        {
            self.time = self.time + delta_time;
            if (self.loop_type == Loop::Forever && self.time > self.duration) {
                self.time = Duration::zero();
            }
        }
        return self.curve.value(self.time.num_milliseconds() as f32/self.duration.num_milliseconds() as f32);
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
    assert_eq!(kf.update(Duration::milliseconds(100)), 0.1);
    assert_eq!(kf.update(Duration::milliseconds(500)), 0.6);
}

#[test]
fn test_animation_from_pon() {
    let mut kf = Animation::from_prop_node(&pon_parser::parse(
        "key_framed { property: this.pos_y, keys: [{ time: 0.0, value: 0.0 }, { time: 1.0, value: 1.0 }], loop: 'forever' }").unwrap()).unwrap();
    assert_eq!(kf.update(Duration::milliseconds(100)), 0.1);
    assert_eq!(kf.update(Duration::milliseconds(500)), 0.6);
}
