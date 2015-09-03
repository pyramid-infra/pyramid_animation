#![feature(convert, box_patterns)]
#[macro_use]
extern crate pyramid;
extern crate time;
extern crate cgmath;

use std::collections::HashMap;

mod animation;
mod curve;

use pyramid::interface::*;
use pyramid::pon::*;
use pyramid::document::*;
use animation::*;

pub struct AnimationSubSystem {
    animations: HashMap<EntityId, (Animation, PropRef)>
}

impl AnimationSubSystem {
    pub fn new() -> AnimationSubSystem {
        AnimationSubSystem {
            animations: HashMap::new()
        }
    }
}


impl ISubSystem for AnimationSubSystem {

    fn on_property_value_change(&mut self, system: &mut ISystem, prop_refs: &Vec<PropRef>) {
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "animation") {
            let pn = system.get_property_value(&pr.entity_id, &pr.property_key.as_str()).unwrap();
            let anim = Animation::from_prop_node(&pn).unwrap();
            let target = system.resolve_named_prop_ref(&pr.entity_id, &anim.property).unwrap();
            self.animations.insert(pr.entity_id, (anim, target));
        }
    }
    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {
        for (_, &mut (ref mut animation, ref mut target)) in self.animations.iter_mut() {
            let value = { animation.update(delta_time) };
            system.set_property(&target.entity_id.clone(), target.property_key.clone(), value);
        }
    }
}
