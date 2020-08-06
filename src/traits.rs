use canvas::Canvas;
use direction::Direction;
use point::Point;

pub trait Entity {
    fn get_name(&self) -> &String;
    fn handle_input(&mut self, direction: Direction);
    fn get_move_prediction(&mut self) -> Point;
    fn set_new_location(&mut self, point: Point);
    fn draw(&self, canvas: &Canvas);
}
