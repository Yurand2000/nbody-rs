use super::*;

mod ffi;

pub struct Bodies {
    pub mass: CudaVec<ffi::float>,
    pub position: CudaVec<ffi::vec2f>,
    pub velocity: CudaVec<ffi::vec2f>,
}

pub struct BodiesInfo {
    pub offset: CudaCell<ffi::vec2f>,
    pub size: CudaCell<ffi::vec2f>,
}

pub struct QuadTree {
    pub mass: CudaVec<ffi::float>,
    pub center_of_mass: CudaVec<ffi::vec2f>,
    pub depth: usize,
}

pub struct Params {
    pub time_step: f32,
    pub theta: f32,
    pub g_constant: f32,
}

pub fn cuda_compute_bodies_info(
    bodies: &mut Bodies,
    info: &mut BodiesInfo,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_compute_bodies_info(bodies.into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn zero_quadtree(
    quadtree: &mut QuadTree,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::zero_quadtree(quadtree.into()) };

    Cuda::make_result((), err)
}

pub fn cuda_quadtree_build(
    bodies: &mut Bodies,
    quadtree: &mut QuadTree,
    info: &mut BodiesInfo,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_quadtree_build(bodies.into(), quadtree.into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn cuda_leapfrog_kick(
    bodies: &mut Bodies,
    quadtree: &mut QuadTree,
    params: &mut Params,
    info: &mut BodiesInfo,
    num_blocks: u64,
    num_threads: u64
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_leapfrog_kick(bodies.into(), quadtree.into(), params.into(), info.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

pub fn cuda_leapfrog_drift(
    bodies: &mut Bodies,
    params: &mut Params,
    num_blocks: u64,
    num_threads: u64,
) -> CudaResult<()> {
    let err =
        unsafe { ffi::cuda_leapfrog_drift(bodies.into(), params.into(), num_blocks, num_threads) };

    Cuda::make_result((), err)
}

impl Bodies {
    pub fn new(num_bodies: usize) -> CudaResult<Self> {
        Ok(Self {
            mass: CudaVec::new(num_bodies)?,
            position: CudaVec::new(num_bodies)?,
            velocity: CudaVec::new(num_bodies)?,
        })
    }
}

impl Into<ffi::bodies> for &mut Bodies {
    fn into(self) -> ffi::bodies {
        unsafe {
            let count = self.mass.len() as u64;
            ffi::bodies {
                mass: self.mass.get_ptr_mut(),
                position: self.position.get_ptr_mut(),
                velocity: self.velocity.get_ptr_mut(),
                count,
            }
        }
    }
}

impl BodiesInfo {
    pub fn new() -> CudaResult<Self> {
        Ok(Self {
            offset: CudaCell::new()?,
            size: CudaCell::new()?,
        })
    }
}

impl Into<ffi::bodies_info> for &mut BodiesInfo {
    fn into(self) -> ffi::bodies_info {
        unsafe {
            ffi::bodies_info {
                offset: self.offset.get_ptr_mut(),
                size: self.size.get_ptr_mut(),
            }
        }
    }
}

impl QuadTree {
    pub fn new(depth: usize) -> CudaResult<Self> {
        use crate::sim::QuadTreeUtils;

        let size = QuadTreeUtils::layer_end(QuadTreeUtils::MAX_DEPTH_GPU);

        Ok(Self {
            mass: CudaVec::new(size)?,
            center_of_mass: CudaVec::new(size)?,
            depth,
        })
    }
}

impl Into<ffi::quadtree> for &mut QuadTree {
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

impl Params {
    pub fn new() -> Self {
        Self {
            time_step: 0.0,
            theta: 0.0,
            g_constant: 0.0,
        }
    }
}

impl Into<ffi::params> for &mut Params {
    fn into(self) -> ffi::params {
        ffi::params {
            time_step: self.time_step,
            theta: self.theta,
            g_constant: self.g_constant,
        }
    }
}