use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
pub struct PlayerInfo {
    pub name: String,
    pub color: String,
}

#[derive(Component)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct WantsToMoveTo {
    pub x: i32,
    pub y: i32,
}
