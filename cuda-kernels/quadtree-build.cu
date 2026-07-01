#include <cuda_runtime.h>
#include <stdint.h>
#include "common.h"

__global__ void __cuda_quadtree_build(
    struct bodies bodies,
    struct quadtree quadtree,
    struct params params,
    u64 per_thread_bodies,
) {
    // Get which bodies this thread has to process
    u64 idx_start = 0;
    u64 idx_end = 0;

    // Build last layer
        // Process all bodies
    u64 layer = quadtree.depth - 1;
    u64 offset = layer_start(layer);
    for (u64 i = idx_start; i < idx_end; i++) {
        float mass = bodies.mass[i];
        vec2f position = bodies.position[i];

        u64 idx = nearest_cell_at_layer(layer, position);
        quadtree.mass[offset + idx] += mass;
        quadtree.center_of_mass[offset + idx].x += position.x * mass;
        quadtree.center_of_mass[offset + idx].y += position.y * mass;
    }

        // Average the center of mass
    for ()

    // Reduce up to a single per-thread cell
        // from last layer to layer 10 (1024 cells)
    u64 per_thread_layers = quadtree.depth > 10 ? quadtree.depth - 10 : 0;
    for (u64 l = 0; l < per_thread_layers; l++) {
        u64 layer = quadtree.depth - l - 1;


    }

    // Reduce to layer zero
    u64 max_depth = min(quadtree.depth, 10);
    for (u64 l = 0; l < max_depth; l++) {
        u64 layer = quadtree.depth - l - 1;
    }
}