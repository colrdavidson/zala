#[derive(Copy, Clone, Debug)]
pub struct Vert {
	pub position: [f32; 2],
	pub tex_coords: [f32; 2],
}

implement_vertex!(Vert, position, tex_coords);
