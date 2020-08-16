extern crate serde;
extern crate stdweb;

use censor::*;
use specs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::traits::*;
use stdweb::web::{event::ClickEvent, event::KeyDownEvent, Date, IEventTarget};

mod canvas;
use canvas::{Canvas, DrawSystem};
mod components;
pub use components::*;
mod entities;
pub use entities::*;
mod map;
pub use map::{closest_valid_map_location, Map, TileType};
mod movement;
use movement::MovementSystem;
mod disappearing;
use disappearing::DisappearingSystem;
mod carry;
use carry::CarrySystem;

pub fn handle_input(ecs: &mut World, input: &str, for_entity: Entity) {
    let new_x;
    let new_y;
    {
        let locations = ecs.read_storage::<Location>();
        let location = locations
            .get(for_entity)
            .expect("Cannot find location for player");
        new_x = location.x;
        new_y = location.y;
    }

    match input {
        "p" => {
            create_poop(ecs, Location { x: new_x, y: new_y });
            return;
        }
        _ => return,
    };
}

pub fn handle_click(ecs: &mut World, x: i32, y: i32, for_entity: Entity) {
    let rect = ecs
        .fetch::<Canvas>()
        .ctx
        .get_canvas()
        .get_bounding_client_rect();
    let canvas = ecs.fetch::<Canvas>();
    let map = ecs.fetch::<Map>();
    let maybe_valid_location = closest_valid_map_location(
        &map,
        Location {
            x: ((x as f64 - rect.get_left() as f64) / canvas.scaled_width) as i32,
            y: ((y as f64 - rect.get_left() as f64) / canvas.scaled_height) as i32,
        },
    );
    if let Some(valid_location) = maybe_valid_location {
        let mut move_tos = ecs.write_storage::<WantsToMoveTo>();
        move_tos
            .insert(
                for_entity,
                WantsToMoveTo {
                    x: valid_location.x,
                    y: valid_location.y,
                },
            )
            .expect("Unable to insert WantsToMoveTo");
    }
}

/// Censor any profanity considering we're about to render the input
pub fn censor_chat_input(chat_input: &str) -> String {
    let censor = Censor::Standard + "cunk";
    censor.censor(chat_input)
}

pub struct State {
    pub ecs: World,
}

impl State {
    /// System for tracking FPS. In main file because depends on stdweb.
    fn update_fps_tracker(&mut self) {
        let mut fps_trackers = self.ecs.write_storage::<FPSTracker>();
        let mut text_renders = self.ecs.write_storage::<TextRenderable>();

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

    fn read_from_local_storage(&mut self) {
        // Check for chat_input
        let chat_input = stdweb::web::window().local_storage().get("chat_input");
        if let Some(chat_msg) = chat_input {
            if chat_msg.len() == 0 {
                return; // Don't render a bubble if nothing was said
            }
            let maybe_entity = get_entity_for_name(&self.ecs, String::from("Ferris"));
            if maybe_entity.is_some() {
                create_chat_bubble(
                    &mut self.ecs,
                    censor_chat_input(&chat_msg),
                    maybe_entity.unwrap(),
                );
            }
        }
        stdweb::web::window().local_storage().remove("chat_input");
    }

    fn run_systems(&mut self) {
        let mut movement_system = MovementSystem {};
        movement_system.run_now(&self.ecs);
        let mut disappearing_system = DisappearingSystem {};
        disappearing_system.run_now(&self.ecs);
        let mut carry_system = CarrySystem {};
        carry_system.run_now(&self.ecs);

        // Draw system should be last
        let mut draw_system = DrawSystem {};
        draw_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn tick(&mut self) {
        // Update the FPS GUI
        self.update_fps_tracker();

        // Check the window local storage for updates
        self.read_from_local_storage();

        // Run all our ECS systems
        self.run_systems();
    }
}

fn main() {
    stdweb::initialize();

    let width: i32 = 100;
    let height: i32 = 100;

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    gs.borrow_mut().ecs.register::<FPSTracker>();
    gs.borrow_mut().ecs.register::<Location>();
    gs.borrow_mut().ecs.register::<PlayerInfo>();
    gs.borrow_mut().ecs.register::<Renderable>();
    gs.borrow_mut().ecs.register::<TextRenderable>();
    gs.borrow_mut().ecs.register::<ChatRenderable>();
    gs.borrow_mut().ecs.register::<GraphicRenderable>();
    gs.borrow_mut().ecs.register::<WantsToMoveTo>();
    gs.borrow_mut().ecs.register::<Disappearing>();
    gs.borrow_mut().ecs.register::<CarriedBy>();

    // Create game helper entities
    create_fps_tracker(&mut gs.borrow_mut().ecs);

    // Create our crabs
    create_crab(&mut gs.borrow_mut().ecs, "Ferris", "red", 50, 50);
    create_crab(&mut gs.borrow_mut().ecs, "Geoff", "blue", 15, 15);
    create_crab(&mut gs.borrow_mut().ecs, "Tammy", "purple", 75, 80);

    // Add the seed for our random number generator
    gs.borrow_mut().ecs.insert(Date::new().get_seconds() as u64);

    // Canvas is where we do all our rendering
    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    // Map contains the map state
    let map = Map::new(&mut gs.borrow_mut().ecs, width, height);
    gs.borrow_mut().ecs.insert(map);

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: KeyDownEvent| {
            let for_entity = get_entity_for_name(&gs.borrow().ecs, String::from("Ferris"))
                .expect("Cannot find entity for event");
            handle_input(&mut gs.borrow_mut().ecs, event.key().as_ref(), for_entity);
        }
    });

    // Link keystrokes to player input via stdweb
    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: ClickEvent| {
            let for_entity = get_entity_for_name(&gs.borrow().ecs, String::from("Ferris"))
                .expect("Cannot find entity for event");
            handle_click(
                &mut gs.borrow_mut().ecs,
                event.client_x(),
                event.client_y(),
                for_entity,
            );
        }
    });

    // Recurive main loop because that's the only way I've found to do it in stdweb
    fn game_loop(gs: Rc<RefCell<State>>, time: u32) {
        let gs = gs.clone();
        stdweb::web::set_timeout(
            move || {
                game_loop(gs.clone(), time);
                gs.borrow_mut().tick();
            },
            time,
        );
    }

    game_loop(gs, 100);

    stdweb::event_loop();
}
