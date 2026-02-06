//! Initial partitioning of small (coarsened) graphs.
//!
//! Implements greedy graph growing (GGP) bisection for the coarsest graph
//! in the multilevel hierarchy.

use crate::graph::Graph;

/// Bisect a small graph using greedy graph growing.
///
/// Returns a partition vector where each entry is 0 or 1.
/// Attempts to balance vertex weight across the two parts.
/// Tries multiple seed vertices and returns the best bisection.
pub fn initial_bisection(g: &Graph) -> Vec<usize> {
    let n = g.n;
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0];
    }

    // Collect candidate seeds: several high-degree vertices for diversity
    let mut candidates: Vec<usize> = Vec::new();
    candidates.push(0);
    candidates.push(n / 2);
    candidates.push(n - 1);
    // Add top-degree vertices
    let mut by_degree: Vec<usize> = (0..n).collect();
    by_degree.sort_by(|&a, &b| g.weighted_degree(b).cmp(&g.weighted_degree(a)));
    for &v in by_degree.iter().take(4) {
        candidates.push(v);
    }
    candidates.sort_unstable();
    candidates.dedup();

    let mut best_part = vec![0usize; n];
    let mut best_cut = i64::MAX;

    for &seed in &candidates {
        let part = grow_bisection(g, seed);
        let cut = g.edge_cut(&part);
        if cut < best_cut {
            best_cut = cut;
            best_part = part;
        }
    }

    best_part
}

/// Grow a bisection from a given seed vertex.
fn grow_bisection(g: &Graph, seed: usize) -> Vec<usize> {
    let n = g.n;
    let mut part = vec![1usize; n];
    let mut in_part0 = vec![false; n];

    let total_weight: i64 = (0..n).map(|u| g.vertex_weight(u)).sum();
    let target = total_weight / 2;
    let mut weight0: i64 = 0;

    in_part0[seed] = true;
    part[seed] = 0;
    weight0 += g.vertex_weight(seed);

    loop {
        if weight0 >= target {
            break;
        }

        let mut best_u = None;
        let mut best_gain = -1i64;

        for u in 0..n {
            if in_part0[u] {
                continue;
            }
            let mut gain = 0i64;
            for k in 0..g.degree(u) {
                let v = g.adjncy[g.xadj[u] + k];
                if in_part0[v] {
                    gain += g.edge_weight(u, k);
                }
            }
            if gain > best_gain || (gain == best_gain && best_u.is_none()) {
                best_gain = gain;
                best_u = Some(u);
            }
        }

        match best_u {
            Some(u) if best_gain > 0 || weight0 < target => {
                in_part0[u] = true;
                part[u] = 0;
                weight0 += g.vertex_weight(u);
            }
            _ => break,
        }
    }

    part
}

/// Partition a small graph into `nparts` using recursive bisection.
///
/// Each entry in the returned vector is a partition ID in `0..nparts`.
pub fn initial_partition(g: &Graph, nparts: usize) -> Vec<usize> {
    if nparts <= 1 || g.n == 0 {
        return vec![0; g.n];
    }

    let bisect = initial_bisection(g);

    if nparts == 2 {
        return bisect;
    }

    // Recursive bisection: split into two subsets, then partition each
    let left_parts = nparts / 2;
    let right_parts = nparts - left_parts;

    // Collect vertices for each side
    let left_verts: Vec<usize> = (0..g.n).filter(|&u| bisect[u] == 0).collect();
    let right_verts: Vec<usize> = (0..g.n).filter(|&u| bisect[u] == 1).collect();

    // Build subgraphs and recursively partition
    let left_sub = build_subgraph(g, &left_verts);
    let right_sub = build_subgraph(g, &right_verts);

    let left_part = initial_partition(&left_sub, left_parts);
    let right_part = initial_partition(&right_sub, right_parts);

    // Map back to original vertex IDs
    let mut part = vec![0usize; g.n];
    for (local_idx, &global_v) in left_verts.iter().enumerate() {
        part[global_v] = left_part[local_idx];
    }
    for (local_idx, &global_v) in right_verts.iter().enumerate() {
        part[global_v] = left_parts + right_part[local_idx];
    }

    part
}

/// Build an induced subgraph from a subset of vertices.
fn build_subgraph(g: &Graph, verts: &[usize]) -> Graph {
    use std::collections::HashMap;

    let n_sub = verts.len();
    if n_sub == 0 {
        return Graph::new(0, vec![0], Vec::new());
    }

    // Map global -> local vertex index
    let mut global_to_local: HashMap<usize, usize> = HashMap::with_capacity(n_sub);
    for (local, &global) in verts.iter().enumerate() {
        global_to_local.insert(global, local);
    }

    let mut xadj = vec![0usize; n_sub + 1];
    let mut adjncy = Vec::new();
    let mut adjwgt = Vec::new();
    let mut vwgt = Vec::with_capacity(n_sub);

    for (local_u, &global_u) in verts.iter().enumerate() {
        vwgt.push(g.vertex_weight(global_u));

        for k in 0..g.degree(global_u) {
            let global_v = g.adjncy[g.xadj[global_u] + k];
            if let Some(&local_v) = global_to_local.get(&global_v) {
                adjncy.push(local_v);
                adjwgt.push(g.edge_weight(global_u, k));
            }
        }
        xadj[local_u + 1] = adjncy.len();
    }

    let mut sub = Graph::new(n_sub, xadj, adjncy);
    sub.adjwgt = adjwgt;
    sub.vwgt = vwgt;
    sub
}
