use crate::boid::Boid;
use crate::bounds::{BoundaryMode, Bounds};
use crate::params::SimulationParams;
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
            let acceleration = acceleration_for(index, &self.boids, &self.neighbour_candidates, &self.settings.params);
            
            {
                let boid = &mut self.boids[index];
                boid.velocity += acceleration * dt;
                boid.velocity = boid.velocity.limit_length(max_speed);

                // Soft turn is applied before updating steering
                if let BoundaryMode::SoftTurn { 
                    margin,
                    turn_acceleration 
                } = &self.settings.boundary_mode {
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
                        self.settings.bounds.apply_bounce(&mut boid.position, &mut boid.velocity);
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
}