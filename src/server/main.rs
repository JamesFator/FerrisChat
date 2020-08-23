extern crate serde;

use specs::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tungstenite::handshake::HandshakeRole;
use tungstenite::{accept, Error, HandshakeError, Message, Result};

use ferris_chat::saveload_system::package_save_state;
use ferris_chat::state::{initialize_ecs, State};

type AsyncStatePtr = Arc<Mutex<Vec<String>>>;

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f,
    }
}

fn handle_client(
    stream: TcpStream,
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
) -> Result<()> {
    let mut socket = accept(stream).map_err(must_not_block)?;
    println!("Accepted client");
    // Send the inital package which includes the map
    let full_save_state = shared_full_state.lock().unwrap().get(0).unwrap().clone();
    socket
        .write_message(Message::text(full_save_state))
        .expect("Failed to write message");
    loop {
        // Send the client the latest information
        let save_state = shared_incr_state.lock().unwrap().get(0).unwrap().clone();
        socket
            .write_message(Message::text(save_state))
            .expect("Failed to write message");
        // match socket.read_message()? {
        //     msg @ Message::Text(_) => {
        //         socket.write_message(msg)?;
        //     }
        //     Message::Close(_) => {
        //         println!("Client left");
        //     }
        //     _ => {}
        // }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn start_server_listener_thread(
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
) {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();

    thread::spawn(move || {
        for stream in server.incoming() {
            let shared_full_state = shared_full_state.clone();
            let shared_incr_state = shared_incr_state.clone();
            thread::spawn(move || match stream {
                Ok(stream) => {
                    if let Err(err) =
                        handle_client(stream, shared_full_state.clone(), shared_incr_state.clone())
                    {
                        match err {
                            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                            e => println!("Error!: {}", e),
                        }
                    }
                }
                Err(e) => println!("Error accepting stream: {}", e),
            });
        }
    });
}

fn start_game_engine(shared_full_state: AsyncStatePtr, shared_incr_state: AsyncStatePtr) {
    let width: i32 = 100;
    let height: i32 = 100;

    let mut gs = State { ecs: World::new() };
    initialize_ecs(&mut gs.ecs, width, height, 1 as u64);
    // Cache the serialized map because that never changes
    let serialized_map = gs.get_serialized_map();

    loop {
        gs.tick();

        // Serialize our ECS
        let serialized_ecs = gs.get_serialized_ecs();

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

        // println!("tick");
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn main() {
    // This is my shitty way to sync save data between threads. I'm new to rust,
    // so I have no idea what I'm doing and if this is bad.
    let shared_full_state = Arc::new(Mutex::new(Vec::with_capacity(1)));
    let shared_incr_state = Arc::new(Mutex::new(Vec::with_capacity(1)));

    // Start listening for client connections in a new thread
    start_server_listener_thread(shared_full_state.clone(), shared_incr_state.clone());

    // Block while running the game engine
    start_game_engine(shared_full_state.clone(), shared_incr_state.clone());
}
