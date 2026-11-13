use std::collections::VecDeque;
use std::sync::Arc;
use smol::lock::{
    Mutex,
    Semaphore,
};
use smol::channel::{
    Sender,
    Receiver,
};

use super::*;

#[derive(bevy::prelude::Resource)]
pub struct AsyncSimulatorFrontend {
    params: SimulationParams,
    bodies: Vec<Body>,

    new_bodies: Sender<Body>,
    body_updates: Receiver<(BodyId, Body)>,
    param_updates: Sender<SimulationParams>,
    comms: AsyncSimulatorComms,
}

pub struct AsyncSimulatorBackend {
    simulator: MultiSimulator,
    new_bodies: Receiver<Body>,
    body_updates: Sender<(BodyId, Body)>,
    param_updates: Receiver<SimulationParams>,
    comms: AsyncSimulatorComms,
    diagnostics: SimulatorDiagnostics,
}

#[derive(Clone)]
pub struct AsyncSimulatorComms {
    exit: Arc<Mutex<bool>>,
    init: Arc<Mutex<Option<ResetData>>>,
    reset: Arc<Mutex<bool>>,
    can_run: Arc<Mutex<bool>>,
    push_updates_from_simulator: Arc<Mutex<bool>>,
    updates_semaphore: Arc<Semaphore>,
    updates_per_second: Arc<Mutex<f64>>,
}

enum ResetData {
    Uninit,
    UpSimulator(SimulationParams),
    #[cfg(feature = "cuda")]
    CUDASimulator(SimulationParams, cuda_simulation::CUDASimulatorParams),
}

struct SimulatorDiagnostics {
    average: f64,
    update_times: VecDeque<std::time::Instant>,
}

pub fn new_async_simulator() -> (AsyncSimulatorBackendBuilder, AsyncSimulatorFrontend) {
    let comms = AsyncSimulatorComms::new();

    let (new_bodies_send, new_bodies_recv) = smol::channel::unbounded();
    let (body_updates_send, body_updates_recv) = smol::channel::unbounded();
    let (param_updates_send, param_updates_recv) = smol::channel::unbounded();

    let backend = AsyncSimulatorBackendBuilder {
        new_bodies: new_bodies_recv,
        body_updates: body_updates_send,
        param_updates: param_updates_recv,
        comms: comms.clone(),
    };

    let frontend = AsyncSimulatorFrontend {
        params: SimulationParams::default(),
        bodies: Vec::new(),
        new_bodies: new_bodies_send,
        body_updates: body_updates_recv,
        param_updates: param_updates_send,
        comms,
    };

    (backend, frontend)
}

pub struct AsyncSimulatorBackendBuilder {
    new_bodies: Receiver<Body>,
    body_updates: Sender<(BodyId, Body)>,
    param_updates: Receiver<SimulationParams>,
    comms: AsyncSimulatorComms,
}

impl AsyncSimulatorBackendBuilder {
    pub fn build(self) -> AsyncSimulatorBackend {
        AsyncSimulatorBackend {
            simulator: MultiSimulator::Uninit,
            new_bodies: self.new_bodies,
            body_updates: self.body_updates,
            param_updates: self.param_updates,
            comms: self.comms,
            diagnostics: SimulatorDiagnostics::new(),
        }
    }
}

impl AsyncSimulatorBackend {
    pub async fn run(&mut self) {
        const PAUSED_DELTA: std::time::Duration =
            std::time::Duration::from_nanos(1_000_000_000 / 10);
        const DELTA: std::time::Duration =
            std::time::Duration::from_nanos(1_000_000_000 / 1000);

        let mut last_instant = std::time::Instant::now();
        while !*self.comms.exit.lock().await {
            self.diagnostics.update();
            *self.comms.updates_per_second.lock().await =
                self.diagnostics.updates_per_second();

            // reinit the simulator?
            if let Some(reset_data) = self.comms.init.lock().await.take() {
                use ResetData::*;

                match reset_data {
                    Uninit =>
                        self.simulator.reinit_empty(),
                    UpSimulator(params) =>
                        self.simulator.init_up(params),
                    #[cfg(feature = "cuda")]
                    CUDASimulator(params, cuda_params) =>
                        self.simulator.init_cuda(params, cuda_params),
                };
            }

            if let MultiSimulator::Uninit = self.simulator {
                smol::Timer::at(last_instant + PAUSED_DELTA).await;
                last_instant = std::time::Instant::now();
                continue;
            }

            // reset the simulator?
            let mut reset = self.comms.reset.lock().await;
            if *reset {
                *reset = false;

                self.simulator.reset();
            }

            // pull body updates?
            while let Ok(body) = self.new_bodies.try_recv() {
                self.simulator.insert_body(body);
            }

            // update params?
            while let Ok(params) = self.param_updates.try_recv() {
                self.simulator.set_params(params);
            }

            // debug features?
            // or run one step here
            if *self.comms.can_run.lock().await {
                self.simulator.step();

                // rate limit at 1000 updates per second.
                smol::Timer::at(last_instant + DELTA).await;
            } else {
                smol::Timer::at(last_instant + PAUSED_DELTA).await;
            }


            // push body updates?
            let mut push_updates = self.comms.push_updates_from_simulator.lock().await;
            if *push_updates {
                *push_updates = false;

                let guard = self.comms.updates_semaphore.acquire().await;

                self.simulator.pull_updates();
                for (id, body) in self.simulator.get_bodies().iter().enumerate() {
                    self.body_updates.send((id, body.clone())).await.unwrap();
                }

                drop(guard);
            }

            last_instant = std::time::Instant::now();
        }
    }
}

