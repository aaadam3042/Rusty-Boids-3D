use crate::boid::Boid;
use crate::bounds::Bounds;
use crate::math::Vec3;

type BoidIndices = Vec<usize>;

/// A bounded 3D spatial grid used to find possible boid neighbours.
///
/// The grid cells are flattened in x-major order. Each bucket contains indices
/// into the world's boid vector; exact distance checks remain the caller's
/// responsibility.
pub(crate) struct SpatialGrid {
    origin: Vec3,
    cell_size: f32,
    dimensions: [usize; 3],
    buckets: Vec<BoidIndices>,
    /// Maps each boid index back to its current flattened bucket index.
    boid_cells: Vec<usize>,
}

impl SpatialGrid {
    pub(crate) fn new(bounds: &Bounds, cell_size: f32, boid_capacity: usize) -> Self {
        assert!(
            cell_size.is_finite() && cell_size > 0.0,
            "spatial-grid cell size must be finite and positive"
        );

        let size = bounds.size();
        let dimensions = [
            Self::dimension_for(size.x, cell_size),
            Self::dimension_for(size.y, cell_size),
            Self::dimension_for(size.z, cell_size),
        ];
        let bucket_count = dimensions
            .into_iter()
            .try_fold(1_usize, usize::checked_mul)
            .expect("spatial-grid dimensions are too large");

        let buckets = (0..bucket_count).map(|_| Vec::new()).collect();

        Self {
            origin: bounds.min(),
            cell_size,
            dimensions,
            buckets,
            boid_cells: vec![0; boid_capacity],
        }
    }

