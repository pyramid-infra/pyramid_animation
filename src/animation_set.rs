#[cfg(test)]
use std::cmp::Ordering;

use time::*;
use animatable::*;
use pyramid::pon::*;

pub struct AnimationSet {
    pub animations: Vec<Box<Animatable>>
}

impl Animatable for AnimationSet {
    fn update(&self, time: Duration) -> Vec<(NamedPropRef, f32)> {
        let mut res = vec![];
        for animation in &self.animations {
            for update in animation.update(time).into_iter() {
                res.push(update);
            }
        }
        res
    }
}

impl<'a> Translatable<'a, AnimationSet> for Pon {
    fn inner_translate(&'a self) -> Result<AnimationSet, PonTranslateErr> {
        let &TypedPon { ref type_name, ref data } = try!(self.translate());
        match type_name.as_str() {
            "animation_set" => {
                let anims = try!(data.translate::<Vec<Box<Animatable>>>());
                Ok(AnimationSet {
                    animations: anims
                })
            },
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}

#[test]
fn test_animation_set_from_pon() {
    let anim_set: AnimationSet = Pon::from_string(
        "animation_set [ fixed_value { property: this.x, value: 0.5 }, fixed_value { property: this.y, value: 0.2 } ]").unwrap().translate().unwrap();
    assert_eq!(anim_set.update(Duration::zero()).sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)), vec![
        (NamedPropRef::new("this", "x"), 0.5),
        (NamedPropRef::new("this", "y"), 0.2)
    ].sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)));
}
