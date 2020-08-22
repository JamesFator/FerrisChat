use crate::components::{CrabAI, CrabAIState, Location, WantsToMoveTo};
use crate::map::{valid_walking_location, Map};
use specs::prelude::*;

pub struct CrabAISystem {}

impl<'a> System<'a> for CrabAISystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, CrabAI>,
        ReadStorage<'a, Location>,
        WriteStorage<'a, WantsToMoveTo>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, map, mut crab_ais, locations, mut move_tos) = data;

        for (entity, mut crab_ai, location) in (&entities, &mut crab_ais, &locations).join() {
            if crab_ai.ticks == crab_ai.tick_interval {
                crab_ai.ticks = 0;
                crab_ai.crab_state = match crab_ai.crab_state {
                    CrabAIState::WalkingRight => CrabAIState::SleepingRight,
                    CrabAIState::SleepingRight => CrabAIState::WalkingLeft,
                    CrabAIState::WalkingLeft => CrabAIState::SleepingLeft,
                    CrabAIState::SleepingLeft => CrabAIState::WalkingRight,
                };
            } else {
                crab_ai.ticks += 1;
            }

            let desired_location = match crab_ai.crab_state {
                CrabAIState::WalkingRight => WantsToMoveTo {
                    x: location.x + 1,
                    y: location.y,
                    speed: crab_ai.walk_speed,
                },
                CrabAIState::WalkingLeft => WantsToMoveTo {
                    x: location.x - 1,
                    y: location.y,
                    speed: crab_ai.walk_speed,
                },
                _ => WantsToMoveTo {
                    x: location.x,
                    y: location.y,
                    speed: crab_ai.walk_speed,
                },
            };
            if !valid_walking_location(&map, &desired_location) {
                continue; // Should not walk off the map
            }
            move_tos
                .insert(entity, desired_location)
                .expect("Unable to insert WantsToMoveTo");
        }
    }
}
