use super::{Location, Renderable, TextRenderable};
use oorandom::Rand32;
use specs::prelude::*;

const MAXTREES: i32 = 30;
const MAXKNIVES: i32 = 1;

pub struct Map {
    pub width: i32,
    pub height: i32,
}

fn create_knife(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable {
            text_renderable: Some(TextRenderable {
                text: String::from("🗡"),
                offset_x: -0.25f64,
                offset_y: -1.25f64,
            }),
            graphic_renderable: None,
            render_order: 1,
        })
        .build();
}

fn create_tree(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable {
            text_renderable: Some(TextRenderable {
                text: String::from("🌴"),
                offset_x: 1.25f64,
                offset_y: -1.25f64,
            }),
            graphic_renderable: None,
            render_order: 5,
        })
        .build();
}

pub fn generate_map(ecs: &mut World, map: &Map) {
    let mut rng = Rand32::new(1u64);
    for _ in 0..MAXTREES {
        create_tree(
            ecs,
            (rng.rand_float() * (map.width - 1) as f32) as i32,
            (rng.rand_float() * (map.height - 1) as f32) as i32,
        );
    }
    for _ in 0..MAXKNIVES {
        create_knife(
            ecs,
            (rng.rand_float() * (map.width - 1) as f32) as i32,
            (rng.rand_float() * (map.height - 1) as f32) as i32,
        );
    }
}
