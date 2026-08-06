use crate::boid::{Boid, BoidId};
use crate::bounds::Bounds;
use crate::math::Vec3;

use rand::{RngExt, SeedableRng};
use rand::rngs::ChaCha8Rng;
use std::f32::consts::TAU;

const MIN_INITIAL_SPEED: f32 = 1.0;

pub struct SpawnConfig {
    count: usize,
    seed: u64,
    initial_speed: f32,
}

impl SpawnConfig {
    pub const fn new(count: u32, seed: u64, initial_speed: f32) -> Self {
        // Count is limited to u32 to match boidID range
        // Also force initial speed to be clamped to a minimum of 1
        Self {
            count: count as usize,
            seed,
            initial_speed: initial_speed.max(MIN_INITIAL_SPEED),
        }
    }
}

pub fn spawn_boids(config: &SpawnConfig, bounds: &Bounds) -> Vec<Boid> {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);

    let min = bounds.min();
    let max = bounds.max();

    let mut boids = Vec::with_capacity(config.count);

    for index in 0..config.count {
        let position = Vec3::new(
            rng.random_range(min.x..max.x),
            rng.random_range(min.y..max.y),
            min.z, // TODO: Unlock z when implementing full 3D features
        );

        let angle = rng.random_range(0.0..TAU);
        let (sin, cos) = angle.sin_cos();

        let velocity = Vec3::new(
            cos * config.initial_speed,
            sin * config.initial_speed,
            0.0, // TODO: Unlock z when implementing full 3D features
        );

        boids.push(Boid::new(index as BoidId, position, velocity));
    }
    
    boids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_boids_with_default_bounds(config: &SpawnConfig) -> Vec<Boid> {
        let bounds = Bounds::try_new(
            Vec3::new(0.0, 0.0, 0.0), 
            Vec3::new(100.0, 100.0, 100.0)
        ).expect("Expected bounds to construct successfully.");

        spawn_boids(config, &bounds)
    }

    #[test]
    fn test_new_min_initial_speed() {
        let config = SpawnConfig::new(100, 123, MIN_INITIAL_SPEED);
        assert_eq!(config.count, 100);
        assert_eq!(config.seed, 123);
        assert_eq!(config.initial_speed, MIN_INITIAL_SPEED);
    }

    #[test]
    fn test_new_initial_speed_above_min() {
        let config = SpawnConfig::new(200, 321, MIN_INITIAL_SPEED + 1.0);
        assert_eq!(config.count, 200);
        assert_eq!(config.seed, 321);
        assert_eq!(config.initial_speed, MIN_INITIAL_SPEED + 1.0);
    }

    #[test]
    fn test_new_initial_speed_below_min() {
        let config = SpawnConfig::new(10, 42, MIN_INITIAL_SPEED - 1.0);
        assert_eq!(config.initial_speed, MIN_INITIAL_SPEED);
    }

    #[test]
    fn test_spawn_boids_successful_count() {
        let config = SpawnConfig::new(100, 123, 5.0);
        let boids = spawn_boids_with_default_bounds(&config);

        assert_eq!(boids.len(), 100)
    }

    #[test]
    fn test_spawn_boids_zero_count_empty() {
        let config = SpawnConfig::new(0, 123, 5.0);
        let boids = spawn_boids_with_default_bounds(&config);

        assert!(boids.is_empty());
    }

    #[test]
    fn test_spawn_boids_assigns_sequential_ids() {
        let config = SpawnConfig::new(10, 123, 5.0);
        let boids = spawn_boids_with_default_bounds(&config);

        for (i, boid) in boids.iter().enumerate() {
            assert_eq!(boid.id, i as u32);
        }
    }

    #[test]
    fn test_spawn_boids_within_bounds() {
        let config = SpawnConfig::new(20, 123, 5.0);
        let bounds = Bounds::try_new(
            Vec3::new(100.0, 100.0, 100.0), 
            Vec3::new(200.0, 200.0, 200.0)
        ).expect("Expected bounds to construct successfully.");
        let boids = spawn_boids(&config, &bounds);

        for boid in boids {
            assert!(bounds.contains(boid.position));
        }
    }

    #[test]
    fn test_spawn_boids_have_initial_speed() {
        let config = SpawnConfig::new(20, 123, 5.0);
        let boids = spawn_boids_with_default_bounds(&config);

        for boid in boids {
            let speed = boid.velocity.length();
            assert!((speed - config.initial_speed).abs() < f32::EPSILON)
        }
    }

    #[test]
    fn test_spawn_boids_is_deterministic() {
        let config = SpawnConfig::new(20, 123, 5.0);
        let boids1 = spawn_boids_with_default_bounds(&config);
        let boids2 = spawn_boids_with_default_bounds(&config);

        assert_eq!(boids1, boids2)
    }

    #[test]
    fn test_spawn_boids_differs_for_different_seeds() {
        let config1 = SpawnConfig::new(20, 123, 5.0);
        let config2 = SpawnConfig::new(20, 456, 5.0);
        let boids1 = spawn_boids_with_default_bounds(&config1);
        let boids2 = spawn_boids_with_default_bounds(&config2);

        let positions_differ = boids1.iter().zip(boids2.iter()).any(|(b1, b2)| b1.position != b2.position);
        assert!(positions_differ);
    }
}