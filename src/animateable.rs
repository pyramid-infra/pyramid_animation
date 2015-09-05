
use time::*;
use pyramid::pon::*;

pub trait Animateable {
    fn update(&self, time: Duration) -> Vec<(NamedPropRef, f32)>;
}


impl<'a> Translatable<'a, Box<Animateable>> for Pon {
    fn inner_translate(&'a self) -> Result<Box<Animateable>, PonTranslateErr> {
        unimplemented!();
    }
}
