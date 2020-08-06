#[derive(Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Copy for Point {}

impl Clone for Point {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.y && self.y == other.y
    }
}

impl Eq for Point {}
