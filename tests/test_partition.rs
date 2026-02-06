use metis_rs::{Graph, partition};

/// Helper: verify that partition is valid (every vertex assigned to 0..nparts).
fn assert_valid_partition(part: &[usize], n: usize, nparts: usize) {
    assert_eq!(part.len(), n);
    for &p in part {
        assert!(p < nparts, "part {} out of range [0, {})", p, nparts);
    }
    // Each part should have at least one vertex (when n >= nparts)
    if n >= nparts {
        for k in 0..nparts {
            assert!(
                part.iter().any(|&p| p == k),
                "part {} is empty",
                k
            );
        }
    }
}

#[test]
fn empty_graph() {
    let g = Graph::new(0, vec![0], vec![]);
    let (cut, part) = partition(&g, 2);
    assert_eq!(cut, 0);
    assert_eq!(part.len(), 0);
}

#[test]
fn single_vertex() {
    let g = Graph::new(1, vec![0, 0], vec![]);
    let (cut, part) = partition(&g, 1);
    assert_eq!(cut, 0);
    assert_eq!(part, vec![0]);
}

#[test]
fn single_vertex_two_parts() {
    let g = Graph::new(1, vec![0, 0], vec![]);
    let (cut, part) = partition(&g, 2);
    assert_eq!(cut, 0);
    assert_eq!(part.len(), 1);
}

#[test]
fn two_vertices_one_edge() {
    // 0 -- 1
    let g = Graph::new(2, vec![0, 1, 2], vec![1, 0]);
    let (cut, part) = partition(&g, 2);
    assert_eq!(part.len(), 2);
    // They must be in different parts for a 2-way partition
    assert_ne!(part[0], part[1]);
    assert_eq!(cut, 1);
}

#[test]
fn path_graph_4_vertices() {
    // 0 - 1 - 2 - 3
    let xadj = vec![0, 1, 3, 5, 6];
    let adjncy = vec![1, 0, 2, 1, 3, 2];
    let g = Graph::new(4, xadj, adjncy);

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 4, 2);
    // Optimal cut for a path of 4 into 2 parts is 1
    assert!(cut >= 1, "cut should be at least 1, got {}", cut);
    assert!(cut <= 2, "cut should be at most 2, got {}", cut);
}

#[test]
fn cycle_graph_6_vertices() {
    // 0-1-2-3-4-5-0
    let xadj = vec![0, 2, 4, 6, 8, 10, 12];
    let adjncy = vec![
        5, 1, // vertex 0
        0, 2, // vertex 1
        1, 3, // vertex 2
        2, 4, // vertex 3
        3, 5, // vertex 4
        4, 0, // vertex 5
    ];
    let g = Graph::new(6, xadj, adjncy);

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 6, 2);
    // A cycle of 6 into 2 parts: optimal cut is 2
    assert!(cut >= 2, "cut should be at least 2, got {}", cut);
}

#[test]
fn complete_graph_k4() {
    // K4: every vertex connected to every other
    let xadj = vec![0, 3, 6, 9, 12];
    let adjncy = vec![
        1, 2, 3, // vertex 0
        0, 2, 3, // vertex 1
        0, 1, 3, // vertex 2
        0, 1, 2, // vertex 3
    ];
    let g = Graph::new(4, xadj, adjncy);

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 4, 2);
    // K4 into 2 parts: balanced cut is 4 (2+2), imbalanced cut is 3 (3+1)
    assert!(cut >= 3, "K4 bisection cut should be >= 3, got {}", cut);
}

#[test]
fn two_cliques_connected_by_bridge() {
    // Two triangles connected by a single edge:
    // Clique A: 0-1-2 (all connected)
    // Clique B: 3-4-5 (all connected)
    // Bridge: 2-3
    let xadj = vec![0, 2, 4, 7, 10, 12, 14];
    let adjncy = vec![
        1, 2,    // vertex 0: -> 1, 2
        0, 2,    // vertex 1: -> 0, 2
        0, 1, 3, // vertex 2: -> 0, 1, 3
        2, 4, 5, // vertex 3: -> 2, 4, 5
        3, 5,    // vertex 4: -> 3, 5
        3, 4,    // vertex 5: -> 3, 4
    ];
    let g = Graph::new(6, xadj, adjncy);

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 6, 2);
    // Optimal: cut the bridge edge -> cut = 1
    assert_eq!(cut, 1, "two cliques with bridge should have cut = 1, got {}", cut);
    // Vertices 0,1,2 should be in one part, 3,4,5 in another
    assert_eq!(part[0], part[1]);
    assert_eq!(part[1], part[2]);
    assert_eq!(part[3], part[4]);
    assert_eq!(part[4], part[5]);
    assert_ne!(part[2], part[3]);
}

