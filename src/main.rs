#[macro_use]
extern crate glium;
extern crate image;
extern crate glium_text;
extern crate time;
extern crate zpu;

pub mod vert;
pub mod keyboard;
pub mod tile;

use std::io::{Write, Read};
use std::fs::File;
use std::io::Cursor;
use std::f32;

use glium::{DisplayBuild, Surface};
use glium::glutin;

use keyboard::Inputs;
use tile::TileAtlas;

#[derive(Copy, Clone)]
struct Vert {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vert, position, tex_coords);

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
	let f = {
		let f = direction;
		let len = ((f[0] * f[0]) + (f[1] * f[1]) + (f[2] * f[2])).sqrt();
		[f[0] / len, f[1] / len, f[2] / len]
	};

	let s = [(up[1] * f[2]) - (up[2] * f[1]),
			 (up[2] * f[0]) - (up[0] * f[2]),
			 (up[0] * f[1]) - (up[1] * f[0])];

	let s_norm = {
		let len = ((s[0] * s[0]) + (s[1] * s[1]) + (s[2] * s[2])).sqrt();
		[s[0] / len, s[1] / len, s[2] / len]
	};

	let u = [(f[1] * s_norm[2]) - (f[2] * s_norm[1]),
			 (f[2] * s_norm[0]) - (f[0] * s_norm[2]),
			 (f[0] * s_norm[1]) - (f[1] * s_norm[0])];

	let p = [(-position[0] * s_norm[0]) - (position[1] * s_norm[1]) - (position[2] * s_norm[2]),
			 (-position[0] * u[0]) - (position[1] * u[1]) - (position[2] * u[2]),
			 (-position[0] * f[0]) - (position[1] * f[1]) - (position[2] * f[2])];

	[
		[s[0], u[0], f[0], 0.0],
		[s[1], u[1], f[1], 0.0],
		[s[2], u[2], f[2], 0.0],
		[p[0], p[1], p[2], 1.0],
	]
}

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point {
            x: x,
            y: y,
        }
    }
}

#[derive(Clone, Copy)]
struct Rect {
    bl: Point,
    tr: Point,
}

impl Rect {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
        Rect {
            bl: Point::new(x1, y1),
            tr: Point::new(x2, y2),
        }
    }

    fn collides(&self, x: f32, y: f32) -> bool {
        if x >= self.bl.x && x <= self.tr.x && y >= self.bl.y && y <= self.tr.y {
            return true;
        }
        return false;
    }
}

#[derive(Clone, Copy)]
struct TileCollide {
    bl: Point,
    tr: Point,
}

impl TileCollide {
    fn new(x1: f32, y1: f32) -> TileCollide {
        TileCollide {
            bl: Point::new(x1 - 0.5, y1 - 0.5),
            tr: Point::new(x1 + 0.5, y1 + 0.5),
        }
    }

    fn partial_scale_new(x1: f32, y1: f32, width: f32, height: f32) -> TileCollide {
        TileCollide {
            bl: Point::new(x1 - 0.5, y1 - 0.5),
            tr: Point::new(x1 + 0.5 + width, y1 + 0.5 + height),
        }
    }

    fn collides(&self, x: f32, y: f32) -> bool {
        if x >= self.bl.x && x <= self.tr.x && y >= self.bl.y && y <= self.tr.y {
            return true;
        }
        return false;
    }
}

struct Entity {
    pos: Point,
    vel: Point,
}

impl Entity {
    fn new(x: f32, y: f32) -> Entity {
        Entity {
            pos: Point::new(x, y),
            vel: Point::new(0.0, 0.0),
        }
    }
}

