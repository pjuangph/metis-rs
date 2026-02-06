//! K-way partitioning via recursive bisection with multilevel refinement.
//!
//! This is the main entry point that orchestrates coarsening, initial
//! partitioning, projection, and refinement.

use crate::coarsen::multilevel_coarsen;
use crate::graph::Graph;
use crate::partition::initial_partition;
use crate::refine::fm_refine;

/// Default coarsening threshold: stop when graph has this many vertices or fewer.
const COARSEN_THRESHOLD: usize = 20;

/// Default number of FM refinement passes per level.
const REFINE_PASSES: usize = 10;

/// Partition a graph into `nparts` parts using multilevel k-way partitioning.
///
/// Returns `(edge_cut, partition)` where `partition[u]` is the 0-based
/// part ID for vertex `u`.
///
/// # Algorithm
///
/// 1. **Coarsening**: Repeatedly contract the graph via heavy-edge matching
///    until it has fewer than `COARSEN_THRESHOLD` vertices.
/// 2. **Initial partitioning**: Partition the small coarsened graph using
///    recursive greedy graph growing.
/// 3. **Uncoarsening + refinement**: Project the partition back through each
///    coarsening level, running FM boundary refinement at each step.
pub fn part_kway(g: &Graph, nparts: usize) -> (i64, Vec<usize>) {
    if g.n == 0 {
        return (0, Vec::new());
    }
    if nparts <= 1 {
        return (0, vec![0; g.n]);
    }
    if g.n <= nparts {
        let part: Vec<usize> = (0..g.n).collect();
        let cut = g.edge_cut(&part);
        return (cut, part);
    }

    // Phase 1: Coarsen
    let levels = multilevel_coarsen(g, COARSEN_THRESHOLD.max(nparts * 2));

    // Phase 2: Initial partition of the coarsest graph
    let coarsest = if levels.is_empty() {
        g.clone()
    } else {
        levels.last().unwrap().graph.clone()
    };

    let mut current_part = initial_partition(&coarsest, nparts);
    fm_refine(&coarsest, &mut current_part, nparts, REFINE_PASSES);

    // Phase 3: Uncoarsen and refine
    // levels[0].cmap maps original vertices -> level 0 coarse vertices
    // levels[1].cmap maps level 0 coarse vertices -> level 1 coarse vertices
    // etc. We project back in reverse order.
    for (i, level) in levels.iter().enumerate().rev() {
        let fine_graph = if i == 0 {
            g.clone()
        } else {
            levels[i - 1].graph.clone()
        };

        let fine_n = fine_graph.n;
        let mut fine_part = vec![0usize; fine_n];
        for u in 0..fine_n {
            fine_part[u] = current_part[level.cmap[u]];
        }

        fm_refine(&fine_graph, &mut fine_part, nparts, REFINE_PASSES);
        current_part = fine_part;
    }

    let cut = g.edge_cut(&current_part);
    (cut, current_part)
}
