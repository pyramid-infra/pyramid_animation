#[cfg(test)]
use std::cmp::Ordering;

use time::*;
use track::*;
use pyramid::pon::*;
use animatable::*;

#[derive(Debug)]
pub struct TrackSet {
    pub tracks: Vec<Box<Track>>
}

impl Track for TrackSet {
    fn value_at(&self, time: Duration) -> Vec<(NamedPropRef, Animatable)> {
        let mut res = vec![];
        for track in &self.tracks {
            for update in track.value_at(time).into_iter() {
                res.push(update);
            }
        }
        res
    }
}

impl Translatable<TrackSet> for Pon {
    fn inner_translate(&self, context: &mut TranslateContext) -> Result<TrackSet, PonTranslateErr> {
        self.as_typed(|&TypedPon { ref type_name, ref data }| {
            match type_name.as_str() {
                "track_set" => {
                    let anims = try!(data.translate::<PonAutoVec<Box<Track>>>(context));
                    Ok(TrackSet {
                        tracks: anims.0
                    })
                },
                s @ _ => Err(PonTranslateErr::UnrecognizedType(s.to_string()))
            }
        })
    }
}

#[test]
fn test_track_set_from_pon() {
    let anim_set: TrackSet = Pon::from_string(
        "track_set [ fixed_value { property: this.x, value: 0.5 }, fixed_value { property: this.y, value: 0.2 } ]")
        .unwrap().translate(&mut TranslateContext::empty()).unwrap();
    assert_eq!(anim_set.value_at(Duration::zero()).sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)), vec![
        (NamedPropRef::new(EntityPath::This, "x"), 0.5),
        (NamedPropRef::new(EntityPath::This, "y"), 0.2)
    ].sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)));
}
