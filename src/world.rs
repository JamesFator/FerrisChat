use canvas::Canvas;
use traits::draw;
use traits::move_entity;
use traits::player_input;
use traits::Location;
use traits::PlayerInfo;
use traits::Velocity;

pub struct GameState {
    pub locations: Vec<Option<Location>>,
    pub velocities: Vec<Option<Velocity>>,
    pub player_infos: Vec<Option<PlayerInfo>>,
}

pub struct World {
    width: u32,
    height: u32,
    pub canvas: Canvas,
    pub entities: Vec<usize>,
    pub game_state: GameState,
}

impl World {
    pub fn new(width: u32, height: u32, canvas: Canvas) -> World {
        World {
            width,
            height,
            canvas,
            entities: Vec::new(),
            game_state: GameState {
                locations: Vec::new(),
                velocities: Vec::new(),
                player_infos: Vec::new(),
            },
        }
    }

    pub fn new_crab(&mut self, name: &str, x: i32, y: i32) {
        self.entities.push(self.entities.len() + 1);
        self.game_state.locations.push(Some(Location { x, y }));
        self.game_state
            .velocities
            .push(Some(Velocity { x: 0, y: 0 }));
        self.game_state.player_infos.push(Some(PlayerInfo {
            name: String::from(name),
            color: String::from("red"),
        }));
    }
}

pub fn handle_input(
    entities: &Vec<usize>,
    game_state: &mut GameState,
    input: &str,
    for_name: &str,
) {
    let new_velocity = match input {
        "ArrowLeft" => Velocity { x: -1, y: 0 },
        "ArrowRight" => Velocity { x: 1, y: 0 },
        "ArrowDown" => Velocity { x: 0, y: 1 },
        "ArrowUp" => Velocity { x: 0, y: -1 },
        _ => Velocity { x: 0, y: 0 },
    };
    if new_velocity.x == 0 && new_velocity.y == 0 {
        return;
    }
    for entity in entities.iter() {
        let mut velocity = game_state.velocities.get_mut(*entity).expect("Entity lost");
        let player_info = game_state.player_infos.get(*entity).expect("Entity lost");
        if velocity.is_some() && player_info.as_ref().unwrap().name == for_name {
            player_input(velocity.as_mut().unwrap(), &new_velocity);
        }
    }
}

pub fn update_game_state(canvas: &Canvas, entities: &Vec<usize>, game_state: &mut GameState) {
    // Move the entities
    for entity in entities.iter() {
        let location = game_state.locations.get_mut(*entity).expect("Entity lost");
        let velocity = game_state.velocities.get_mut(*entity).expect("Entity lost");
        if location.is_some() && velocity.is_some() {
            move_entity(location.as_mut().unwrap(), velocity.as_mut().unwrap());
        }
    }

    // // Clear the canvas to draw again
    canvas.clear_all();

    // Draw the entities
    for entity in entities.iter() {
        let location = game_state.locations.get(*entity).expect("Entity lost");
        let player_info = game_state
            .player_infos
            .get_mut(*entity)
            .expect("Entity lost");
        if location.is_some() && player_info.is_some() {
            draw(
                &canvas,
                location.as_ref().unwrap(),
                player_info.as_ref().unwrap(),
            );
        }
    }
}
