use futures_util::future::{select, Either};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tungstenite::{Message, Result};

use ferris_chat::saveload_system::{deserialize_player_input, serialize_player_input, PlayerInput};

pub type AsyncStatePtr = Arc<Mutex<Vec<String>>>;

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
    shared_input_queue: AsyncStatePtr,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    println!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    let mut player_id: Option<String> = None;

    // Send the inital package which includes the map
    let full_save_state = shared_full_state.lock().unwrap().get(0).unwrap().clone();
    ws_sender.send(Message::Text(full_save_state)).await?;
    let mut num_full_state_sends: i8 = 5;

    let mut msg_fut = ws_receiver.next();
    let mut tick_fut = interval.next();
    loop {
        match select(msg_fut, tick_fut).await {
            Either::Left((msg, tick_fut_continue)) => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() || msg.is_binary() {
                            if player_id.is_none() {
                                let player_input = deserialize_player_input(msg.to_string());
                                if let PlayerInput::CreatePlayer { id, name: _ } = player_input {
                                    player_id = Some(id);
                                }
                            }
                            // Push the message onto the player input queue
                            shared_input_queue.lock().unwrap().push(msg.to_string());
                        } else if msg.is_close() {
                            break;
                        }
                        tick_fut = tick_fut_continue; // Continue waiting for tick.
                        msg_fut = ws_receiver.next(); // Receive next WebSocket message.
                    }
                    None => break, // WebSocket stream terminated.
                };
            }
            Either::Right((_, msg_fut_continue)) => {
                let save_state;
                if num_full_state_sends > 0 {
                    // For the first few sends, send the full map to ensure they get the map
                    save_state = shared_full_state.lock().unwrap().get(0).unwrap().clone();
                    num_full_state_sends -= 1;
                } else {
                    save_state = shared_incr_state.lock().unwrap().get(0).unwrap().clone();
                }
                ws_sender.send(Message::Text(save_state)).await?;
                msg_fut = msg_fut_continue; // Continue receiving the WebSocket message.
                tick_fut = interval.next(); // Wait for next tick.
            }
        }
    }

    println!("Connection closed: {}", peer);

    // Queue input to delete their entity if any
    if let Some(player_id) = player_id {
        let delete_input = serialize_player_input(PlayerInput::DeletePlayer { id: player_id });
        shared_input_queue.lock().unwrap().push(delete_input);
    }

    Ok(())
}

async fn run(
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
    shared_input_queue: AsyncStatePtr,
) {
    let addr = "127.0.0.1:3012";
    let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(handle_connection(
            peer,
            stream,
            shared_full_state.clone(),
            shared_incr_state.clone(),
            shared_input_queue.clone(),
        ));
    }
}

pub fn start_async_server(
    shared_full_state: AsyncStatePtr,
    shared_incr_state: AsyncStatePtr,
    shared_input_queue: AsyncStatePtr,
) {
    thread::spawn(move || {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(run(
            shared_full_state.clone(),
            shared_incr_state.clone(),
            shared_input_queue.clone(),
        ));
    });
}
