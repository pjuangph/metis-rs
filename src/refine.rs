//! Partition refinement using the Kernighan-Lin / Fiduccia-Mattheyses algorithm.
//!
//! After projecting a partition from a coarser graph back to a finer one,
//! this module improves the partition by swapping boundary vertices between
//! parts to reduce the edge cut while maintaining balance.

use crate::graph::Graph;

/// Maximum allowed imbalance factor (5% above perfect balance).
const MAX_IMBALANCE: f64 = 1.05;

/// Refine a k-way partition using boundary FM-style swaps.
///
/// Performs up to `max_passes` passes. Each pass iterates over boundary
/// vertices and moves them to the neighboring part that yields the greatest
/// edge-cut reduction while maintaining balance.
pub fn fm_refine(g: &Graph, part: &mut [usize], nparts: usize, max_passes: usize) {
    if g.n == 0 || nparts <= 1 {
        return;
    }

    for _pass in 0..max_passes {
        let improved = fm_pass(g, part, nparts);
        if !improved {
            break;
        }
    }
}

/// Single FM refinement pass. Returns `true` if any improvement was made.
fn fm_pass(g: &Graph, part: &mut [usize], nparts: usize) -> bool {
    let n = g.n;

    // Compute part weights
    let mut part_weight = vec![0i64; nparts];
    for u in 0..n {
        part_weight[part[u]] += g.vertex_weight(u);
    }
    let total_weight: i64 = part_weight.iter().sum();
    let max_part_weight = (total_weight as f64 * MAX_IMBALANCE / nparts as f64).ceil() as i64;

    let mut improved = false;
    let mut locked = vec![false; n];

    // Iterate: find best move among all boundary vertices
    for _iter in 0..n {
        let mut best_u = None;
        let mut best_to = 0usize;
        let mut best_gain = i64::MIN;

        for u in 0..n {
            if locked[u] {
                continue;
            }

            let from = part[u];

            // Compute external edges per part
            let mut ext = vec![0i64; nparts];
            let mut int = 0i64;
            for k in 0..g.degree(u) {
                let v = g.adjncy[g.xadj[u] + k];
                let w = g.edge_weight(u, k);
                if part[v] == from {
                    int += w;
                } else {
                    ext[part[v]] += w;
                }
            }

            // Check if this is a boundary vertex
            let is_boundary = ext.iter().any(|&e| e > 0);
            if !is_boundary {
                continue;
            }

            // Try moving to each neighboring part
            for to in 0..nparts {
                if to == from || ext[to] == 0 {
                    continue;
                }

                let vw = g.vertex_weight(u);

                // Balance check: would `to` exceed max?
                if part_weight[to] + vw > max_part_weight {
                    continue;
                }

                // Gain = external edges to `to` - internal edges in `from`
                let gain = ext[to] - int;

                if gain > best_gain {
                    best_gain = gain;
                    best_u = Some(u);
                    best_to = to;
                }
            }
        }

        match best_u {
            Some(u) if best_gain > 0 => {
                let from = part[u];
                let vw = g.vertex_weight(u);
                part_weight[from] -= vw;
                part_weight[best_to] += vw;
                part[u] = best_to;
                locked[u] = true;
                improved = true;
            }
            _ => break, // No profitable move found
        }
    }

    improved
}
