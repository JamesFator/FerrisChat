use crate::components::*;
use crate::map::{get_random_location_of_tile, Map, TileType};
use oorandom::Rand32;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::ops::Range;

const MAXTREES: i32 = 20;
const MAXKNIVES: i32 = 1;

pub fn get_entity_for_name(ecs: &World, name: &String) -> Option<Entity> {
    let entities = ecs.entities();
    let player_infos = ecs.read_storage::<PlayerInfo>();
    for (entity, player_info) in (&entities, &player_infos).join() {
        if player_info.name == *name {
            return Some(entity);
        }
    }
    return None;
}

//
// GUI entities
//

pub fn add_fps_tracker(ecs: &mut World, tracker: &FPSTracker) {
    // First delete any existing trackers so we don't write over eachother
    {
        let entities = ecs.entities();
        let mut existing_tracker: Option<Entity> = None;
        {
            let fps_trackers = ecs.read_storage::<FPSTracker>();
            for (entity, _) in (&entities, &fps_trackers).join() {
                existing_tracker = Some(entity);
                break;
            }
        }
        if let Some(entity) = existing_tracker {
            entities
                .delete(entity)
                .expect("Failed to delete existing FPSTracker");
        }
    }
    ecs.create_entity()
        .with(tracker.clone())
        .with(Location { x: 5, y: 5 })
        .with(Renderable { render_order: 0 })
        .with(TextRenderable {
            text: format!("FPS: {}", tracker.prev_fps),
            font_size: 20_f64,
            offset_x: 0_f64,
            offset_y: 0_f64,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}

//
// Crab entities
//

pub fn spawn_crab(mut ecs: &mut World, map: &Map, mut rng: &mut Rand32, name: &str, ai: bool) {
    let location = match ai {
        true => get_random_location_of_tile(&map, &mut rng, TileType::Grass),
        false => get_random_location_of_tile(&map, &mut rng, TileType::Sand),
    };
    let mut crab = ecs
        .create_entity()
        .with(location)
        .with(PlayerInfo {
            name: String::from(name),
        })
        .with(Renderable { render_order: 2 })
        .with(TextRenderable {
            text: String::from(name),
            font_size: 20_f64,
            offset_x: 0_f64,
            offset_y: 5_f64,
        })
        .with(GraphicRenderable {
            image_name: String::from("rustacean_right"),
            offset_x: -5_f64,
            offset_y: -5_f64,
        })
        .with(GraphicAnimatable {
            image_names: vec![
                String::from("rustacean_right"),
                String::from("rustacean_left"),
            ],
            tick_interval: 2,
            ticks: 0,
        });

    if ai {
        let crab_state = if rng.rand_float() > 0.5 {
            CrabAIState::SleepingRight
        } else {
            CrabAIState::SleepingLeft
        };
        crab = crab.with(CrabAI {
            crab_state: crab_state,
            tick_interval: rng.rand_range(Range { start: 20, end: 30 }) as i16,
            ticks: 0,
            walk_speed: 1,
            sleep_duration: rng.rand_range(Range { start: 15, end: 20 }) as i16,
        });
    }

    let entity = crab.marked::<SimpleMarker<EntityMarker>>().build();

    // It not AI, then player character, so spawn on beach
    if !ai {
        create_wave_for_entity(&mut ecs, &entity);
    }
}

fn create_wave_for_entity(ecs: &mut World, for_entity: &Entity) {
    ecs.create_entity()
        .with(CarriedBy { owner: *for_entity })
        .with(Renderable { render_order: 0 })
        .with(GraphicRenderable {
            image_name: String::from("water_wave_0"),
            offset_x: -5_f64,
            offset_y: -5_f64,
        })
        .with(GraphicAnimatable {
            image_names: vec![
                String::from("water_wave_0"),
                String::from("water_wave_1"),
                String::from("water_wave_2"),
                String::from("water_wave_2"),
            ],
            tick_interval: 3,
            ticks: 0,
        })
        .with(Disappearing {
            total_ticks: 9,
            ticks_left: 15,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}

pub fn create_chat_bubble(ecs: &mut World, text: String, for_entity: Entity) {
    // Delete any other chat bubbles this player has had up until now
    {
        let entities = ecs.entities();
        let carried_bys = ecs.read_storage::<CarriedBy>();
        let mut chats_to_remove = Vec::new();
        for (entity, carried_by) in (&entities, &carried_bys).join() {
            if for_entity.eq(&carried_by.owner) {
                chats_to_remove.push(entity);
            }
        }
        for entity in chats_to_remove {
            entities
                .delete(entity)
                .expect("Could not delete chat bubble");
        }
    }
    ecs.create_entity()
        .with(CarriedBy { owner: for_entity })
        .with(Renderable { render_order: 0 })
        .with(ChatRenderable {
            text: text,
            offset_x: -5_f64,
            offset_y: -10_f64,
        })
        .with(Disappearing {
            total_ticks: 100,
            ticks_left: 100,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}

//
// World entities
//

pub fn create_knife(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable { render_order: 3 })
        .with(TextRenderable {
            text: String::from("🔪"),
            font_size: 40_f64,
            offset_x: 5_f64,
            offset_y: 0_f64,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}

pub fn create_tree(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable { render_order: 4 })
        .with(TextRenderable {
            text: String::from("🌴"),
            font_size: 40_f64,
            offset_x: 0_f64,
            offset_y: 0_f64,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}

/// Fill the map with entities
pub fn fill_map(ecs: &mut World, map: &Map, mut rng: &mut Rand32) {
    for _ in 0..MAXTREES {
        let location = get_random_location_of_tile(&map, &mut rng, TileType::Grass);
        create_tree(ecs, location.x, location.y);
    }
    for _ in 0..MAXKNIVES {
        let location = get_random_location_of_tile(&map, &mut rng, TileType::Grass);
        create_knife(ecs, location.x, location.y);
    }
}

pub fn create_poop(ecs: &mut World, location: Location) {
    ecs.create_entity()
        .with(location)
        .with(Renderable { render_order: 3 })
        .with(TextRenderable {
            text: String::from("💩"),
            font_size: 20_f64,
            offset_x: 0_f64,
            offset_y: 3_f64,
        })
        .with(Disappearing {
            total_ticks: 100,
            ticks_left: 100,
        })
        .marked::<SimpleMarker<EntityMarker>>()
        .build();
}