    /// Clears and repopulates the grid while retaining bucket allocations.
    pub(crate) fn rebuild(&mut self, boids: &[Boid]) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }

        self.boid_cells.resize(boids.len(), 0);

        for (boid_index, boid) in boids.iter().enumerate() {
            let cell_index = self.cell_for(boid.position);
            self.buckets[cell_index].push(boid_index);
            self.boid_cells[boid_index] = cell_index;
        }
    }

    /// Collects possible neighbours from every cell touched by `search_radius`.
    ///
    /// Results are sorted by boid index so steering accumulation follows the
    /// same deterministic order as a full traversal. The querying boid may be
    /// present and should be skipped by the caller.
    pub(crate) fn collect_candidates(
        &self,
        position: Vec3,
        search_radius: f32,
        candidates: &mut Vec<usize>,
    ) {
        debug_assert!(position.is_finite());
        debug_assert!(search_radius.is_finite() && search_radius >= 0.0);

        candidates.clear();

        let [cell_x, cell_y, cell_z] = self.coordinates_for(position);
        let cell_radius = (search_radius / self.cell_size).ceil() as usize;

        let min_x = cell_x.saturating_sub(cell_radius);
        let min_y = cell_y.saturating_sub(cell_radius);
        let min_z = cell_z.saturating_sub(cell_radius);
        let max_x = cell_x
            .saturating_add(cell_radius)
            .min(self.dimensions[0] - 1);
        let max_y = cell_y
            .saturating_add(cell_radius)
            .min(self.dimensions[1] - 1);
        let max_z = cell_z
            .saturating_add(cell_radius)
            .min(self.dimensions[2] - 1);

        for z in min_z..=max_z {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let bucket_index = self.flatten([x, y, z]);
                    candidates.extend_from_slice(&self.buckets[bucket_index]);
                }
            }
        }
    }

    /// Moves a boid between buckets after an in-place position update.
    ///
    /// This keeps later boids in a sequential step aware of earlier boids that
    /// crossed a cell boundary.
    pub(crate) fn update_boid(&mut self, boid_index: usize, new_position: Vec3) {
        let old_cell = self.boid_cells[boid_index];
        let new_cell = self.cell_for(new_position);

        if old_cell == new_cell {
            return;
        }

        let position_in_bucket = self.buckets[old_cell]
            .iter()
            .position(|&index| index == boid_index)
            .expect("boid is missing from its recorded spatial-grid bucket");

        self.buckets[old_cell].swap_remove(position_in_bucket);
        self.buckets[new_cell].push(boid_index);
        self.boid_cells[boid_index] = new_cell;
    }

    fn dimension_for(axis_size: f32, cell_size: f32) -> usize {
        ((axis_size / cell_size).ceil() as usize).max(1)
    }

    fn cell_for(&self, position: Vec3) -> usize {
        self.flatten(self.coordinates_for(position))
    }

    fn coordinates_for(&self, position: Vec3) -> [usize; 3] {
        [
            self.coordinate_for_axis(position.x, self.origin.x, self.dimensions[0]),
            self.coordinate_for_axis(position.y, self.origin.y, self.dimensions[1]),
            self.coordinate_for_axis(position.z, self.origin.z, self.dimensions[2]),
        ]
    }

    fn coordinate_for_axis(&self, value: f32, origin: f32, dimension: usize) -> usize {
        let offset = ((value - origin) / self.cell_size).floor();
        (offset as usize).min(dimension - 1)
    }

    fn flatten(&self, [x, y, z]: [usize; 3]) -> usize {
        x + self.dimensions[0] * (y + self.dimensions[1] * z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(min: Vec3, max: Vec3) -> Bounds {
        Bounds::try_new(min, max).expect("test bounds should be valid")
    }

    fn boid(id: u32, position: Vec3) -> Boid {
        Boid::new(id, position, Vec3::ZERO)
    }

    #[test]
    fn test_new_calculates_three_dimensional_bucket_count() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(25.0, 35.0, 45.0));
        let grid = SpatialGrid::new(&bounds, 10.0, 5);

        assert_eq!(grid.dimensions, [3, 4, 5]);
        assert_eq!(grid.buckets.len(), 60);
        assert_eq!(grid.boid_cells.len(), 5);
    }

    #[test]
    fn test_flatten_uses_x_major_order() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(30.0, 30.0, 30.0));
        let grid = SpatialGrid::new(&bounds, 10.0, 0);

        assert_eq!(grid.flatten([0, 0, 0]), 0);
        assert_eq!(grid.flatten([1, 0, 0]), 1);
        assert_eq!(grid.flatten([0, 1, 0]), 3);
        assert_eq!(grid.flatten([0, 0, 1]), 9);
        assert_eq!(grid.flatten([2, 2, 2]), 26);
    }

    #[test]
    fn test_cell_coordinates_are_relative_to_bounds_and_clamped() {
        let bounds = bounds(Vec3::new(10.0, 20.0, 30.0), Vec3::new(40.0, 50.0, 60.0));
        let grid = SpatialGrid::new(&bounds, 10.0, 0);

        assert_eq!(grid.coordinates_for(Vec3::new(10.0, 20.0, 30.0)), [0, 0, 0]);
        assert_eq!(grid.coordinates_for(Vec3::new(29.9, 39.9, 49.9)), [1, 1, 1]);
        assert_eq!(grid.coordinates_for(Vec3::new(40.0, 50.0, 60.0)), [2, 2, 2]);
        assert_eq!(grid.coordinates_for(Vec3::new(-5.0, 80.0, 45.0)), [0, 2, 1]);
    }

    #[test]
    fn test_rebuild_assigns_boids_and_resizes_reverse_lookup() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(30.0, 30.0, 30.0));
        let mut grid = SpatialGrid::new(&bounds, 10.0, 0);
        let boids = vec![
            boid(0, Vec3::new(1.0, 1.0, 1.0)),
            boid(1, Vec3::new(15.0, 1.0, 1.0)),
            boid(2, Vec3::new(19.0, 1.0, 1.0)),
        ];

        grid.rebuild(&boids);

        assert_eq!(grid.buckets[grid.flatten([0, 0, 0])], vec![0]);
        assert_eq!(grid.buckets[grid.flatten([1, 0, 0])], vec![1, 2]);
        assert_eq!(grid.boid_cells.len(), boids.len());
    }

    #[test]
    fn test_collect_candidates_searches_surrounding_cells_in_index_order() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(40.0, 40.0, 40.0));
        let mut grid = SpatialGrid::new(&bounds, 10.0, 4);
        let boids = vec![
            boid(0, Vec3::new(15.0, 15.0, 15.0)),
            boid(1, Vec3::new(25.0, 25.0, 25.0)),
            boid(2, Vec3::new(35.0, 15.0, 15.0)),
            boid(3, Vec3::new(5.0, 5.0, 5.0)),
        ];
        grid.rebuild(&boids);

        let mut candidates = vec![usize::MAX];
        grid.collect_candidates(boids[0].position, 10.0, &mut candidates);

        assert_eq!(candidates, vec![0, 1, 3]);
    }

    #[test]
    fn test_collect_candidates_can_search_more_than_one_cell() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(50.0, 10.0, 10.0));
        let mut grid = SpatialGrid::new(&bounds, 10.0, 2);
        let boids = vec![
            boid(0, Vec3::new(5.0, 5.0, 5.0)),
            boid(1, Vec3::new(25.0, 5.0, 5.0)),
        ];
        grid.rebuild(&boids);

        let mut candidates = Vec::new();
        grid.collect_candidates(boids[0].position, 20.0, &mut candidates);

        assert_eq!(candidates, vec![0, 1]);
    }

    #[test]
    fn test_update_boid_moves_membership_between_buckets() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(30.0, 10.0, 10.0));
        let mut grid = SpatialGrid::new(&bounds, 10.0, 2);
        let boids = vec![
            boid(0, Vec3::new(5.0, 5.0, 5.0)),
            boid(1, Vec3::new(15.0, 5.0, 5.0)),
        ];
        grid.rebuild(&boids);

        grid.update_boid(0, Vec3::new(25.0, 5.0, 5.0));

        assert!(grid.buckets[grid.flatten([0, 0, 0])].is_empty());
        assert_eq!(grid.buckets[grid.flatten([2, 0, 0])], vec![0]);
        assert_eq!(grid.boid_cells[0], grid.flatten([2, 0, 0]));
    }

    #[test]
    fn test_update_boid_within_same_cell_keeps_membership() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(20.0, 10.0, 10.0));
        let mut grid = SpatialGrid::new(&bounds, 10.0, 1);
        let boids = vec![boid(0, Vec3::new(1.0, 1.0, 1.0))];
        grid.rebuild(&boids);

        grid.update_boid(0, Vec3::new(9.0, 9.0, 9.0));

        assert_eq!(grid.buckets[grid.flatten([0, 0, 0])], vec![0]);
        assert_eq!(grid.boid_cells[0], grid.flatten([0, 0, 0]));
    }

    #[test]
    #[should_panic(expected = "cell size must be finite and positive")]
    fn test_new_rejects_zero_cell_size() {
        let bounds = bounds(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        let _grid = SpatialGrid::new(&bounds, 0.0, 0);
    }
}
