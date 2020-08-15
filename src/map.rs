use super::{create_knife, create_tree};
use oorandom::Rand32;
use specs::prelude::*;

const MAXTREES: i32 = 30;
const MAXKNIVES: i32 = 1;

pub struct Map {
    pub width: i32,
    pub height: i32,
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
