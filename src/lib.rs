#![feature(convert, box_patterns)]
#[macro_use]
extern crate pyramid;
extern crate time;
extern crate cgmath;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub mod animatable;
pub mod track;
pub mod curve_track;
pub mod track_set;
pub mod weighted_tracks;
pub mod curve;

use time::*;

use pyramid::interface::*;
use pyramid::pon::*;
use pyramid::document::*;
use pyramid::system::*;

pub use track::*;
pub use track_set::*;
pub use curve_track::*;
pub use curve::*;
pub use animatable::*;

struct EntityAnimation {
    track: Box<Track>,
    cached_resolved_named_prop_refs: HashMap<NamedPropRef, PropRef>
}

pub struct AnimationSubSystem {
    animations: HashMap<EntityId, EntityAnimation>,
    start_time: Timespec
}

impl AnimationSubSystem {
    pub fn new() -> AnimationSubSystem {
        AnimationSubSystem {
            animations: HashMap::new(),
            start_time: time::get_time()
        }
    }
}


impl ISubSystem for AnimationSubSystem {

    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>) {
        let doc = system.document_mut();
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "animation") {
            let pon = &*doc.get_property(&pr.entity_id, &pr.property_key.as_str()).unwrap();
            pon.as_resolved(|pon| {
                match pon {
                    &Pon::Nil => {}, // Ignore nil pons
                    pn @ _ => {
                        match pn.translate::<Box<Track>>(&mut TranslateContext { document: Some(doc) }) {
                            Ok(anim) => {
                                self.animations.insert(pr.entity_id, EntityAnimation {
                                    track: anim,
                                    cached_resolved_named_prop_refs: HashMap::new()
                                });
                            },
                            Err(err) => { println!("Failed to translate animation: {:?}", err.to_string()); }
                        };
                    }
                };
                Ok(())
            }).unwrap()
        }
    }
    fn update(&mut self, system: &mut System) {
        let time = time::get_time() - self.start_time;
        for (entity_id, entity_animation) in self.animations.iter_mut() {
            let to_update = { entity_animation.track.value_at(time) };
            for (named_prop_ref, value) in to_update {
                let target = match entity_animation.cached_resolved_named_prop_refs.entry(named_prop_ref.clone()) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(system.document().resolve_named_prop_ref(entity_id, &named_prop_ref).unwrap())
                };
                system.document_mut().set_property(&target.entity_id.clone(), &target.property_key, value.to_pon()).unwrap();
            }
        }
    }
}
