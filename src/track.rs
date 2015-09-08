
use time::*;
use pyramid::pon::*;
use curve_track::*;
use track_set::*;
use weighted_tracks::*;

pub trait Track {
    fn value_at(&self, time: Duration) -> Vec<(NamedPropRef, f32)>;
}


impl<'a> Translatable<'a, Box<Track>> for Pon {
    fn inner_translate(&'a self) -> Result<Box<Track>, PonTranslateErr> {
        let &TypedPon { ref type_name, .. } = try!(self.translate());
        match type_name.as_str() {
            "key_framed" => Ok(Box::new(try!(self.translate::<CurveTrack>()))),
            "fixed_value" => Ok(Box::new(try!(self.translate::<CurveTrack>()))),
            "track_set" => Ok(Box::new(try!(self.translate::<TrackSet>()))),
            "weighted_tracks" => Ok(Box::new(try!(self.translate::<WeightedTracks>()))),
            s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
        }
    }
}
