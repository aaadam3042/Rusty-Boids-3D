use crate::math::Vec3;

pub type BoidId = u32;

#[derive(Debug, PartialEq)]
pub struct Boid {
    pub id: BoidId,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Boid {
    pub(crate) const fn new(id: BoidId, position: Vec3, velocity: Vec3) -> Self {
        Self {
            id,
            position,
            velocity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_boid() {
        let boid = Boid::new(1, Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(boid.id, 1);
        assert_eq!(boid.position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(boid.velocity, Vec3::new(1.0, 1.0, 1.0));
    }
}
