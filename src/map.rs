use crate::components::{Location, WantsToMoveTo};
use oorandom::Rand32;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::ops::Range;

pub fn euclidean_distance(a: &Location, b: &Location) -> f64 {
    ((a.x - b.x).pow(2) as f64 + (a.y - b.y).pow(2) as f64).sqrt()
}

/// Determine if two locations are close enough where we'd consider it reasonable for
/// the player to pick up an item.
pub fn within_pickup_distance(a: &Location, b: &Location) -> bool {
    euclidean_distance(&a, &b) < 5.0
}

#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum TileType {
    Void,
    Water,
    Sand,
    Grass,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<TileType>>,
}

/// Generate a map containing void tiles where we don't want the isometric map rendering.
/// I chose this over some sort of tile mask because I don't want to send all that mask
/// data to the clients.
fn blank_isometric_map(width: usize, height: usize) -> Vec<Vec<TileType>> {
    let mut void_map = vec![vec![TileType::Void; height as usize]; width as usize];
    for index in 0..min(width - 1, height - 1) {
        void_map[index][index] = TileType::Water;
        for span in 1..(min(width, height) as f64 / 2.1) as usize {
            if index + span < width - 1 {
                void_map[index + span][index] = TileType::Water;
            }
            if index + span < height - 1 {
                void_map[index][index + span] = TileType::Water;
            }
        }
    }

    void_map
}

impl Map {
    /// Generate new map by running 3 cellular automata simulations
    /// One which defines the sand, and two which define the grass at different
    /// densities. I hate this, but it's my first time doing something like this
    /// so I'm giving up for now.
    pub fn new(mut rng: &mut Rand32, width: i32, height: i32) -> Map {
        let blank_tiles = blank_isometric_map(width as usize, height as usize);
        let mut map = Map {
            width,
            height,
            tiles: blank_tiles.clone(),
        };
        let mut grass_map = Map {
            width,
            height,
            tiles: blank_tiles.clone(),
        };
        let mut grass_dense_map = Map {
            width,
            height,
            tiles: blank_tiles,
        };

        // Generate the island
        cellular_automata_map(&mut map, &mut rng, 9);

        // Generate less dense, outer grass map
        cellular_automata_map(&mut grass_map, &mut rng, 8);

        let pct_grass = 0.80;
        let pct_sand = (1_f64 - pct_grass) / 2_f64;
        for x in 0usize..(grass_map.width as usize - 1usize) {
            for y in 0usize..(grass_map.width as usize - 1usize) {
                if grass_map.tiles[x][y] != TileType::Water
                    && grass_map.tiles[x][y] != TileType::Void
                {
                    let scaled_x =
                        (x as f64 * pct_grass + grass_map.width as f64 * pct_sand) as usize;
                    let scaled_y =
                        (y as f64 * pct_grass + grass_map.height as f64 * pct_sand) as usize;
                    if map.tiles[scaled_x][scaled_y] != TileType::Water {
                        map.tiles[scaled_x][scaled_y] = TileType::Grass;
                    }
                }
            }
        }

        // Generate a more dense, inner grass map
        cellular_automata_map(&mut grass_dense_map, &mut rng, 18);

        let pct_grass = 0.50;
        let pct_sand = (1_f64 - pct_grass) / 2_f64;
        for x in 0..width - 1 {
            for y in 0..height - 1 {
                if grass_dense_map.tiles[x as usize][y as usize] != TileType::Water
                    && grass_dense_map.tiles[x as usize][y as usize] != TileType::Void
                {
                    let scaled_x =
                        (x as f64 * pct_grass + grass_dense_map.width as f64 * pct_sand) as usize;
                    let scaled_y =
                        (y as f64 * pct_grass + grass_dense_map.height as f64 * pct_sand) as usize;
                    if map.tiles[scaled_x][scaled_y] != TileType::Water {
                        map.tiles[scaled_x][scaled_y] = TileType::Grass;
                    }
                }
            }
        }

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
    } else if map.tiles[wants_to_move.x as usize][wants_to_move.y as usize] == TileType::Void {
        return false; // Cannot travel to void
    }
    true
}

/// Return a location which is not TileType::Water
pub fn get_random_location_of_tile(
    map: &Map,
    rng: &mut Rand32,
    tile_type: Option<TileType>,
) -> Location {
    let mut x;
    let mut y;
    // Right side of island is where we spawn, so limit x range search for Sand
    let x_range = match tile_type {
        Some(TileType::Sand) => Range {
            start: (map.width as f64 * 0.90) as u32,
            end: map.width as u32 - 1,
        },
        _ => Range {
            start: 1 as u32,
            end: map.width as u32 - 1,
        },
    };
    let y_range = Range {
        start: 1 as u32,
        end: map.width as u32 - 1,
    };
    loop {
        x = rng.rand_range(x_range.clone());
        y = rng.rand_range(y_range.clone());
        let cur_tile = map.tiles[x as usize][y as usize];
        if tile_type.is_none() && cur_tile != TileType::Void && cur_tile != TileType::Water {
            break;
        } else if tile_type.is_some() && cur_tile == tile_type.unwrap() {
            break;
        }
    }
    Location {
        x: x as i32,
        y: y as i32,
    }
}

/// Modify the tiles structure to create something that looks like an island
fn cellular_automata_map(map: &mut Map, rng: &mut Rand32, iterations: i32) {
    for x in 0usize..(map.width as usize - 1usize) {
        for y in 0usize..(map.width as usize - 1usize) {
            if x < 1 || x as i32 > map.width - 1 || y < 1 || y as i32 > map.height - 1 {
                continue;
            }
            if map.tiles[x][y] == TileType::Void {
                continue; // We should never place anything on the void
            }
            if rng.rand_float() > 0.55 {
                map.tiles[x][y] = TileType::Sand;
            }
        }
    }

    // Iteratively apply cellular automata rules
    for _ in 0..iterations {
        let mut new_tiles = map.tiles.clone();

        for x in 0usize..(map.width as usize - 1usize) {
            for y in 0usize..(map.width as usize - 1usize) {
                if map.tiles[x][y] == TileType::Void {
                    continue; // We should never place anything on the void
                }
                if x < 1 || x as i32 > map.width - 1 || y < 1 || y as i32 > map.height - 1 {
                    continue;
                }
                let mut neighbors = 0;
                if map.tiles[x - 1][y] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x + 1][y] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x][y - 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x][y + 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x - 1][y - 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x + 1][y - 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x - 1][y + 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }
                if map.tiles[x + 1][y + 1] as i32 <= TileType::Water as i32 {
                    neighbors += 1
                }

                if neighbors > 4 {
                    new_tiles[x][y] = TileType::Water;
                } else {
                    new_tiles[x][y] = TileType::Sand;
                }
            }
        }
        map.tiles = new_tiles.clone();
    }
}
