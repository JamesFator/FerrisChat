use super::{create_knife, create_tree, Location, STANDARD_TILE};
use oorandom::Rand32;
use specs::prelude::*;
use std::cmp;
use std::ops::Range;

const MAXTREES: i32 = 30;
const MAXKNIVES: i32 = 1;
const EDGE_BUFFER: f64 = STANDARD_TILE / 2_f64;

pub struct Map {
    pub width: i32,
    pub height: i32,
}

/// Given x+y, return location closest to desired where an entity can travel.
/// None if user clicked off canvas.
pub fn closest_valid_map_location(map: &Map, location: Location) -> Option<Location> {
    let edge_buffer = EDGE_BUFFER as i32;
    if location.x < 0 || location.x > map.width {
        return None;
    } else if location.y < 0 || location.y > map.height {
        return None;
    }
    Some(Location {
        x: cmp::min(cmp::max(location.x, edge_buffer), map.width - edge_buffer),
        y: cmp::min(map.height - edge_buffer, cmp::max(location.y, edge_buffer)),
    })
}

pub fn generate_map(ecs: &mut World, map: &Map) {
    let edge_buffer = EDGE_BUFFER as u32;
    let width = map.width as u32;
    let height = map.height as u32;
    let mut rng = Rand32::new(1u64);
    for _ in 0..MAXTREES {
        create_tree(
            ecs,
            rng.rand_range(Range {
                start: edge_buffer,
                end: width - edge_buffer,
            }) as i32,
            rng.rand_range(Range {
                start: edge_buffer,
                end: height - edge_buffer,
            }) as i32,
        );
    }
    for _ in 0..MAXKNIVES {
        create_knife(
            ecs,
            rng.rand_range(Range {
                start: edge_buffer,
                end: width - edge_buffer,
            }) as i32,
            rng.rand_range(Range {
                start: edge_buffer,
                end: height - edge_buffer,
            }) as i32,
        );
    }
}
