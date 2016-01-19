use data::{Vec2, Vec3};

pub struct Model {
    pub paths: Vec<Path>
}

pub struct Path {
    pub colour: Vec3,
    pub points: Vec<Point>,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub location: Vec2,
    pub curve_bias: f64
}

pub mod io;
pub mod rendering;
