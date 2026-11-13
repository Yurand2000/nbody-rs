#ifndef CUDA_VEC2F
#define CUDA_VEC2F

#include <float.h>

extern "C" {

typedef struct {
    float x;
    float y;
} vec2f;

}

__device__ inline vec2f __vec2f_zero() {
    return { .x = 0.0f, .y = 0.0f };
}

__device__ inline vec2f __vec2f_max_value() {
    return { .x = FLT_MAX, .y = FLT_MAX };
}

__device__ inline vec2f __vec2f_min_value() {
    return { .x = -FLT_MAX, .y = -FLT_MAX };
}

__device__ inline vec2f __vec2f_add(const vec2f a, const vec2f b) {
    return { .x = a.x + b.x, .y = a.y + b.y };
}

__device__ inline vec2f __vec2f_sub(const vec2f a, const vec2f b) {
    return { .x = a.x - b.x, .y = a.y - b.y };
}

__device__ inline vec2f __vec2f_comp_mul(const vec2f a, const vec2f b) {
    return { .x = a.x * b.x, .y = a.y * b.y };
}

__device__ inline vec2f __vec2f_comp_div(const vec2f a, const vec2f b) {
    return { .x = a.x / b.x, .y = a.y / b.y };
}

__device__ inline vec2f __vec2f_mul(const vec2f a, const float b) {
    return { .x = a.x * b, .y = a.y * b };
}

__device__ inline vec2f __vec2f_div(const vec2f a, const float b) {
    return { .x = a.x / b, .y = a.y / b };
}

__device__ inline float __vec2f_magnitude_sqr(vec2f vec) {
    return vec.x * vec.x + vec.y * vec.y;
}

__device__ inline float __vec2f_magnitude(vec2f vec) {
    return sqrtf(__vec2f_magnitude_sqr(vec));
}

__device__ inline vec2f __vec2f_normalize(vec2f vec) {
    return __vec2f_div(vec, __vec2f_magnitude(vec));
}

__device__ inline vec2f __vec2f_comp_min(const vec2f a, const vec2f b) {
    return { .x = min(a.x, b.x), .y = min(a.y, b.y) };
}

__device__ inline vec2f __vec2f_comp_max(const vec2f a, const vec2f b) {
    return { .x = max(a.x, b.x), .y = max(a.y, b.y) };
}

#endif