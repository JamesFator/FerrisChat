use canvas::Canvas;
use direction::Direction;
use point::Point;
use traits::Entity;

#[derive(Debug)]
pub struct Crab {
    name: String,
    location: Point,
    direction: Option<Direction>,
}

impl Crab {
    pub fn new(name: &str, x: u32, y: u32) -> Crab {
        Crab {
            name: String::from(name),
            location: Point { x, y },
            direction: None,
        }
    }
}

impl Entity for Crab {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn handle_input(&mut self, direction: Direction) {
        self.direction = Some(direction)
    }

    fn get_move_prediction(&mut self) -> Point {
        let cached_direction = self.direction;
        self.direction = None;
        match cached_direction {
            None => self.location,
            Some(Direction::Up) => Point {
                x: self.location.x,
                y: if self.location.y > 0 { self.location.y - 1 } else { 0 },
            },
            Some(Direction::Down) => Point {
                x: self.location.x,
                y: self.location.y + 1,
            },
            Some(Direction::Right) => Point {
                x: self.location.x + 1,
                y: self.location.y,
            },
            Some(Direction::Left) => Point {
                x: if self.location.x > 0 { self.location.x - 1 } else { 0 },
                y: self.location.y,
            },
        }
    }

    fn set_new_location(&mut self, point: Point) {
        self.location = point;
    }

    fn draw(&self, canvas: &Canvas) {
        canvas.draw(self.location.x, self.location.y, "green");
    }
}
