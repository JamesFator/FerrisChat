use super::{Location, Map, WantsToMoveTo};
use specs::prelude::*;
use std::cmp;

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Location>,
        WriteStorage<'a, WantsToMoveTo>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut locations, mut move_tos) = data;

        for (mut location, move_to) in (&mut locations, &move_tos).join() {
            location.x = cmp::min(cmp::max(move_to.x, 0), map.width - 1);
            location.y = cmp::min(map.height - 1, cmp::max(move_to.y, 0));
        }

        move_tos.clear();
    }
}
