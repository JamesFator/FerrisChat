use super::*;

pub fn get_entity_for_name(ecs: &World, name: String) -> Option<Entity> {
    let entities = ecs.entities();
    let player_infos = ecs.read_storage::<PlayerInfo>();
    for (entity, player_info) in (&entities, &player_infos).join() {
        if player_info.name == name {
            return Some(entity);
        }
    }
    return None;
}

pub fn create_crab(ecs: &mut World, name: &str, color: &str, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(PlayerInfo {
            name: String::from(name),
        })
        .with(Renderable { render_order: 1 })
        .with(TextRenderable {
            text: String::from(name),
            offset_x: -STANDARD_TILE / 5_f64,
            offset_y: STANDARD_TILE / 2_f64,
        })
        .with(GraphicRenderable {
            color: String::from(color),
            offset_x: -STANDARD_TILE / 2_f64,
            offset_y: -STANDARD_TILE / 2_f64,
        })
        .build();
}

pub fn create_chat_bubble(ecs: &mut World, text: String, for_entity: Entity) {
    // Delete any other chat bubbles this player has had up until now
    {
        let entities = ecs.entities();
        let carried_bys = ecs.read_storage::<CarriedBy>();
        let mut chats_to_remove = Vec::new();
        for (entity, carried_by) in (&entities, &carried_bys).join() {
            if for_entity.eq(&carried_by.owner) {
                chats_to_remove.push(entity);
            }
        }
        for entity in chats_to_remove {
            entities
                .delete(entity)
                .expect("Could not delete chat bubble");
        }
    }
    ecs.create_entity()
        .with(CarriedBy { owner: for_entity })
        .with(Renderable { render_order: 0 })
        .with(ChatRenderable {
            text: text,
            offset_x: -STANDARD_TILE / 2.5_f64,
            offset_y: -STANDARD_TILE * 0.75,
        })
        .with(Disappearing {
            total_ticks: 100,
            ticks_left: 100,
        })
        .build();
}

pub fn create_knife(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable { render_order: 1 })
        .with(TextRenderable {
            text: String::from("🔪"),
            offset_x: STANDARD_TILE / 3_f64,
            offset_y: -STANDARD_TILE / 2_f64,
        })
        .build();
}

pub fn create_tree(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Location { x, y })
        .with(Renderable { render_order: 4 })
        .with(TextRenderable {
            text: String::from("🌴"),
            offset_x: 0_f64,
            offset_y: 0_f64,
        })
        .build();
}

pub fn create_poop(ecs: &mut World, location: Location) {
    ecs.create_entity()
        .with(location)
        .with(Renderable { render_order: 3 })
        .with(TextRenderable {
            text: String::from("💩"),
            offset_x: -8_f64,
            offset_y: STANDARD_TILE / 3_f64,
        })
        .with(Disappearing {
            total_ticks: 100,
            ticks_left: 100,
        })
        .build();
}
