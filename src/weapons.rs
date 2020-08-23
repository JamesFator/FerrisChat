use crate::components::{CarriedBy, Location, PlayerInfo, WantsToStab};
use crate::entities::{create_blood_splatter, delete_player};
use crate::map::within_pickup_distance;
use specs::prelude::*;

pub struct StabSystem {}

/// StabSystem isn't a standard system because it needs to create entities
/// and I didn't find any better way to do this
impl StabSystem {
    pub fn run_now_manually(mut ecs: &mut World) {
        // Build list of stabbings for later so we don't modify holding immutable reference
        let mut stabbings = Vec::new();
        {
            let entities = ecs.entities();
            let player_infos = ecs.read_storage::<PlayerInfo>();
            let locations = ecs.read_storage::<Location>();
            let carried_bys = ecs.read_storage::<CarriedBy>();
            let stabbies = ecs.read_storage::<WantsToStab>();

            // A WantsToStab can only kill if it has a CarriedBy
            for (item_location, carried_by, _) in (&locations, &carried_bys, &stabbies).join() {
                // WantsToStab can only kill PlayerInfos (and can't be the carrier)
                for (player_entity, player_location, _) in
                    (&entities, &locations, &player_infos).join()
                {
                    if player_entity != carried_by.owner {
                        if within_pickup_distance(item_location, player_location) {
                            stabbings.push((player_entity, player_location.clone()));
                        }
                    }
                }
            }

            for (stabbing_victim, _) in stabbings.iter() {
                // Insert a CarriedBy component for the item
                delete_player(&entities, &carried_bys, *stabbing_victim);
            }
        }

        for (_, victim_location) in stabbings {
            // Add a blood splatter to highlight what happened here
            create_blood_splatter(&mut ecs, victim_location);
        }
    }
}
