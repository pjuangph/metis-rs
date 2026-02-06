//! Graph coarsening via heavy-edge matching.
//!
//! Implements the multilevel coarsening phase: repeatedly contract the graph
//! by matching vertices along heavy edges until the graph is small enough
//! for direct partitioning.

use crate::graph::Graph;

/// Result of a single coarsening level.
#[derive(Clone, Debug)]
pub struct CoarsenLevel {
    /// Coarsened graph.
    pub graph: Graph,
    /// Mapping from fine vertex `u` to coarse vertex: `cmap[u]`.
    pub cmap: Vec<usize>,
    /// Number of coarse vertices.
    pub nc: usize,
}

/// Coarsen the graph by heavy-edge matching.
///
/// Visits vertices in random (deterministic) order, greedily matching each
/// unmatched vertex with its heaviest unmatched neighbor.
pub fn coarsen_once(g: &Graph) -> CoarsenLevel {
    let n = g.n;
    let mut matched = vec![false; n];
    let mut cmap = vec![0usize; n];
    let mut nc = 0usize;

    // Visit in natural order (deterministic; could shuffle for randomization)
    for u in 0..n {
        if matched[u] {
            continue;
        }

        // Find heaviest unmatched neighbor
        let mut best_v = None;
        let mut best_w = -1i64;
        for k in 0..g.degree(u) {
            let v = g.adjncy[g.xadj[u] + k];
            if !matched[v] && v != u {
                let w = g.edge_weight(u, k);
                if w > best_w {
                    best_w = w;
                    best_v = Some(v);
                }
            }
        }

        if let Some(v) = best_v {
            // Match u and v into coarse vertex nc
            cmap[u] = nc;
            cmap[v] = nc;
            matched[u] = true;
            matched[v] = true;
        } else {
            // Unmatched singleton
            cmap[u] = nc;
            matched[u] = true;
        }
        nc += 1;
    }

    // Build coarsened graph
    let graph = build_coarse_graph(g, &cmap, nc);

    CoarsenLevel { graph, cmap, nc }
}

/// Build the coarsened graph from the fine graph and vertex mapping.
fn build_coarse_graph(g: &Graph, cmap: &[usize], nc: usize) -> Graph {
    use std::collections::HashMap;

    // Accumulate coarse vertex weights
    let mut cvwgt = vec![0i64; nc];
    for u in 0..g.n {
        cvwgt[cmap[u]] += g.vertex_weight(u);
    }

    // Accumulate coarse edges
    // For each coarse vertex cu, collect neighbors with accumulated weights
    let mut adj_map: Vec<HashMap<usize, i64>> = vec![HashMap::new(); nc];

    for u in 0..g.n {
        let cu = cmap[u];
        for k in 0..g.degree(u) {
            let v = g.adjncy[g.xadj[u] + k];
            let cv = cmap[v];
            if cu != cv {
                let w = g.edge_weight(u, k);
                *adj_map[cu].entry(cv).or_insert(0) += w;
            }
        }
    }

    // Convert to CSR
    let mut xadj = vec![0usize; nc + 1];
    let mut adjncy = Vec::new();
    let mut adjwgt = Vec::new();

    for cu in 0..nc {
        let mut neighbors: Vec<(usize, i64)> = adj_map[cu].drain().collect();
        neighbors.sort_by_key(|&(v, _)| v);
        for (v, w) in neighbors {
            adjncy.push(v);
            adjwgt.push(w);
        }
        xadj[cu + 1] = adjncy.len();
    }

    Graph {
        n: nc,
        xadj,
        adjncy,
        adjwgt,
        vwgt: cvwgt,
    }
}

/// Coarsen the graph repeatedly until it has fewer than `threshold` vertices.
///
/// Returns a stack of coarsening levels (finest to coarsest).
pub fn multilevel_coarsen(g: &Graph, threshold: usize) -> Vec<CoarsenLevel> {
    let mut levels = Vec::new();
    let mut current = g.clone();

    while current.n > threshold {
        let level = coarsen_once(&current);
        // Stop if coarsening made no progress
        if level.nc >= current.n {
            break;
        }
        current = level.graph.clone();
        levels.push(level);
    }

    levels
}
