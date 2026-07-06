#ifndef CUDA_COMMON
#define CUDA_COMMON

#include <stdint.h>

#include "vec2f.h"

extern "C" {

typedef uint64_t u64;

struct bodies {
    float *mass;
    vec2f *position;
    vec2f *velocity;
    u64 count;
};

struct normalization_data {
    vec2f *offset;
    vec2f *size;
};

struct quadtree {
    float *mass;
    vec2f *center_of_mass;
    u64 depth;
};

struct params {
    // Simulation Params
    float time_step;
    float theta;
    float g_constant;
};

}

// Utils
__host__ __device__ inline u64 ceil_div_u64(u64 a, u64 b) {
  return (a + b - 1) / b;
}

__device__ inline u64 __log2_u64(u64 x) {
    return 31 - __clz(x);
}

inline u64 is_pow2_u64(u64 x) {
    return x && !(x & (x - 1));
}

inline u64 is_pow4_u64(u64 x) {
    if (!x) return 0;
    if (x & (x - 1)) return 0;
    u64 idx = 63 - __builtin_clz(x);
    return (idx & 1) == 0;
}

__device__ inline u64 __thread_idx() {
    return blockDim.x * blockIdx.x + threadIdx.x;
}

__device__ inline u64 __grid_size() {
    return blockDim.x * gridDim.x;
}

// Body indices when iterating over bodies
__host__ __device__ inline u64 __per_thread_bodies__(u64 num_bodies, u64 num_blocks, u64 num_threads) {
    return ceil_div_u64(num_bodies, num_blocks * num_threads);
}

__host__ __device__ inline u64 __thread_body_start__(u64 num_bodies, u64 num_blocks, u64 block_idx, u64 num_threads, u64 thread_idx) {
    return (num_threads * block_idx + thread_idx) * __per_thread_bodies__(num_bodies, num_blocks, num_threads);
}

__device__ inline u64 __thread_body_start(u64 num_bodies) {
    return __thread_body_start__(num_bodies, gridDim.x, blockIdx.x, blockDim.x, threadIdx.x);
}

__host__ __device__ inline u64 __thread_body_end__(u64 num_bodies, u64 num_blocks, u64 block_idx, u64 num_threads, u64 thread_idx) {
    return min(
        (num_threads * block_idx + thread_idx + 1) * __per_thread_bodies__(num_bodies, num_blocks, num_threads),
        num_bodies
    );
}

__device__ inline u64 __thread_body_end(u64 num_bodies) {
    return __thread_body_end__(num_bodies, gridDim.x, blockIdx.x, blockDim.x, threadIdx.x);
}

// Cell Indies when iterating over the quadtree layers
__host__ __device__ inline u64 __layer_start(u64 depth) {
    u64 size = 0;
    u64 tmp = 1;
    for (u64 i = 0; i < depth; i++){
        size += tmp;
        tmp *= 4;
    }

    return size;
}

__host__ __device__ inline u64 __layer_end(u64 depth) {
    return __layer_start(depth + 1);
}

__device__ inline u64 __layer_dim(u64 depth) {
    return 1 << depth;
}

__device__ inline u64 __layer_size(u64 depth) {
    return __layer_dim(depth) * __layer_dim(depth);
}

__device__ inline u64 __thread_layer_start(u64 depth) {
    u64 idx = __thread_idx();
    u64 layer_size = __layer_size(depth);
    u64 num_threads = __grid_size();
    return __layer_start(depth) + ceil_div_u64(layer_size * idx, num_threads);
}

__device__ inline u64 __thread_layer_end(u64 depth) {
    u64 idx = __thread_idx() + 1;
    u64 layer_size = __layer_size(depth);
    u64 num_threads = __grid_size();
    return __layer_start(depth) + ceil_div_u64(layer_size * idx, num_threads);
}

__device__ inline float __layer_width(u64 depth) {
    return 1.0f / (float)(__layer_dim(depth));
}

__device__ inline u64 __nearest_cell_in_layer(vec2f position) {
    if (position.x <= 1.0) {
        if (position.y <= 1.0) {
            return 0;
        } else {
            return 1;
        }
    } else {
        if (position.y <= 1.0) {
            return 2;
        } else {
            return 3;
        }
    }
}

__device__ inline vec2f __position_in_layer(u64 cell) {
    switch (cell) {
        case 0:
        default:
            return { .x = 0.0f, .y = 0.0f };
        case 1:
            return { .x = 0.0f, .y = 1.0f };
        case 2:
            return { .x = 1.0f, .y = 0.0f };
        case 3:
            return { .x = 1.0f, .y = 1.0f };
    }
}

__device__ inline u64 __nearest_cell_at_layer(u64 depth, vec2f position) {
    u64 out = 0;
    float layer_dim = __layer_dim(1);
    for (u64 l = 0; l < depth; l++) {
        u64 cell = __nearest_cell_in_layer(__vec2f_mul(position, layer_dim));
        out = out * 4 + cell;
        position = __vec2f_sub(position, __vec2f_div(__position_in_layer(cell), layer_dim));
        layer_dim *= 2.0;
    }

    return out;
}

__device__ inline vec2f __normalize_pos(vec2f position, const struct normalization_data info) {
    return {
        .x = (position.x - info.offset->x) / info.size->x,
        .y = 1.0f - ((position.y - info.offset->y) / info.size->y),
    };
}

#endif