#[test]
fn grid_4x4_into_4_parts() {
    // 4x4 grid graph (16 vertices)
    // Vertex (r,c) = r*4 + c
    let n = 16;
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for r in 0..4 {
        for c in 0..4 {
            let u = r * 4 + c;
            if c + 1 < 4 {
                let v = r * 4 + c + 1;
                adj[u].push(v);
                adj[v].push(u);
            }
            if r + 1 < 4 {
                let v = (r + 1) * 4 + c;
                adj[u].push(v);
                adj[v].push(u);
            }
        }
    }

    let mut xadj = vec![0usize];
    let mut adjncy = Vec::new();
    for neighbors in &adj {
        for &v in neighbors {
            adjncy.push(v);
        }
        xadj.push(adjncy.len());
    }

    let g = Graph::new(n, xadj, adjncy);
    let (cut, part) = partition(&g, 4);
    assert_valid_partition(&part, n, 4);

    // Check balance: each part should have ~4 vertices
    let mut counts = vec![0usize; 4];
    for &p in &part {
        counts[p] += 1;
    }
    for (k, &c) in counts.iter().enumerate() {
        assert!(
            c >= 2 && c <= 6,
            "part {} has {} vertices, expected ~4",
            k,
            c
        );
    }

    // 4x4 grid into 4 parts: optimal cut is 8 (4 quad blocks, 2 cuts each direction)
    // Allow some slack for heuristic
    assert!(cut >= 8, "4x4 grid 4-way cut should be >= 8, got {}", cut);
    assert!(cut <= 16, "4x4 grid 4-way cut should be <= 16, got {}", cut);
}

#[test]
fn weighted_vertices() {
    // 0 -- 1 -- 2, vertex weights [10, 1, 10]
    // Optimal 2-way: {0} vs {1,2} or {0,1} vs {2} to balance weight
    let mut g = Graph::new(3, vec![0, 1, 3, 4], vec![1, 0, 2, 1]);
    g.vwgt = vec![10i64, 1, 10];

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 3, 2);
    assert!(cut >= 1);
}

#[test]
fn weighted_edges() {
    // Triangle: 0-1 (weight 100), 1-2 (weight 1), 0-2 (weight 1)
    // Optimal 2-way: cut the two weak edges, keep the heavy edge intact
    let mut g = Graph::new(
        3,
        vec![0, 2, 4, 6],
        vec![1, 2, 0, 2, 0, 1],
    );
    g.adjwgt = vec![100, 1, 100, 1, 1, 1];

    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 3, 2);
    // If the heavy edge is preserved, cut = 2 (two weight-1 edges)
    // If not, cut could be up to 101
    assert!(cut <= 101);
}

#[test]
fn nparts_equals_n() {
    // Each vertex in its own part
    // 0-1, 1-2
    let g = Graph::new(3, vec![0, 1, 3, 4], vec![1, 0, 2, 1]);
    let (_cut, part) = partition(&g, 3);
    assert_eq!(part.len(), 3);
    // Each vertex should be in a unique part
    let mut seen = vec![false; 3];
    for &p in &part {
        assert!(p < 3);
        seen[p] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

#[test]
fn nparts_one() {
    let g = Graph::new(4, vec![0, 1, 3, 5, 6], vec![1, 0, 2, 1, 3, 2]);
    let (cut, part) = partition(&g, 1);
    assert_eq!(cut, 0);
    assert!(part.iter().all(|&p| p == 0));
}

#[test]
fn disconnected_graph() {
    // Two disconnected edges: 0-1, 2-3
    let g = Graph::new(
        4,
        vec![0, 1, 2, 3, 4],
        vec![1, 0, 3, 2],
    );
    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, 4, 2);
    // Optimal: put each connected component in a different part -> cut = 0
    assert_eq!(cut, 0, "disconnected components should give cut = 0, got {}", cut);
}

#[test]
fn star_graph() {
    // Center vertex 0 connected to 1,2,3,4,5
    let n = 6;
    let xadj = vec![0, 5, 6, 7, 8, 9, 10];
    let adjncy = vec![
        1, 2, 3, 4, 5, // vertex 0
        0,             // vertex 1
        0,             // vertex 2
        0,             // vertex 3
        0,             // vertex 4
        0,             // vertex 5
    ];
    let g = Graph::new(n, xadj, adjncy);
    let (cut, part) = partition(&g, 2);
    assert_valid_partition(&part, n, 2);
    // Center must be in one part, so at least some leaves cross
    assert!(cut >= 1);
}
