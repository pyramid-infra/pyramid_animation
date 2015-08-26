#![feature(convert)]
extern crate pyramid;
extern crate time;

use std::collections::HashMap;

mod animation;
mod curve;

use pyramid::interface::*;
use pyramid::propnode::*;
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

#[no_mangle]
pub fn new() -> Box<SubSystem> {
    Box::new(AnimationSubSystem::new())
}


impl SubSystem for AnimationSubSystem {

    fn on_entity_added(&mut self, system: &mut System, entity_id: &EntityId) {
        let prop_refs: Vec<PropRef> = { system.get_properties(&entity_id).unwrap() };
        self.on_property_value_change(system, &prop_refs);
    }
    fn on_property_value_change(&mut self, system: &mut System, prop_refs: &Vec<PropRef>) {
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "animation") {
            let pn = system.get_property_value(&pr.entity_id, &pr.property_key.as_str()).unwrap();
            let anim = Animation::from_prop_node(&pn).unwrap();
            let target = system.resolve_named_prop_ref(&pr.entity_id, &anim.property).unwrap();
            self.animations.insert(pr.entity_id, (anim, target));
        }
    }
    fn update(&mut self, system: &mut System, delta_time: time::Duration) {
        let mut to_set = vec![];
        {
            //println!("Update animations {}", self.animations.len());
            for (_, &mut (ref mut animation, ref mut target)) in self.animations.iter_mut() {
                //println!("Update anim");
                let value = { animation.update(delta_time) };
                to_set.push((target.entity_id.clone(), target.property_key.clone(), PropNode::Float(value)));
            }
        }
        {
            for t in to_set {
                system.set_property(&t.0, t.1, t.2);
            }
        }
    }
}
