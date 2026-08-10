use crate::boid::Boid;
use crate::bounds::{BoundaryMode, Bounds};
use crate::params::SimulationParams;
use crate::spawn::{SpawnConfig, spawn_boids};
use crate::steering::acceleration_for;

pub struct WorldSettings {
    params: SimulationParams,
    bounds: Bounds,
    boundary_mode: BoundaryMode,
}

pub struct World {
    boids: Vec<Boid>,
    settings: WorldSettings,
}

impl World {
    pub fn new(boids: Vec<Boid>, settings: WorldSettings) -> Self {
        Self {
            boids,
            settings
        }
    }

    pub fn from_config(spawn_config: SpawnConfig, settings: WorldSettings) -> Self {
        let boids = spawn_boids(&spawn_config, &settings.bounds);
        Self::new(boids, settings)
    }

    pub fn step(&mut self, dt: f32) {
        let mut accelerations = Vec::with_capacity(self.boids.len());

        // Step 1: Calculate the acceleration forces on each boid
        for index in 0..self.boids.len() {
            accelerations.push(acceleration_for(index, &self.boids, &self.settings.params));
        }

        let max_speed = self.settings.params.max_speed();

        // Step 2: update each boid
        for (boid, acceleration) in self.boids.iter_mut().zip(accelerations) {
            boid.velocity += acceleration * dt;
            boid.velocity = boid.velocity.limit_length(max_speed);
           
           boid.position += boid.velocity * dt;

            match &self.settings.boundary_mode {
                BoundaryMode::Wrap => {
                    self.settings.bounds.apply_wrap(&mut boid.position);
                }
                BoundaryMode::Bounce => {
                    self.settings.bounds.apply_bounce(&mut boid.position, &mut boid.velocity);
                }
            }

            // TODO: Remove once we implment z axis
            boid.position.z = 0.0;
            boid.velocity.z = 0.0;
        }
    }

    pub fn boids(&self) -> &[Boid] {
        &self.boids
    }
}