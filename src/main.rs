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
use ferris_chat::saveload_system::load_game;
use ferris_chat::state::{handle_chat_input, handle_click, handle_input, initialize_ecs, State};

pub struct GUIComponents {
    pub fps_tracker: FPSTracker,
}

pub fn handle_client_click(mut ecs: &mut World, x: i32, y: i32, for_name: String) {
    let relative_x;
    let relative_y;
    {
        let rect = ecs
            .fetch::<Canvas>()
            .ctx
            .get_canvas()
            .get_bounding_client_rect();
        let canvas = ecs.fetch::<Canvas>();
        relative_x = ((x as f64 - rect.get_left() as f64) / canvas.scaled_width) as i32;
        relative_y = ((y as f64 - rect.get_top() as f64) / canvas.scaled_height) as i32;
    }
    handle_click(&mut ecs, relative_x, relative_y, for_name)
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
        handle_chat_input(&mut ecs, &chat_msg, String::from("Ferris"));
    }
    stdweb::web::window().local_storage().remove("chat_input");
}

fn rendering_tick(state: &mut State, gui: &mut GUIComponents) {
    let remote_session_save_state = stdweb::web::window()
        .local_storage()
        .get("save_state")
        .expect("Cannot read local storage");
    if remote_session_save_state != "" {
        // If save state exists in local storage, then we're connected to a remote session.
        // Load that into our ECS instead of running systems manually.
        load_game(&mut state.ecs, remote_session_save_state);
    } else {
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
}

fn main() {
    stdweb::initialize();

    let width: i32 = 100;
    let height: i32 = 100;

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
    );

    js! {
        // Clear save_state storage in case server is not up
        window.localStorage.setItem("save_state", "");

        // Attempt to connect to server
        let socket = new WebSocket("ws://127.0.0.1:3012");

        // socket.onopen = function(e) {
        //     socket.send(@{msg});
        // };

        socket.onmessage = function(event) {
            console.log("save data received");
            window.localStorage.setItem("save_state", event.data);
        };
    }

    // Canvas is where we do all our rendering
    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: KeyDownEvent| {
            handle_input(
                &mut gs.borrow_mut().ecs,
                event.key().as_ref(),
                String::from("Ferris"),
            );
        }
    });

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: ClickEvent| {
            handle_client_click(
                &mut gs.borrow_mut().ecs,
                event.client_x(),
                event.client_y(),
                String::from("Ferris"),
            );
        }
    });

    // Recurive main loop because that's the only way I've found to do it in stdweb
    fn game_loop(gs: Rc<RefCell<State>>, gui: Rc<RefCell<GUIComponents>>, time: u32) {
        let gs = gs.clone();
        let gui = gui.clone();
        stdweb::web::set_timeout(
            move || {
                game_loop(gs.clone(), gui.clone(), time);
                rendering_tick(&mut gs.borrow_mut(), &mut gui.borrow_mut());
            },
            time,
        );
    }

    game_loop(gs, gui, 100);

    stdweb::event_loop();
}
