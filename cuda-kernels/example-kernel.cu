#include <cuda_runtime.h>
#include <stdint.h>

__global__ void __add_vector(const float* a, const float* b, float* c, unsigned int n) {
    unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;

    if (i < n)
        c[i] = a[i] + b[i];
}

extern "C" {

cudaError_t add_vector(const float* a, const float* b, float* c, unsigned int n) {
    __add_vector<<<1, 32>>>(a, b, c, n);
    return cudaGetLastError();
}

}
