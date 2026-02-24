# Surjective Sum Checker

## Overview

This project implements a highly parallelized, brute-force computational check for a specific Diophantine-like equation involving surjective functions.

Specifically, it is currently unknown if there exists an integer $`m \not\in \{ 1,2,5,7 \}`$ such that:

$$`x^{\{ m \}} + y^{\{ m \}} = z^{\{ m \}}`$$

Where $`k^{\{ m \}}`$ denotes the number of surjective functions from a set of size $m$ to a set of size $k$.

This program attempts to find counterexamples or solutions up to $m = 3000$. To reduce the massive search space, the check operates under the heuristic assumption that $x = z - 1$. Based on preliminary observations, if a solution exists, it is highly probable that $x$ and $z$ are strictly adjacent.

**Reference:** [arXiv:2501.08762](https://arxiv.org/abs/2501.08762)

## Implementation Details

The computational search is implemented in [Rust](https://rust-lang.org/) and is heavily optimized for speed using dynamic programming, parallelization, and unimodal binary search.

### 1. Dynamic Programming Generation

Calculating the number of surjections directly for large numbers is computationally expensive. Instead, the program generates the sequence of surjections for $m$ iteratively from the sequence for $m - 1$. It uses a known recurrence relation, updating the values in-place within a fixed-size array (`SurjectiveLine`).

### 2. Modulo Arithmetic for Performance

Because the number of surjections grows rapidly, the values quickly exceed the bounds of a standard 64-bit integer (`u64`). To prioritize computational throughput and avoid the heavy overhead of large integer (`BigInt`) arithmetic, the program computes all values modulo a large prime (`100000000003`). While this introduces a slight theoretical risk of a false positive (a hash collision), it allows the program to process thousands of values in seconds.

### 3. Parallelization

The workload is distributed across multiple CPU threads using the `rayon` crate:

- **Chunked Generation:** During the DP generation phase, the array is sliced into independent chunks, allowing multiple threads to compute the next row of surjections simultaneously.
- **Parallel Search:** The outer loop iterating through possible values of $z$ is parallelized using `.into_par_iter()`.

### 4. Unimodal Binary Search

For any given $m$, the sequence of surjection counts $x^{\{m\}}$ increases to a peak and then decreases, forming a unimodal curve. The program exploits this property to search for $y$ efficiently:

1. It finds the `change_point` (the peak of the sequence) using a custom binary search.
2. It splits the array into a monotonically increasing left half and a monotonically decreasing right half.
3. For a given $x$ and $z$, it calculates the target difference ($z^{\{m\}} - x^{\{m\}}$) and performs concurrent $O(\log n)$ binary searches on both halves of the array to see if a valid $y$ exists.

## Running the Code

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

To run the checker with release optimizations (highly recommended for speed):

```bash
cargo run --release
```

## Possible Future Improvements

- use a proof assistant (e.g. [Lean](https://lean-lang.org/)) to further constrict the search space
- improve the binary search (exploit the number distribution?)
- manually write SIMD for AVX512
