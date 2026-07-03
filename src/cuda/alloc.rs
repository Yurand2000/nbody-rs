use super::*;
use super::ffi::*;

pub struct CudaVec<T> {
    ptr: *mut T,
    size: usize
}

impl<T> CudaVec<T> {
    pub fn new(size: usize) -> CudaResult<Self> {
        let mut ptr = std::ptr::null_mut();

        let err = unsafe { cudaMalloc(&mut ptr, size_of::<T>() * size) };

        Cuda::make_result(Self { ptr: ptr.cast(), size }, err)
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn copy_in(&mut self, data: &[T], offset: usize) -> CudaResult<()> {
        if self.size < offset + data.len() {
            return Err(CudaError::CopyIn());
        }

        let err = unsafe {
            cudaMemcpy(
                self.ptr.add(offset).cast(),
                data.as_ptr().cast(),
                data.len() * size_of::<T>(),
                CUDA_MEMCPY_HOST_TO_DEVICE,
            )
        };

        Cuda::make_result((), err)
    }

    pub fn copy_in_single(&mut self, data: &T, offset: usize) -> CudaResult<()> {
        self.copy_in(std::slice::from_ref(data), offset)
    }

    pub fn copy_out(&self, data: &mut [T], offset: usize) -> CudaResult<()> {
        if self.size < offset + data.len() {
            return Err(CudaError::CopyOut());
        }

        let err = unsafe {
            cudaMemcpy(
                data.as_mut_ptr().cast(),
                self.ptr.add(offset).cast_const().cast(),
                data.len() * size_of::<T>(),
                CUDA_MEMCPY_DEVICE_TO_HOST,
            )
        };

        Cuda::make_result((), err)
    }

    pub fn copy_out_single(&self, data: &mut T, offset: usize) -> CudaResult<()> {
        self.copy_out(std::slice::from_mut(data), offset)
    }

    pub unsafe fn get_ptr(&self) -> *const T {
        self.ptr.cast_const()
    }

    pub unsafe fn get_ptr_mut(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Drop for CudaVec<T> {
    fn drop(&mut self) {
        unsafe { cudaFree(self.ptr.cast()); }
    }
}

pub struct CudaCell<T> {
    cell: CudaVec<T>,
}

impl<T> CudaCell<T> {
    pub fn new() -> CudaResult<Self> {
        Ok(Self { cell: CudaVec::new(1)? })
    }

    pub fn write(&mut self, data: &T) -> CudaResult<()> {
        self.cell.copy_in_single(data, 0)
    }

    pub fn read(&self, data: &mut T) -> CudaResult<()> {
        self.cell.copy_out_single(data, 0)
    }

    pub unsafe fn get_ptr(&self) -> *const T {
        unsafe { self.cell.get_ptr() }
    }

    pub unsafe fn get_ptr_mut(&mut self) -> *mut T {
        unsafe { self.cell.get_ptr_mut() }
    }
}