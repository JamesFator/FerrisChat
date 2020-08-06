extern crate stdweb;

mod canvas;
mod crab;
mod direction;
mod point;
mod traits;
mod world;

use canvas::Canvas;
use crab::Crab;
use world::World;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    stdweb::initialize();

    let canvas = Canvas::new("#canvas", 20, 20);
    let world = Rc::new(RefCell::new(World::new(20, 20, canvas)));

    let crab = Box::new(Crab::new("ferris", 9, 9));
    world.borrow_mut().add_entity(crab);

    // Initial draw of the world
    world.borrow_mut().update();

    stdweb::web::document().add_event_listener({
        let world = world.clone();
        move |event: KeyDownEvent| {
            world.borrow_mut().handle_input(event.key().as_ref(), "ferris");
        }
    });

    fn game_loop(world: Rc<RefCell<World>>, time: u32) {
        stdweb::web::set_timeout(
            move || {
                game_loop(world.clone(), time);
                world.borrow_mut().update();
            },
            time,
        );
    }

    game_loop(world, 100);

    stdweb::event_loop();
}
