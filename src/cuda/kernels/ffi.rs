pub use std::ffi::c_float as float;

#[repr(C)]
pub struct vec2f {
    pub x: float,
    pub y: float,
}

#[repr(C)]
pub struct bodies {
    pub mass: *mut float,
    pub position: *mut vec2f,
    pub velocity: *mut vec2f,
    pub count: u64,
}

#[repr(C)]
pub struct bodies_info {
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
    pub unsafe fn cuda_compute_bodies_info(
        bodies: bodies,
        info: bodies_info,
        num_blocks: u64,
        num_threads: u64,
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn zero_quadtree(
        quadtree: quadtree
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn cuda_quadtree_build(
        bodies: bodies,
        quadtree: quadtree,
        info: bodies_info,
        num_blocks: u64,
        num_threads: u64,
    ) -> crate::cuda::ffi::cudaError_t;

    pub unsafe fn cuda_leapfrog_kick(
        bodies: bodies,
        quadtree: quadtree,
        params: params,
        info: bodies_info,
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