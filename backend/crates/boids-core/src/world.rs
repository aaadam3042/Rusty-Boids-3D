use crate::boid::Boid;
use crate::bounds::{BoundaryMode, Bounds};
use crate::params::{ParamsError, SimulationParams};
use crate::spatial_grid::SpatialGrid;
use crate::spawn::{SpawnConfig, spawn_boids};
use crate::steering::acceleration_for;

pub struct WorldSettings {
    pub params: SimulationParams,
    pub bounds: Bounds,
    pub boundary_mode: BoundaryMode,
}

pub struct World {
    boids: Vec<Boid>,
    settings: WorldSettings,
    spatial_grid: SpatialGrid,
    neighbour_candidates: Vec<usize>,
}

impl World {
    pub fn new(boids: Vec<Boid>, settings: WorldSettings) -> Self {
        let mut spatial_grid = SpatialGrid::new(
            &settings.bounds,
            settings.params.perception_radius(),
            boids.len(),
        );

        spatial_grid.rebuild(&boids);

        Self {
            boids,
            settings,
            spatial_grid,
            neighbour_candidates: Vec::new(),
        }
    }

    pub fn from_config(spawn_config: SpawnConfig, settings: WorldSettings) -> Self {
        let boids = spawn_boids(&spawn_config, &settings.bounds);
        Self::new(boids, settings)
    }

    pub fn step(&mut self, dt: f32) {
        let perception_radius = self.settings.params.perception_radius();
        let max_speed = self.settings.params.max_speed();
        for index in 0..self.boids.len() {
            let position = self.boids[index].position;

            self.spatial_grid.collect_candidates(
                position,
                perception_radius,
                &mut self.neighbour_candidates,
            );

            // Adding steering to boids in place is important to maintain movement
            let acceleration = acceleration_for(
                index,
                &self.boids,
                &self.neighbour_candidates,
                &self.settings.params,
            );

            {
                let boid = &mut self.boids[index];
                boid.velocity += acceleration * dt;
                boid.velocity = boid.velocity.limit_length(max_speed);

                // Soft turn is applied before updating steering
                if let BoundaryMode::SoftTurn {
                    margin,
                    turn_acceleration,
                } = &self.settings.boundary_mode
                {
                    self.settings.bounds.apply_soft_turn(
                        boid.position,
                        &mut boid.velocity,
                        *margin,
                        *turn_acceleration,
                        dt,
                    );
                }

                boid.position += boid.velocity * dt;

                match &self.settings.boundary_mode {
                    BoundaryMode::Wrap => {
                        self.settings.bounds.apply_wrap(&mut boid.position);
                    }
                    BoundaryMode::Bounce => {
                        self.settings
                            .bounds
                            .apply_bounce(&mut boid.position, &mut boid.velocity);
                    }
                    BoundaryMode::SoftTurn { .. } => {}
                }
            }

            // Deal with the spatial grid
            let new_position = self.boids[index].position;
            self.spatial_grid.update_boid(index, new_position);
        }
    }

    pub fn boids(&self) -> &[Boid] {
        &self.boids
    }

    pub fn bounds(&self) -> &Bounds {
        &self.settings.bounds
    }

    pub fn params(&self) -> &SimulationParams {
        &self.settings.params
    }

    pub fn set_weights(
        &mut self,
        cohesion_weight: f32,
        alignment_weight: f32,
        separation_weight: f32,
    ) -> Result<(), ParamsError> {
        let updated_params = self.settings.params.try_with_weights(
            cohesion_weight,
            alignment_weight,
            separation_weight,
        )?;

        self.settings.params = updated_params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    fn empty_world() -> World {
        let bounds = Bounds::try_new(Vec3::ZERO, Vec3::new(100.0, 100.0, 100.0))
            .expect("expected valid test bounds");
        let settings = WorldSettings {
            params: SimulationParams::default(),
            bounds,
            boundary_mode: BoundaryMode::Bounce,
        };

        World::new(Vec::new(), settings)
    }

    #[test]
    fn set_weights_replaces_all_weights_atomically() {
        let mut world = empty_world();

        world
            .set_weights(4.0, 5.0, 6.0)
            .expect("expected valid replacement weights");

        assert_eq!(world.params().cohesion_weight(), 4.0);
        assert_eq!(world.params().alignment_weight(), 5.0);
        assert_eq!(world.params().separation_weight(), 6.0);
    }

    #[test]
    fn set_weights_preserves_previous_values_when_replacement_is_invalid() {
        let mut world = empty_world();
        let original = *world.params();

        let result = world.set_weights(4.0, -1.0, 6.0);

        assert_eq!(
            result,
            Err(ParamsError::MustBeNonNegative("alignment_weight"))
        );
        assert_eq!(*world.params(), original);
    }
}
