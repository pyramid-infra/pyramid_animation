
use time::*;
use pyramid::pon::*;
use curve_track::*;
use track_set::*;
use weighted_tracks::*;
use animatable::*;
use std::fmt::Debug;

pub trait Track : Debug {
    fn value_at(&self, time: Duration) -> Vec<(NamedPropRef, Animatable)>;
}


impl<'a, 'b> Translatable<'a, 'b, Box<Track>> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<Box<Track>, PonTranslateErr> {
        let &TypedPon { ref type_name, .. } = try!(self.translate(context));
        match type_name.as_str() {
            "key_framed" => Ok(Box::new(try!(self.translate::<CurveTrack>(context)))),
            "fixed_value" => Ok(Box::new(try!(self.translate::<CurveTrack>(context)))),
            "track_set" => Ok(Box::new(try!(self.translate::<TrackSet>(context)))),
            "weighted_tracks" => Ok(Box::new(try!(self.translate::<WeightedTracks>(context)))),
            "track_from_resource" => unimplemented!(),
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}
