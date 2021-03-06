extern crate serde;

use censor::*;
use oorandom::Rand32;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use crate::animation::{AnimationSystem, DisappearingSystem};
use crate::carry::{CarrySystem, PickUpSystem};
use crate::components::*;
use crate::crab_ai::CrabAISystem;
use crate::entities::*;
use crate::map::{valid_walking_location, Map};
use crate::movement::MovementSystem;
use crate::saveload_system::{serialize_ecs, serialize_map, PlayerInput};
use crate::weapons::StabSystem;

pub fn handle_input(ecs: &mut World, input: &str, player_id: &String) {
    let maybe_entity;
    {
        maybe_entity = get_player_with_id(&ecs, &player_id);
        if maybe_entity.is_none() {
            println!("Entity ID {} doesn't exist!", &player_id);
            return;
        }
    }

    let for_entity = maybe_entity.unwrap();
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

pub fn handle_click(ecs: &mut World, x: i32, y: i32, player_id: &String) {
    let maybe_entity;
    {
        maybe_entity = get_player_with_id(&ecs, &player_id);
        if maybe_entity.is_none() {
            println!("Entity ID {} doesn't exist!", &player_id);
            return;
        }
    }

    let for_entity = maybe_entity.unwrap();
    let map = ecs.fetch::<Map>();
    let desired_location = WantsToMoveTo { x, y, speed: 2 };
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

/// Censor any profanity considering we're about to render the input
pub fn handle_chat_input(mut ecs: &mut World, chat_input: &str, player_id: &String) {
    let maybe_entity;
    {
        maybe_entity = get_player_with_id(&ecs, &player_id);
        if maybe_entity.is_none() {
            println!("Entity ID {} doesn't exist!", &player_id);
            return;
        }
    }

    let for_entity = maybe_entity.unwrap();
    create_chat_bubble(&mut ecs, censor_chat_input(&chat_input), for_entity);
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
        let mut pick_up_system = PickUpSystem {}; // PickUp before Carry so we can update location
        pick_up_system.run_now(&self.ecs);
        let mut carry_system = CarrySystem {};
        carry_system.run_now(&self.ecs);
        let mut crab_ai_system = CrabAISystem {};
        crab_ai_system.run_now(&self.ecs);
        StabSystem::run_now_manually(&mut self.ecs);
        let mut animation_system = AnimationSystem {};
        animation_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    pub fn get_serialized_map(&self) -> String {
        serialize_map(&self.ecs.fetch::<Map>())
    }

    pub fn get_serialized_ecs(&mut self) -> String {
        serialize_ecs(&mut self.ecs)
    }

    pub fn handle_player_input(&mut self, player_input: PlayerInput) {
        match player_input {
            PlayerInput::CreatePlayer { id, name } => {
                spawn_crab(&mut self.ecs, &id, &censor_chat_input(&name), false)
            }
            PlayerInput::DeletePlayer { id } => delete_player_with_id(&mut self.ecs, &id),
            PlayerInput::SpecialInput { id, input } => handle_input(&mut self.ecs, &input, &id),
            PlayerInput::Click { id, x, y } => handle_click(&mut self.ecs, x, y, &id),
            PlayerInput::Chat { id, message } => handle_chat_input(&mut self.ecs, &message, &id),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        // Run all our ECS systems
        self.run_systems();
    }
}

pub fn initialize_ecs(mut ecs: &mut World, width: i32, height: i32, seed: u64) {
    ecs.register::<FPSTracker>();
    ecs.register::<Location>();
    ecs.register::<PlayerInfo>();
    ecs.register::<Renderable>();
    ecs.register::<TextRenderable>();
    ecs.register::<ChatRenderable>();
    ecs.register::<GraphicRenderable>();
    ecs.register::<GraphicAnimatable>();
    ecs.register::<WantsToMoveTo>();
    ecs.register::<Disappearing>();
    ecs.register::<CarriedBy>();
    ecs.register::<CrabAI>();
    ecs.register::<WantsToBePickedUp>();
    ecs.register::<WantsToStab>();

    // Serialization helpers
    ecs.register::<SimpleMarker<EntityMarker>>();
    ecs.insert(SimpleMarkerAllocator::<EntityMarker>::new());

    // Psuedo random number generator we'll use
    let mut rng = Rand32::new(seed);

    // Map contains the map state
    let map = Map::new(&mut rng, width, height);

    // Create some initial entities to our map
    fill_map(&mut ecs, &map, &mut rng);

    // Insert resources into ECS
    ecs.insert(map);
    ecs.insert(rng);

    // Create our crabs
    spawn_crab(&mut ecs, "Chris", "Chris", true);
    spawn_crab(&mut ecs, "Tammy", "Tammy", true);
}
