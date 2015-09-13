
#[cfg(test)]
use std::cmp::Ordering;
#[cfg(test)]
use curve_track::*;

use std::collections::HashMap;
use time::*;
use track::*;
use pyramid::pon::*;
use animatable::*;

#[derive(Debug)]
pub struct WeightedTrack {
    pub weight: f32,
    pub track: Box<Track>
}

#[derive(Debug)]
pub struct WeightedTracks {
    pub tracks: Vec<WeightedTrack>
}

impl Track for WeightedTracks {
    fn value_at(&self, time: Duration) -> Vec<(NamedPropRef, Animatable)> {
        let mut by_props: HashMap<NamedPropRef, Animatable> = HashMap::new();
        for track in &self.tracks {
            for update in track.track.value_at(time) {
                let new_value = match by_props.get(&update.0) {
                    Some(value) => value.add_weighted(track.weight, &update.1),
                    None => update.1.weighted(track.weight)
                };
                by_props.insert(update.0, new_value);
            }
        }
        by_props.into_iter().collect()
    }
}

impl<'a, 'b> Translatable<'a, 'b, WeightedTracks> for Pon {
    fn inner_translate(&'a self, context: &mut TranslateContext<'b>) -> Result<WeightedTracks, PonTranslateErr> {
        unimplemented!()
    }
}


#[test]
fn test_tracks() {
    let setup = WeightedTracks {
        tracks: vec![
            WeightedTrack { weight: 0.1, track: Box::new(CurveTrack::new_fixed_value(NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(10.0))) },
            WeightedTrack { weight: 0.5, track: Box::new(CurveTrack::new_fixed_value(NamedPropRef::new(EntityPath::This, "y"), Animatable::new_float(10.0))) },
            WeightedTrack { weight: 0.2, track: Box::new(CurveTrack::new_fixed_value(NamedPropRef::new(EntityPath::This, "y"), Animatable::new_float(100.0))) },
        ]
    };
    assert_eq!(setup.value_at(Duration::zero()).sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)), vec![
        (NamedPropRef::new(EntityPath::This, "x"), Animatable::new_float(1.0)),
        (NamedPropRef::new(EntityPath::This, "y"), Animatable::new_float(25.0))
    ].sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)));
}
