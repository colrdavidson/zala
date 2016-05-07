#[derive(Copy, Clone, Debug)]
pub struct Vert {
	pub position: [f32; 2],
	pub tex_coords: [f32; 2],
}

implement_vertex!(Vert, position, tex_coords);

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x: x,
            y: y,
        }
    }
}
