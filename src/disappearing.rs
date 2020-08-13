use super::Disappearing;
use specs::prelude::*;

pub struct DisappearingSystem {}

impl<'a> System<'a> for DisappearingSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, Disappearing>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut disappearings) = data;

        for (entity, mut disappearing) in (&entities, &mut disappearings).join() {
            disappearing.ticks_left -= 1;
            if disappearing.ticks_left == 0 {
                entities
                    .delete(entity)
                    .expect("Failed to delete disappearing entity");
            }
        }
    }
}
