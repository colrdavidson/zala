#[macro_use]
extern crate glium;
extern crate image;
extern crate glium_text;
extern crate time;
extern crate zpu;

pub mod vert;
pub mod keyboard;
pub mod tile;
pub mod particle;

use std::io::{Write, Read};
use std::fs::File;
use std::io::Cursor;
use std::f32;

use glium::{Surface};
use glium::glutin::{self, Event, WindowEvent, KeyboardInput};

//use particle::Particle;
use keyboard::Inputs;
use tile::TileAtlas;
use tile::TileCollide;
use tile::Door;

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

struct Entity {
    pos: vert::Point,
    vel: vert::Point,
}

impl Entity {
    fn new(x: f32, y: f32) -> Entity {
        Entity {
            pos: vert::Point::new(x, y),
            vel: vert::Point::new(0.0, 0.0),
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let (width, height) = (640, 480);
    let ratio = height as f32 / width as f32;
    let display_builder = glutin::WindowBuilder::new()
        .with_dimensions((width, height).into())
        .with_title(format!("Zala"));
    let context = glutin::ContextBuilder::new();
        //.with_vsync(true);
    let display = glium::Display::new(display_builder, context, &events_loop).unwrap();

    let term_img = image::load(Cursor::new(&include_bytes!("../assets/term.png")[..]), image::PNG).unwrap().to_rgba();
    let term_dims = term_img.dimensions();
    let term_img = glium::texture::RawImage2d::from_raw_rgba_reversed(&term_img.into_raw(), term_dims);
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

    let cursor_vert_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec4 v_color;

        uniform mat4 model;
        uniform vec4 color;

        void main() {
            v_color = color;
            gl_Position = model * vec4(position, 0.0, 1.0);
        }
    "#;

    let cursor_frag_shader_src = r#"
        #version 140

        in vec4 v_color;
        out vec4 color;

        void main() {
            color = v_color;
        }
    "#;

    let game_frag_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let ui_frag_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let game_program = glium::Program::from_source(&display, game_vert_shader_src, game_frag_shader_src, None).unwrap();
    let ui_program = glium::Program::from_source(&display, ui_vert_shader_src, ui_frag_shader_src, None).unwrap();
    let cursor_program = glium::Program::from_source(&display, cursor_vert_shader_src, cursor_frag_shader_src, None).unwrap();

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

    let zero = f32::consts::PI / 2.0;
    let mut rot = zero;

    let mut inputs = Inputs::new();

    let tile_gap = 2.0;

    let turret_rect = TileCollide::new(1.0, 1.0);
    let term_rect = TileCollide::new(3.0, 3.0);
    let eng_rect = TileCollide::new(1.0, 3.0);
    let chair_rect = TileCollide::new(5.0, 14.0);
    let mut door = Door::new(vec![29, 30], 2.0, 4.0, true);

    let mut collidables = vec![term_rect, chair_rect, door.get_state().collision_box, turret_rect, eng_rect];

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

    let mut player = Entity::new(2.0, 2.0);
    let mut ship_power = 0.0;

    let mut cur_x = terminal.last().unwrap().len();
    let mut cur_y = terminal.len() - 1;
    //let mut bullet = Particle::new(11, 1.0, 1.0, 0.0, 20.0);

    'main: loop {
        let start_time = time::precise_time_ns();

        //let mut typed = false;
        let mut result = zpu::zpu::ZResult::new(false, None);

        if acc_time > 5.0 {
            result = zpu.step();
            acc_time = 0.0;
        }

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => return,
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: key, .. }, .. }, .. } => {
                    if key.is_some() && !term_ui {
                        let key = key.unwrap();
                        inputs.update(key, state);
                    } else if key.is_some() && state == glium::glutin::ElementState::Pressed && term_ui {
                        inputs.release_keys();
                        let key = key.unwrap();
                        let mut char_to_add = '~';
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
                                if !terminal.is_empty() && terminal[cur_y].len() >= 1 {
                                    if cur_x == terminal[cur_y].len() {
                                        terminal[cur_y].pop();
                                        cur_x -= 1;
                                    } else {
                                        cur_x -= 1;
                                    }
                                } else if terminal.len() >= 2 {
                                    if cur_y < terminal.len() {
                                        terminal.remove(cur_y);
                                    }
                                    cur_y -= 1;
                                    cur_x = terminal[cur_y].len();
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
                                    if terminal[cur_y].len() > cur_x {
                                        println!("cursor: {},{}", cur_x, cur_y);
                                    }
                                    cur_y += 1;
                                    cur_x = 0;
                                    terminal.insert(cur_y, String::new());
                                }
                            }
                            glium::glutin::VirtualKeyCode::Escape => {
                                term_ui = false;
                                term_collide = false;
                            },
                            glium::glutin::VirtualKeyCode::Up => {
                                if cur_y > 0 {
                                    cur_y -= 1;
                                }
                            },
                            glium::glutin::VirtualKeyCode::Down => {
                                cur_y += 1;
                            },
                            glium::glutin::VirtualKeyCode::Left => {
                                if cur_x > 0 {
                                    cur_x -= 1;
                                }
                            },
                            glium::glutin::VirtualKeyCode::Right => {
                                cur_x += 1;
                            },
                            _ => { },
                        }

                        if shift {
                            char_to_add = match char_to_add {
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
                        }
                        if char_to_add != '~' {
                            terminal[cur_y].push(char_to_add);
                            cur_x += 1;
                        }
                    } else if key.is_some() && state == glium::glutin::ElementState::Released && term_ui {
                        let key = key.unwrap();
                        match key {
                            glium::glutin::VirtualKeyCode::LShift => { shift = false; },
                            _ => { },
                        }
                    }
                },
                _ => (),
            }
        });

        if result.output.is_some() {
            let output = result.output.unwrap();
            if output.port == 0 {
                terminal.push(String::from(format!(";{}", output.data)));
            } else if output.port == 1 {
                terminal.push(String::from(format!(";{}", (output.data as u8) as char)));
            } else if output.port == 2 {
                if output.data > 0 {
                    eng_id = on_engine_id;
                    ship_power = 1.0;
                } else {
                    ship_power = 0.0;
                    eng_id = off_engine_id;
                }
            } else if output.port == 3 {
                if ship_power > 0.0 && tur_id == on_turret_id {
                    rot = zero + (output.data as f32) / 10.0;
                }
            } else if output.port == 4 {
                if ship_power > 0.0 && tur_id == on_turret_id {
                    rot = zero - ((output.data as f32) / 10.0);
                }
            } else if output.port == 5 {
                if output.data > 0 && ship_power > 0.0 {
                    tur_id = on_turret_id;
                    ship_power -= 0.1;
                    //bullet.reset();
                } else {
                    tur_id = off_turret_id;
                }
            } else if output.port == 6 {
                if output.data > 0 {
                    if ship_power > 0.0 {
                        door.close();
                    }
                } else {
                    if ship_power > 0.0 {
                        door.open();
                    }
                }
            }
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

        //bullet.update(dt);

        let friction = -0.4;

        let x_acc = friction * player.vel.x + cx;
        let y_acc = friction * player.vel.y + cy;

        let tmpx = (0.5 * x_acc * dt * dt) + player.vel.x * dt + player.pos.x;
        let tmpy = (0.5 * y_acc * dt * dt) + player.vel.y * dt + player.pos.y;

        collidables.insert(2, door.get_state().collision_box);
        collidables.remove(3);
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

        camera.pos.x = player.pos.x * tile_gap;
        camera.pos.y = player.pos.y * tile_gap;

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

        let door_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [2.0 * tile_gap, 4.0 * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

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
                [1.0 * tile_gap, 3.0 * tile_gap, 0.0, 1.0f32],
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
                [5.0 * tile_gap, 14.0 * tile_gap, 0.0, 1.0f32],
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
                [player.pos.x * tile_gap, player.pos.y * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        /*let bullet_uniform = uniform! {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [bullet.c_pos.x * tile_gap, bullet.c_pos.y * tile_gap, 0.0, 1.0f32],
            ],
            view: view,
            perspective: perspective,
            tex: tile_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };
        let bullet_buffer = tile_atlas.atlas.get(bullet.sprite).unwrap();*/

        let door_buffer = tile_atlas.atlas.get(door.get_state().sprite).unwrap();
        let term_buffer = tile_atlas.atlas.get(term_id).unwrap();
        let chair_buffer = tile_atlas.atlas.get(chair_id).unwrap();
        let turret_base_buffer = tile_atlas.atlas.get(turret_base_id).unwrap();

        let player_buffer = tile_atlas.atlas.get(player_id).unwrap();
        let eng_buffer = tile_atlas.atlas.get(eng_id).unwrap();
        let tur_buffer = tile_atlas.atlas.get(tur_id).unwrap();

        target.draw(door_buffer, &indices, &game_program, &door_uniform, &params).unwrap();
        target.draw(term_buffer, &indices, &game_program, &term_uniform, &params).unwrap();
        target.draw(chair_buffer, &indices, &game_program, &chair_uniform, &params).unwrap();
        target.draw(turret_base_buffer, &indices, &game_program, &base_uniforms, &params).unwrap();

        target.draw(tur_buffer, &indices, &game_program, &turret_uniforms, &params).unwrap();
        target.draw(eng_buffer, &indices, &game_program, &engine_uniform, &params).unwrap();

        /*if bullet.render {
            target.draw(bullet_buffer, &indices, &game_program, &bullet_uniform, &params).unwrap();
        }*/
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
                    [-0.99, 0.95 - ((i as f32) * 0.05), 0.0, 1.0],
                ];

                let console_text = glium_text::TextDisplay::new(&text_system, &font, line.as_str());
                glium_text::draw(&console_text, &text_system, &mut target, console_matrix, (0.0, 1.0, 0.0, 1.0));
            }

            let cursor_buffer = &termui_buffer;
            let cursor_uniform = uniform! {
                model: [
                    [0.005 * ratio, 0.0, 0.0, 0.0],
                    [0.0, 0.0225, 0.0, 0.0],
                    [0.0, 0.0, 0.1, 0.0],
                    [-0.995 + ((cur_x as f32) * 0.0235), 0.96 - ((cur_y as f32) * 0.05), 0.0, 1.0f32],
                ],
                color: [1.0, 1.0, 0.0, 1.0f32],
            };

            target.draw(cursor_buffer, &indices, &cursor_program, &cursor_uniform, &params).unwrap();

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
        } else {
            let console_matrix = [
                [0.035 * ratio, 0.0, 0.0, 0.0],
                [0.0, 0.035, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.65, 0.95, 0.0, 1.0],
            ];
            let console_text = glium_text::TextDisplay::new(&text_system, &font, format!("Ship Power: {}", ship_power).as_str());
            glium_text::draw(&console_text, &text_system, &mut target, console_matrix, (1.0, 1.0, 1.0, 1.0));
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

        let end_time = time::precise_time_ns();
		dt = ((end_time - start_time) as f32 / 1e6) / 60.0;
        acc_time += dt;
    }
}
