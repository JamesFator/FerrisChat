use canvas::Canvas;
use std::cmp;


pub struct Location {
    pub x: i32,
    pub y: i32,
}

pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

pub struct PlayerInfo {
    pub name: String,
    pub color: String,
}

pub fn player_input(velocity: &mut Velocity, new_velocity: &Velocity) {
    velocity.x = new_velocity.x;
    velocity.y = new_velocity.y;
}

pub fn move_entity(location: &mut Location, velocity: &mut Velocity) {
    location.x += velocity.x;
    location.y += velocity.y;

    velocity.x = 0;
    velocity.y = 0;

    // Detect collisions with other entities
    // TODO: bounds of world should be entities?
    let width = 20;
    let height = 20;
    location.x = cmp::min(cmp::max(location.x, 0), width - 1);
    location.y = cmp::min(height - 1, cmp::max(location.y, 0));
}

pub fn draw(canvas: &Canvas, location: &Location, player_info: &PlayerInfo) {
    canvas.draw(location.x as u32, location.y as u32, &player_info.color);
}
