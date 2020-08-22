extern crate serde;

use specs::prelude::*;
use std::{thread, time};

use ferris_chat::state::{initialize_ecs, State};

fn main() {
    let width: i32 = 100;
    let height: i32 = 100;

    let mut gs = State { ecs: World::new() };
    initialize_ecs(&mut gs.ecs, width, height, 1 as u64);

    loop {
        gs.tick();
        thread::sleep(time::Duration::from_millis(100));
    }
}
