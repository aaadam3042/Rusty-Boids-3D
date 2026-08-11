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
            average_vec += other_boid.velocity;
            num_neighbours += 1;
        }
    }

    if num_neighbours <= 0 {
        return Vec3::ZERO;
    }

    average_vec /= num_neighbours as f32;

    average_vec - current_boid.velocity 
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
            center += other_boid.position;
            num_neighbours += 1;
        }
    }

    if num_neighbours <= 0 {
        return Vec3::ZERO;
    }
  
    center /= num_neighbours as f32;
    
    // Return the displacement vector, ie direction and distance
    // This means that the further we are the stronger the pull
    center - current_boid.position
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
        }
    }

    separation_vec
}
