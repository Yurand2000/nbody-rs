#include <cuda_runtime.h>
#include <stdint.h>
#include <float.h>
#include "common.h"
#include "quadtree-iter.h"

__device__ vec2f __cuda_leapfrog_force(
    struct bodies bodies,
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    u64 body_id
) {
    struct visit_result result;
    vec2f total_force = __vec2f_zero();

    float theta = params.theta / __vec2f_magnitude(*info.size);

    struct quadtree_iter iter =
        __cuda_quadtree_iter_new(
            quadtree,
            params,
            info,
            bodies.position[body_id]
        );

    while (__cuda_quadtree_iter_next(
        &result,
        &iter,
        quadtree,
        bodies.position[body_id],
        theta
    )) {
        // If the body belongs to the cell, update the cell's mass and CoM
        // float mass = quadtree.mass[result.cell];
        // vec2f com = quadtree.center_of_mass[result.cell];

        // if (result.belongs_to_cell) {
        //     com = __vec2f_mul(com, mass);
        //     mass -= bodies.mass[body_id];
        //     if (mass < FLT_EPSILON)
        //         continue;

        //     com = __vec2f_sub(com, __vec2f_mul(bodies.position[body_id], bodies.mass[body_id]));
        //     com = __vec2f_div(com, mass);
        // }

        // vec2f diff = __vec2f_sub(com, bodies.position[body_id]);

        // // Remove interactions when they are too close
        // if (powf(__vec2f_magnitude(diff), 3.0f) <= mass + bodies.mass[body_id])
        //     continue;

        // vec2f direction = __vec2f_normalize(diff);
        // float magnitude = params.g_constant * mass * bodies.mass[body_id] /
        //                   __vec2f_magnitude_sqr(diff);

        // total_force = __vec2f_add(total_force, __vec2f_mul(direction, magnitude));
    }

    __cuda_quadtree_iter_free(iter);

    return total_force;
}

__global__ void __cuda_leapfrog_kick(
    struct bodies bodies,
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info
) {
    for (u64 i = __thread_body_start(bodies.count); i < __thread_body_end(bodies.count); i++) {
        float mass = bodies.mass[i];
        vec2f* velocity = &bodies.velocity[i];

        // compute total force
        vec2f force = __cuda_leapfrog_force(bodies, quadtree, params, info, i);

        // compute acceleration
        // if (__vec2f_magnitude_sqr(force) < FLT_EPSILON)
        //     return;

        // vec2f acceleration = __vec2f_div(force, mass);

        // // v <= v + \Delta_t / 2 * a
        // *velocity = __vec2f_add(
        //     *velocity,
        //     __vec2f_mul(acceleration, params.time_step / 2.0)
        // );
    }
}

__global__ void __cuda_leapfrog_drift(
    struct bodies bodies,
    struct params params
) {
    for (u64 i = __thread_body_start(bodies.count); i < __thread_body_end(bodies.count); i++) {
        vec2f* position = &bodies.position[i];
        const vec2f velocity = bodies.velocity[i];

        // x <= x + \Delta_t * v
        *position = __vec2f_add(
            *position,
            __vec2f_mul(velocity, params.time_step)
        );
    }
}

extern "C" {

cudaError_t cuda_leapfrog_kick(
    struct bodies bodies,
    struct quadtree quadtree,
    struct params params,
    struct normalization_data info,
    u64 num_blocks,
    u64 num_threads
) {
    cudaError_t err;

    __cuda_leapfrog_kick<<<num_blocks, num_threads>>>(bodies, quadtree, params, info);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

cudaError_t cuda_leapfrog_drift(
    struct bodies bodies,
    struct params params,
    u64 num_blocks,
    u64 num_threads
) {
    cudaError_t err;

    __cuda_leapfrog_drift<<<num_blocks, num_threads>>>(bodies, params);
    err = cudaGetLastError();
    if (err != cudaSuccess)
        return err;

    return cudaSuccess;
}

}