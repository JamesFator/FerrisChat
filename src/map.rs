use super::{Location, WantsToMoveTo};
use oorandom::Rand32;
use std::ops::Range;

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
    pub fn new(mut rng: &mut Rand32, width: i32, height: i32) -> Map {
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

/// Return a location which is not TileType::Water
pub fn get_random_location_of_tile(map: &Map, rng: &mut Rand32, tile_type: TileType) -> Location {
    let mut x;
    let mut y;
    // Right side of island is where we spawn, so limit x range search for Sand
    let x_range = match tile_type {
        TileType::Sand => Range {
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
        if map.tiles[x as usize][y as usize] == tile_type {
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
