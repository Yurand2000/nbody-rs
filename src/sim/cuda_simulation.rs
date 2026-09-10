use crate::cuda::kernels::*;
use super::*;

pub struct CUDASimulator {
    host_bodies: Vec<Body>,
    bodies: CudaBodies,
    host_bodies_need_update: bool,
    bodies_need_update: bool,
    norm_data: CudaNormalizationData,
    quadtree: CudaQuadTree,
    params: CudaParams,
}

impl CUDASimulator {
    const MAX_NUM_BODIES: usize = 100000;
    const QUADTREE_DEPTH: usize = 8;

    pub fn new() -> Self {
        Self {
            host_bodies: Vec::new(),
            bodies: CudaBodies::new(Self::MAX_NUM_BODIES).unwrap(),
            host_bodies_need_update: false,
            bodies_need_update: false,
            norm_data: CudaNormalizationData::new().unwrap(),
            quadtree: CudaQuadTree::new(Self::QUADTREE_DEPTH).unwrap(),
            params: CudaParams::new(),
        }
    }

    fn upload_bodies(&mut self) {
        if !self.bodies_need_update { return; }

    let start = std::time::Instant::now();

        for (idx, body) in self.host_bodies.iter().enumerate() {
            self.bodies.write_body(body, idx).unwrap();
        }

    let total = std::time::Instant::now() - start;
    println!("+ bodies upload: {total:?}");

        self.bodies_need_update = false;
    }

    fn download_bodies(&mut self) {
        if !self.host_bodies_need_update { return; }

    let start = std::time::Instant::now();

        self.bodies.read_bodies(&mut self.host_bodies).unwrap();

    let total = std::time::Instant::now() - start;
    // println!("- bodies download: {total:?}");

        self.host_bodies_need_update = false;
    }
}

impl Simulator for CUDASimulator {
    fn get_params(&self) -> SimulationParams {
        SimulationParams {
            time_step: self.params.time_step,
            theta: self.params.theta,
            g_constant: self.params.g_constant,
        }
    }

    fn set_params(&mut self, params: SimulationParams) {
        self.params.g_constant = params.g_constant;
        self.params.theta = params.theta;
        self.params.time_step = params.time_step;
    }

    fn insert_body(&mut self, body: Body) -> BodyId {
        let last = self.host_bodies.len();
        self.host_bodies.push(body);
        self.bodies_need_update = true;
        last
    }

    fn step(&mut self) {
        let num_blocks = 16;
        let num_threads = 256;

        // Upload bodies if the user inserted some
        self.upload_bodies();

    let start = std::time::Instant::now();

        // Compute quadtree normalization data
        compute_normalization_data(&mut self.bodies, self.host_bodies.len(), &mut self.norm_data, num_blocks, num_threads).unwrap();

        // Compute quadtree
        zero_quadtree(&mut self.quadtree).unwrap();
        quadtree_build(&mut self.bodies, self.host_bodies.len(), &mut self.quadtree, &mut self.norm_data, num_blocks, num_threads).unwrap();

        crate::cuda::Cuda::device_synchronize().unwrap();

        // First kick
        leapfrog_kick(&mut self.bodies, self.host_bodies.len(), &mut self.quadtree, &mut self.params, &mut self.norm_data, num_blocks, num_threads).unwrap();

        crate::cuda::Cuda::device_synchronize().unwrap();

        // // Drift
        leapfrog_drift(&mut self.bodies, self.host_bodies.len(), &mut self.params, num_blocks, num_threads).unwrap();

        // Re-compute quadtree normalization data
        compute_normalization_data(&mut self.bodies, self.host_bodies.len(), &mut self.norm_data, num_blocks, num_threads).unwrap();

        // Re-compute quadtree
        zero_quadtree(&mut self.quadtree).unwrap();
        quadtree_build(&mut self.bodies, self.host_bodies.len(), &mut self.quadtree, &mut self.norm_data, num_blocks, num_threads).unwrap();

        // Second kick
        leapfrog_kick(&mut self.bodies, self.host_bodies.len(), &mut self.quadtree, &mut self.params, &mut self.norm_data, num_blocks, num_threads).unwrap();

        self.host_bodies_need_update = true;

    let total = std::time::Instant::now() - start;
    // println!("  step time: {total:?}");

    }

    fn pull_updates(&mut self) {
        // Download bodies if the simulation advanced
        self.download_bodies();
    }

    fn get_bodies(&self) -> &[Body] {
        &self.host_bodies
    }

    fn get_body(&self, id: BodyId) -> &Body {
        &self.host_bodies[id]
    }
}

impl SimulatorExt for CUDASimulator {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> {
        let mut tree = QuadTree::new(self.quadtree.depth.min(QuadTreeUtils::MAX_DEPTH_CPU), (&self.norm_data).into());

    println!("quadtree norm: {:?}", tree.norm);
    println!("quadtree norm uni: {:?}", NormalizationData::new(&self.host_bodies));
    let start = std::time::Instant::now();

        self.quadtree.read_cells(&mut tree.cells).unwrap();

    let total = std::time::Instant::now() - start;
    println!("quadtree download: {total:?}");

        for (idx, cell) in tree.cells.iter().enumerate() {
            if cell.mass.is_nan() || cell.position.x.is_nan() || cell.position.y.is_nan() {
                println!("{} {} {:?}", idx, cell.mass, cell.position);
            }
        }

        Some(tree)
    }

    fn get_forces(&self, quadtree: &QuadTree<QuadTreeCell>) -> Option<Vec<Vec2f>> {

    let start = std::time::Instant::now();

        let mut forces = crate::cuda::CudaVec::new(self.host_bodies.len()).unwrap();
        leapfrog_kick_debug(&self.bodies, self.host_bodies.len(), &self.quadtree, &self.params,
                            &self.norm_data, &mut forces, 1, 1).unwrap();

    let total = std::time::Instant::now() - start;
    println!("leapfrog forces compute: {total:?}");

    let start = std::time::Instant::now();

        let mut out_forces = vec![Vec2f::zeros(); self.host_bodies.len()];
        let mut tmp_vec = ffi::vec2f::new();
        for i in 0 .. self.host_bodies.len() {
            forces.copy_out_single(&mut tmp_vec, i).unwrap();
            out_forces[i] = Vec2f::new(tmp_vec.x, tmp_vec.y);
        }

    let total = std::time::Instant::now() - start;
    println!("+ leapfrog forces download: {total:?}");

        for (i, force) in out_forces.iter().enumerate() {
            if force.x.is_nan() || force.y.is_nan() {
                println!("force {} {:?}", i, force);
            }
        }

        Some(out_forces)
    }

    fn get_body_forces(&self, quadtree: &QuadTree<QuadTreeCell>, id: BodyId) -> Option<Vec<SingleBodyForce>> { None }
}
