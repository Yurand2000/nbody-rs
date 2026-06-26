use std::ffi::*;

pub const CUDA_MEMCPY_HOST_TO_DEVICE: i32 = 1;
pub const CUDA_MEMCPY_DEVICE_TO_HOST: i32 = 2;

#[allow(non_camel_case_types)]
pub type cudaError_t = i32;

unsafe extern "C" {
    pub unsafe fn cudaGetErrorString(error: cudaError_t) -> *const std::ffi::c_char;
    pub unsafe fn cudaMalloc(devPtr: *mut *mut c_void, size: usize) -> cudaError_t;
    pub unsafe fn cudaFree(devPtr: *mut c_void) -> cudaError_t;
    pub unsafe fn cudaMemcpy(dst: *mut c_void, src: *const c_void, count: usize, kind: i32) -> cudaError_t;
}