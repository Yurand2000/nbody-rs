#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include <assert.h>
#include <stdio.h>
#include "common.h"

__device__ void combine_cell_from_layer_below(float* mass, vec2f* mass_pos, u64 c);

__global__ void __cuda_quadtree_last_layer_per_body(
    struct bodies bodies,
    struct quadtree quadtree,
    struct normalization_data info
) {
    // Build last layer
    // #### Process all bodies
    u64 offset = __layer_start(quadtree.depth);
    for (u64 i = __thread_body_start(bodies.count); i < __thread_body_end(bodies.count); i++) {
        u64 idx = __nearest_cell_at_layer(quadtree.depth, __normalize_pos(bodies.position[i], info));

        atomicAdd(&quadtree.mass[offset + idx], bodies.mass[i]);
        atomicAdd(&quadtree.center_of_mass[offset + idx].x, bodies.position[i].x * bodies.mass[i]);
        atomicAdd(&quadtree.center_of_mass[offset + idx].y, bodies.position[i].y * bodies.mass[i]);
    }
}

__global__ void __cuda_quadtree_last_layer_per_cell(
    struct bodies bodies,
    struct quadtree quadtree,
    struct normalization_data info
) {
    // Build last layer
    // #### Average the center of mass
    for (u64 cell = __thread_layer_start(quadtree.depth); cell < __thread_layer_end(quadtree.depth); cell++) {
        if (quadtree.mass[cell] > FLT_EPSILON) {
            quadtree.center_of_mass[cell] =
                __vec2f_div(quadtree.center_of_mass[cell], quadtree.mass[cell]);
        }
    }
}

__global__ void __cuda_quadtree_reduce_at_layer(
    struct quadtree quadtree,
    u64 layer
) {
    float *mass = quadtree.mass;
    vec2f *com = quadtree.center_of_mass;

    for (u64 c = __thread_layer_start(layer); c < __thread_layer_end(layer); c++) {
        u64 in_layer_cell = c - __layer_start(layer);
        u64 cn = in_layer_cell * 4 + __layer_start(layer + 1);

        mass[c] = mass[cn] + mass[cn+1] + mass[cn+2] + mass[cn+3];

        com[c].x =
            com[cn].x * mass[cn] + com[cn+1].x * mass[cn+1] +
            com[cn+2].x * mass[cn+2] + com[cn+3].x * mass[cn+3];
        com[c].y =
            com[cn].y * mass[cn] + com[cn+1].y * mass[cn+1] +
            com[cn+2].y * mass[cn+2] + com[cn+3].y * mass[cn+3];

        com[c] = __vec2f_div(com[c], mass[c]);
    }
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

    __cuda_quadtree_last_layer_per_body<<<num_blocks, num_threads>>>(bodies, quadtree, info);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    __cuda_quadtree_last_layer_per_cell<<<num_blocks, num_threads>>>(bodies, quadtree, info);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    for (u64 l = 0; l < quadtree.depth; l++) {
        u64 layer = quadtree.depth - 1 - l;

        __cuda_quadtree_reduce_at_layer<<<num_blocks, num_threads>>>(quadtree, layer);
        err = cudaGetLastError();
        if (err != cudaSuccess)
            return err;
    }

    return cudaSuccess;
}

}