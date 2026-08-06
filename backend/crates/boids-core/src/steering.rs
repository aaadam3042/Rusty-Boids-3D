use std::fmt::Alignment;

use crate::boid::Boid;
use crate::math::Vec3;
use crate::params::SimulationParams;

pub(crate) fn acceleration_for(index: usize, boids: &[Boid], params: &SimulationParams) -> Vec3 {
    let alignment = alignment_for(index, boids, params.perception_radius()) * params.alignment_weight();
    let cohesion = cohesion_for(index, boids, params.perception_radius()) * params.cohesion_weight();
    let separation = separation_for(index, boids, params.separation_radius()) * params.separation_weight();

    (alignment + cohesion + separation).limit_length(params.max_acceleration())
}

fn alignment_for(index: usize, boids: &[Boid], perception_radius: f32) -> Vec3 {
    let current_boid = &boids[index];
    let perception_radius_sq = perception_radius * perception_radius;

    let mut average_vec = Vec3::new(0.0, 0.0, 0.0);
    let mut num_neighbours = 0;

    for (other_index, other_boid) in boids.iter().enumerate() {
        if other_index == index {
            continue;
        }

        if current_boid.position.distance_squared(other_boid.position) < perception_radius_sq {
            average_vec.x += other_boid.velocity.x;
            average_vec.y += other_boid.velocity.y;
            // average_vec += other_boid.velocity; TODO: Fix when 3d
            num_neighbours += 1;
        }
    }

    if num_neighbours <= 0 {
        return Vec3::ZERO;
    }

    average_vec /= num_neighbours as f32;

    average_vec.x -= current_boid.velocity.x;
    average_vec.y -= current_boid.velocity.y;
    average_vec
    // average_vec - current_boid.velocity TODO: Fix when 3D
}

fn cohesion_for(index: usize, boids: &[Boid], perception_radius: f32) -> Vec3 {
    let current_boid = &boids[index];
    let perception_radius_sq = perception_radius * perception_radius;

    let mut center = Vec3::new(0.0, 0.0, 0.0);
    let mut num_neighbours = 0;

    for (other_index, other_boid) in boids.iter().enumerate() {
        if other_index == index {
            continue;
        }

        if current_boid.position.distance_squared(other_boid.position) < perception_radius_sq {
            center.x += other_boid.position.x;
            center.y += other_boid.position.y;
            // center += other_boid.position TODO: Unlock when implementing 3D - This does the above + z axis
            num_neighbours += 1;
        }
    }

    if num_neighbours <= 0 {
        return Vec3::ZERO;
    }
  
    center /= num_neighbours as f32;
    
    // Return the displacement vector, ie direction and distance
    // This means that the further we are the stronger the pull
    center.x -= current_boid.position.x;
    center.y -= current_boid.position.y;
    center
    // center - current_boid.position TODO: Unlock when implementing 3D
}

fn separation_for(index: usize, boids: &[Boid], separation_radius: f32) -> Vec3 {
    let current_boid = &boids[index];
    let separation_radius_sq = separation_radius * separation_radius;

    let mut separation_vec = Vec3::new(0.0, 0.0, 0.0);

    for (other_index, other_boid) in boids.iter().enumerate() {
        if other_index == index {
            continue;
        }

        if current_boid.position.distance_squared(other_boid.position) < separation_radius_sq {
            separation_vec += current_boid.position - other_boid.position; 
            separation_vec.z = 0.0; // TODO: Remove when 3D
        }
    }

    separation_vec
}

#[cfg(test)] 
mod tests {
    use super::*;

    #[test]
    fn test_() {
        // TODO: Implement later
    }
}