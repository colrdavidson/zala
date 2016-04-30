#[macro_use]
extern crate glium;
extern crate image;
extern crate glium_text;
extern crate time;
extern crate zpu;

pub mod vert;
pub mod keyboard;

use std::io::{Write, Read};
use std::fs::File;
use std::io::Cursor;
use std::f32;

use glium::{DisplayBuild, Surface};
use glium::glutin;

use keyboard::Inputs;

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

struct Camera {
    pos: Point,
    vel: Point,
}

impl Camera {
    fn new(x: f32, y: f32) -> Camera {
        Camera {
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

    let atlas_img = image::load(Cursor::new(&include_bytes!("../assets/atlas.png")[..]), image::PNG).unwrap().to_rgba();
	let atlas_dims = atlas_img.dimensions();
	let atlas_img = glium::texture::RawImage2d::from_raw_rgba_reversed(atlas_img.into_raw(), atlas_dims);
	let atlas_tex = glium::texture::SrgbTexture2d::new(&display, atlas_img).unwrap();

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

    let term = atlas_verts(39, 64);
    let player = atlas_verts(11, 64);

    let top_wall = atlas_verts(15, 64);
    let left_wall = atlas_verts(6, 64);
    let bl_corner = atlas_verts(5, 64);
    let br_corner = atlas_verts(21, 64);
    let tl_corner = atlas_verts(7, 64);
    let tr_corner = atlas_verts(23, 64);
    let right_wall = atlas_verts(22, 64);
    let bot_wall = atlas_verts(13, 64);
    let floor = atlas_verts(14, 64);

    let on_engine = atlas_verts(3, 64);
    let off_engine = atlas_verts(4, 64);

    let on_turret = atlas_verts(12, 64);
    let off_turret = atlas_verts(20, 64);
    let turret_base = atlas_verts(28, 64);

    let door_closed = atlas_verts(29, 64);
    let door_open = atlas_verts(30, 64);

    let term_buffer = glium::VertexBuffer::new(&display, &term).unwrap();
    let player_buffer = glium::VertexBuffer::new(&display, &player).unwrap();

    let top_wall_buffer = glium::VertexBuffer::new(&display, &top_wall).unwrap();
    let left_wall_buffer = glium::VertexBuffer::new(&display, &left_wall).unwrap();
    let right_wall_buffer = glium::VertexBuffer::new(&display, &right_wall).unwrap();
    let bot_wall_buffer = glium::VertexBuffer::new(&display, &bot_wall).unwrap();

    let bl_corner_buffer = glium::VertexBuffer::new(&display, &bl_corner).unwrap();
    let br_corner_buffer = glium::VertexBuffer::new(&display, &br_corner).unwrap();
    let tl_corner_buffer = glium::VertexBuffer::new(&display, &tl_corner).unwrap();
    let tr_corner_buffer = glium::VertexBuffer::new(&display, &tr_corner).unwrap();

    let floor_buffer = glium::VertexBuffer::new(&display, &floor).unwrap();

    let on_engine_buffer = glium::VertexBuffer::new(&display, &on_engine).unwrap();
    let off_engine_buffer = glium::VertexBuffer::new(&display, &off_engine).unwrap();

    let on_turret_buffer = glium::VertexBuffer::new(&display, &on_turret).unwrap();
    let off_turret_buffer = glium::VertexBuffer::new(&display, &off_turret).unwrap();
    let turret_base_buffer = glium::VertexBuffer::new(&display, &turret_base).unwrap();

    let door_open_buffer = glium::VertexBuffer::new(&display, &door_open).unwrap();
    let door_closed_buffer = glium::VertexBuffer::new(&display, &door_closed).unwrap();

    let zero = f32::consts::PI / 2.0;
    let mut rot = zero;

    let mut inputs = Inputs::new();

    let mut map_str = String::new();
    File::open("assets/map").unwrap().read_to_string(&mut map_str).unwrap();
    let map_strs: Vec<&str> = map_str.split_whitespace().collect();
    let mut map = Vec::new();
    for n in map_strs.iter() {
        let val: u32 = n.parse().unwrap();
        map.push(val);
    }

    let mut camera = Camera::new(10.0, 5.0);

    let term_rect = Rect::new(5.0, 5.0, 7.0, 7.0);
    let turret_rect = Rect::new(1.0, 1.0, 3.0, 3.0);

    let collidables = vec![term_rect, turret_rect];

    let mut dt = 0.0;
    let mut acc_time = 0.0;
    let speed = 0.2;

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

    let mut eng_buffer = &off_engine_buffer;
    let mut tur_buffer = &off_turret_buffer;
    let mut err = zpu::assembler::assemble_program("programs/hello.asm", "programs/zpu.bin");
    let mut zpu = zpu::zpu::ZPU::new("programs/zpu.bin");

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    'main: loop {
        let mut typed = false;
        let start_time = time::precise_time_ns();
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
                    eng_buffer = &on_engine_buffer;
                } else {
                    eng_buffer = &off_engine_buffer;
                }
            } else if output.port == 3 {
                rot = zero + (output.data as f32) / 10.0;
            } else if output.port == 4 {
                rot = zero - ((output.data as f32) / 10.0);
            } else if output.port == 5 {
                if output.data > 0 {
                    tur_buffer = &on_turret_buffer;
                } else {
                    tur_buffer = &off_turret_buffer;
                }
            }
        }

        if !typed {
            let term_text = terminal.last_mut().unwrap();
            term_text.push('|');
        }

        let mut cy = 0.0;
        let mut cx = 0.0;
        let mut turret_inputs = Vec::new();
        if inputs.has_update() && !term_ui {
            for key in inputs.keys.iter() {
                if *key.1 == keyboard::KeyState::Pressed {
                    match *key.0 {
                        keyboard::Action::RotateLeft => { turret_inputs.push(key.0); },
                        keyboard::Action::RotateRight => { turret_inputs.push(key.0); },
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

        let x_acc = friction * camera.vel.x + cx;
        let y_acc = friction * camera.vel.y + cy;

        let tmpx = (0.5 * x_acc * dt * dt) + camera.vel.x * dt + camera.pos.x;
        let tmpy = (0.5 * y_acc * dt * dt) + camera.vel.y * dt + camera.pos.y;

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
            camera.pos.x = tmpx;
            camera.pos.y = tmpy;
            camera.vel.x = (x_acc * dt) + camera.vel.x;
            camera.vel.y = (y_acc * dt) + camera.vel.y;
        } else {
            collided = false;
            camera.vel.x = 0.0;
            camera.vel.y = 0.0;
        }

		let mut target = display.draw();
		target.clear_color(0.0, 0.0, 0.0, 1.0);

        let view = view_matrix(&[camera.pos.x, camera.pos.y, -15.0], &[0.0, 0.0, 1.0], &[0.0, 1.0, 0.0]);

        let base_uniforms = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [turret_rect.bl.x * 2.0, turret_rect.bl.y * 2.0, 0.0, 1.0f32],
            ],
    		view: view,
            perspective: perspective,
            tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let width = 11;
        let height = 17;
        for y in 0..height {
            for x in 0..width {
                let wall_uniform = uniform! {
                    model: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [(x as f32) * 2.0, (y as f32) * 2.0, 0.0, 1.0f32],
                    ],
                    view: view,
                    perspective: perspective,
                    tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                };

                let idx = y * width + x;
                match map[idx] {
                    5 => target.draw(&tl_corner_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    6 => target.draw(&left_wall_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    7 => target.draw(&bl_corner_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    13 => target.draw(&top_wall_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    14 => target.draw(&floor_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    15 => target.draw(&bot_wall_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    21 => target.draw(&tr_corner_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    22 => target.draw(&right_wall_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    23 => target.draw(&br_corner_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    29 => target.draw(&door_closed_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    30 => target.draw(&door_open_buffer, &indices, &game_program, &wall_uniform, &params).unwrap(),
                    _ => (),
                }
            }
        }

        for key in turret_inputs {
            match *key {
                keyboard::Action::RotateLeft => { rot -= 0.02; },
                keyboard::Action::RotateRight => { rot += 0.02; },
                keyboard::Action::Space => {
                    tur_buffer = &on_turret_buffer;
                },
                _ => { },
            }
        }

        let turret_uniforms = uniform! {
            model: [
                [rot.sin(), rot.cos(), 0.0, 0.0],
                [-rot.cos(), rot.sin(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [turret_rect.bl.x * 2.0, turret_rect.bl.y * 2.0, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let term_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [3.0 * 2.0, 3.0 * 2.0, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let engine_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [5.0 * 2.0, 3.0 * 2.0, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let player_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [camera.pos.x, camera.pos.y, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: atlas_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        target.draw(&term_buffer, &indices, &game_program, &term_uniform, &params).unwrap();

        target.draw(&turret_base_buffer, &indices, &game_program, &base_uniforms, &params).unwrap();
        target.draw(tur_buffer, &indices, &game_program, &turret_uniforms, &params).unwrap();
        target.draw(eng_buffer, &indices, &game_program, &engine_uniform, &params).unwrap();
        target.draw(&player_buffer, &indices, &game_program, &player_uniform, &params).unwrap();

//        println!("ui: {}, collide: {}", term_ui, term_collide);
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
