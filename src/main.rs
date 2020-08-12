extern crate serde;
extern crate stdweb;

mod canvas;
use canvas::Canvas;
mod components;
pub use components::*;
mod map;
pub use map::{draw, handle_input, Map};
mod movement;
use movement::MovementSystem;

use specs::prelude::*;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut movement_system = MovementSystem {};
        movement_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn tick(&mut self) {
        // Run all our ECS systems
        self.run_systems();

        let canvas = self.ecs.fetch::<Canvas>();

        // Clear the canvas to draw again
        canvas.clear_all();

        let locations = self.ecs.read_storage::<Location>();
        let player_infos = self.ecs.read_storage::<PlayerInfo>();
        for (location, player_info) in (&locations, &player_infos).join() {
            draw(&canvas, &location, &player_info);
        }
    }
}

fn main() {
    stdweb::initialize();

    let width: i32 = 20;
    let height: i32 = 20;

    let gs = Rc::new(RefCell::new(State { ecs: World::new() }));
    gs.borrow_mut().ecs.register::<PlayerInfo>();
    gs.borrow_mut().ecs.register::<Location>();
    gs.borrow_mut().ecs.register::<WantsToMoveTo>();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 9, y: 9 })
        .with(PlayerInfo {
            name: String::from("Ferris"),
            color: String::from("red"),
        })
        .build();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 1, y: 1 })
        .with(PlayerInfo {
            name: String::from("Geoff"),
            color: String::from("blue"),
        })
        .build();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 17, y: 15 })
        .with(PlayerInfo {
            name: String::from("Tammy"),
            color: String::from("purple"),
        })
        .build();

    let canvas = Canvas::new("#canvas", width as u32, height as u32);
    gs.borrow_mut().ecs.insert(canvas);

    let map = Map { width, height };
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
