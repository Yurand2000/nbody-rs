#ifndef CUDA_QUADTREE_ITER
#define CUDA_QUADTREE_ITER

#include "common.h"

extern "C" {

struct visit_result {
    u64 cell;
    bool belongs_to_cell;
};

struct visit_cell {
    u64 current;
    u64 max;
};

struct quadtree_iter {
    struct visit_cell *visit_stack;
    u64 stack_size;
    u64 *nearest_cells;
};

__device__ struct quadtree_iter __cuda_quadtree_iter_init(
    struct quadtree_iter iterators,
    u64 thread_idx,
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    vec2f body_position
);

__device__ bool __cuda_quadtree_iter_next(
    struct visit_result *result,
    struct quadtree_iter *iter,
    struct quadtree quadtree,
    vec2f body_position,
    float theta
);

cudaError_t cuda_quadtree_iter_alloc(
    struct quadtree_iter *iterators,
    u64 quadtree_depth,
    u64 grid_size
);

cudaError_t cuda_quadtree_iter_free(
    struct quadtree_iter *iterators
);

}

#endif