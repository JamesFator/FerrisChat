use super::{valid_walking_location, CrabAI, CrabAIState, Location, Map, WantsToMoveTo};
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
            if crab_ai.ticks_since_move == crab_ai.total_ticks_per_move {
                crab_ai.ticks_since_move = 0;
                crab_ai.crab_state = match crab_ai.crab_state {
                    CrabAIState::WalkingRight => CrabAIState::SleepingRight,
                    CrabAIState::SleepingRight => CrabAIState::WalkingLeft,
                    CrabAIState::WalkingLeft => CrabAIState::SleepingLeft,
                    CrabAIState::SleepingLeft => CrabAIState::WalkingRight,
                };
            } else {
                crab_ai.ticks_since_move += 1;
            }

            let desired_location = match crab_ai.crab_state {
                CrabAIState::WalkingRight => WantsToMoveTo {
                    x: location.x + 1,
                    y: location.y,
                    speed: crab_ai.speed,
                },
                CrabAIState::WalkingLeft => WantsToMoveTo {
                    x: location.x - 1,
                    y: location.y,
                    speed: crab_ai.speed,
                },
                _ => WantsToMoveTo {
                    x: location.x,
                    y: location.y,
                    speed: crab_ai.speed,
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