fn main() {
    let (width, height) = (640, 480);
    let ratio = height as f32 / width as f32;
    let display = glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_title(format!("Zala"))
        .with_vsync()
        .build_glium().unwrap();

    let term_img = image::load(Cursor::new(&include_bytes!("../assets/term.png")[..]), image::PNG).unwrap().to_rgba();
    let term_dims = term_img.dimensions();
    let term_img = glium::texture::RawImage2d::from_raw_rgba_reversed(term_img.into_raw(), term_dims);
    let term_tex = glium::texture::SrgbTexture2d::new(&display, term_img).unwrap();

    let text_system = glium_text::TextSystem::new(&display);
	let font_file = std::fs::File::open(&std::path::Path::new("assets/greenscr.ttf")).unwrap();
	let font = glium_text::FontTexture::new(&display, font_file, 11).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let game_vert_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = perspective * view * model * vec4(position, 0.0, 1.0);
        }
    "#;

    let ui_vert_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 model;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = model * vec4(position, 0.0, 1.0);
        }
    "#;

    let frag_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;
        void main() {
            color = texture(tex, v_tex_coords);
            if (color.a == 0.0) { discard; }
        }
    "#;

    let game_program = glium::Program::from_source(&display, game_vert_shader_src, frag_shader_src, None).unwrap();
    let ui_program = glium::Program::from_source(&display, ui_vert_shader_src, frag_shader_src, None).unwrap();

    let perspective = {
        let fov: f32 = 3.141592 / 3.0;
        let zfar = 1024.0;
        let znear = 0.1;
        let f = 1.0 / (fov / 2.0).tan();

        [
            [f * ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (zfar+znear)/(zfar-znear), 1.0],
            [0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
        ]
    };

    let vert1 = Vert { position: [-1.0, -1.0], tex_coords: [ 0.0, 0.0] };
    let vert2 = Vert { position: [-1.0,  1.0], tex_coords: [ 0.0, 1.0] };
    let vert3 = Vert { position: [ 1.0, -1.0], tex_coords: [ 1.0, 0.0] };
    let vert4 = Vert { position: [ 1.0, -1.0], tex_coords: [ 1.0, 0.0] };
    let vert5 = Vert { position: [-1.0,  1.0], tex_coords: [ 0.0, 1.0] };
    let vert6 = Vert { position: [ 1.0,  1.0], tex_coords: [ 1.0, 1.0] };
    let verts = [vert1, vert2, vert3, vert4, vert5, vert6];
    let termui_buffer = glium::VertexBuffer::immutable(&display, &verts).unwrap();

    let tile_atlas = TileAtlas::new(&display, "assets/atlas.png", 64);

    let term_id = 39;
    let chair_id = 38;
    let player_id = 11;

    let on_engine_id = 3;
    let off_engine_id = 4;

    let on_turret_id = 12;
    let off_turret_id = 20;
    let turret_base_id = 28;

//    let door_closed_id = 29;
//    let door_open_id = 30;

    let zero = f32::consts::PI / 2.0;
    let mut rot = zero;

    let mut inputs = Inputs::new();

    let tile_gap = 2.0;

    let turret_rect = TileCollide::new(1.0, 1.0);
    let term_rect = TileCollide::new(3.0, 3.0);
    let eng_rect = TileCollide::new(5.0, 3.0);
    let chair_rect = TileCollide::new(7.0, 3.0);

    let mut collidables = vec![term_rect, turret_rect, eng_rect, chair_rect];

    let mut map_str = String::new();
    File::open("assets/map").unwrap().read_to_string(&mut map_str).unwrap();
    let size = map_str.lines().nth(0).unwrap();
    let size_parts: Vec<&str> = size.split_whitespace().collect();

    let map_width: usize = size_parts[0].parse().unwrap();
    let map_height: usize = size_parts[1].parse().unwrap();

    let map_str_v: Vec<&str> = map_str.splitn(2, "\n").collect();
    let map_str = map_str_v[1];
    let map_strs: Vec<&str> = map_str.split_whitespace().collect();
    let mut map = Vec::new();
    for n in map_strs.iter() {
        let val: u32 = n.parse().unwrap();
        map.push(val);
    }

    for x in 0..map_width {
        for y in 0..map_height {
            let idx = y * map_width + x;
            match map[idx as usize] {
                5 => collidables.push(TileCollide::new(x as f32, y as f32)),
                6 => collidables.push(TileCollide::partial_scale_new(x as f32, y as f32, -0.6, 0.0)),
                7 => collidables.push(TileCollide::new(x as f32, y as f32)),
                13 => collidables.push(TileCollide::partial_scale_new(x as f32, y as f32, 0.0, -0.6)),
                15 => collidables.push(TileCollide::partial_scale_new(x as f32, (y as f32) + 0.6, 0.0, -0.5)),
                21 => collidables.push(TileCollide::new(x as f32, y as f32)),
                22 => collidables.push(TileCollide::partial_scale_new((x as f32) + 0.6, y as f32, -0.5, 0.0)),
                23 => collidables.push(TileCollide::new(x as f32, y as f32)),
                _ => (),
            }
        }
    }

    let mut camera = Entity::new(10.0, 5.0);

    let mut dt = 0.0;
    let mut acc_time = 0.0;
    let speed = 0.125;

    let mut collided = false;
    let mut term_collide = false;
    let mut term_ui = false;

    let mut shift = false;

    let mut term_string = String::new();
    let mut guide_string = String::new();

    let mut term_file = File::open("programs/hello.asm").unwrap();
    let mut guide_file = File::open("docs/zpu_ref").unwrap();

    term_file.read_to_string(&mut term_string).unwrap();
    guide_file.read_to_string(&mut guide_string).unwrap();

    let mut terminal = Vec::new();
    let mut guide = Vec::new();

    for line in term_string.lines() {
        terminal.push(String::from(line));
    }
    for line in guide_string.lines() {
        guide.push(String::from(line));
    }

    let mut eng_id = off_engine_id;
    let mut tur_id = off_turret_id;
    let mut err = zpu::assembler::assemble_program("programs/hello.asm", "programs/zpu.bin");
    let mut zpu = zpu::zpu::ZPU::new("programs/zpu.bin");

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    let mut player = Entity::new(1.0, 5.0);

    'main: loop {
        let start_time = time::precise_time_ns();

        let mut typed = false;
        let mut result = zpu::zpu::ZResult::new(false, None);

        if acc_time > 5.0 {
            result = zpu.step();
            acc_time = 0.0;
        }

        for event in display.poll_events() {
            match event {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(state, _, key) => {
                    if key.is_some() && !term_ui {
                        let key = key.unwrap();
                        inputs.update(key, state);
                    } else if key.is_some() && state == glium::glutin::ElementState::Pressed && term_ui {
                        inputs.release_keys();
                        let key = key.unwrap();
                        let mut char_to_add = '|';
                        match key {
                            glium::glutin::VirtualKeyCode::A => char_to_add = 'a',
                            glium::glutin::VirtualKeyCode::B => char_to_add = 'b',
                            glium::glutin::VirtualKeyCode::C => char_to_add = 'c',
                            glium::glutin::VirtualKeyCode::D => char_to_add = 'd',
                            glium::glutin::VirtualKeyCode::E => char_to_add = 'e',
                            glium::glutin::VirtualKeyCode::F => char_to_add = 'f',
                            glium::glutin::VirtualKeyCode::G => char_to_add = 'g',
                            glium::glutin::VirtualKeyCode::H => char_to_add = 'h',
                            glium::glutin::VirtualKeyCode::I => char_to_add = 'i',
                            glium::glutin::VirtualKeyCode::J => char_to_add = 'j',
                            glium::glutin::VirtualKeyCode::K => char_to_add = 'k',
                            glium::glutin::VirtualKeyCode::L => char_to_add = 'l',
                            glium::glutin::VirtualKeyCode::M => char_to_add = 'm',
                            glium::glutin::VirtualKeyCode::N => char_to_add = 'n',
                            glium::glutin::VirtualKeyCode::O => char_to_add = 'o',
                            glium::glutin::VirtualKeyCode::P => char_to_add = 'p',
                            glium::glutin::VirtualKeyCode::Q => char_to_add = 'q',
                            glium::glutin::VirtualKeyCode::R => char_to_add = 'r',
                            glium::glutin::VirtualKeyCode::S => char_to_add = 's',
                            glium::glutin::VirtualKeyCode::T => char_to_add = 't',
                            glium::glutin::VirtualKeyCode::U => char_to_add = 'u',
                            glium::glutin::VirtualKeyCode::V => char_to_add = 'v',
                            glium::glutin::VirtualKeyCode::W => char_to_add = 'w',
                            glium::glutin::VirtualKeyCode::X => char_to_add = 'x',
                            glium::glutin::VirtualKeyCode::Y => char_to_add = 'y',
                            glium::glutin::VirtualKeyCode::Z => char_to_add = 'z',
                            glium::glutin::VirtualKeyCode::Period => char_to_add = '.',
                            glium::glutin::VirtualKeyCode::Comma => char_to_add = ',',
                            glium::glutin::VirtualKeyCode::Apostrophe => char_to_add = '\'',
                            glium::glutin::VirtualKeyCode::Semicolon => char_to_add = ';',
                            glium::glutin::VirtualKeyCode::LBracket => char_to_add = '[',
                            glium::glutin::VirtualKeyCode::RBracket => char_to_add = ']',
                            glium::glutin::VirtualKeyCode::Key0 => char_to_add = '0',
                            glium::glutin::VirtualKeyCode::Key1 => char_to_add = '1',
                            glium::glutin::VirtualKeyCode::Key2 => char_to_add = '2',
                            glium::glutin::VirtualKeyCode::Key3 => char_to_add = '3',
                            glium::glutin::VirtualKeyCode::Key4 => char_to_add = '4',
                            glium::glutin::VirtualKeyCode::Key5 => char_to_add = '5',
                            glium::glutin::VirtualKeyCode::Key6 => char_to_add = '6',
                            glium::glutin::VirtualKeyCode::Key7 => char_to_add = '7',
                            glium::glutin::VirtualKeyCode::Key8 => char_to_add = '8',
                            glium::glutin::VirtualKeyCode::Key9 => char_to_add = '9',

                            glium::glutin::VirtualKeyCode::LShift => { shift = true; },
                            glium::glutin::VirtualKeyCode::Space => char_to_add = ' ',
                            glium::glutin::VirtualKeyCode::Back => {
                                if !terminal.is_empty() && terminal.last().unwrap().len() >= 1 {
                                    terminal.last_mut().unwrap().pop();
                                } else if terminal.len() >= 2 {
                                    terminal.pop();
                                }
                            },
                            glium::glutin::VirtualKeyCode::Return => {
                                if shift {
                                    let mut file = File::create("programs/hello.asm").unwrap();
                                    for line in terminal.iter() {
                                        let mut nline = line.clone();
                                        nline.push('\n');
                                        file.write_all(nline.as_bytes()).unwrap();
                                    }
                                    file.sync_data().unwrap();
                                    err = zpu::assembler::assemble_program("programs/hello.asm", "programs/zpu.bin");
                                    zpu.load_program("programs/zpu.bin");
                                } else {
                                    terminal.push(String::new());
                                }
                            }
                            glium::glutin::VirtualKeyCode::Escape => {
                                term_ui = false;
                                term_collide = false;
                            },
                            _ => { },
                        }

                        if char_to_add != '|' {
                            let term_text = terminal.last_mut().unwrap();
                            if shift {
                                let print_char = match char_to_add {
                                    '1' => '!',
                                    '2' => '@',
                                    '3' => '#',
                                    '4' => '$',
                                    '5' => '%',
                                    '6' => '^',
                                    '7' => '&',
                                    '8' => '*',
                                    '9' => '(',
                                    '0' => ')',
                                    '[' => '{',
                                    ']' => '}',
                                    ';' => ':',
                                    '\'' => '\"',
                                    ',' => '<',
                                    '.' => '>',
                                    '/' => '?',
                                    '-' => '_',
                                    '=' => '+',
                                    _ => char_to_add.to_uppercase().next().unwrap(),
                                };
                                term_text.push(print_char);
                            } else {
                                term_text.push(char_to_add);
                            }
                            typed = true;
                        }
                    } else if key.is_some() && state == glium::glutin::ElementState::Released && term_ui {
                        let key = key.unwrap();
                        match key {
                            glium::glutin::VirtualKeyCode::LShift => { shift = false; },
                            _ => { },
                        }
                        typed = true;
                    }
                },
                _ => (),
            }
        }

        if result.output.is_some() {
            let output = result.output.unwrap();
            if output.port == 1 {
                terminal.push(String::from(format!(";{}", (output.data as u8) as char)));
            } else if output.port == 0 {
                terminal.push(String::from(format!(";{}", output.data)));
            } else if output.port == 2 {
                if output.data > 0 {
                    eng_id = on_engine_id;
                } else {
                    eng_id = off_engine_id;
                }
            } else if output.port == 3 {
                rot = zero + (output.data as f32) / 10.0;
            } else if output.port == 4 {
                rot = zero - ((output.data as f32) / 10.0);
            } else if output.port == 5 {
                if output.data > 0 {
                    tur_id = on_turret_id;
                } else {
                    tur_id = off_turret_id;
                }
            }
        }

        if !typed {
            let term_text = terminal.last_mut().unwrap();
            term_text.push('|');
        }

        let mut cy = 0.0;
        let mut cx = 0.0;
        if inputs.has_update() && !term_ui {
            for key in inputs.keys.iter() {
                if *key.1 == keyboard::KeyState::Pressed {
                    match *key.0 {
                        keyboard::Action::Left => { cx -= speed; },
                        keyboard::Action::Right => { cx += speed; },
                        keyboard::Action::Up => { cy += speed; },
                        keyboard::Action::Down => { cy -= speed; },
                        keyboard::Action::Enter => { term_ui = true; },
                        keyboard::Action::Back => { break 'main; }
                        _ => { },
                    }
                }
            }
        }

        let friction = -0.4;

        let x_acc = friction * player.vel.x + cx;
        let y_acc = friction * player.vel.y + cy;

        let tmpx = (0.5 * x_acc * dt * dt) + player.vel.x * dt + player.pos.x;
        let tmpy = (0.5 * y_acc * dt * dt) + player.vel.y * dt + player.pos.y;

        for (i, item) in collidables.iter().enumerate() {
            if item.collides(tmpx, tmpy) {
                collided = true;
                if i == 0 {
                    term_collide = true;
                }
            }
        }

        if !term_collide {
            term_ui = false;
        }

        if !collided {
            player.pos.x = tmpx;
            player.pos.y = tmpy;
            player.vel.x = (x_acc * dt) + player.vel.x;
            player.vel.y = (y_acc * dt) + player.vel.y;
        } else {
            collided = false;
            player.vel.x = -player.vel.x * 0.20;
            player.vel.y = -player.vel.y * 0.20;
        }

        camera.pos.x = player.pos.x * 2.0;
        camera.pos.y = player.pos.y * 2.0;

		let mut target = display.draw();
		target.clear_color(0.0, 0.0, 0.0, 1.0);

        let view = view_matrix(&[camera.pos.x, camera.pos.y, -15.0], &[0.0, 0.0, 1.0], &[0.0, 1.0, 0.0]);

        let base_uniforms = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [1.0 * tile_gap, 1.0 * tile_gap, 0.0, 1.0f32],
            ],
    		view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        for y in 0..map_height {
            for x in 0..map_width {
                let wall_uniform = uniform! {
                    model: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [(x as f32) * tile_gap, (y as f32) * tile_gap, 0.0, 1.0f32],
                    ],
                    view: view,
                    perspective: perspective,
                    tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                };

                let idx = y * map_width + x;
                let tile = tile_atlas.atlas.get(map[idx] as usize);
                if tile.is_some() {
                    let tile = tile.unwrap();
                    target.draw(tile, &indices, &game_program, &wall_uniform, &params).unwrap();
                }
            }
        }

        let turret_uniforms = uniform! {
            model: [
                [rot.sin(), rot.cos(), 0.0, 0.0],
                [-rot.cos(), rot.sin(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [1.0 * tile_gap, 1.0 * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let term_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [3.0 * tile_gap, 3.0 * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let engine_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [5.0 * tile_gap, 3.0 * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let chair_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [7.0 * tile_gap, 3.0 * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let player_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [player.pos.x * 2.0, player.pos.y * 2.0, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let term_buffer = tile_atlas.atlas.get(term_id).unwrap();
        let chair_buffer = tile_atlas.atlas.get(chair_id).unwrap();
        let turret_base_buffer = tile_atlas.atlas.get(turret_base_id).unwrap();

        let player_buffer = tile_atlas.atlas.get(player_id).unwrap();
        let eng_buffer = tile_atlas.atlas.get(eng_id).unwrap();
        let tur_buffer = tile_atlas.atlas.get(tur_id).unwrap();

        target.draw(term_buffer, &indices, &game_program, &term_uniform, &params).unwrap();
        target.draw(chair_buffer, &indices, &game_program, &chair_uniform, &params).unwrap();
        target.draw(turret_base_buffer, &indices, &game_program, &base_uniforms, &params).unwrap();

        target.draw(tur_buffer, &indices, &game_program, &turret_uniforms, &params).unwrap();
        target.draw(eng_buffer, &indices, &game_program, &engine_uniform, &params).unwrap();

        target.draw(player_buffer, &indices, &game_program, &player_uniform, &params).unwrap();

        if term_ui && term_collide {
            let termui_left_uniform = uniform! {
                model: [
                    [0.25, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-0.75, 0.5, 0.0, 1.0f32],
                ],
                tex: term_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };
            let termui_right_uniform = uniform! {
                model: [
                    [0.35, 0.0, 0.0, 0.0],
                    [0.0, 1.25, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.70, 0.35, 0.0, 1.0f32],
                ],
                tex: term_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            target.draw(&termui_buffer, &indices, &ui_program, &termui_left_uniform, &params).unwrap();
            target.draw(&termui_buffer, &indices, &ui_program, &termui_right_uniform, &params).unwrap();

            for (i, line) in terminal.iter().enumerate() {
                let console_matrix = [
                    [0.035 * ratio, 0.0, 0.0, 0.0],
                    [0.0, 0.035, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-1.0, 0.95 - ((i as f32) * 0.05), 0.0, 1.0],
                ];

                let console_text = glium_text::TextDisplay::new(&text_system, &font, line.as_str());
                glium_text::draw(&console_text, &text_system, &mut target, console_matrix, (0.0, 1.0, 0.0, 1.0));
            }

            for (i, line) in guide.iter().enumerate() {
                let console_matrix = [
                    [0.035 * ratio, 0.0, 0.0, 0.0],
                    [0.0, 0.035, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.35, 0.95 - ((i as f32) * 0.05), 0.0, 1.0],
                ];

                let console_text = glium_text::TextDisplay::new(&text_system, &font, line.as_str());
                glium_text::draw(&console_text, &text_system, &mut target, console_matrix, (1.0, 1.0, 1.0, 1.0));
            }
        }

        if !err.compile_err.is_empty() {
            let console_matrix = [
                [0.035 * ratio, 0.0, 0.0, 0.0],
                [0.0, 0.035, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, -0.95, 0.0, 1.0],
            ];
            {
                let console_text = glium_text::TextDisplay::new(&text_system, &font, err.compile_err.as_str());
                glium_text::draw(&console_text, &text_system, &mut target, console_matrix, (1.0, 0.0, 0.0, 1.0));
            }
        }

        target.finish().unwrap();

        if !typed {
            let term_text = terminal.last_mut().unwrap();
            term_text.pop();
        }

        let end_time = time::precise_time_ns();
		dt = ((end_time - start_time) as f32 / 1e6) / 60.0;
        acc_time += dt;
    }
}
