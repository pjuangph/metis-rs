# metis-rs

Pure Rust multilevel k-way graph partitioner inspired by [METIS](http://glaros.dtc.umn.edu/gkhome/metis/metis/overview). No C dependencies.

## Features

- **Heavy-edge matching coarsening** — contracts the graph by matching vertices along heaviest edges
- **Multi-seed greedy graph growing** — initial bisection with multiple seed candidates for quality
- **FM-style boundary refinement** — Fiduccia-Mattheyses swaps to reduce edge cut while maintaining balance
- **Recursive bisection** — k-way partitioning via recursive bisection through the multilevel hierarchy
- **Vertex and edge weights** — optional weighted graphs for non-uniform workloads

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
metis = { path = "../metis-rs" }
```

### Quick Start

```rust
use metis_rs::{Graph, partition};

// A simple 4-vertex path graph: 0-1-2-3
// Stored in CSR format: xadj[u]..xadj[u+1] indexes into adjncy
let xadj = vec![0, 1, 3, 5, 6];
let adjncy = vec![1, 0, 2, 1, 3, 2];
let g = Graph::new(4, xadj, adjncy);

let (edge_cut, part) = partition(&g, 2);
assert_eq!(part.len(), 4);
assert!(part.iter().all(|&p| p < 2));
```

### Weighted Graphs

```rust
use metis_rs::{Graph, partition};

// Triangle graph with vertex weights
let g = Graph::new(3, vec![0, 2, 4, 6], vec![1, 2, 0, 2, 0, 1])
    .with_vwgt(vec![10, 1, 10])
    .with_adjwgt(vec![5, 1, 5, 1, 1, 1]);

let (edge_cut, part) = partition(&g, 2);
```

## API

### `Graph`

CSR (Compressed Sparse Row) graph representation.

| Field | Type | Description |
|-------|------|-------------|
| `n` | `usize` | Number of vertices |
| `xadj` | `Vec<usize>` | Row pointers (length `n + 1`) |
| `adjncy` | `Vec<usize>` | Column indices (neighbor lists) |
| `adjwgt` | `Vec<i64>` | Edge weights (empty = all 1) |
| `vwgt` | `Vec<i64>` | Vertex weights (empty = all 1) |

### `partition(g, nparts) -> (i64, Vec<usize>)`

Partition graph `g` into `nparts` parts. Returns `(edge_cut, partition)` where `partition[u]` is the 0-based part ID for vertex `u`.

## Algorithm

1. **Coarsen**: Repeatedly contract the graph via heavy-edge matching until small (~20 vertices)
2. **Initial partition**: Bisect the coarsest graph using greedy graph growing with multiple seeds, then recursively bisect for k-way
3. **Uncoarsen + refine**: Project the partition back through each level, running FM boundary refinement to minimize edge cut

## Project Structure

```
src/
  lib.rs        # Public API
  graph.rs      # CSR graph struct
  coarsen.rs    # Heavy-edge matching coarsening
  partition.rs  # Greedy graph growing bisection
  refine.rs     # FM boundary refinement
  kway.rs       # Multilevel k-way orchestration
tests/
  test_partition.rs
```

## License

MIT
