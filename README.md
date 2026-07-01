# N-Body RS

N-Body Simulation written in Rust and CUDA.

## Features

- N-Body 2D Simulation
    - Barnes-Hut approximation
    - Leapfrog integration
- Computing Solutions
    - Single-Thread
    - Multi-Thread CPU-only
    - CUDA

### General Compute Pipeline

- Build QuadTree
- Kick Step
    - Compute per-body forces
    - Half-step velocity update
- Drift Step
    - Position Update
- Re-build QuadTree
- Kick Step
    - *Same as previous*

### Barnes-Hut approximation

*MISSING: What is Barnes-Hut approximation?*

A quadtree of depth $ h $ (with $ h \in \mathbb{N} $) consists on a vector of 2D arrays, one for each layer $ l \in [0,h)$. Each layer $ l $ is a square 2D array whose dimentions are $ 2^l $ cells long, for a total of $ 4^l $ cells.

Each cell, for this n-body simulation, is a structure containing a total mass $ m $ (with $ m \in \mathbb{R} $) and the center of mass $ c $ (2D-vector, $ c \in \mathbb{R}^2$).

## Single-Thread Solution

### Quadtree Structure

The CPU specific quadtree is built as a flattened array for all the layers, each cell containing a `struct` containing three `f32` (C-like `float`s), whose first value is the mass and the other two represent the center of mass coordinates.

## CUDA Solution

### Quadtree Structure

The CUDA specific quadtree is instead a flattened struct of array, i.e. one array containing only the mass values and another one containing center of mass coordinates.

### Quadtree Building

### Quadtree Interrogation