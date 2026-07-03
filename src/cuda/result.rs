use super::*;
use super::ffi::*;

pub struct __CudaError(cudaError_t);

#[derive(Debug)]
pub enum CudaError {
    Cuda(__CudaError),
    CopyIn(),
    CopyOut(),
    QuadtreeDepth(),
}

impl std::fmt::Debug for __CudaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = unsafe { std::ffi::CStr::from_ptr(cudaGetErrorString(self.0)) };

        f.debug_tuple("CudaError").field(&str).finish()
    }
}

pub type CudaResult<T> = Result<T, CudaError>;

impl Cuda {
    pub fn make_result<T>(value: T, err: cudaError_t) -> CudaResult<T> {
        if err == 0 {
            Ok(value)
        } else {
            Err(CudaError::Cuda(__CudaError(err)))
        }
    }
}