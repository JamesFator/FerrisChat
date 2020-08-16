use super::{create_knife, create_tree, WantsToMoveTo};
use oorandom::Rand32;
use specs::prelude::*;
use std::ops::Range;

const MAXTREES: i32 = 20;
const MAXKNIVES: i32 = 1;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Water,
    Sand,
    Grass,
}

#[derive(PartialEq, Copy, Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<TileType>>,

    // Helper for a list of x,y iterations
    pub iter: Vec<Point>,
}

impl Map {
    /// Generate new map by running 3 cellular automata simulations
    /// One which defines the sand, and two which define the grass at different
    /// densities. I hate this, but it's my first time doing something like this
    /// so I'm giving up for now.
    pub fn new(ecs: &mut World, mut rng: &mut Rand32, width: i32, height: i32) -> Map {
        // Pre-build a vector of points to iterate over for code cleanlyness
        // later down the line. I don't know if this is a good idea or bad idea.
        let mut iter = Vec::new();
        for x in 0..width - 1 {
            for y in 0..height - 1 {
                iter.push(Point {
                    x: x as usize,
                    y: y as usize,
                });
            }
        }

        let mut map = Map {
            width,
            height,
            tiles: vec![vec![TileType::Water; height as usize]; width as usize],
            iter: iter.clone(),
        };
        let mut grass_map = Map {
            width,
            height,
            tiles: vec![vec![TileType::Water; height as usize]; width as usize],
            iter: iter.clone(),
        };
        let mut grass_dense_map = Map {
            width,
            height,
            tiles: vec![vec![TileType::Water; height as usize]; width as usize],
            iter: iter.clone(),
        };

        // Generate the island
        cellular_automata_map(&mut map, &mut rng, 9);

        // Generate less dense, outer grass map
        cellular_automata_map(&mut grass_map, &mut rng, 8);

        let pct_grass = 0.80;
        let pct_sand = (1_f64 - pct_grass) / 2_f64;
        for p in grass_map.iter.iter() {
            if grass_map.tiles[p.x][p.y] != TileType::Water {
                let scaled_x =
                    (p.x as f64 * pct_grass + grass_map.width as f64 * pct_sand) as usize;
                let scaled_y =
                    (p.y as f64 * pct_grass + grass_map.height as f64 * pct_sand) as usize;
                if map.tiles[scaled_x][scaled_y] != TileType::Water {
                    map.tiles[scaled_x][scaled_y] = TileType::Grass;
                }
            }
        }

        // Generate a more dense, inner grass map
        cellular_automata_map(&mut grass_dense_map, &mut rng, 18);

        let pct_grass = 0.50;
        let pct_sand = (1_f64 - pct_grass) / 2_f64;
        for p in grass_dense_map.iter.iter() {
            if grass_dense_map.tiles[p.x][p.y] != TileType::Water {
                let scaled_x =
                    (p.x as f64 * pct_grass + grass_dense_map.width as f64 * pct_sand) as usize;
                let scaled_y =
                    (p.y as f64 * pct_grass + grass_dense_map.height as f64 * pct_sand) as usize;
                if map.tiles[scaled_x][scaled_y] != TileType::Water {
                    map.tiles[scaled_x][scaled_y] = TileType::Grass;
                }
            }
        }

        // Add some initial entities to our map
        fill_map(ecs, &map, &mut rng);

        map
    }
}

/// Given x+y, return true if an entity can walk there. False if it's water or outside map
pub fn valid_walking_location(map: &Map, wants_to_move: &WantsToMoveTo) -> bool {
    if wants_to_move.x < 0 || wants_to_move.x > map.width - 1 {
        return false;
    } else if wants_to_move.y < 0 || wants_to_move.y > map.height - 1 {
        return false;
    } else if map.tiles[wants_to_move.x as usize][wants_to_move.y as usize] == TileType::Water {
        return false; // Cannot travel to water
    }
    true
}

/// Modify the tiles structure to create something that looks like an island
fn cellular_automata_map(map: &mut Map, rng: &mut Rand32, iterations: i32) {
    for p in map.iter.iter() {
        if p.x < 1 || p.x as i32 > map.width - 1 || p.y < 1 || p.y as i32 > map.height - 1 {
            continue;
        }
        if rng.rand_float() > 0.55 {
            map.tiles[p.x][p.y] = TileType::Sand;
        }
    }

    // Iteratively apply cellular automata rules
    for _ in 0..iterations {
        let mut new_tiles = map.tiles.clone();

        for p in map.iter.iter() {
            if p.x < 1 || p.x as i32 > map.width - 1 || p.y < 1 || p.y as i32 > map.height - 1 {
                continue;
            }
            let mut neighbors = 0;
            if map.tiles[p.x - 1][p.y] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x + 1][p.y] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x][p.y - 1] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x][p.y + 1] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x - 1][p.y - 1] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x + 1][p.y - 1] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x - 1][p.y + 1] == TileType::Water {
                neighbors += 1
            }
            if map.tiles[p.x + 1][p.y + 1] == TileType::Water {
                neighbors += 1
            }

            if neighbors > 4 {
                new_tiles[p.x][p.y] = TileType::Water;
            } else {
                new_tiles[p.x][p.y] = TileType::Sand;
            }
        }
        map.tiles = new_tiles.clone();
    }
}

/// Fill the map with entities
fn fill_map(ecs: &mut World, map: &Map, rng: &mut Rand32) {
    let width = map.width as u32;
    let height = map.height as u32;
    for _ in 0..MAXTREES {
        let x = rng.rand_range(Range {
            start: 10,
            end: width - 11,
        }) as i32;
        let y = rng.rand_range(Range {
            start: 10,
            end: height - 11,
        }) as i32;
        if map.tiles[x as usize][y as usize] != TileType::Water {
            create_tree(ecs, x, y);
        }
    }
    for _ in 0..MAXKNIVES {
        let x = rng.rand_range(Range {
            start: 10,
            end: width - 11,
        }) as i32;
        let y = rng.rand_range(Range {
            start: 10,
            end: height - 11,
        }) as i32;
        if map.tiles[x as usize][y as usize] != TileType::Water {
            create_knife(ecs, x, y);
        }
    }
}
