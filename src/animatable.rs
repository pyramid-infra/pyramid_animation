
use cgmath::*;
use pyramid::pon::*;
use std::cmp;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Animatable {
    pub value: Vec<f32>
}

impl Animatable {
    pub fn new(value: Vec<f32>) -> Animatable {
        Animatable {
            value: value
        }
    }
    pub fn new_float(value: f32) -> Animatable {
        Animatable::new(vec![value])
    }
    pub fn add_weighted(&self, weight: f32, next_value: &Animatable) -> Animatable {
        let mut res = vec![];
        for i in 0..cmp::min(self.value.len(), next_value.value.len()) {
            res.push(self.value[i] + weight * next_value.value[i]);
        }
        Animatable { value: res }
    }
    pub fn weighted(&self, weight: f32) -> Animatable {
        Animatable {
            value: self.value.iter().map(|x| x * weight).collect()
        }
    }
}

impl ToPon for Animatable {
    fn to_pon(&self) -> Pon {
        match self.value.len() {
            1 => self.value[0].to_pon(),
            3 => Vector3::new(self.value[0], self.value[1], self.value[2]).to_pon(),
            4 => Vector4::new(self.value[3], self.value[0], self.value[1], self.value[2]).to_pon(),
            _ => unreachable!()
        }
    }
}

pub trait Interpolateable {
    fn interpolate(a: &Self, b: &Self, p: &f32) -> Self;
}

impl Interpolateable for Animatable {
    fn interpolate(a: &Animatable, b: &Animatable, p: &f32) -> Animatable {
        let mut res = vec![];
        for i in 0..cmp::min(a.value.len(), b.value.len()) {
            res.push(a.value[i] * (1.0 - p) + b.value[i] * p);
        }
        Animatable { value: res }
    }
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

impl<'a> Translatable<'a, Animatable> for Pon {
    fn inner_translate(&'a self) -> Result<Animatable, PonTranslateErr> {
        if let Ok(v) = self.translate::<f32>() {
            Ok(Animatable { value: vec![v] })
        } else if let Ok(v) = self.translate::<Cow<Vec<f32>>>() {
            Ok(Animatable { value: v.into_owned() })
        } else if let Ok(v) = self.translate::<Vector3<f32>>() {
            Ok(Animatable { value: vec![v.x, v.y, v.z] })
        } else if let Ok(v) = self.translate::<Vector4<f32>>() {
            Ok(Animatable { value: vec![v.x, v.y, v.z, v.w] })
        } else {
            Err(PonTranslateErr::InvalidValue { value: self.to_string() })
        }
    }
}
