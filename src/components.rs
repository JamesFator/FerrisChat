use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
pub struct FPSTracker {
    pub for_time: u64,
    pub seen_frames: u16,
}

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
    pub font_size: f64,
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
    pub image_name: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component)]
pub struct GraphicAnimatable {
    pub image_names: Vec<String>,
    pub tick_interval: i16,
    pub ticks: i16,
}

#[derive(Component)]
pub struct WantsToMoveTo {
    pub x: i32,
    pub y: i32,
    pub speed: i16,
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

pub enum CrabAIState {
    WalkingRight,
    SleepingRight,
    WalkingLeft,
    SleepingLeft,
}

#[derive(Component)]
pub struct CrabAI {
    pub crab_state: CrabAIState,
    pub tick_interval: i16,
    pub ticks: i16,
    pub walk_speed: i16,
    pub sleep_duration: i16,
}
