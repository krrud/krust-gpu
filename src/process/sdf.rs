use cgmath::{Vector3, InnerSpace};

struct PointCloud {
    points: Vec<Vector3<f32>>,
    distances: Vec<f32>,
}

impl PointCloud {
    fn new(points: Vec<Vector3<f32>>) -> Self {
        let distances = vec![0.0; points.len()];
        Self { points, distances }
    }

    fn closest_tri_distance(&self, p: Vector3<f32>) -> f32 {
        let mut min_distance = f32::INFINITY;

        for q in &self.points {
            let distance = (p - *q).magnitude();
            if distance < min_distance {
                min_distance = distance;
            }
        }
        min_distance
    }

    fn compute_distances(&mut self) {
        for (i, p) in self.points.iter().enumerate() {
            let mut min_distance = f32::INFINITY;

            for (j, q) in self.points.iter().enumerate() {
                if i != j {
                    let distance = (*p - *q).magnitude();
                    if distance < min_distance {
                        min_distance = distance;
                    }
                }
            }
            self.distances[i] = min_distance;
        }
    }
}
