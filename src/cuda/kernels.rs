use super::*;

mod ffi;

pub struct CudaBodies {
    pub mass: CudaVec<ffi::float>,
    pub position: CudaVec<ffi::vec2f>,
    pub velocity: CudaVec<ffi::vec2f>,
}

pub struct CudaNormalizationData {
    pub offset: CudaCell<ffi::vec2f>,
    pub size: CudaCell<ffi::vec2f>,
}

pub struct CudaQuadTree {
    pub mass: CudaVec<ffi::float>,
    pub center_of_mass: CudaVec<ffi::vec2f>,
    pub depth: usize,
}

pub struct CudaParams {
    pub time_step: f32,
    pub theta: f32,
    pub g_constant: f32,
}

pub fn compute_normalization_data(
    bodies: &mut CudaBodies,
    num_bodies: usize,
    info: &mut CudaNormalizationData,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_compute_normalization_data((bodies, num_bodies).into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn zero_quadtree(
    quadtree: &mut CudaQuadTree,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::zero_quadtree(quadtree.into()) };

    Cuda::make_result((), err)
}

pub fn quadtree_build(
    bodies: &mut CudaBodies,
    num_bodies: usize,
    quadtree: &mut CudaQuadTree,
    info: &mut CudaNormalizationData,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_quadtree_build((bodies, num_bodies).into(), quadtree.into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn leapfrog_kick(
    bodies: &mut CudaBodies,
    num_bodies: usize,
    quadtree: &mut CudaQuadTree,
    params: &mut CudaParams,
    info: &mut CudaNormalizationData,
    num_blocks: u64,
    num_threads: u64
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_leapfrog_kick((bodies, num_bodies).into(), quadtree.into(), params.into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn leapfrog_drift(
    bodies: &mut CudaBodies,
    num_bodies: usize,
    params: &mut CudaParams,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_leapfrog_drift((bodies, num_bodies).into(), params.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

impl CudaBodies {
    pub fn new(capacity: usize) -> CudaResult<Self> {
        Ok(Self {
            mass: CudaVec::new(capacity)?,
            position: CudaVec::new(capacity)?,
            velocity: CudaVec::new(capacity)?,
        })
    }

    pub fn write_body(&mut self, body: &crate::sim::Body, idx: usize) -> CudaResult<()>{
        self.mass.copy_in_single(&body.mass, idx)?;
        self.position.copy_in_single(&(&body.position).into(), idx)?;
        self.velocity.copy_in_single(&(&body.velocity).into(), idx)?;

        Ok(())
    }

    pub fn read_body(&self, idx: usize) -> CudaResult<crate::sim::Body> {
        let mut mass = 0.0;
        let mut position = ffi::vec2f::new();
        let mut velocity = ffi::vec2f::new();

        self.mass.copy_out_single(&mut mass, idx)?;
        self.position.copy_out_single(&mut position, idx)?;
        self.velocity.copy_out_single(&mut velocity, idx)?;

        Ok(crate::sim::Body {
            mass,
            position: position.into(),
            velocity: velocity.into(),
        })
    }
}

impl Into<ffi::bodies> for (&mut CudaBodies, usize) {
    fn into(self) -> ffi::bodies {
        let (bodies, num_bodies) = self;

        unsafe {
            ffi::bodies {
                mass: bodies.mass.get_ptr_mut(),
                position: bodies.position.get_ptr_mut(),
                velocity: bodies.velocity.get_ptr_mut(),
                count: num_bodies as u64,
            }
        }
    }
}

impl CudaNormalizationData {
    pub fn new() -> CudaResult<Self> {
        Ok(Self {
            offset: CudaCell::new()?,
            size: CudaCell::new()?,
        })
    }
}

impl Into<ffi::normalization_data> for &mut CudaNormalizationData {
    fn into(self) -> ffi::normalization_data {
        unsafe {
            ffi::normalization_data {
                offset: self.offset.get_ptr_mut(),
                size: self.size.get_ptr_mut(),
            }
        }
    }
}

impl CudaQuadTree {
    pub fn new(depth: usize) -> CudaResult<Self> {
        use crate::sim::QuadTreeUtils;

        if depth > QuadTreeUtils::MAX_DEPTH_GPU {
            return Err(CudaError::QuadtreeDepth())
        }

        let size = QuadTreeUtils::layer_end(depth);

        Ok(Self {
            mass: CudaVec::new(size)?,
            center_of_mass: CudaVec::new(size)?,
            depth,
        })
    }
}

impl Into<ffi::quadtree> for &mut CudaQuadTree {
    fn into(self) -> ffi::quadtree {
        unsafe {
            ffi::quadtree {
                mass: self.mass.get_ptr_mut(),
                center_of_mass: self.center_of_mass.get_ptr_mut(),
                depth: self.depth as u64,
            }
        }
    }
}

impl CudaParams {
    pub fn new() -> Self {
        Self {
            time_step: 0.0,
            theta: 0.0,
            g_constant: 0.0,
        }
    }
}

impl Into<ffi::params> for &mut CudaParams {
    fn into(self) -> ffi::params {
        ffi::params {
            time_step: self.time_step,
            theta: self.theta,
            g_constant: self.g_constant,
        }
    }
}

impl Into<crate::sim::NormalizationData> for &CudaNormalizationData {
    fn into(self) -> crate::sim::NormalizationData {
        let mut offset = ffi::vec2f::new();
        let mut size = ffi::vec2f::new();

        self.offset.read(&mut offset).unwrap();
        self.size.read(&mut size).unwrap();

        crate::sim::NormalizationData {
            offset: offset.into(),
            size: size.into(),
            min_distance: 1.0,
        }
    }
}