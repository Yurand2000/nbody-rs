#!/usr/bin/env bash
# Runs target/release/bench following the first three experiments of REPORT.md (Results).
# Sizes left as "...." in the report are configurable via environment variables.
set -euo pipefail
trap 'echo "== interrupted"; exit 130' INT TERM

cd "$(dirname "$0")/.."
BENCH=target/release/bench

ITERS=${ITERS:-1000}
ITERS_THETA=${ITERS_THETA:-50}               # iterations used by the theta experiment
DT=${DT:-0.01}
CUDA_BLOCKS=${CUDA_BLOCKS:-"16 64 256 1024"}
CUDA_THREADS=${CUDA_THREADS:-"16 64 256"}
# grid:<length> spawns (2*length)^2 bodies:
# 100, 196, 324, 576, 1024, 1764, 3136, 5476, 10000, 19600, 40000
GRID_SIZES=${GRID_SIZES:-"5 7 9 12 16 21 28 37 50 70 100"}
THETA_GRID=${THETA_GRID:-30}                 # length for the theta experiment (3600 bodies)
THETAS=${THETAS:-"0 0.125 0.25 0.375 0.5 0.625 0.75 0.875 1 1.125 1.25"}
STEPS_GRID=${STEPS_GRID:-50}                 # length for the grid scenario of experiment 3 (10000 bodies)
CUDA_BLOCKS_THETA=${CUDA_BLOCKS_THETA:-"16 64 256"} # blocks used by the theta experiment
BACKENDS_STEPS=${BACKENDS_STEPS:-"cpu cuda:16x256 cuda:64x256 cuda:256x256"}  # backends used by experiment 3

make_backends() {  # $1 = blocks list
    local out="" blocks threads
    for blocks in $1; do
        for threads in $CUDA_THREADS; do
            out="cuda:${blocks}x${threads} $out"  # prepend: largest config first
        done
    done
    echo "cpu $out"
}
BACKENDS=$(make_backends "$CUDA_BLOCKS")
BACKENDS_THETA=$(make_backends "$CUDA_BLOCKS_THETA")

DATA=report/data
mkdir -p "$DATA"

count() { echo $#; }
NB=$(count $BACKENDS)
NBT=$(count $BACKENDS_THETA)
NBS=$(count $BACKENDS_STEPS)
TOTAL=$(( NB * $(count $GRID_SIZES) + NBT * $(count $THETAS) + NBS * 2 ))
DONE=0
START=$SECONDS

fmt() { printf '%dh%02dm%02ds' $(($1 / 3600)) $(($1 % 3600 / 60)) $(($1 % 60)); }

run() {
    local name t0
    name=$(echo "$*" | tr -c 'A-Za-z0-9.\n' '_' | sed 's/_\+/_/g; s/^_//; s/_$//')
    DONE=$((DONE + 1))
    if [ -e "$DATA/$name.txt" ] && grep -q -e "--- OUTPUT ---" "$DATA/$name.txt"; then
        echo "== [$DONE/$TOTAL] skip (exists): $*"
        return
    fi
    echo "== [$DONE/$TOTAL] $*"
    t0=$SECONDS
    {
        echo "# params: $*"
        "$BENCH" "$@" 2>/dev/null
    } > "$DATA/$name.txt" || echo "== [$DONE/$TOTAL] exited with error (ignored)"
    echo "== [$DONE/$TOTAL] took $(fmt $((SECONDS - t0))), total elapsed $(fmt $((SECONDS - START)))"
}

experiment_done() {
    echo "##### $1 finished: $DONE/$TOTAL runs, elapsed $(fmt $((SECONDS - START)))"
}

# 1. Increasing number of bodies, grid scenario, theta = 0.5
for b in $BACKENDS; do
    for n in $GRID_SIZES; do
        run -s "grid:$n" -n "$ITERS" -t "$DT" -T 0.5 -b "$b"
    done
done
experiment_done "experiment 1"

# 2. Fixed number of bodies, varying theta
for b in $BACKENDS_THETA; do
    for theta in $THETAS; do
        run -s "grid:$THETA_GRID" -n "$ITERS_THETA" -t "$DT" -T "$theta" -b "$b"
    done
done
experiment_done "experiment 2"

# 3. Grid and orbit scenarios, theta = 0.5
for b in $BACKENDS_STEPS; do
    run -s "grid:$STEPS_GRID" -n "$ITERS" -t "$DT" -T 0.5 -b "$b"
    run -s orbit -n "$ITERS" -t "$DT" -T 0.5 -b "$b"
done
experiment_done "experiment 3"
