#ifndef CUDA_COMMON
#define CUDA_COMMON

#include <stdint.h>

extern "C" {

typedef struct vec2f {
    float x;
    float y;
};

struct bodies {
    float *mass;
    vec2f *position;
    vec2f *velocity;
    u64 count;
}

struct quadtree {
    float *mass;
    vec2f *center_of_mass;
    u64 depth;
}

struct params {
    float time_step;
    float theta;
    float g_constant;
}

}

__device__ u64 layer_start(u64 depth) {
    return 0;
}

__device__ u64 nearest_cell_at_layer(u64 depth, vec2f position) {
    return 0;
}

__device__ u64 first_cell_at_next_layer(u64 idx) {
    return idx * 4;
}

__device__ u64 parent_cell_at_prev_layer(u64 idx) {
    return idx / 4;
}

#endif