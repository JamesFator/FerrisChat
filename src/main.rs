extern crate serde;
extern crate stdweb;

use specs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

mod canvas;
use canvas::{Canvas, DrawSystem};
mod components;
pub use components::*;
mod entities;
pub use entities::*;
mod map;
pub use map::{generate_map, Map};
mod movement;
use movement::MovementSystem;
mod disappearing;
use disappearing::DisappearingSystem;
mod carry;
use carry::CarrySystem;

pub fn handle_input(ecs: &mut World, input: &str, for_name: &str) {
    let mut new_poop_location: Option<Location> = None;
    {
        let entities = ecs.entities();
        let locations = ecs.read_storage::<Location>();
        let player_infos = ecs.read_storage::<PlayerInfo>();
        let mut move_tos = ecs.write_storage::<WantsToMoveTo>();
        for (entity, location, player_info) in (&entities, &locations, &player_infos).join() {
            if player_info.name != for_name {
                continue;
            }
            let mut new_x = location.x;
            let mut new_y = location.y;

            match input {
                "ArrowLeft" => new_x -= 1,
                "ArrowRight" => new_x += 1,
                "ArrowDown" => new_y += 1,
                "ArrowUp" => new_y -= 1,
                "p" => {
                    new_poop_location = Some(Location {
                        x: location.x,
                        y: location.y,
                    });
                }
                _ => return,
            };
            move_tos
                .insert(entity, WantsToMoveTo { x: new_x, y: new_y })
                .expect("Unable to insert WantsToMoveTo");
        }
    }
    if new_poop_location.is_some() {
        create_poop(ecs, new_poop_location.unwrap());
    }
}

pub struct State {
    pub ecs: World,
}

impl State {
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

    fn read_from_local_storage(&mut self) {
        // Check for chat_input
        let chat_input = stdweb::web::window().local_storage().get("chat_input");
        if chat_input.is_some() {
            let maybe_entity = get_entity_for_name(&self.ecs, String::from("Ferris"));
            if maybe_entity.is_some() {
                create_chat_bubble(&mut self.ecs, &chat_input.unwrap(), maybe_entity.unwrap());
            }
        }
        stdweb::web::window().local_storage().remove("chat_input");
    }

    fn tick(&mut self) {
        // Check the window local storage for updates
        self.read_from_local_storage();

        // Run all our ECS systems
        self.run_systems();
    }
}

fn main() {
    stdweb::initialize();

    let width: i32 = 20;
    let height: i32 = 20;

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    gs.borrow_mut().ecs.register::<Location>();
    gs.borrow_mut().ecs.register::<PlayerInfo>();
    gs.borrow_mut().ecs.register::<Renderable>();
    gs.borrow_mut().ecs.register::<TextRenderable>();
    gs.borrow_mut().ecs.register::<ChatRenderable>();
    gs.borrow_mut().ecs.register::<GraphicRenderable>();
    gs.borrow_mut().ecs.register::<WantsToMoveTo>();
    gs.borrow_mut().ecs.register::<Disappearing>();
    gs.borrow_mut().ecs.register::<CarriedBy>();

    // Create our crabs
    create_crab(&mut gs.borrow_mut().ecs, "Ferris", "red", 9, 9);
    create_crab(&mut gs.borrow_mut().ecs, "Geoff", "blue", 1, 1);
    create_crab(&mut gs.borrow_mut().ecs, "Tammy", "purple", 17, 15);

    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    let map = Map { width, height };
    generate_map(&mut gs.borrow_mut().ecs, &map);
    gs.borrow_mut().ecs.insert(map);

    stdweb::web::document().add_event_listener({
        let gs = gs.clone();
        move |event: KeyDownEvent| {
            handle_input(&mut gs.borrow_mut().ecs, event.key().as_ref(), "Ferris");
        }
    });

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
