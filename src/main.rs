extern crate stdweb;

mod canvas;
mod traits;
mod world;

use canvas::Canvas;
use traits::*;
use world::*;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    stdweb::initialize();

    let canvas = Rc::new(RefCell::new(Canvas::new("#canvas", 20, 20)));
    // let world = Rc::new(RefCell::new(World::new(20, 20, canvas)));
    let entities = Rc::new(RefCell::new(Vec::new()));
    entities.borrow_mut().push(0 as usize);
    let game_state = Rc::new(RefCell::new(GameState {
        locations: Vec::new(),
        velocities: Vec::new(),
        player_infos: Vec::new(),
    }));
    game_state
        .borrow_mut()
        .locations
        .push(Some(Location { x: 9, y: 9 }));
    game_state
        .borrow_mut()
        .velocities
        .push(Some(Velocity { x: 0, y: 0 }));
    game_state.borrow_mut().player_infos.push(Some(PlayerInfo {
        name: String::from("ferris"),
        color: String::from("red"),
    }));

    // world.borrow_mut().new_crab("ferris", 9, 9);

    // Initial draw of the world
    // world.borrow_mut().update();

    // TODO: Pull entities and game state out of world?

    stdweb::web::document().add_event_listener({
        let entities = entities.clone();
        let game_state = game_state.clone();
        move |event: KeyDownEvent| {
            handle_input(
                &entities.borrow(),
                &mut game_state.borrow_mut(),
                event.key().as_ref(),
                "ferris",
            );
        }
    });

    fn game_loop(
        canvas: Rc<RefCell<Canvas>>,
        entities: Rc<RefCell<Vec<usize>>>,
        game_state: Rc<RefCell<GameState>>,
        time: u32,
    ) {
        let canvas = canvas.clone();
        let entities = entities.clone();
        let game_state = game_state.clone();
        stdweb::web::set_timeout(
            move || {
                game_loop(canvas.clone(), entities.clone(), game_state.clone(), time);
                update_game_state(
                    &canvas.borrow(),
                    &entities.borrow(),
                    &mut game_state.borrow_mut(),
                );
            },
            time,
        );
    }

    game_loop(canvas, entities, game_state, 50);

    stdweb::event_loop();
}
