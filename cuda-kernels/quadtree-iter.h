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
    struct visit_cell *visit_stack_base;
    struct visit_cell *visit_stack_top;
    u64 *nearest_cells;
};

__device__ bool __cuda_quadtree_iter_next(
    struct visit_result *result,
    struct quadtree_iter *iter,
    struct quadtree quadtree,
    vec2f body_position,
    float theta
);

__device__ struct quadtree_iter __cuda_quadtree_iter_new(
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    vec2f body_position
);

__device__ void __cuda_quadtree_iter_free(
    struct quadtree_iter iter
);

}

#endif