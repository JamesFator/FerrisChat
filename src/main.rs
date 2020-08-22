extern crate serde;
#[macro_use]
extern crate stdweb;

use specs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::traits::*;
use stdweb::web::{event::ClickEvent, event::KeyDownEvent, Date, IEventTarget};

use ferris_chat::canvas::{Canvas, DrawSystem};
use ferris_chat::components::*;
use ferris_chat::entities::*;
use ferris_chat::state::{handle_chat_input, handle_click, handle_input, initialize_ecs, State};

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
        relative_y = ((y as f64 - rect.get_left() as f64) / canvas.scaled_height) as i32;
    }
    handle_click(&mut ecs, relative_x, relative_y, for_name)
}

/// System for tracking FPS. In main file because depends on stdweb.
fn update_fps_tracker(ecs: &mut World) {
    let mut fps_trackers = ecs.write_storage::<FPSTracker>();
    let mut text_renders = ecs.write_storage::<TextRenderable>();

    let now = Date::new().get_seconds() as u64;
    for (mut fps_tracker, mut text_render) in (&mut fps_trackers, &mut text_renders).join() {
        if fps_tracker.for_time != now {
            fps_tracker.for_time = now;
            text_render.text = String::from(format!("FPS: {}", fps_tracker.seen_frames));
            fps_tracker.seen_frames = 0;
        }
        fps_tracker.seen_frames += 1;
    }
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

fn rendering_tick(state: &mut State) {
    // Update the FPS GUI
    update_fps_tracker(&mut state.ecs);

    // Check the window local storage for updates
    read_from_local_storage(&mut state.ecs);

    state.tick();

    // Invoke the draw system last
    let mut draw_system = DrawSystem {};
    draw_system.run_now(&state.ecs);
}

fn main() {
    stdweb::initialize();

    let width: i32 = 100;
    let height: i32 = 100;

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    initialize_ecs(
        &mut gs.borrow_mut().ecs,
        width,
        height,
        Date::new().get_seconds() as u64,
    );

    // Canvas is where we do all our rendering
    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    // Create game helper entities
    create_fps_tracker(&mut gs.borrow_mut().ecs);

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
    fn game_loop(gs: Rc<RefCell<State>>, time: u32) {
        let gs = gs.clone();
        stdweb::web::set_timeout(
            move || {
                game_loop(gs.clone(), time);
                rendering_tick(&mut gs.borrow_mut());
            },
            time,
        );
    }

    game_loop(gs, 100);

    stdweb::event_loop();
}
