use super::{CarriedBy, Location};
use specs::prelude::*;

pub struct CarrySystem {}

impl<'a> System<'a> for CarrySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Location>,
        ReadStorage<'a, CarriedBy>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut locations, carried_bys) = data;

        // Build a list of new locations for each CarriedBy entity so we
        // don't mutate locations while we have immutable reference
        let mut new_locations = Vec::new();
        for (entity, carried_by) in (&entities, &carried_bys).join() {
            if let Some(carrier_location) = locations.get(carried_by.owner) {
                new_locations.push((entity, carrier_location.clone()));
            }
        }

        for (entity, new_location) in new_locations {
            locations
                .insert(entity, new_location)
                .expect("Failed to insert Location");
        }
    }
}
