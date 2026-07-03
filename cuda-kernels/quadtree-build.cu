#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include <assert.h>
#include "common.h"

__device__ void combine_cell_from_layer_below(float* mass, vec2f* mass_pos, u64 c);

__global__ void __cuda_quadtree_last_layer(
    struct bodies bodies,
    struct quadtree quadtree,
    struct normalization_data info
) {
    // Helper variables
    u64 layer_last = quadtree.depth - 1;

    // Build last layer
    // #### Process all bodies
    u64 offset = __layer_start(layer_last);
    for (u64 i = __thread_body_start(bodies.count); i < __thread_body_end(bodies.count); i++) {
        u64 idx = __nearest_cell_at_layer(layer_last, __normalize_pos(bodies.position[i], info));

        atomicAdd(&quadtree.mass[offset + idx], bodies.mass[i]);
        atomicAdd(&quadtree.center_of_mass[offset + idx].x, bodies.position[i].x * bodies.mass[i]);
        atomicAdd(&quadtree.center_of_mass[offset + idx].y, bodies.position[i].y * bodies.mass[i]);
    }

    __syncthreads();

    // Build last layer
    // #### Average the center of mass
    for (u64 cell = __thread_layer_start(layer_last); cell < __thread_layer_end(layer_last); cell++) {
        if (quadtree.mass[cell] < FLT_EPSILON)
            continue;

        quadtree.center_of_mass[cell] =
            __vec2f_div(quadtree.center_of_mass[cell], quadtree.mass[cell]);
    }
}

__global__ void __cuda_quadtree_reduce(
    struct quadtree quadtree
) {
    // Helper variables
    u64 layer_last = quadtree.depth - 1;
    u64 layer_threads = __log2_u64(blockDim.x) + __log2_u64(gridDim.x);

    // Reduce up to a single per-thread cell
        // from last layer to layer 10 (1024 cells)
    u64 per_thread_layers = quadtree.depth > layer_threads ? quadtree.depth - layer_threads : 0;
    for (u64 l = 0; l < per_thread_layers; l++) {
        u64 layer = layer_last - l - 1;

        for (u64 cell = __thread_layer_start(layer); cell < __thread_layer_end(layer); cell++) {
            combine_cell_from_layer_below(quadtree.mass, quadtree.center_of_mass, cell);
        }
    }

    __syncthreads();

    // Reduce to layer zero
    u64 max_depth = min(layer_last, (u64)(layer_threads - 1));
    u64 cell = threadIdx.x;
    for (u64 l = 0; l < max_depth; l++) {
        if (cell % 4 != 0) {
            return;
        }

        combine_cell_from_layer_below(quadtree.mass, quadtree.center_of_mass, cell);
        cell /= 4;

        __syncthreads();
    }
}

// Average the position and masses of cells at the next layer
__device__ void combine_cell_from_layer_below(float* mass, vec2f* mass_pos, u64 c) {
    mass[c] = mass[4*c] + mass[4*c+1] + mass[4*c+2] + mass[4*c+3];

    mass_pos[c].x =
        mass_pos[4*c].x * mass[4*c] + mass_pos[4*c+1].x * mass[4*c+1] +
        mass_pos[4*c+2].x * mass[4*c+2] + mass_pos[4*c+3].x * mass[4*c+3];
    mass_pos[c].y =
        mass_pos[4*c].y * mass[4*c] + mass_pos[4*c+1].y * mass[4*c+1] +
        mass_pos[4*c+2].y * mass[4*c+2] + mass_pos[4*c+3].y * mass[4*c+3];

    mass_pos[c] = __vec2f_div(mass_pos[c], mass[c]);
}

extern "C" {

cudaError_t zero_quadtree(
    struct quadtree quadtree
) {
    cudaError_t err;

    cudaMemset(quadtree.mass, 0, sizeof(float) * __layer_end(quadtree.depth));
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    cudaMemset(quadtree.center_of_mass, 0, sizeof(vec2f) * __layer_end(quadtree.depth));
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

cudaError_t cuda_quadtree_build(
    struct bodies bodies,
    struct quadtree quadtree,
    struct normalization_data info,
    u64 num_blocks,
    u64 num_threads
) {
    cudaError_t err;

    assert(is_pow4_u64(num_blocks));
    assert(is_pow4_u64(num_threads));

    __cuda_quadtree_last_layer<<<num_blocks, num_threads>>>(bodies, quadtree, info);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    __cuda_quadtree_reduce<<<num_blocks, num_threads>>>(quadtree);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

}