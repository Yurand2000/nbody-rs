pub mod up_simulation;
pub mod cuda_simulation;

pub type Vec2 = nalgebra::Vector2<f32>;

pub type BodyId = usize;

pub struct Body {
    pub mass: f32,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub struct Simulator {
    pub params: SimulationParams,
    bodies: Vec<Body>,
}

pub enum SimulationType {
    SingleCore,
    MultiCore,
    CUDA(cuda_simulation::CudaData),
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
            SimulationType::CUDA(_) => todo!(),
        }
    }

    pub fn get_bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn get_body(&self, id: BodyId) -> &Body {
        &self.bodies[id]
    }
}

pub struct QuadTreeUtils;

impl QuadTreeUtils {
    pub const MAX_DEPTH_CPU: usize = 10;
    pub const MAX_DEPTH_GPU: usize = 15;

    /// Number of cells for a tree of the given `depth`.
    #[inline(always)]
    pub fn tree_size(depth: usize) -> usize {
        Self::layer_end(depth)
    }

    /// Number of cells at the given `depth`.
    #[inline(always)]
    pub fn layer_size(depth: usize) -> usize {
        1 << (2 * depth)
    }

    /// Number of cells per dimension at given `depth`:
    /// `layer_dim(x) * layer_dim(x) == layer_size(x)`
    #[inline(always)]
    pub fn layer_dim(depth: usize) -> usize {
        1 << depth
    }

    /// Width of one cell at the given `depth`.
    #[inline(always)]
    pub fn layer_width(depth: usize) -> f32 {
        1.0 / Self::layer_dim(depth) as f32
    }

    /// Start offset of the given layer in the whole array.
    #[inline(always)]
    pub fn layer_start(depth: usize) -> usize {
        let mut size = 0;
        let mut tmp = 1;
        for _ in 0..depth {
            size += tmp;
            tmp *= 4;
        }

        size
    }

    /// End (excluded) offset of the given layer in the whole array.
    #[inline(always)]
    pub fn layer_end(depth: usize) -> usize {
        Self::layer_start(depth + 1)
    }

    /// Relative nearest cell to `normalized_pos` at `depth`.
    #[inline(always)]
    pub fn nearest_cell_at_layer(depth: usize, normalized_pos: Vec2) -> usize {
        let mut out = 0;
        let mut layer_dim = QuadTreeUtils::layer_dim(1) as f32;
        let mut pos = normalized_pos;
        for _ in 0 .. depth {
            let cell = Self::nearest_cell_in_layer(pos * layer_dim);
            out = Self::first_position_at_next_layer(out) + cell;
            pos -= Self::position_in_layer(cell) / layer_dim;
            layer_dim *= 2.0;
        }

        out
    }

    #[inline(always)]
    /// Inverse of `nearest_cell_in_layer`
    pub fn nearest_cell_in_layer(pos: Vec2) -> usize {
        if pos.x <= 1.0 {
            if pos.y <= 1.0 {
                0
            } else {
                1
            }
        } else {
            if pos.y <= 1.0 {
                2
            } else {
                3
            }
        }
    }

    /// Nearest cell position to relative cell of `index` at `depth`.
    #[inline(always)]
    pub fn position_at_layer(depth: usize, index: usize) -> Vec2 {
        let mut out = Vec2::zeros();
        let mut curr_idx = index;
        let mut curr_size = Self::layer_width(depth);
        for _ in 0 .. depth {
            out += Self::position_in_layer(curr_idx % 4) * curr_size;
            curr_size *= 2.0;
            curr_idx = curr_idx / 4;
        }

        out
    }

    #[inline(always)]
    /// Inverse of `nearest_cell_in_layer`
    pub fn position_in_layer(index: usize) -> Vec2 {
        const OFFSETS: [Vec2; 4] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
        ];

        OFFSETS[index]
    }

    #[inline(always)]
    pub fn first_position_at_next_layer(index: usize) -> usize {
        index * 4
    }
}