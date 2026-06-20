pub struct Point(i32, i32);

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point(x, y)
    }

    pub fn xy(xy: i32) -> Self {
        Self::new(xy, xy)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }
}
