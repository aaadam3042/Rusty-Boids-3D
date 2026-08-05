use crate::boid::Boid;
use crate::bounds::{BoundaryMode, Bounds};
use crate::params::SimulationParams;

struct World {
    boids: Vec<Boid>,
    params: SimulationParams,
    bounds: Bounds,
    bounds_mode: BoundaryMode,
    elapsed: f32,
}

impl World {
    pub fn new() -> Self {

    }

    pub fn from_config(config: Config) {

    }

    pub fn seeded() -> Self {

    }

    pub fn step(&mut self) {

    }

    pub fn boids(&self) -> &[Boid] {
        &self.boids
    }

    pub fn params(&self) -> &SimulationParams {
        &self.params
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
}