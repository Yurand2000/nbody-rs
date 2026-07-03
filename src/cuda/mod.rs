pub mod alloc;
pub mod ffi;
pub mod kernels;
pub mod result;

pub use alloc::*;
pub use result::*;

pub struct Cuda;

impl Cuda {
    pub fn device_synchronize() -> CudaResult<()> {
        let err =
            unsafe { ffi::cudaDeviceSynchronize() };

        Self::make_result((), err)
    }
}