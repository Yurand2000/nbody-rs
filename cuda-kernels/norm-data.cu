#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include <assert.h>
#include "common.h"

__global__ void __cuda_compute_bodies_info(
    struct bodies bodies,
    vec2f *block_min,
    vec2f *block_max
) {
    // Allocate per-block bodies_info arrays
    extern __shared__ vec2f pos_data[];

    vec2f *min_pos = pos_data;
    vec2f *max_pos = pos_data + gridDim.x;

    u64 idx = threadIdx.x;

    min_pos[idx] = __vec2f_zero();
    max_pos[idx] = __vec2f_zero();
    for (u64 i = __thread_body_start(bodies.count); i < __thread_body_end(bodies.count); i++) {
        min_pos[idx] = __vec2f_comp_min(min_pos[idx], bodies.position[i]);
        max_pos[idx] = __vec2f_comp_max(max_pos[idx], bodies.position[i]);
    }

    // Reduce min/max positions (POSSIBLE PARALLELIZED REDUCE)
    if (idx != 0)
        return;

    block_min = &block_min[ blockIdx.x ];
    block_max = &block_max[ blockIdx.x ];

    *block_min = min_pos[0];
    *block_max = max_pos[0];
    for (u64 i = 1; i < blockDim.x; i++) {
        *block_min = __vec2f_comp_min(*block_min, min_pos[i]);
        *block_max = __vec2f_comp_max(*block_max, max_pos[i]);
    }
}

__global__ void __cuda_reduce_block_bodies_info(
    struct normalization_data info,
    vec2f *block_min,
    vec2f *block_max,
    u64 num_blocks
) {
    // Reduce min/max positions (POSSIBLE PARALLELIZED REDUCE)
    vec2f global_min, global_max;

    global_min = block_min[0];
    global_max = block_max[0];
    for (u64 i = 1; i < num_blocks; i++) {
        global_min = __vec2f_comp_min(global_min, block_min[i]);
        global_max = __vec2f_comp_max(global_max, block_max[i]);
    }

    // Apply a small delta
    global_min.x -= 10.0f;
    global_min.y -= 10.0f;
    global_max.x += 10.0f;
    global_max.y += 10.0f;

    *(info.offset) = global_min;
    *(info.size) = __vec2f_sub(global_max, global_min);
}

extern "C" {

cudaError_t cuda_compute_normalization_data(
    struct bodies bodies,
    struct normalization_data info,
    u64 num_blocks,
    u64 num_threads
) {
    cudaError_t err;

    vec2f* data = NULL;
    err = cudaMalloc((void**)(&data), sizeof(vec2f) * 2 * num_blocks);
    if (err != cudaSuccess)
        return err;

    vec2f* block_min = data;
    vec2f* block_max = data + num_blocks;

    u64 shared_bytes = sizeof(vec2f) * 2 * num_threads;
    __cuda_compute_bodies_info<<<num_blocks, num_threads, shared_bytes>>>(bodies, block_min, block_max);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        cudaFree(data);
        return err;
    }

    __cuda_reduce_block_bodies_info<<<1, 1>>>(info, block_min, block_max, num_blocks);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        cudaFree(data);
        return err;
    }

    cudaFree(data);
    return cudaSuccess;
}

}