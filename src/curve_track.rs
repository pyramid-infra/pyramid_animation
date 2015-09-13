
use time::*;

use curve::*;
use track::*;
use pyramid::pon::*;
use cgmath::*;
use animatable::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Loop {
    Forever,
    Once
}

#[derive(PartialEq, Debug, Clone)]
pub enum CurveTime {
    /// The curve is expected to have keys between 0 and 1
    Relative,
    /// The curve is expected to have keys between 0 and duration
    Absolute
}


#[derive(Debug)]
pub struct CurveTrack {
    pub curve: Box<Curve<Animatable>>,
    pub offset: Duration,
    pub property: NamedPropRef,
    pub loop_type: Loop,
    pub duration: Duration,
    pub curve_time: CurveTime
}

impl CurveTrack {
    pub fn new_fixed_value(property: NamedPropRef, value: Animatable) -> CurveTrack {
        CurveTrack {
            curve: Box::new(FixedValueCurve { value: value }),
            offset: Duration::zero(),
            property: property,
            loop_type: Loop::Forever,
            duration: Duration::weeks(1),
            curve_time: CurveTime::Absolute
        }
    }
}

impl Track for CurveTrack {
    fn value_at(&self, time: Duration) -> Vec<(NamedPropRef, Animatable)> {
        let time = time - self.offset;
        let time = if time > self.duration {
            if self.loop_type == Loop::Forever {
                Duration::milliseconds(time.num_milliseconds() % self.duration.num_milliseconds())
            } else {
                return vec![]
            }
        } else {
            time
        };
        let time = match self.curve_time {
            CurveTime::Absolute => time.num_milliseconds() as f32 / 1000.0,
            CurveTime::Relative => time.num_milliseconds() as f32 / self.duration.num_milliseconds() as f32
        };
        return vec![(self.property.clone(), self.curve.value(time))];
    }
}


impl<'a, 'b> Translatable<'a, 'b, Loop> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Loop, PonTranslateErr> {
        match try!(self.translate(context)) {
            "forever" => Ok(Loop::Forever),
            "once" => Ok(Loop::Once),
            _ => Err(PonTranslateErr::InvalidValue { value: format!("{:?}", self) })
        }
    }
}
impl<'a, 'b> Translatable<'a, 'b, CurveTime> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<CurveTime, PonTranslateErr> {
        match try!(self.translate(context)) {
            "absolute" => Ok(CurveTime::Absolute),
            "relative" => Ok(CurveTime::Relative),
            _ => Err(PonTranslateErr::InvalidValue { value: format!("{:?}", self) })
        }
    }
}

impl<'a, 'b> Translatable<'a, 'b, Key<Animatable>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Key<Animatable>, PonTranslateErr> {
        match self {
            &Pon::Object(..) => {
                let time: f32 = try!(self.field_as::<f32>("time", context));
                let value = try!(self.field_as("value", context));
                Ok(Key(time, value))
            },
            &Pon::Array(ref arr) => {
                let time: f32 = try!(arr[0].translate::<f32>(context));
                let value = try!(arr[1].translate(context));
                Ok(Key(time, value))
            },
            &Pon::FloatArray(ref arr) => Ok(Key(arr[0], Animatable { value: vec![arr[1]] })),
            _ => {
                Err(PonTranslateErr::MismatchType { expected: "Object or Array".to_string(), found: format!("{:?}", self) })
            }
        }
    }
}

impl<'a, 'b> Translatable<'a, 'b, CurveTrack> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<CurveTrack, PonTranslateErr> {
        let &TypedPon { ref type_name, ref data } = try!(self.translate(context));
        match type_name.as_str() {
            "key_framed" => {
                let property: &NamedPropRef = try!(try!(data.field("property")).as_reference());
                let duration: f32 = try!(data.field_as_or("duration", 1.0, context));
                let loop_type = try!(data.field_as_or("loop", Loop::Once, context));
                let curve_time = try!(data.field_as_or("curve_time", CurveTime::Absolute, context));
                let keys_array: &Vec<Pon> = try!(data.field_as("keys", context));
                let first_key = &keys_array[0];
                let curve: Box<Curve<Animatable>> = {
                    let as_float: Result<Key<Animatable>, PonTranslateErr> = first_key.translate(context);
                    if let Ok(..) = as_float {
                        let keys: Vec<Key<Animatable>> = try!(data.field_as("keys", context));
                        Box::new(LinearKeyFrameCurve {
                            keys: keys
                        })
                    } else {
                        return Err(PonTranslateErr::Generic(format!("Unrecognized keys: {:?}", first_key)))
                    }
                };
                Ok(CurveTrack {
                    curve: curve,
                    offset: Duration::zero(),
                    property: property.clone(),
                    loop_type: loop_type,
                    duration: Duration::milliseconds((duration*1000.0) as i64),
                    curve_time: curve_time
                })
            },
            "fixed_value" => {
                let property: &NamedPropRef = try!(try!(data.field("property")).as_reference());
                let value = try!(data.field_as::<Animatable>("value", context));
                Ok(CurveTrack::new_fixed_value(property.clone(), value))
            },
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}



#[test]
fn test_animation() {
    let kf = CurveTrack {
        curve: Box::new(LinearKeyFrameCurve {
            keys: vec![Key(0.0, Animatable::new_float(0.0)), Key(1.0, Animatable::new_float(1.0))]
        }),
        offset: Duration::zero(),
        property: NamedPropRef::new(EntityPath::This, "x"),
        loop_type: Loop::Once,
        duration: Duration::seconds(1),
        curve_time: CurveTime::Absolute
    };
    assert_eq!(kf.value_at(Duration::milliseconds(100)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.1))]);
    assert_eq!(kf.value_at(Duration::milliseconds(600)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.6))]);
}

#[test]
fn test_animation_from_pon() {
    let kf: CurveTrack = Pon::from_string(
        "key_framed { property: this.x, keys: [{ time: 0.0, value: 0.0 }, { time: 1.0, value: 1.0 }], loop: 'forever' }")
        .unwrap().translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(kf.value_at(Duration::milliseconds(100)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.1))]);
    assert_eq!(kf.value_at(Duration::milliseconds(600)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.6))]);
}

#[test]
fn test_animation_from_pon_alternative_syntax() {
    let kf: CurveTrack = Pon::from_string(
        "key_framed { property: this.x, keys: [[0.0, 0.0], { time: 1.0, value: 1.0 }], loop: 'forever' }")
        .unwrap().translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(kf.value_at(Duration::milliseconds(100)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.1))]);
    assert_eq!(kf.value_at(Duration::milliseconds(600)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(0.6))]);
}

#[test]
fn test_animation_from_pon_multivalue() {
    let kf: CurveTrack = Pon::from_string(
        "key_framed { property: this.x, keys: [[0.0, [0.0, 10.0]], [1.0, [-2.0, 0.0]]], loop: 'forever' }")
        .unwrap().translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(kf.value_at(Duration::milliseconds(500)), vec![(NamedPropRef::new(EntityPath::This, "x"), Animatable::new(vec![-1.0, 5.0]))]);
}
