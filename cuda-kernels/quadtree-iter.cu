#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include "common.h"
#include "quadtree-iter.h"

__device__ bool __cuda_quadtree_iter_next(
    struct visit_result *result,
    struct quadtree_iter *iter,
    struct quadtree quadtree,
    vec2f body_position,
    float theta
) {
    while (iter->visit_stack_top >= iter->visit_stack_base) {
        u64 curr_depth = iter->visit_stack_top - iter->visit_stack_base;
        u64 curr_cell = iter->visit_stack_top->current;
        u64 max_cell = iter->visit_stack_top->max;

        if (curr_cell >= max_cell) {
            // We have already visited the whole subtree
            iter->visit_stack_top -= 1;
            continue;
        }

        // Visit current cell
        iter->visit_stack_top->current += 1;

        u64 cell_idx = __layer_start(curr_depth) + curr_cell;

        // Stop visit at zero-mass subtree
        if (quadtree.mass[cell_idx] < FLT_EPSILON)
            continue;

        vec2f diff_position = __vec2f_sub(body_position, quadtree.center_of_mass[cell_idx]);
        float ratio = __layer_width(curr_depth) / __vec2f_magnitude(diff_position);
        if ((ratio < theta) || (curr_depth >= quadtree.depth - 1)) {
            // return current cell
            *result = {
                .cell = cell_idx,
                .belongs_to_cell = cell_idx == iter->nearest_cells[curr_depth]
            };
            return true;
        } else {
            // visit subtree
            iter->visit_stack_top += 1;
            iter->visit_stack_top->current = cell_idx * 4;
            iter->visit_stack_top->max = cell_idx * 4 + 4;
        }
    }

    return false;
}

__device__ struct quadtree_iter __cuda_quadtree_iter_new(
    struct quadtree quadtree,
    struct params params,
    struct bodies_info info,
    vec2f body_position
) {
    struct quadtree_iter iter;

    // Initialize the iterator
    iter.visit_stack_base = (struct visit_cell*) malloc(sizeof(vec2f) * quadtree.depth);
    iter.nearest_cells = (u64*) malloc(sizeof(u64) * quadtree.depth);

    // Initialize Visit Stack
    iter.visit_stack_base->current = 0;
    iter.visit_stack_base->max = 1;

    // Initialize Nearest Cells
    for (u64 layer = 0; layer < quadtree.depth - 1; layer++) {
        iter.nearest_cells[layer] =
            __nearest_cell_at_layer(layer, __normalize_pos(body_position, &info));
    }

    // Set the current stack top at the stack base
    iter.visit_stack_top = iter.visit_stack_base;

    return iter;
}

__device__ void __cuda_quadtree_iter_free(
    struct quadtree_iter iter
) {
    free(iter.visit_stack_base);
    free(iter.nearest_cells);
}