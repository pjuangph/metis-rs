//! Pure Rust multilevel k-way graph partitioner inspired by METIS.
//!
//! No C dependencies. Implements:
//! - Heavy-edge matching coarsening
//! - Greedy graph growing initial partitioning
//! - FM-style boundary refinement
//! - Recursive bisection for k-way partitioning
//!
//! # Example
//!
//! ```
//! use metis_rs::{Graph, partition};
//!
//! // A simple 4-vertex path graph: 0-1-2-3
//! let xadj = vec![0, 1, 3, 5, 6];
//! let adjncy = vec![1, 0, 2, 1, 3, 2];
//! let g = Graph::new(4, xadj, adjncy);
//!
//! let (edge_cut, part) = partition(&g, 2);
//! assert_eq!(part.len(), 4);
//! // Each vertex should be assigned to part 0 or 1
//! assert!(part.iter().all(|&p| p < 2));
//! ```

pub mod coarsen;
pub mod graph;
pub mod kway;
pub mod partition;
pub mod refine;

pub use graph::Graph;
pub use kway::part_kway;

/// Partition a graph into `nparts` parts.
///
/// Returns `(edge_cut, partition)` where:
/// - `edge_cut` is the total weight of edges crossing partition boundaries
/// - `partition[u]` is the 0-based part ID for vertex `u`
pub fn partition(g: &Graph, nparts: usize) -> (i64, Vec<usize>) {
    part_kway(g, nparts)
}
