#[derive(Copy, Clone)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }

    pub fn is_equal(&self, x: usize, y: usize) -> bool {
        self.x == x && self.y == y
    }
}
