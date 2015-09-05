
use time::*;
use animateable::*;
use pyramid::pon::*;

pub struct AnimationSet {
    pub animations: Vec<Box<Animateable>>
}

impl Animateable for AnimationSet {
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
        unimplemented!()
    }
}
