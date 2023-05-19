use std::ops::Sub;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Self { x, y }
    }

    #[inline]
    pub fn square_length(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn length(&self)-> f32{
        self.square_length().sqrt()
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self
    {
        let one_minus_t = 1.0 - t;
        Self::new(one_minus_t * self.x + t * other.x, one_minus_t * self.y + t * other.y)
    }

    #[inline]
    pub fn cross(self, other: Self) -> f32
    {
        self.x * other.y - self.y * other.x
    }
}

impl Sub for Point{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}