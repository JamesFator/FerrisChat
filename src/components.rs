use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::*;

// Serialization helper code. Need to implement ConvertSaveload for each type that contains an
// Entity.
pub struct SerializeMe;

#[derive(Component, ConvertSaveload, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct FPSTracker {
    pub for_time: u64,
    pub seen_frames: u16,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct PlayerInfo {
    pub name: String,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub render_order: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct TextRenderable {
    pub text: String,
    pub font_size: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ChatRenderable {
    pub text: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct GraphicRenderable {
    pub image_name: String,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct GraphicAnimatable {
    pub image_names: Vec<String>,
    pub tick_interval: i16,
    pub ticks: i16,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMoveTo {
    pub x: i32,
    pub y: i32,
    pub speed: i16,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Disappearing {
    pub total_ticks: u32,
    pub ticks_left: u32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct CarriedBy {
    pub owner: Entity,
}

#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum CrabAIState {
    WalkingRight,
    SleepingRight,
    WalkingLeft,
    SleepingLeft,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct CrabAI {
    pub crab_state: CrabAIState,
    pub tick_interval: i16,
    pub ticks: i16,
    pub walk_speed: i16,
    pub sleep_duration: i16,
}
