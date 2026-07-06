#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include "common.h"
#include "quadtree-iter.h"

extern "C" {

__device__ void __visit_stack_push(
    struct quadtree_iter *iter,
    u64 current,
    u64 max
) {
    iter->visit_stack[iter->stack_size] = {
        .current = current,
        .max = max
    };
    iter->stack_size += 1;
}

__device__ void __visit_stack_pop(
    struct quadtree_iter *iter
) {
    iter->stack_size -= 1;
}

__device__ u64 __visit_stack_depth(
    struct quadtree_iter *iter
) {
    return iter->stack_size - 1;
}

__device__ struct visit_cell *__visit_stack_top(
    struct quadtree_iter *iter
) {
    return &(iter->visit_stack[iter->stack_size - 1]);
}

__device__ bool __visit_stack_is_empty(
    struct quadtree_iter *iter
) {
    return iter->stack_size == 0;
}

__device__ bool __cuda_quadtree_iter_next(
    struct visit_result *result,
    struct quadtree_iter *iter,
    struct quadtree quadtree,
    vec2f body_position,
    float theta
) {
    while (!__visit_stack_is_empty(iter)) {
        u64 curr_depth = __visit_stack_depth(iter);
        struct visit_cell *top = __visit_stack_top(iter);
        u64 curr_cell = top->current;
        u64 max_cell = top->max;

        if (curr_cell >= max_cell) {
            // We have already visited the whole subtree
            __visit_stack_pop(iter);
            continue;
        }

        // Visit current cell
        top->current += 1;

        u64 cell_idx = __layer_start(curr_depth) + curr_cell;

        // Stop visit at zero-mass subtree
        if (quadtree.mass[cell_idx] < FLT_EPSILON)
            continue;

        vec2f diff_position = __vec2f_sub(body_position, quadtree.center_of_mass[cell_idx]);
        float ratio = theta;
        if (__vec2f_magnitude_sqr(diff_position) >= FLT_EPSILON)
            ratio = __layer_width(curr_depth) / __vec2f_magnitude(diff_position);

        if ((ratio < theta) || (curr_depth >= quadtree.depth - 1)) {
            // return current cell
            *result = {
                .cell = cell_idx,
                .belongs_to_cell = curr_cell == iter->nearest_cells[curr_depth]
            };
            return true;
        } else {
            // visit subtree
            __visit_stack_push(iter, curr_cell * 4, curr_cell * 4 + 4);
        }
    }

    return false;
}

__device__ struct quadtree_iter __cuda_quadtree_iter_init(
    struct quadtree_iter iterators,
    u64 thread_idx,
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    vec2f body_position
) {
    struct quadtree_iter iter;

    // Initialize the iterator
    iter.visit_stack = &iterators.visit_stack[thread_idx * (quadtree.depth + 1)];
    iter.stack_size = 0;
    iter.nearest_cells = &iterators.nearest_cells[thread_idx * (quadtree.depth + 1)];

    // Initialize Visit Stack
    __visit_stack_push(&iter, 0, 1);

    // Initialize Nearest Cells
    for (u64 layer = 0; layer <= quadtree.depth; layer++) {
        iter.nearest_cells[layer] =
            __nearest_cell_at_layer(layer, __normalize_pos(body_position, info));
    }

    return iter;
}

cudaError_t cuda_quadtree_iter_alloc(
    struct quadtree_iter *iterators,
    u64 quadtree_depth,
    u64 grid_size
) {
    cudaError_t err;

    // Allocate iterators
    err = cudaMalloc((void**)(&iterators->visit_stack), sizeof(vec2f) * grid_size * (quadtree_depth + 1));
    if (err != cudaSuccess)
        return err;

    err = cudaMalloc((void**)(&iterators->nearest_cells), sizeof(u64) * grid_size * (quadtree_depth + 1));
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

cudaError_t cuda_quadtree_iter_free(
    struct quadtree_iter *iterators
) {
    cudaError_t err;

    err = cudaFree(iterators->visit_stack);
    if (err != cudaSuccess)
        return err;

    err = cudaFree(iterators->nearest_cells);
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

}