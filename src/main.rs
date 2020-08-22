extern crate serde;
#[macro_use]
extern crate stdweb;

use censor::*;
use oorandom::Rand32;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
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
pub use map::{get_random_location_of_tile, valid_walking_location, Map, TileType};
mod movement;
use movement::MovementSystem;
mod carry;
use carry::CarrySystem;
mod crab_ai;
use crab_ai::CrabAISystem;
mod animation;
use animation::{AnimationSystem, DisappearingSystem};
mod saveload_system;
use saveload_system::save_game;
mod string_writer;

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
    let desired_location = WantsToMoveTo {
        x: ((x as f64 - rect.get_left() as f64) / canvas.scaled_width) as i32,
        y: ((y as f64 - rect.get_left() as f64) / canvas.scaled_height) as i32,
        speed: 2,
    };
    let msg = format!("{}, {}", desired_location.x, desired_location.y);
    js! {
        let socket = new WebSocket("ws://127.0.0.1:3012");

        socket.onopen = function(e) {
            socket.send(@{msg});
        };

        socket.onmessage = function(event) {
            console.log(event.data);
        };
    }
    if valid_walking_location(&map, &desired_location) {
        let mut move_tos = ecs.write_storage::<WantsToMoveTo>();
        move_tos
            .insert(for_entity, desired_location)
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
        let mut crab_ai_system = CrabAISystem {};
        crab_ai_system.run_now(&self.ecs);
        let mut animation_system = AnimationSystem {};
        animation_system.run_now(&self.ecs);

        // Draw system should be last
        let mut draw_system = DrawSystem {};
        draw_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    pub fn save_state(&mut self) {
        // let data = serde_json::to_string(&*self.ecs.fetch::<Map>()).unwrap();
        // stdweb::console!(log, format!("{}", data));
        save_game(&mut self.ecs);
    }

    fn tick(&mut self) {
        // Update the FPS GUI
        self.update_fps_tracker();

        // Check the window local storage for updates
        self.read_from_local_storage();

        // Run all our ECS systems
        self.run_systems();

        // self.save_state();
    }
}

fn main() {
    stdweb::initialize();

    let width: i32 = 100;
    let height: i32 = 100;

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    gs.borrow_mut().ecs.register::<SerializationHelper>();
    gs.borrow_mut().ecs.register::<FPSTracker>();
    gs.borrow_mut().ecs.register::<Location>();
    gs.borrow_mut().ecs.register::<PlayerInfo>();
    gs.borrow_mut().ecs.register::<Renderable>();
    gs.borrow_mut().ecs.register::<TextRenderable>();
    gs.borrow_mut().ecs.register::<ChatRenderable>();
    gs.borrow_mut().ecs.register::<GraphicRenderable>();
    gs.borrow_mut().ecs.register::<GraphicAnimatable>();
    gs.borrow_mut().ecs.register::<WantsToMoveTo>();
    gs.borrow_mut().ecs.register::<Disappearing>();
    gs.borrow_mut().ecs.register::<CarriedBy>();
    gs.borrow_mut().ecs.register::<CrabAI>();

    // Serialization helpers
    gs.borrow_mut().ecs.register::<SimpleMarker<SerializeMe>>();
    gs.borrow_mut()
        .ecs
        .insert(SimpleMarkerAllocator::<SerializeMe>::new());

    // Canvas is where we do all our rendering
    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    // Psuedo random number generator we'll use
    let mut rng = Rand32::new(Date::new().get_seconds() as u64);

    // Map contains the map state
    let map = Map::new(&mut rng, width, height);

    // Create some initial entities to our map
    fill_map(&mut gs.borrow_mut().ecs, &map, &mut rng);

    // Create game helper entities
    create_fps_tracker(&mut gs.borrow_mut().ecs);

    // Create our crabs
    spawn_crab(&mut gs.borrow_mut().ecs, &map, &mut rng, "Ferris", false);
    spawn_crab(&mut gs.borrow_mut().ecs, &map, &mut rng, "Chris", true);
    spawn_crab(&mut gs.borrow_mut().ecs, &map, &mut rng, "Tammy", true);

    // Insert resources into ECS
    gs.borrow_mut().ecs.insert(map);
    gs.borrow_mut().ecs.insert(rng);
    gs.borrow_mut().save_state();

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
