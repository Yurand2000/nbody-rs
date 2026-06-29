pub mod up_simulation;

pub type Vec2 = nalgebra::Vector2<f32>;

pub type BodyId = usize;

pub struct Body {
    pub mass: f32,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub struct Simulator {
    params: SimulationParams,
    bodies: Vec<Body>,
}

pub enum SimulationType {
    SingleCore,
    MultiCore,
    CUDA,
}

pub struct SimulationParams {
    pub typ: SimulationType,
    pub time_step: f32,
    pub theta: f32,
    pub grav_constant: f32,
}

impl Simulator {
    pub fn new(params: SimulationParams) -> Self {
        Self { params, bodies: Vec::new() }
    }

    pub fn insert_body(&mut self, body: Body) -> BodyId {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    pub fn step(&mut self) {
        match self.params.typ {
            SimulationType::SingleCore => self.step_uniproc(),
            SimulationType::MultiCore => todo!(),
            SimulationType::CUDA => todo!(),
        }
    }

    pub fn get_body(&self, id: BodyId) -> &Body {
        &self.bodies[id]
    }
}