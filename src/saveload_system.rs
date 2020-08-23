use super::components::*;
use super::map::Map;
use super::string_writer::StringWriter;
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};

#[derive(Default, Serialize, Deserialize, Clone)]
struct OptimisticGameSave {
    serialized_ecs: String,
    maybe_serialized_map: Option<String>,
}

/// Magic stolen from "Roguelike Tutorial - In Rust" (See README.md)
/// Macro to serialize ECS components and entities
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<EntityMarker>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

/// Convert our ECS into a JSON string
pub fn serialize_ecs(ecs: &mut World) -> String {
    let data = (
        ecs.entities(),
        ecs.read_storage::<SimpleMarker<EntityMarker>>(),
    );

    let mut writer = StringWriter::new();
    let mut serializer = serde_json::Serializer::new(&mut writer);
    serialize_individually!(
        ecs,
        serializer,
        data,
        FPSTracker,
        Location,
        PlayerInfo,
        Renderable,
        TextRenderable,
        ChatRenderable,
        GraphicRenderable,
        GraphicAnimatable,
        WantsToMoveTo,
        Disappearing,
        CarriedBy,
        CrabAI
    );

    // TODO: Compress serde?
    writer.to_string()
}

pub fn serialize_map(map: &Map) -> String {
    serde_json::to_string(&map).unwrap()
}

/// Package our save state into a struct that optionally includes the map.
/// This allows us to only send the large map data when the client's map needs to be updated
pub fn package_save_state(serialized_ecs: String, maybe_serialized_map: Option<String>) -> String {
    serde_json::to_string(&OptimisticGameSave {
        serialized_ecs,
        maybe_serialized_map,
    })
    .unwrap()
}

/// Magic stolen from "Roguelike Tutorial - In Rust" (See README.md)
/// Macro to deserialize ECS components and entities
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

/// Update the ECS to reflect the world deserialized from the OptimisticGameSave JSON string
pub fn load_game(ecs: &mut World, package_save_str: String) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    // Extract our save package from the string
    let package_save_state: OptimisticGameSave = serde_json::from_str(&package_save_str).unwrap();

    // Deserialize the ECS because we know that will be there
    let mut de = serde_json::Deserializer::from_str(&package_save_state.serialized_ecs);
    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<EntityMarker>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<EntityMarker>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            FPSTracker,
            Location,
            PlayerInfo,
            Renderable,
            TextRenderable,
            ChatRenderable,
            GraphicRenderable,
            GraphicAnimatable,
            WantsToMoveTo,
            Disappearing,
            CarriedBy,
            CrabAI
        );
    }

    // If the map was in this package, copy the map over to our instance
    if let Some(serialized_map) = package_save_state.maybe_serialized_map {
        let mut map_ref = ecs.write_resource::<Map>();
        let new_map: Map = serde_json::from_str(&serialized_map).unwrap();
        *map_ref = new_map.clone();
    }
}
