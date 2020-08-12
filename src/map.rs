use super::*;
use canvas::Canvas;

pub struct Map {
    pub width: i32,
    pub height: i32,
}

pub fn handle_input(ecs: &mut World, input: &str, for_name: &str) {
    let entities = ecs.entities();
    let locations = ecs.read_storage::<Location>();
    let player_infos = ecs.read_storage::<PlayerInfo>();
    let mut move_tos = ecs.write_storage::<WantsToMoveTo>();
    for (entity, location, player_info) in (&entities, &locations, &player_infos).join() {
        if player_info.name != for_name {
            continue;
        }
        let mut new_x = location.x;
        let mut new_y = location.y;

        match input {
            "ArrowLeft" => new_x -= 1,
            "ArrowRight" => new_x += 1,
            "ArrowDown" => new_y += 1,
            "ArrowUp" => new_y -= 1,
            _ => return,
        };
        move_tos
            .insert(entity, WantsToMoveTo { x: new_x, y: new_y })
            .expect("Unable to insert WantsToMoveTo");
    }
}

pub fn draw(canvas: &Canvas, location: &Location, player_info: &PlayerInfo) {
    canvas.draw(location.x as u32, location.y as u32, &player_info.color);
}
