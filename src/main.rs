extern crate serde;
extern crate stdweb;

use specs::prelude::*;

use stdweb::traits::*;
use stdweb::web::{event::KeyDownEvent, IEventTarget};

use std::cell::RefCell;
use std::rc::Rc;

mod canvas;
use canvas::{Canvas, DrawSystem};
mod components;
pub use components::*;
mod map;
pub use map::{generate_map, Map};
mod movement;
use movement::MovementSystem;
mod disappearing;
use disappearing::DisappearingSystem;

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
        ecs.create_entity()
            .with(new_poop_location.unwrap())
            .with(Renderable {
                text_renderable: Some(TextRenderable {
                    text: String::from("💩"),
                    offset_x: 1.25f64,
                    offset_y: 1.25f64,
                }),
                graphic_renderable: None,
                render_order: 3,
            })
            .with(Disappearing {
                total_ticks: 100,
                ticks_left: 100,
            })
            .build();
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

        // Draw system should be last
        let mut draw_system = DrawSystem {};
        draw_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn read_from_local_storage(&mut self) {
        // Check for chat_input
        let chat_input = stdweb::web::window().local_storage().get("chat_input");
        if chat_input.is_some() {
            let chat_message = &chat_input.unwrap();
            self.ecs
                .create_entity()
                .with(Location { x: 4, y: 15 })
                .with(PlayerInfo {
                    name: String::from(chat_message),
                })
                .with(Renderable {
                    text_renderable: Some(TextRenderable {
                        text: String::from(chat_message),
                        offset_x: 1f64,
                        offset_y: 1.5f64,
                    }),
                    graphic_renderable: Some(GraphicRenderable {
                        color: String::from("green"),
                        offset_x: 1f64,
                        offset_y: 1f64,
                    }),
                    render_order: 1,
                })
                .with(Disappearing {
                    total_ticks: 100,
                    ticks_left: 100,
                })
                .build();
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
    gs.borrow_mut().ecs.register::<PlayerInfo>();
    gs.borrow_mut().ecs.register::<Renderable>();
    gs.borrow_mut().ecs.register::<Location>();
    gs.borrow_mut().ecs.register::<WantsToMoveTo>();
    gs.borrow_mut().ecs.register::<Disappearing>();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 9, y: 9 })
        .with(PlayerInfo {
            name: String::from("Ferris"),
        })
        .with(Renderable {
            text_renderable: Some(TextRenderable {
                text: String::from("Ferris"),
                offset_x: 1f64,
                offset_y: 1.5f64,
            }),
            graphic_renderable: Some(GraphicRenderable {
                color: String::from("red"),
                offset_x: 1f64,
                offset_y: 1f64,
            }),
            render_order: 1,
        })
        .build();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 1, y: 1 })
        .with(PlayerInfo {
            name: String::from("Geoff"),
        })
        .with(Renderable {
            text_renderable: Some(TextRenderable {
                text: String::from("Geoff"),
                offset_x: 1f64,
                offset_y: 1.5f64,
            }),
            graphic_renderable: Some(GraphicRenderable {
                color: String::from("blue"),
                offset_x: 1f64,
                offset_y: 1f64,
            }),
            render_order: 1,
        })
        .build();

    gs.borrow_mut()
        .ecs
        .create_entity()
        .with(Location { x: 17, y: 15 })
        .with(PlayerInfo {
            name: String::from("Tammy"),
        })
        .with(Renderable {
            text_renderable: Some(TextRenderable {
                text: String::from("Tammy"),
                offset_x: 1f64,
                offset_y: 1.5f64,
            }),
            graphic_renderable: Some(GraphicRenderable {
                color: String::from("purple"),
                offset_x: 1f64,
                offset_y: 1f64,
            }),
            render_order: 1,
        })
        .build();

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
