use crate::components::{Disappearing, GraphicAnimatable, GraphicRenderable, WantsToBePickedUp};
use specs::prelude::*;

pub struct AnimationSystem {}

impl<'a> System<'a> for AnimationSystem {
    type SystemData = (
        WriteStorage<'a, GraphicRenderable>,
        WriteStorage<'a, GraphicAnimatable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut renderables, mut animatables) = data;

        for (render, animation) in (&mut renderables, &mut animatables).join() {
            // Tick the animation if we're still waiting
            if animation.ticks != animation.tick_interval {
                animation.ticks += 1;
                continue;
            }
            // If we're done waiting, swap to the next graphic
            animation.ticks = 0;
            let mut next_index = 1 + animation
                .image_names
                .iter()
                .position(|r| r == &render.image_name)
                .unwrap();
            if animation.image_names.len() == next_index {
                next_index = 0; // Wrap around to the first image
            }

            // Update the render
            render.image_name = animation.image_names.get(next_index).unwrap().clone();
        }
    }
}

pub struct DisappearingSystem {}

impl<'a> System<'a> for DisappearingSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Disappearing>,
        ReadStorage<'a, WantsToBePickedUp>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut disappearings, pickups) = data;

        for (entity, mut disappearing) in (&entities, &mut disappearings).join() {
            if let Some(_) = pickups.get(entity) {
                continue; // Don't disappear if it's an item that wants to be carried.
            }
            disappearing.ticks_left -= 1;
            if disappearing.ticks_left == 0 {
                entities
                    .delete(entity)
                    .expect("Failed to delete disappearing entity");
            }
        }
    }
}
