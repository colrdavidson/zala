use std::fs::File;
use std::path::Path;

use glium;
use image;
use vert::Vert;
use vert::Point;

#[derive(Clone, Copy)]
pub struct TileCollide {
    pub bl: Point,
    pub tr: Point,
}

impl TileCollide {
    pub fn new(x1: f32, y1: f32) -> TileCollide {
        TileCollide {
            bl: Point::new(x1 - 0.5, y1 - 0.5),
            tr: Point::new(x1 + 0.5, y1 + 0.5),
        }
    }

    pub fn partial_scale_new(x1: f32, y1: f32, width: f32, height: f32) -> TileCollide {
        TileCollide {
            bl: Point::new(x1 - 0.5, y1 - 0.5),
            tr: Point::new(x1 + 0.5 + width, y1 + 0.5 + height),
        }
    }

    pub fn collides(&self, x: f32, y: f32) -> bool {
        if x >= self.bl.x && x <= self.tr.x && y >= self.bl.y && y <= self.tr.y {
            return true;
        }
        return false;
    }
}

pub struct TileState {
    pub triggered: bool,
    pub sprite: usize,
    pub collision_box: TileCollide,
}

pub struct Door {
    pub frames: Vec<usize>,
    pub open_collide: TileCollide,
    pub closed_collide: TileCollide,
    pub closed: bool,
}

impl Door {
    pub fn new(ids: Vec<usize>, x: f32, y: f32, closed: bool) -> Door {
        Door {
            frames: ids,
            open_collide: TileCollide::new(-1.0, -1.0),
            closed_collide: TileCollide::partial_scale_new(x, y + 0.6, 0.0, -0.5),
            closed: closed,
        }
    }

    pub fn get_state(&self) -> TileState {
        if self.closed {
            TileState {
                triggered: self.closed,
                sprite: self.frames[0],
                collision_box: self.closed_collide,
            }
        } else {
            TileState {
                triggered: self.closed,
                sprite: self.frames[1],
                collision_box: self.open_collide,
            }
        }
    }

    pub fn open(&mut self) {
        self.closed = false;
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn trigger(&mut self) {
        self.closed = !self.closed;
    }
}

pub struct Turret {
    pub states: [usize; 2],
    pub base: usize,
    pub collide: TileCollide,
    pub on: bool,
}

impl Turret {
    pub fn new(ids: [usize; 3], x: f32, y: f32, on: bool) -> Turret {
        Turret {
            states: [ids[0], ids[1]],
            base: ids[2],
            collide: TileCollide::new(x, y),
            on: on,
        }
    }

    pub fn get_state(&self) -> TileState {
        if self.on {
            TileState {
                triggered: self.on,
                sprite: self.states[0],
                collision_box: self.collide,
            }
        } else {
            TileState {
                triggered: self.on,
                sprite: self.states[1],
                collision_box: self.collide,
            }
        }
    }

    pub fn poweron(&mut self) {
        self.on = true;
    }

    pub fn poweroff(&mut self) {
        self.on = false;
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

fn ship_verts(bl_corner: usize, sheet_entries: usize) -> Vec<Vert> {
    let num_entries = sheet_entries;
    let col_num = (num_entries as f32).sqrt();
    let row_num = (num_entries as f32).sqrt();

    let scalar = 1.0 / ((num_entries as f32) / col_num);

    let base_y = bl_corner % (num_entries / (col_num as usize));
    let base_x = bl_corner / (num_entries / (row_num as usize));
    let base_x = (base_x as f32) * scalar;
    let base_y = (base_y as f32) * scalar;

    let bottom_left =  [base_x, base_y];
    let bottom_right = [base_x + (scalar * 2.0), base_y];
    let top_left = 	   [base_x, base_y + (scalar * 2.0)];
    let top_right =	   [base_x + (scalar * 2.0), base_y + (scalar * 2.0)];

    let vert1 = Vert { position: [-1.0, -1.0], tex_coords: bottom_left };
    let vert2 = Vert { position: [-1.0,  1.0], tex_coords: top_left };
    let vert3 = Vert { position: [ 1.0, -1.0], tex_coords: bottom_right };
    let vert4 = Vert { position: [ 1.0, -1.0], tex_coords: bottom_right };
    let vert5 = Vert { position: [-1.0,  1.0], tex_coords: top_left };
    let vert6 = Vert { position: [ 1.0,  1.0], tex_coords: top_right };
    vec![vert1, vert2, vert3, vert4, vert5, vert6]
}

impl TileAtlas {
    pub fn new(display: &glium::backend::glutin::Display, filename: &str, num_entries: u32, ship: Vec<u32>) -> TileAtlas {
        let f = File::open(filename).unwrap();
        let f = std::io::BufReader::new(f);
        let atlas_img = image::load(f, image::PNG).unwrap().to_rgba();
    	let atlas_dims = atlas_img.dimensions();
    	let atlas_img = glium::texture::RawImage2d::from_raw_rgba_reversed(&atlas_img.into_raw(), atlas_dims);
    	let atlas_tex = glium::texture::SrgbTexture2d::new(display, atlas_img).unwrap();

        let mut atlas = Vec::new();
        for i in 0..num_entries {
            let vert_vec = atlas_verts(i as usize, num_entries as usize);
            let verts = glium::VertexBuffer::immutable(display, &vert_vec).unwrap();
            atlas.push(verts);
        }

        let ship_vert_vec = ship_verts(ship[0] as usize, num_entries as usize);
        let ship_verts = glium::VertexBuffer::immutable(display, &ship_vert_vec).unwrap();
        atlas.push(ship_verts);

        TileAtlas {
            texture: atlas_tex,
            num_entries: num_entries,
            atlas: atlas,
        }
    }
}
