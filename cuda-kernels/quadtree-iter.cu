#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include "common.h"
#include "quadtree-iter.h"

extern "C" {

__device__ bool __cuda_quadtree_iter_next(
    struct visit_result *result,
    struct quadtree_iter *iter,
    struct quadtree quadtree,
    vec2f body_position,
    float theta
) {
    while (iter->stack_size > 0) {
        u64 curr_depth = iter->stack_size - 1;
        u64 curr_cell = iter->visit_stack[curr_depth].current;
        u64 max_cell = iter->visit_stack[curr_depth].max;

        if (curr_cell >= max_cell) {
            // We have already visited the whole subtree
            iter->stack_size -= 1;
            continue;
        }

        // Visit current cell
        iter->visit_stack[curr_depth].current += 1;

        u64 cell_idx = __layer_start(curr_depth) + curr_cell;

        // Stop visit at zero-mass subtree
        if (quadtree.mass[cell_idx] < FLT_EPSILON)
            continue;

        vec2f diff_position = __vec2f_sub(body_position, quadtree.center_of_mass[cell_idx]);
        float ratio = __layer_width(curr_depth) / __vec2f_magnitude(diff_position);
        if ((ratio < theta) || (curr_depth >= quadtree.depth)) {
            // return current cell
            *result = {
                .cell = cell_idx,
                .belongs_to_cell = cell_idx == iter->nearest_cells[curr_depth]
            };
            return true;
        } else {
            // visit subtree
            iter->stack_size += 1;
            iter->visit_stack[curr_depth+1].current = curr_cell * 4;
            iter->visit_stack[curr_depth+1].max = curr_cell * 4 + 4;
        }
    }

    return false;
}

__device__ struct quadtree_iter __cuda_quadtree_iter_new(
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    vec2f body_position
) {
    struct quadtree_iter iter;

    // Initialize the iterator
    iter.visit_stack = (struct visit_cell*) malloc(sizeof(vec2f) * (quadtree.depth + 1));
    iter.nearest_cells = (u64*) malloc(sizeof(u64) * (quadtree.depth + 1));

    // Initialize Visit Stack
    iter.visit_stack[0].current = 0;
    iter.visit_stack[0].max = 1;
    iter.stack_size = 1;

    // Initialize Nearest Cells
    for (u64 layer = 0; layer <= quadtree.depth; layer++) {
        iter.nearest_cells[layer] =
            __nearest_cell_at_layer(layer, __normalize_pos(body_position, info));
    }

    return iter;
}

__device__ void __cuda_quadtree_iter_free(
    struct quadtree_iter iter
) {
    free(iter.visit_stack);
    free(iter.nearest_cells);
}

}