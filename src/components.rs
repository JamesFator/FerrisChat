use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
pub struct PlayerInfo {
    pub name: String,
}

pub struct TextRenderable {
    pub text: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

pub struct GraphicRenderable {
    pub color: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component)]
pub struct Renderable {
    pub text_renderable: Option<TextRenderable>,
    pub graphic_renderable: Option<GraphicRenderable>,
    pub render_order: i32,
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

#[derive(Component)]
pub struct Disappearing {
    pub total_ticks: u32,
    pub ticks_left: u32,
}
