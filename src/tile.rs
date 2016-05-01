use std::fs::File;

use glium;
use image;
use vert::Vert;

pub struct Tile {
    pub id: u32,
}

impl Tile {
    pub fn new(id: u32) -> Tile {
        Tile {
            id: id,
        }
    }
}

pub struct TileAtlas {
    pub texture: glium::texture::SrgbTexture2d,
    pub num_entries: u32,
    pub atlas: Vec<glium::VertexBuffer<Vert>>,
}

fn atlas_verts(entry: usize, sheet_entries: usize) -> Vec<Vert> {
    let num_entries = sheet_entries;
    let col_num = (num_entries as f32).sqrt();
    let row_num = (num_entries as f32).sqrt();

    let scalar = 1.0 / ((num_entries as f32) / col_num);

    let base_y = entry % (num_entries / (col_num as usize));
    let base_x = entry / (num_entries / (row_num as usize));
    let base_x = (base_x as f32) * scalar;
    let base_y = (base_y as f32) * scalar;

    let bottom_left =  [base_x, base_y];
    let bottom_right = [base_x + scalar, base_y];
    let top_left = 	   [base_x, base_y + scalar];
    let top_right =	   [base_x + scalar, base_y + scalar];

    let vert1 = Vert { position: [-1.0, -1.0], tex_coords: bottom_left };
    let vert2 = Vert { position: [-1.0,  1.0], tex_coords: top_left };
    let vert3 = Vert { position: [ 1.0, -1.0], tex_coords: bottom_right };
    let vert4 = Vert { position: [ 1.0, -1.0], tex_coords: bottom_right };
    let vert5 = Vert { position: [-1.0,  1.0], tex_coords: top_left };
    let vert6 = Vert { position: [ 1.0,  1.0], tex_coords: top_right };
    vec![vert1, vert2, vert3, vert4, vert5, vert6]
}

impl TileAtlas {
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade, filename: &str, num_entries: u32) -> TileAtlas {
        let atlas_img = image::load(File::open(filename).unwrap(), image::PNG).unwrap().to_rgba();
    	let atlas_dims = atlas_img.dimensions();
    	let atlas_img = glium::texture::RawImage2d::from_raw_rgba_reversed(atlas_img.into_raw(), atlas_dims);
    	let atlas_tex = glium::texture::SrgbTexture2d::new(display, atlas_img).unwrap();

        let mut atlas = Vec::new();
        for i in 0..num_entries {
            let vert_vec = atlas_verts(i as usize, num_entries as usize);
            let verts = glium::VertexBuffer::new(display, &vert_vec).unwrap();
            atlas.push(verts);
        }

        TileAtlas {
            texture: atlas_tex,
            num_entries: num_entries,
            atlas: atlas,
        }
    }
}
