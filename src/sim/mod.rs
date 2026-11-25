pub mod async_simulator;

pub mod up_simulation;
#[cfg(feature = "cuda")]
pub mod cuda_simulation;

pub mod quadtree;

pub use quadtree::*;
pub type Vec2f = nalgebra::Vector2<f32>;
pub type BodyId = usize;

#[derive(Debug, Clone)]
pub struct Body {
    pub mass: f32,
    pub position: Vec2f,
    pub velocity: Vec2f,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SimulationParams {
    pub time_step: f32,
    pub theta: f32,
    pub g_constant: f32,
}

pub trait Simulator {
    fn get_params(&self) -> SimulationParams;
    fn set_params(&mut self, params: SimulationParams);
    fn reset(&mut self);
    fn insert_body(&mut self, body: Body) -> BodyId;
    fn step(&mut self);
    fn pull_updates(&mut self);
    fn get_bodies(&self) -> &[Body];
    fn get_body(&self, id: BodyId) -> &Body;
}

#[derive(Clone)]
pub struct SingleBodyForce {
    pub com: Vec2f,
    pub force: Vec2f,
}

pub trait SimulatorExt {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> { None }
    fn get_forces(&self) -> Option<Vec<Vec2f>> { None }
    fn get_body_forces(&self, _id: BodyId) -> Option<Vec<SingleBodyForce>> { None }
}

pub enum MultiSimulator {
    Uninit,
    UpSimulator(up_simulation::UpSimulator),

    #[cfg(feature = "cuda")]
    CUDASimulator(cuda_simulation::CUDASimulator),
}

impl MultiSimulator {
    pub fn empty() -> Self { Self::Uninit }

    pub fn init_up(&mut self, params: SimulationParams) {
        *self = Self::UpSimulator(up_simulation::UpSimulator::new(params));
    }

    #[cfg(feature = "cuda")]
    pub fn init_cuda(&mut self, params: SimulationParams, cuda_params: cuda_simulation::CUDASimulatorParams) {
        *self = Self::CUDASimulator(cuda_simulation::CUDASimulator::new(cuda_params));

        self.set_params(params);
    }

    pub fn reinit_empty(&mut self) {
        *self = Self::Uninit;
    }
}

impl Simulator for MultiSimulator {
    fn get_params(&self) -> SimulationParams {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_params(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_params(),
        }
    }

    fn set_params(&mut self, params: SimulationParams) {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.set_params(params),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.set_params(params),
        }
    }

    fn reset(&mut self) {
        use MultiSimulator::*;

        match self {
            Uninit => (),
            UpSimulator(sim) => sim.reset(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.reset(),
        }
    }

    fn insert_body(&mut self, body: Body) -> BodyId {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.insert_body(body),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.insert_body(body),
        }
    }

    fn step(&mut self) {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.step(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.step(),
        }
    }

    fn pull_updates(&mut self) {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.pull_updates(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.pull_updates(),
        }
    }

    fn get_bodies(&self) -> &[Body] {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_bodies(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_bodies(),
        }
    }

    fn get_body(&self, id: BodyId) -> &Body {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_body(id),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_body(id),
        }
    }
}

impl SimulatorExt for MultiSimulator {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_quadtree(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_quadtree(),
        }
    }

    fn get_forces(&self) -> Option<Vec<Vec2f>> {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_forces(),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_forces(),
        }
    }

    fn get_body_forces(&self, id: BodyId) -> Option<Vec<SingleBodyForce>> {
        use MultiSimulator::*;

        match self {
            Uninit => unreachable!(),
            UpSimulator(sim) => sim.get_body_forces(id),
            #[cfg(feature = "cuda")]
            CUDASimulator(sim) => sim.get_body_forces(id),
        }
    }
}
