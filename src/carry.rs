use crate::components::{CarriedBy, Location, PlayerInfo, Renderable, WantsToBePickedUp};
use crate::map::within_pickup_distance;
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

pub struct PickUpSystem {}

impl<'a> System<'a> for PickUpSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, PlayerInfo>,
        ReadStorage<'a, Location>,
        WriteStorage<'a, WantsToBePickedUp>,
        WriteStorage<'a, CarriedBy>,
        WriteStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_infos, locations, mut pickups, mut carried_bys, mut renders) = data;

        // Build list of new CarriedBy pairs so we don't mutate while we have immutable reference
        let mut pick_up_pairs = Vec::new();
        for (item_entity, item_location, _) in (&entities, &locations, &pickups).join() {
            for (player_entity, player_location, _) in (&entities, &locations, &player_infos).join()
            {
                if within_pickup_distance(item_location, player_location) {
                    pick_up_pairs.push((item_entity, player_entity));
                }
            }
        }

        for (item_entity, player_entity) in pick_up_pairs {
            // Insert a CarriedBy component for the item
            carried_bys
                .insert(
                    item_entity,
                    CarriedBy {
                        owner: player_entity,
                    },
                )
                .expect("Failed to give player item");
            // Remove the WantsToBePickedUp component for the item
            pickups.remove(item_entity);
            // Bump up render order so item appears on top
            renders
                .insert(item_entity, Renderable { render_order: 0 })
                .expect("Failed to increase render order");
        }
    }
}
