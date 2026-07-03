pub mod up_simulation;
pub mod cuda_simulation;

pub mod quadtree;

pub use quadtree::*;

pub type Vec2f = nalgebra::Vector2<f32>;

pub type BodyId = usize;

pub struct Body {
    pub mass: f32,
    pub position: Vec2f,
    pub velocity: Vec2f,
}

#[derive(Debug, Clone, Copy)]
pub struct SimulationParams {
    pub time_step: f32,
    pub theta: f32,
    pub g_constant: f32,
}

pub trait Simulator {
    fn get_params(&self) -> SimulationParams;
    fn set_params(&mut self, params: SimulationParams);
    fn insert_body(&mut self, body: Body) -> BodyId;
    fn step(&mut self);
    fn pull_updates(&mut self);
    fn get_bodies(&self) -> &[Body];
    fn get_body(&self, id: BodyId) -> &Body;
}

pub struct SingleBodyForce {
    pub com: Vec2f,
    pub force: Vec2f,
}

pub trait SimulatorExt {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> { None }
    fn get_forces(&self, _quadtree: &QuadTree<QuadTreeCell>) -> Option<Vec<Vec2f>> { None }
    fn get_body_forces(&self, _quadtree: &QuadTree<QuadTreeCell>, _id: BodyId) -> Option<Vec<SingleBodyForce>> { None }
}

pub enum MultiSimulator {
    UpSimulator(up_simulation::UpSimulator),
    CUDASimulator(cuda_simulation::CUDASimulator),
}

impl MultiSimulator {
    pub fn new_up(params: SimulationParams) -> Self {
        Self::UpSimulator(up_simulation::UpSimulator::new(params))
    }

    pub fn new_cuda(params: SimulationParams) -> Self {
        let mut sim = Self::CUDASimulator(cuda_simulation::CUDASimulator::new());

        sim.set_params(params);
        sim
    }
}

impl Simulator for MultiSimulator {
    fn get_params(&self) -> SimulationParams {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_params(),
            CUDASimulator(sim) => sim.get_params(),
        }
    }

    fn set_params(&mut self, params: SimulationParams) {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.set_params(params),
            CUDASimulator(sim) => sim.set_params(params),
        }
    }

    fn insert_body(&mut self, body: Body) -> BodyId {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.insert_body(body),
            CUDASimulator(sim) => sim.insert_body(body),
        }
    }

    fn step(&mut self) {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.step(),
            CUDASimulator(sim) => sim.step(),
        }
    }

    fn pull_updates(&mut self) {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.pull_updates(),
            CUDASimulator(sim) => sim.pull_updates(),
        }
    }

    fn get_bodies(&self) -> &[Body] {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_bodies(),
            CUDASimulator(sim) => sim.get_bodies(),
        }
    }

    fn get_body(&self, id: BodyId) -> &Body {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_body(id),
            CUDASimulator(sim) => sim.get_body(id),
        }
    }
}

impl SimulatorExt for MultiSimulator {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_quadtree(),
            CUDASimulator(sim) => sim.get_quadtree(),
        }
    }

    fn get_forces(&self, quadtree: &QuadTree<QuadTreeCell>) -> Option<Vec<Vec2f>> {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_forces(quadtree),
            CUDASimulator(sim) => sim.get_forces(quadtree),
        }
    }

    fn get_body_forces(&self, quadtree: &QuadTree<QuadTreeCell>, id: BodyId) -> Option<Vec<SingleBodyForce>> {
        use MultiSimulator::*;

        match self {
            UpSimulator(sim) => sim.get_body_forces(quadtree, id),
            CUDASimulator(sim) => sim.get_body_forces(quadtree, id),
        }
    }
}