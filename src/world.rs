use canvas::Canvas;
use direction::Direction;
use point::Point;
use std::cmp;
use traits::Entity;

pub struct World {
    width: u32,
    height: u32,
    canvas: Canvas,
    entities: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn new(width: u32, height: u32, canvas: Canvas) -> World {
        World {
            width,
            height,
            canvas,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Box<dyn Entity>) {
        self.entities.push(entity);
    }

    pub fn handle_input(&mut self, input: &str, for_name: &str) {
        let entity = self
            .entities
            .iter_mut()
            .find(|x| x.get_name() == for_name).unwrap();
        match input {
            "ArrowLeft" => entity.handle_input(Direction::Left),
            "ArrowRight" => entity.handle_input(Direction::Right),
            "ArrowDown" => entity.handle_input(Direction::Down),
            "ArrowUp" => entity.handle_input(Direction::Up),
            _ => ()
        };
    }

    fn validate_move(&self, point: &Point) -> Point {
        assert!(point.x < self.width);
        assert!(point.y < self.height);
        Point {
            x: cmp::min(cmp::max(point.x, 0), self.width - 1),
            y: cmp::min(self.height - 1, cmp::max(point.y, 0)),
        }
    }

    pub fn update(&mut self) {
        // Move the entities
        for entity in self.entities.iter_mut() {
            let predicted_location = entity.get_move_prediction();
            // TODO: Call validate_move instead
            // let p = self.validate_move(&predicted_location);
            // Maybe pass lambda?
            let p = Point {
                x: cmp::min(cmp::max(predicted_location.x, 0), self.width - 1),
                y: cmp::min(self.height - 1, cmp::max(predicted_location.y, 0)),
            };
            entity.set_new_location(p);
        }
        
        // Clear the canvas to draw again
        self.canvas.clear_all();

        // Draw the entities
        self.entities.iter().for_each(|entity| entity.draw(&self.canvas));
    }
}
