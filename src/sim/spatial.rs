use glam::Vec2;

/// Flat O(1) Spatial Distance Matrix and Pre-sorted Neighbors Index
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    pub count: usize,
    matrix: Vec<f32>,
    sorted_neighbors: Vec<Vec<(usize, f32)>>,
}

impl SpatialIndex {
    pub fn build(positions: &[Vec2]) -> Self {
        let count = positions.len();
        let mut matrix = vec![0.0f32; count * count];
        let mut sorted_neighbors = Vec::with_capacity(count);

        // Precompute all pairwise distances
        for i in 0..count {
            let mut neighbors_for_i = Vec::with_capacity(count - 1);
            for j in 0..count {
                if i == j {
                    matrix[i * count + j] = 0.0;
                } else {
                    let d = positions[i].distance(positions[j]);
                    matrix[i * count + j] = d;
                    neighbors_for_i.push((j, d));
                }
            }
            // Sort neighbors ascending by distance
            neighbors_for_i.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            sorted_neighbors.push(neighbors_for_i);
        }

        Self {
            count,
            matrix,
            sorted_neighbors,
        }
    }

    /// O(1) lookup of distance between planet a and planet b
    #[inline(always)]
    pub fn distance(&self, a: usize, b: usize) -> f32 {
        if a < self.count && b < self.count {
            self.matrix[a * self.count + b]
        } else {
            f32::MAX
        }
    }

    /// Returns all neighbors of planet `from` within `max_radius`, ordered nearest to farthest
    pub fn neighbors_within(&self, from: usize, max_radius: f32) -> &[(usize, f32)] {
        if from >= self.count {
            return &[];
        }
        let list = &self.sorted_neighbors[from];
        // Binary search to find cutoff index where distance exceeds max_radius
        let idx = list.partition_point(|&(_, d)| d <= max_radius);
        &list[..idx]
    }

    /// Returns the nearest neighbor to planet `from`
    pub fn nearest(&self, from: usize) -> Option<(usize, f32)> {
        if from < self.count && !self.sorted_neighbors[from].is_empty() {
            Some(self.sorted_neighbors[from][0])
        } else {
            None
        }
    }
}
