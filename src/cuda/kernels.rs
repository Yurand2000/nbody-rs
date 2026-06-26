use super::*;
use std::ffi::*;

pub mod ffi {
    use std::ffi::*;

    unsafe extern "C" {
        pub unsafe fn add_vector(
            a: *const c_float,
            b: *const c_float,
            c: *mut c_float,
            n: c_uint
        ) -> crate::cuda::ffi::cudaError_t;
    }
}

pub unsafe fn add_vector(a: &CudaMem, b: &CudaMem, c: &mut CudaMem, n: usize) -> CudaResult<()> {
    let err =
        unsafe {
            ffi::add_vector(
                a.ptr as *const c_float,
                b.ptr as *const c_float,
                c.ptr as *mut c_float,
                n as c_uint)
        };

    Cuda::make_result((), err)
}