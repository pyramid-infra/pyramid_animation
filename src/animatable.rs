
use time::*;
use pyramid::pon::*;
use animation::*;
use animation_set::*;
use tracks::*;

pub trait Animatable {
    fn update(&self, time: Duration) -> Vec<(NamedPropRef, f32)>;
}


impl<'a> Translatable<'a, Box<Animatable>> for Pon {
    fn inner_translate(&'a self) -> Result<Box<Animatable>, PonTranslateErr> {
        let &TypedPon { ref type_name, .. } = try!(self.translate());
        match type_name.as_str() {
            "key_framed" => Ok(Box::new(try!(self.translate::<Animation>()))),
            "fixed_value" => Ok(Box::new(try!(self.translate::<Animation>()))),
            "animation_set" => Ok(Box::new(try!(self.translate::<AnimationSet>()))),
            "tracks" => Ok(Box::new(try!(self.translate::<Tracks>()))),
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}
