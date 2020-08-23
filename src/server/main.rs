extern crate serde;

use specs::prelude::*;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use ferris_chat::saveload_system::{deserialize_player_input, package_save_state};
use ferris_chat::state::{initialize_ecs, State};
mod websocket_server;
use websocket_server::{start_async_server, AsyncStatePtr};

fn start_game_engine(
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
    shared_input_queue: AsyncStatePtr,
) {
    let width: i32 = 100;
    let height: i32 = 100;

    let mut gs = State { ecs: World::new() };
    initialize_ecs(&mut gs.ecs, width, height, 1 as u64);
    // Cache the serialized map because that never changes
    let serialized_map = gs.get_serialized_map();

    loop {
        {
            // Process the player input queue
            let mut input_queue = shared_input_queue.lock().unwrap();
            for player_input in input_queue.iter() {
                println!("Received input: {:?}", player_input);
                gs.handle_player_input(deserialize_player_input((&player_input).to_string()));
            }
            input_queue.clear();
        }

        gs.tick();

        // Serialize our ECS
        let serialized_ecs = gs.get_serialized_ecs();

        {
            // Save a copy of the incremental state for existing clients
            let mut incr_state_mut = shared_incr_state.lock().unwrap();
            incr_state_mut.clear();
            incr_state_mut.push(package_save_state(serialized_ecs.clone(), None));
            // Save a copy of the full state for new clients
            let mut full_state_mut = shared_full_state.lock().unwrap();
            full_state_mut.clear();
            full_state_mut.push(package_save_state(
                serialized_ecs,
                Some(serialized_map.clone()),
            ));
        }

        // println!("tick");
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn main() {
    // This is my shitty way to sync save data between threads. I'm new to rust,
    // so I have no idea what I'm doing and if this is bad.
    let shared_full_state = Arc::new(Mutex::new(Vec::with_capacity(1)));
    let shared_incr_state = Arc::new(Mutex::new(Vec::with_capacity(1)));
    let shared_input_queue = Arc::new(Mutex::new(Vec::new()));

    // Start listening for client connections in a new thread
    start_async_server(
        shared_full_state.clone(),
        shared_incr_state.clone(),
        shared_input_queue.clone(),
    );

    // Block while running the game engine
    start_game_engine(
        shared_full_state.clone(),
        shared_incr_state.clone(),
        shared_input_queue.clone(),
    );
}
