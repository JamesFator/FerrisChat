use super::components::*;
use super::string_writer::StringWriter;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) -> String {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    let state_string: String;
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        // let writer = File::create("/Volumes/RAMDisk/savegame.json").unwrap();
        let mut writer = StringWriter::new();
        let mut serializer = serde_json::Serializer::new(&mut writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            SerializationHelper,
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

        // TODO: Compress serde
        state_string = writer.to_string();
        super::stdweb::console!(log, format!("{}", state_string));
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");

    state_string
}

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

pub fn load_game(ecs: &mut World, state_string: String) {
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

    let mut de = serde_json::Deserializer::from_str(&state_string);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            SerializationHelper,
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

    // let mut deleteme: Option<Entity> = None;
    // {
    //     let entities = ecs.entities();
    //     let helper = ecs.read_storage::<SerializationHelper>();
    //     let player = ecs.read_storage::<Player>();
    //     let position = ecs.read_storage::<Location>();
    //     for (e, h) in (&entities, &helper).join() {
    //         let mut worldmap = ecs.write_resource::<super::map::Map>();
    //         // *worldmap = h.map.clone();
    //         // worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
    //         deleteme = Some(e);
    //     }
    // }
    // ecs.delete_entity(deleteme.unwrap())
    //     .expect("Unable to delete helper");
}
