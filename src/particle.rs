use vert::Point;

pub struct Particle {
    pub o_lifespan: f32,
    pub c_lifespan: f32,
    pub sprite: usize,
    pub o_pos: Point,
    pub c_pos: Point,
    pub vel: Point,
    pub angle: f32,
    pub render: bool
}

impl Particle {
    pub fn new(sprite_id: usize, x: f32, y: f32, angle: f32, lifespan: f32) -> Particle {
        Particle {
            o_lifespan: lifespan,
            c_lifespan: lifespan,
            sprite: sprite_id,
            o_pos: Point::new(x, y),
            c_pos: Point::new(x, y),
            vel: Point::new(0.0, 0.0),
            angle: angle,
            render: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.c_lifespan > 0.0 {
            self.c_pos.x += 0.2 * dt;
            self.c_pos.y += 0.2 * dt;
            self.c_lifespan -= dt;
        } else {
            self.c_pos.x = self.o_pos.x;
            self.c_pos.y = self.o_pos.y;
            self.render = false;
        }
    }

    pub fn reset(&mut self) {
        self.c_lifespan = self.o_lifespan;
        self.render = true;
    }
}
