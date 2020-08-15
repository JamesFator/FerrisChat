use specs::prelude::*;
use specs_derive::*;

#[derive(Component, Clone, Copy)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct PlayerInfo {
    pub name: String,
}

#[derive(Component)]
pub struct Renderable {
    pub render_order: i32,
}

#[derive(Component)]
pub struct TextRenderable {
    pub text: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component)]
pub struct ChatRenderable {
    pub text: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component)]
pub struct GraphicRenderable {
    pub color: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component)]
pub struct WantsToMoveTo {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Disappearing {
    pub total_ticks: u32,
    pub ticks_left: u32,
}

#[derive(Component)]
pub struct CarriedBy {
    pub owner: Entity,
}
