use super::*;
use canvas::Canvas;

pub struct Map {
    pub width: i32,
    pub height: i32,
}

pub fn handle_input(ecs: &mut World, input: &str, for_name: &str) {
    let mut should_create_poop = false;
    let mut poop_x = 0;
    let mut poop_y = 0;
    {
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
                "p" => {
                    should_create_poop = true;
                    poop_x = location.x;
                    poop_y = location.y;
                }
                _ => return,
            };
            move_tos
                .insert(entity, WantsToMoveTo { x: new_x, y: new_y })
                .expect("Unable to insert WantsToMoveTo");
        }
    }
    if should_create_poop {
        ecs.create_entity()
            .with(Location {
                x: poop_x,
                y: poop_y,
            })
            .with(PlayerInfo {
                name: String::from("💩"),
                color: String::from(""),
            })
            .with(Disappearing {
                total_ticks: 100,
                ticks_left: 100,
            })
            .build();
    }
}

pub fn draw(
    canvas: &Canvas,
    location: &Location,
    player_info: &PlayerInfo,
    disappearing: Option<&Disappearing>,
) {
    let alpha = match disappearing {
        None => 1f64,
        Some(disappearing) => disappearing.ticks_left as f64 / disappearing.total_ticks as f64,
    };
    canvas.draw(
        location.x as u32,
        location.y as u32,
        &player_info.color,
        &player_info.name,
        alpha,
    );
}
