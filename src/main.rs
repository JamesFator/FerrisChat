extern crate serde;
#[macro_use]
extern crate stdweb;

use specs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::traits::*;
use stdweb::web::{event::ClickEvent, event::KeyDownEvent, Date, IEventTarget};

mod canvas;
use canvas::{Canvas, DrawSystem};
use ferris_chat::components::*;
use ferris_chat::entities::*;
use ferris_chat::saveload_system::{load_game, serialize_player_input, PlayerInput};
use ferris_chat::state::{handle_chat_input, handle_click, handle_input, initialize_ecs, State};

pub struct GUIComponents {
    pub fps_tracker: FPSTracker,
}

fn handle_client_input(mut ecs: &mut World, input: &str) {
    match input {
        "p" => {}
        _ => return,
    }

    let player_id = ecs.fetch::<String>().to_string();
    let player_input = PlayerInput::SpecialInput {
        id: player_id.clone(),
        input: input.into(),
    };
    stdweb::web::window()
        .local_storage()
        .insert("player_input", &serialize_player_input(player_input))
        .expect("Failed to write player_input to local_storage");
    handle_input(&mut ecs, input, &player_id);
}

fn handle_client_click(mut ecs: &mut World, screen_x: i32, screen_y: i32) {
    let player_id;
    let x: i32;
    let y: i32;
    {
        player_id = ecs.fetch::<String>().to_string();
        let rect = ecs
            .fetch::<Canvas>()
            .ctx
            .get_canvas()
            .get_bounding_client_rect();
        let canvas = ecs.fetch::<Canvas>();
        let iso_coordinates = canvas.convert_from_screen(
            screen_x as f64,
            screen_y as f64,
            rect.get_top(),
            rect.get_left(),
        );
        x = iso_coordinates.0 as i32;
        y = iso_coordinates.1 as i32;
    }
    let player_input = PlayerInput::Click {
        id: player_id.clone(),
        x,
        y,
    };
    stdweb::web::window()
        .local_storage()
        .insert("player_input", &serialize_player_input(player_input))
        .expect("Failed to write player_input to local_storage");
    handle_click(&mut ecs, x, y, &player_id);
}

/// System for tracking FPS. In main file because depends on stdweb.
fn update_fps_tracker(mut ecs: &mut World, fps_tracker: &mut FPSTracker) {
    // Update the tracker data
    {
        let now = Date::new().get_seconds() as u64;
        if fps_tracker.for_time != now {
            fps_tracker.for_time = now;
            fps_tracker.prev_fps = fps_tracker.seen_frames;
            fps_tracker.seen_frames = 0;
        }
        fps_tracker.seen_frames += 1;
    }
    add_fps_tracker(&mut ecs, &fps_tracker);
}

fn read_from_local_storage(mut ecs: &mut World) {
    // Check for chat_input
    let chat_input = stdweb::web::window().local_storage().get("chat_input");
    if let Some(chat_msg) = chat_input {
        if chat_msg.len() == 0 {
            return; // Don't render a bubble if nothing was said
        }
        let player_id = ecs.fetch::<String>().to_string();
        let player_input = PlayerInput::Chat {
            id: player_id.clone(),
            message: chat_msg.clone(),
        };
        stdweb::web::window()
            .local_storage()
            .insert("player_input", &serialize_player_input(player_input))
            .expect("Failed to write player_input to local_storage");
        handle_chat_input(&mut ecs, &chat_msg, &player_id);
    }
    stdweb::web::window().local_storage().remove("chat_input");
}

fn create_player(mut ecs: &mut World) {
    let player_id = ecs.fetch::<String>().to_string();
    if get_player_with_id(&ecs, &player_id).is_some() {
        return; // Player is alive and healthy
    }
    let player_name = match stdweb::web::window().local_storage().get("player_name") {
        Some(name) => name.to_string(),
        None => String::from(""),
    };
    let player_input = PlayerInput::CreatePlayer {
        id: player_id.clone(),
        name: player_name.clone(),
    };
    stdweb::web::window()
        .local_storage()
        .insert("player_input", &serialize_player_input(player_input))
        .expect("Failed to write player_input to local_storage");
    spawn_crab(&mut ecs, &player_id, &player_name, false);
}

fn rendering_tick(state: &mut State, gui: &mut GUIComponents) {
    let remote_session_save_state = stdweb::web::window().local_storage().get("save_state");
    let mut remote_session = false;
    if let Some(save_state) = remote_session_save_state {
        if save_state.len() > 0 {
            remote_session = true;
            // If save state exists in local storage, then we're connected to a remote session.
            // Load that into our ECS instead of running systems manually.
            load_game(&mut state.ecs, save_state);
            stdweb::web::window().local_storage().remove("save_state");
        }
    }
    if !remote_session {
        // If no remote sesson save state, then run our ECS locally.
        state.tick();
    }

    // Check the window local storage for updates
    read_from_local_storage(&mut state.ecs);

    // Update the FPS GUI
    update_fps_tracker(&mut state.ecs, &mut gui.fps_tracker);

    // Invoke the draw system last
    let mut draw_system = DrawSystem {};
    draw_system.run_now(&state.ecs);

    // Check if our character is alive. If not, create them
    create_player(&mut state.ecs);
}

fn main() {
    stdweb::initialize();

    let width: i32 = 100;
    let height: i32 = 100;
    // TODO: Make this a better UUID. If two people join at the same time, they'll clash
    let player_id = format!("{}", Date::new().get_seconds());

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    let gui = Rc::new(RefCell::new(GUIComponents {
        fps_tracker: FPSTracker {
            for_time: 0,
            seen_frames: 0,
            prev_fps: 0,
        },
    }));
    initialize_ecs(
        &mut gs.borrow_mut().ecs,
        width,
        height,
        Date::new().get_seconds() as u64,
        // 1,
    );

    js! {
        var player_name = prompt("Please enter your crab's name");
        window.localStorage.setItem("player_name", player_name);

        // Clear save_state storage in case server is not up
        window.localStorage.setItem("save_state", "");

        // Attempt to connect to server
        var socket = new WebSocket("ws://192.168.1.83:3012");

        socket.onmessage = function(event) {
            console.log("save data received");
            window.localStorage.setItem("save_state", event.data);

            // Send player_input back
            var player_input = window.localStorage.getItem("player_input");
            if (player_input !== null && player_input != "") {
                socket.send(player_input);
                window.localStorage.setItem("player_input", "");
            }
        };
    }

    // Canvas is where we do all our rendering
    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    // Insert the current player ID
    gs.borrow_mut().ecs.insert(player_id.clone());

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: KeyDownEvent| {
            handle_client_input(&mut gs.borrow_mut().ecs, event.key().as_ref());
        }
    });

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: ClickEvent| {
            handle_client_click(&mut gs.borrow_mut().ecs, event.client_x(), event.client_y());
        }
    });

    // Recurive main loop because that's the only way I've found to do it in stdweb
    fn game_loop(gs: Rc<RefCell<State>>, gui: Rc<RefCell<GUIComponents>>, time: u32) {
        let gs = gs.clone();
        let gui = gui.clone();
        stdweb::web::set_timeout(
            move || {
                game_loop(gs.clone(), gui.clone(), 100);
                rendering_tick(&mut gs.borrow_mut(), &mut gui.borrow_mut());
            },
            time,
        );
    }

    game_loop(gs, gui, 10);

    stdweb::event_loop();
}
