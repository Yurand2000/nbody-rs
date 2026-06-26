use std::mem::size_of;
use self::ffi::*;

pub mod ffi;
pub mod kernels;

pub struct Cuda;

pub struct CudaMem {
    ptr: *mut std::ffi::c_void,
    size: usize
}

impl CudaMem {
    pub fn new(size: usize) -> CudaResult<Self> {
        let mut ptr = std::ptr::null_mut();

        let err = unsafe { cudaMalloc(&mut ptr, size) };

        Cuda::make_result(Self { ptr, size }, err)
    }

    pub unsafe fn copy_in<T>(&mut self, data: &[T]) -> CudaResult<()> {
        if self.size != data.len() * size_of::<T>() {
            unimplemented!()
        }

        let err = unsafe {
            cudaMemcpy(
                self.ptr,
                data.as_ptr() as *const std::ffi::c_void,
                self.size,
                CUDA_MEMCPY_HOST_TO_DEVICE,
            )
        };

        Cuda::make_result((), err)
    }

    pub unsafe fn copy_out<T>(&self, data: &mut [T]) -> CudaResult<()> {
        if self.size != data.len() * size_of::<T>() {
            unimplemented!()
        }

        let err = unsafe {
            cudaMemcpy(
                data.as_mut_ptr() as *mut std::ffi::c_void,
                self.ptr,
                self.size,
                CUDA_MEMCPY_DEVICE_TO_HOST,
            )
        };

        Cuda::make_result((), err)
    }
}

impl Drop for CudaMem {
    fn drop(&mut self) {
        unsafe { cudaFree(self.ptr); }
    }
}

pub type CudaResult<T> = Result<T, CudaError>;

pub struct CudaError(cudaError_t);

impl Cuda {
    fn make_result<T>(value: T, err: cudaError_t) -> CudaResult<T> {
        if err == 0 {
            Ok(value)
        } else {
            Err(CudaError(err))
        }
    }
}

impl std::fmt::Debug for CudaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = unsafe { std::ffi::CStr::from_ptr(cudaGetErrorString(self.0)) };

        f.debug_tuple("CudaError").field(&str).finish()
    }
}

impl std::fmt::Display for CudaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = unsafe { std::ffi::CStr::from_ptr(cudaGetErrorString(self.0)) };

        write!(f, "{}", str.to_string_lossy())
    }
}