//! CSR graph representation for partitioning.

/// A graph stored in Compressed Sparse Row (CSR) format.
///
/// Vertices are numbered `0..n`. For vertex `u`, its neighbors are
/// `adjncy[xadj[u]..xadj[u+1]]` with corresponding edge weights
/// `adjwgt[xadj[u]..xadj[u+1]]`.
#[derive(Clone, Debug)]
pub struct Graph {
    /// Number of vertices.
    pub n: usize,
    /// Row pointers (length `n + 1`).
    pub xadj: Vec<usize>,
    /// Column indices (flattened neighbor lists).
    pub adjncy: Vec<usize>,
    /// Edge weights aligned with `adjncy`. If empty, all edges have weight 1.
    pub adjwgt: Vec<i64>,
    /// Vertex weights. If empty, all vertices have weight 1.
    pub vwgt: Vec<i64>,
}

impl Graph {
    /// Create a graph from CSR arrays.
    pub fn new(n: usize, xadj: Vec<usize>, adjncy: Vec<usize>) -> Self {
        assert_eq!(xadj.len(), n + 1);
        Self {
            n,
            xadj,
            adjncy,
            adjwgt: Vec::new(),
            vwgt: Vec::new(),
        }
    }

    /// Set edge weights.
    pub fn with_adjwgt(mut self, adjwgt: Vec<i64>) -> Self {
        assert_eq!(adjwgt.len(), self.adjncy.len());
        self.adjwgt = adjwgt;
        self
    }

    /// Set vertex weights.
    pub fn with_vwgt(mut self, vwgt: Vec<i64>) -> Self {
        assert_eq!(vwgt.len(), self.n);
        self.vwgt = vwgt;
        self
    }

    /// Degree of vertex `u`.
    pub fn degree(&self, u: usize) -> usize {
        self.xadj[u + 1] - self.xadj[u]
    }

    /// Neighbors of vertex `u`.
    pub fn neighbors(&self, u: usize) -> &[usize] {
        &self.adjncy[self.xadj[u]..self.xadj[u + 1]]
    }

    /// Edge weight for the `k`-th neighbor of `u` (0-indexed within neighbor list).
    pub fn edge_weight(&self, u: usize, k: usize) -> i64 {
        if self.adjwgt.is_empty() {
            1
        } else {
            self.adjwgt[self.xadj[u] + k]
        }
    }

    /// Vertex weight for `u`.
    pub fn vertex_weight(&self, u: usize) -> i64 {
        if self.vwgt.is_empty() {
            1
        } else {
            self.vwgt[u]
        }
    }

    /// Total weight of all edges incident to `u`.
    pub fn weighted_degree(&self, u: usize) -> i64 {
        let start = self.xadj[u];
        let end = self.xadj[u + 1];
        if self.adjwgt.is_empty() {
            (end - start) as i64
        } else {
            self.adjwgt[start..end].iter().sum()
        }
    }

    /// Total edge cut for a given partition assignment.
    pub fn edge_cut(&self, part: &[usize]) -> i64 {
        let mut cut = 0i64;
        for u in 0..self.n {
            for k in 0..self.degree(u) {
                let v = self.adjncy[self.xadj[u] + k];
                if part[u] != part[v] {
                    cut += self.edge_weight(u, k);
                }
            }
        }
        cut / 2 // each edge counted twice
    }
}
