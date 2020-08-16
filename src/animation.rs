use super::{Disappearing, GraphicAnimatable, GraphicRenderable};
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
