pub type Vec2 = nalgebra::Vector2<f32>;

pub type BodyId = usize;

pub struct Body {
    pub mass: f32,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub struct System {
    params: SystemParams,
    bodies: Vec<Body>,
}

pub struct SystemParams {
    pub time_step: f32,
}

impl System {
    pub fn new(params: SystemParams) -> Self {
        Self { params, bodies: Vec::new() }
    }

    pub fn insert_body(&mut self, body: Body) -> BodyId {
        self.bodies.push(body);
        self.bodies.len() - 1
    }
}