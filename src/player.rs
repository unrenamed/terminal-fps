#[derive(Debug, Default)]
pub struct Player {
    x: f32,
    y: f32,
    a: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, a: f32) -> Self {
        Self { x, y, a }
    }

    pub fn x(&self) -> f32 {
        return self.x;
    }

    pub fn y(&self) -> f32 {
        return self.y;
    }

    pub fn a(&self) -> f32 {
        return self.a;
    }

    pub fn move_forward(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn move_back(&mut self, dx: f32, dy: f32) {
        self.x -= dx;
        self.y -= dy;
    }

    pub fn turn_left(&mut self, da: f32) {
        self.a -= da;
    }

    pub fn turn_right(&mut self, da: f32) {
        self.a += da;
    }
}
