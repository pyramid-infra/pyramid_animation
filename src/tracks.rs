
#[cfg(test)]
use std::cmp::Ordering;
#[cfg(test)]
use animation::*;

use std::collections::HashMap;
use time::*;
use animatable::*;
use pyramid::pon::*;

pub struct Track {
    pub weight: f32,
    pub animation: Box<Animatable>
}

pub struct Tracks {
    pub tracks: Vec<Track>
}

impl Animatable for Tracks {
    fn update(&self, time: Duration) -> Vec<(NamedPropRef, f32)> {
        let mut by_props = HashMap::new();
        for track in &self.tracks {
            for update in track.animation.update(time) {
                let new_value = match by_props.get(&update.0) {
                    Some(value) => value + track.weight * update.1,
                    None => track.weight * update.1
                };
                by_props.insert(update.0, new_value);
            }
        }
        by_props.into_iter().collect()
    }
}

impl<'a> Translatable<'a, Tracks> for Pon {
    fn inner_translate(&'a self) -> Result<Tracks, PonTranslateErr> {
        unimplemented!()
    }
}


#[test]
fn test_tracks() {
    let setup = Tracks {
        tracks: vec![
            Track { weight: 0.1, animation: Box::new(Animation::new_fixed_value(NamedPropRef::new(EntityPath::This, "x"), 10.0)) },
            Track { weight: 0.5, animation: Box::new(Animation::new_fixed_value(NamedPropRef::new(EntityPath::This, "y"), 10.0)) },
            Track { weight: 0.2, animation: Box::new(Animation::new_fixed_value(NamedPropRef::new(EntityPath::This, "y"), 100.0)) },
        ]
    };
    assert_eq!(setup.update(Duration::zero()).sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)), vec![
        (NamedPropRef::new(EntityPath::This, "x"), 1.0),
        (NamedPropRef::new(EntityPath::This, "y"), 25.0)
    ].sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)));
}
