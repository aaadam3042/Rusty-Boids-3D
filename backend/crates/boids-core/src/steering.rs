use crate::boid::Boid;
use crate::math::Vec3;
use crate::params::SimulationParams;

pub(crate) fn acceleration_for(index: usize, boids: &[Boid], params: &SimulationParams) -> Vec3 {
    let mut alignment = Vec3::ZERO;
    let mut cohesion = Vec3::ZERO;

    let current_boid = &boids[index];
    let perception_radius_sq = params.perception_radius() * params.perception_radius();
    let separation_radius_sq = params.separation_radius() * params.separation_radius();

    
    let mut num_neighbours = 0;
    let mut alignment_vec = Vec3::new(0.0, 0.0, 0.0); 
    let mut cohesion_center = Vec3::new(0.0, 0.0, 0.0); 
    let mut separation_vec = Vec3::new(0.0, 0.0, 0.0);

    for (other_index, other_boid) in boids.iter().enumerate() {
        if other_index == index {
            continue;
        }

        let distance_sq = current_boid.position.distance_squared(other_boid.position);

        if distance_sq < perception_radius_sq {
            alignment_vec += other_boid.velocity;
            cohesion_center += other_boid.position;

            num_neighbours += 1;
        }

        if distance_sq < separation_radius_sq {
            separation_vec += current_boid.position - other_boid.position; 
        }
    }

    if num_neighbours > 0 {
        alignment_vec /= num_neighbours as f32;
        cohesion_center /= num_neighbours as f32;

        alignment = alignment_vec - current_boid.velocity;
        cohesion = cohesion_center - current_boid.position;
    }
    alignment *= params.alignment_weight();
    cohesion *= params.cohesion_weight();
    let separation = separation_vec * params.separation_weight();

    (alignment + cohesion + separation).limit_length(params.max_acceleration())
}