impl AsyncSimulatorFrontend {
    pub fn init_up(&mut self, params: SimulationParams) {
        self.params = params;
        *smol::block_on(self.comms.init.lock()) = Some(ResetData::UpSimulator(params));
    }

    #[cfg(feature = "cuda")]
    pub fn init_cuda(&mut self, params: SimulationParams, cuda_params: cuda_simulation::CUDASimulatorParams) {
        self.params = params;
        *smol::block_on(self.comms.init.lock()) = Some(ResetData::CUDASimulator(params, cuda_params));
    }

    pub fn reinit_empty(&mut self) {
        *smol::block_on(self.comms.init.lock()) = Some(ResetData::Uninit);
    }

    pub fn get_steps_per_second(&self) -> f64 {
        *smol::block_on(self.comms.updates_per_second.lock())
    }

    pub fn set_run(&mut self) {
        *smol::block_on(self.comms.can_run.lock()) = true;
    }

    pub fn set_pause(&mut self) {
        *smol::block_on(self.comms.can_run.lock()) = false;
    }
}

impl Simulator for AsyncSimulatorFrontend {
    fn get_params(&self) -> SimulationParams {
        self.params
    }

    fn set_params(&mut self, params: SimulationParams) {
        self.params = params;
        smol::block_on(self.param_updates.send(params)).unwrap();
    }

    fn reset(&mut self) {
        self.bodies.clear();
        while let Ok(_) = self.body_updates.try_recv() { }
        *smol::block_on(self.comms.reset.lock()) = true;
    }

    fn insert_body(&mut self, body: Body) -> BodyId {
        smol::block_on(self.new_bodies.send(body.clone())).unwrap();
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    fn step(&mut self) {

    }

    fn pull_updates(&mut self) {
        *smol::block_on(self.comms.push_updates_from_simulator.lock()) = true;

        let guard = smol::block_on(self.comms.updates_semaphore.acquire());

        if self.bodies.is_empty() {
            while let Ok(_) = self.body_updates.try_recv() { }
        } else {
            while let Ok((id, body)) = self.body_updates.try_recv() {
                self.bodies[id] = body;
            }
        }

        drop(guard);
    }

    fn get_bodies(&self) -> &[Body] {
        &self.bodies
    }

    fn get_body(&self, id: BodyId) -> &Body {
        &self.bodies[id]
    }
}

impl SimulatorExt for AsyncSimulatorFrontend {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> { None }

    fn get_forces(&self, _quadtree: &QuadTree<QuadTreeCell>) -> Option<Vec<Vec2f>> { None }

    fn get_body_forces(&self, _quadtree: &QuadTree<QuadTreeCell>, _id: BodyId) -> Option<Vec<SingleBodyForce>> { None }
}

impl Drop for AsyncSimulatorFrontend {
    fn drop(&mut self) {
        *smol::block_on(self.comms.exit.lock()) = true;
    }
}

impl AsyncSimulatorComms {
    pub fn new() -> Self {
        Self {
            exit: Arc::new(Mutex::new(false)),
            init: Arc::new(Mutex::new(None)),
            reset: Arc::new(Mutex::new(false)),
            can_run: Arc::new(Mutex::new(false)),
            push_updates_from_simulator: Arc::new(Mutex::new(false)),
            updates_semaphore: Arc::new(Semaphore::new(1)),
            updates_per_second: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl SimulatorDiagnostics {
    pub fn new() -> Self {
        Self {
            average: 0.0,
            update_times: VecDeque::new(),
        }
    }

    pub fn updates_per_second(&self) -> f64 {
        self.average
    }

    pub fn update(&mut self) {
        const WINDOW_SIZE: std::time::Duration = std::time::Duration::from_millis(1000);

        let now = std::time::Instant::now();
        self.update_times.push_back(now);

        while let Some(&oldest) = self.update_times.front() {
            if now - oldest > WINDOW_SIZE {
                self.update_times.pop_front();
            } else {
                break;
            }
        }

        self.average = self.update_times.len() as f64 / WINDOW_SIZE.as_secs_f64();
    }
}
