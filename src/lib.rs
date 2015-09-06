#![feature(convert, box_patterns)]
#[macro_use]
extern crate pyramid;
extern crate time;
extern crate cgmath;

use std::collections::HashMap;

mod animatable;
mod animation;
mod animation_set;
mod tracks;
mod curve;

use time::*;

use pyramid::interface::*;
use pyramid::pon::*;
use pyramid::document::*;
use animatable::*;

pub struct AnimationSubSystem {
    animations: HashMap<EntityId, Box<Animatable>>,
    time: Duration
}

impl AnimationSubSystem {
    pub fn new() -> AnimationSubSystem {
        AnimationSubSystem {
            animations: HashMap::new(),
            time: Duration::zero()
        }
    }
}


impl ISubSystem for AnimationSubSystem {

    fn on_property_value_change(&mut self, system: &mut ISystem, prop_refs: &Vec<PropRef>) {
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "animation") {
            match system.get_property_value(&pr.entity_id, &pr.property_key.as_str()).unwrap() {
                Pon::Nil => {}, // Ignore nil pons
                pn @ _ => {
                    match pn.translate::<Box<Animatable>>() {
                        Ok(anim) => { self.animations.insert(pr.entity_id, anim); },
                        Err(err) => { println!("Failed to translate animation: {:?}", err.to_string()); }
                    };
                }
            }
        }
    }
    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {
        self.time = self.time + delta_time;
        for (entity_id, animation) in self.animations.iter() {
            let to_update = { animation.update(self.time) };
            for (named_prop_ref, value) in to_update {
                let target = system.resolve_named_prop_ref(entity_id, &named_prop_ref).unwrap();
                system.set_property(&target.entity_id.clone(), &target.property_key, Pon::Float(value)).unwrap();
            }
        }
    }
}
