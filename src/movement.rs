use crate::components::{Location, WantsToMoveTo};
use crate::map::Map;
use specs::prelude::*;

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Location>,
        ReadStorage<'a, WantsToMoveTo>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (_map, mut locations, move_tos) = data;

        for (mut location, move_to) in (&mut locations, &move_tos).join() {
            let speed = move_to.speed as i32;
            let dist_x = move_to.x - location.x;
            let dist_y = move_to.y - location.y;
            let mut new_x = location.x;
            let mut new_y = location.y;
            if dist_x == 0 && dist_y == 0 {
                continue; // This entity has reached its desination
            } else if dist_x.abs() > dist_y.abs() {
                if speed > dist_x.abs() {
                    new_x = move_to.x;
                } else if dist_x > 0 {
                    new_x += speed;
                } else {
                    new_x -= speed;
                }
            } else {
                if speed > dist_y.abs() {
                    new_y = move_to.y;
                } else if dist_y > 0 {
                    new_y += speed;
                } else {
                    new_y -= speed;
                }
            }
            location.x = new_x;
            location.y = new_y;
        }
    }
}
