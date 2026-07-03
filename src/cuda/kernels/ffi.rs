pub use std::ffi::c_float as float;

#[repr(C)]
pub struct vec2f {
    pub x: float,
    pub y: float,
}

impl vec2f {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<&crate::sim::Vec2f> for vec2f {
    fn from(value: &crate::sim::Vec2f) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl Into<crate::sim::Vec2f> for vec2f {
    fn into(self) -> crate::sim::Vec2f {
        crate::sim::Vec2f::new(self.x, self.y)
    }
}

#[repr(C)]
pub struct bodies {
    pub mass: *mut float,
    pub position: *mut vec2f,
    pub velocity: *mut vec2f,
    pub count: u64,
}

#[repr(C)]
pub struct normalization_data {
    pub offset: *mut vec2f,
    pub size: *mut vec2f,
}

#[repr(C)]
pub struct quadtree {
    pub mass: *mut float,
    pub center_of_mass: *mut vec2f,
    pub depth: u64,
}

#[repr(C)]
pub struct params {
    pub time_step: float,
    pub theta: float,
    pub g_constant: float,
}

unsafe extern "C" {
    pub unsafe fn cuda_compute_normalization_data(
        bodies: bodies,
        info: normalization_data,
        num_blocks: u64,
        num_threads: u64,
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn zero_quadtree(
        quadtree: quadtree
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn cuda_quadtree_build(
        bodies: bodies,
        quadtree: quadtree,
        info: normalization_data,
        num_blocks: u64,
        num_threads: u64,
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn cuda_leapfrog_kick(
        bodies: bodies,
        quadtree: quadtree,
        params: params,
        info: normalization_data,
        num_blocks: u64,
        num_threads: u64
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn cuda_leapfrog_drift(
        bodies: bodies,
        params: params,
        num_blocks: u64,
        num_threads: u64,
    ) -> crate::cuda::ffi::cudaError_t;
}