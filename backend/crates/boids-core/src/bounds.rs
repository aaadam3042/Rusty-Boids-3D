use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsError {
    NonFinite,
    InvalidAxis,
}

pub enum BoundaryMode {
    Bounce,
    Wrap, 
    SoftTurn {
        margin: f32,
        turn_acceleration: f32,
    },
}

#[derive(Debug, PartialEq)]
pub struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    pub fn try_new(min: Vec3, max: Vec3) -> Result<Self, BoundsError> {
        if !min.is_finite() || !max.is_finite() {
            return Err(BoundsError::NonFinite);
        }

        if min.x >= max.x || min.y >= max.y || min.z >= max.z {
            return Err(BoundsError::InvalidAxis);
        }

        Ok(Self { min, max })
    }

    pub(crate) fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

// Getters
impl Bounds {
    pub const fn min(&self) -> Vec3 {
        self.min
    }

    pub const fn max(&self) -> Vec3 {
        self.max
    }
}

// Mutating Bounds functions
impl Bounds {
    pub(crate) fn apply_wrap(&self, position: &mut Vec3) {
        if position.x < self.min.x {
            position.x = self.max.x - (self.min.x - position.x) % self.size().x;
        } else if position.x > self.max.x {
            position.x = self.min.x + (position.x - self.max.x) % self.size().x;
        }

        if position.y < self.min.y {
            position.y = self.max.y - (self.min.y - position.y) % self.size().y;
        } else if position.y > self.max.y {
            position.y = self.min.y + (position.y - self.max.y) % self.size().y;
        }

        if position.z < self.min.z {
            position.z = self.max.z - (self.min.z - position.z) % self.size().z;
        } else if position.z > self.max.z {
            position.z = self.min.z + (position.z - self.max.z) % self.size().z;
        }
    }

    pub(crate) fn apply_bounce(&self, position: &mut Vec3, velocity: &mut Vec3) {
        if position.x <= self.min.x {
            position.x = self.min.x;
            if velocity.x < 0.0 { velocity.x = -velocity.x; }
        } else if position.x >= self.max.x {
            position.x = self.max.x;
            if velocity.x > 0.0 { velocity.x = -velocity.x; }
        }

        if position.y <= self.min.y {
            position.y = self.min.y;
            if velocity.y < 0.0 { velocity.y = -velocity.y };
        } else if position.y >= self.max.y {
            position.y = self.max.y;
            if velocity.y > 0.0 { velocity.y = -velocity.y };
        }

        if position.z <= self.min.z {
            position.z = self.min.z;
            if velocity.z < 0.0 { velocity.z = -velocity.z };
        } else if position.z >= self.max.z {
            position.z = self.max.z;
            if velocity.z > 0.0 { velocity.z = -velocity.z };
        }
    }

    pub(crate) fn apply_soft_turn(&self, position: Vec3, velocity: &mut Vec3, margin: f32, turn_acceleration: f32, dt:f32) {
        let velocity_change = turn_acceleration * dt;

        if position.x < self.min.x + margin {
            velocity.x += velocity_change;
        }

        if position.x > self.max.x - margin {
            velocity.x -= velocity_change;
        }

        if position.y < self.min.y + margin {
            velocity.y += velocity_change;
        }

        if position.y > self.max.y - margin {
            velocity.y -= velocity_change;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_test_bounds() -> Bounds {
        setup_bounds(
            [0.0, 0.0, 0.0],
            [500.0, 500.0, 500.0]
        ).expect("expected valid default test bounds")
    }

    fn setup_bounds(min: [f32; 3], max: [f32; 3]) -> Result<Bounds, BoundsError> {
        let min = Vec3::new(min[0], min[1], min[2]);
        let max = Vec3::new(max[0],max[1], max[2]);
        Bounds::try_new(min, max)
    }

    #[test]
    fn test_try_new_valid() {
        let bounds = setup_bounds(
            [10.0, 10.0, 10.0],
            [110.0, 110.0, 110.0]    
        );
        assert!(bounds.is_ok());

        let bounds = setup_bounds(
            [-10.0, -10.0, -10.0],
            [10.0, 10.0, 10.0]    
        );
        assert!(bounds.is_ok());
    }

    #[test]
    fn test_try_new_invalid_non_finite() {
        let bounds = setup_bounds(
            [f32::NAN, 10.0, 10.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::NonFinite));
    
        let bounds = setup_bounds(
            [10.0, 10.0, 10.0],
            [f32::NAN, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::NonFinite));

        let bounds = setup_bounds(
            [f32::INFINITY, 10.0, 10.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::NonFinite));
    
        let bounds = setup_bounds(
            [10.0, 10.0, 10.0],
            [f32::INFINITY, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::NonFinite));
    }

    #[test]
    fn test_try_new_invalid_zero_sized_axis() {
        let bounds = setup_bounds(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [110.0, 0.0, 0.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [0.0, 110.0, 0.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [0.0, 0.0, 110.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [110.0, 110.0, 110.0],
            [110.0, 110.0, 110.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));
    }

    #[test]
    fn test_try_new_invalid_reversed_axis() {
        let bounds = setup_bounds(
            [110.0, 110.0, 110.0],
            [10.0, 10.0, 10.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [110.0, 0.0, 0.0],
            [10.0, 10.0, 10.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [0.0, 110.0, 0.0],
            [10.0, 10.0, 10.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));

        let bounds = setup_bounds(
            [0.0, 0.0, 110.0],
            [10.0, 10.0, 10.0]
        );
        assert_eq!(bounds, Err(BoundsError::InvalidAxis));
    }

    #[test]
    fn test_size() {
        let bounds = default_test_bounds();
        assert_eq!(bounds.size(), Vec3 {x: 500.0, y: 500.0, z: 500.0});

        let bounds = setup_bounds(
            [10.0, 10.0, 10.0],
            [110.0, 110.0, 110.0]    
        ).expect("test bounds should be valid");
        assert_eq!(bounds.size(), Vec3 {x: 100.0, y: 100.0, z: 100.0});

        let bounds = setup_bounds(
            [-10.0, -10.0, -10.0],
            [10.0, 10.0, 10.0]    
        ).expect("test bounds should be valid");
        assert_eq!(bounds.size(), Vec3 {x: 20.0, y: 20.0, z: 20.0});
    }

    

    #[test]
    fn test_no_wrap_inside() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(250.0, 250.0, 250.0);

        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(250.0, 250.0, 250.0))
    }

    #[test]
    fn test_no_wrap_on_boundary() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(0.0, 0.0, 0.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(0.0, 0.0, 0.0));

        position = Vec3::new(500.0, 500.0, 500.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(500.0, 500.0, 500.0));

        position = Vec3::new(0.0, 500.0, 0.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(0.0, 500.0, 0.0));

        position = Vec3::new(500.0, 0.0, 500.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(500.0, 0.0, 500.0));
    }

    #[test]
    fn test_wrap_below_minimum() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, 250.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(490.0, 250.0, 250.0));

        // try other axes in the same test
        position = Vec3::new(250.0, -10.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(250.0, 490.0, 250.0));

        position = Vec3::new(250.0, 250.0, -20.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(250.0, 250.0, 480.0));
    }

    #[test]
    fn test_wrap_above_maximum() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(510.0, 250.0, 250.0);

        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(10.0, 250.0, 250.0));

        // try other axes in the same test
        position = Vec3::new(250.0, 510.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(250.0, 10.0, 250.0));

        position = Vec3::new(250.0, 250.0, 520.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(250.0, 250.0, 20.0));
    }

    #[test]
    fn test_wrap_preserve_unaffected_axes() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, 123.0, 321.0);

        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(490.0, 123.0, 321.0));

        // mutate and ensure unaffected axes preserved
        position = Vec3::new(123.0, -5.0, 321.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(123.0, 495.0, 321.0));

        position = Vec3::new(123.0, 321.0, 505.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(123.0, 321.0, 5.0));
    }

    #[test]
    fn test_wrap_multiple_axes() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, 510.0, -20.0);

        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(490.0, 10.0, 480.0));
    }

    #[test]
    fn test_wrap_multiple_bounds_widths() {
        let bounds = default_test_bounds();
        // Position is 520 units beyond max (520 = 500 + 20), should wrap with % 500
        let mut position = Vec3::new(1020.0, 250.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(20.0, 250.0, 250.0));

        let mut position = Vec3::new(-510.0, 250.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(490.0, 250.0, 250.0));
    }

    #[test]
    fn test_wrap_exact_bounds_width() {
        let bounds = default_test_bounds();
        // Position exactly at width boundary (500 units beyond max)
        let mut position = Vec3::new(1000.0, 250.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(0.0, 250.0, 250.0));

        // Position exactly at width boundary below min (500 units below)
        let mut position = Vec3::new(-500.0, 250.0, 250.0);
        bounds.apply_wrap(&mut position);
        assert_eq!(position, Vec3::new(500.0, 250.0, 250.0));
    }

    #[test]
    fn test_bounce_no_change_inside() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(250.0, 250.0, 250.0);
        let mut velocity = Vec3::new(1.0, -2.0, 3.0);

        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(250.0, 250.0, 250.0));
        assert_eq!(velocity, Vec3::new(1.0, -2.0, 3.0));
    }

    #[test]
    fn test_bounce_on_boundary() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(0.0, 0.0, 0.0);
        let mut velocity = Vec3::new(-5.0, -5.0, -5.0);

        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(velocity, Vec3::new(5.0, 5.0, 5.0));

        velocity = Vec3::new(0.0, 0.0, 0.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(velocity, Vec3::new(0.0, 0.0, 0.0));

        velocity = Vec3::new(5.0, 5.0, 5.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(velocity, Vec3::new(5.0, 5.0, 5.0));

        position = Vec3::new(500.0, 500.0, 500.0);
        velocity = Vec3::new(10.0, 10.0, 10.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(500.0, 500.0, 500.0));
        assert_eq!(velocity, Vec3::new(-10.0, -10.0, -10.0));

        velocity = Vec3::new(0.0, 0.0, 0.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(500.0, 500.0, 500.0));
        assert_eq!(velocity, Vec3::new(0.0, 0.0, 0.0));

        velocity = Vec3::new(-10.0, -10.0, -10.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(500.0, 500.0, 500.0));
        assert_eq!(velocity, Vec3::new(-10.0, -10.0, -10.0));
    }

    #[test]
    fn test_bounce_below_minimum() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, 250.0, 250.0);
        let mut velocity = Vec3::new(-3.0, 2.0, -1.0);

        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 250.0, 250.0));
        assert_eq!(velocity, Vec3::new(3.0, 2.0, -1.0));

        // other axes
        position = Vec3::new(250.0, -5.0, 250.0);
        velocity = Vec3::new(1.0, -4.0, 2.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(250.0, 0.0, 250.0));
        assert_eq!(velocity, Vec3::new(1.0, 4.0, 2.0));

        position = Vec3::new(250.0, 250.0, -20.0);
        velocity = Vec3::new(1.0, 2.0, -6.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(250.0, 250.0, 0.0));
        assert_eq!(velocity, Vec3::new(1.0, 2.0, 6.0));
    }

    #[test]
    fn test_bounce_above_maximum() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(510.0, 250.0, 250.0);
        let mut velocity = Vec3::new(3.0, -2.0, 1.0);

        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(500.0, 250.0, 250.0));
        assert_eq!(velocity, Vec3::new(-3.0, -2.0, 1.0));

        // other axes
        position = Vec3::new(250.0, 510.0, 250.0);
        velocity = Vec3::new(-1.0, 4.0, -2.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(250.0, 500.0, 250.0));
        assert_eq!(velocity, Vec3::new(-1.0, -4.0, -2.0));

        position = Vec3::new(250.0, 250.0, 520.0);
        velocity = Vec3::new(-1.0, 2.0, 6.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(250.0, 250.0, 500.0));
        assert_eq!(velocity, Vec3::new(-1.0, 2.0, -6.0));
    }

    #[test]
    fn test_bounce_multiple_axes() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, 510.0, -20.0);
        let mut velocity = Vec3::new(-3.0, 4.0, -5.0);

        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 500.0, 0.0));
        assert_eq!(velocity, Vec3::new(3.0, -4.0, 5.0));
    }

    #[test]
    fn test_bounce_outside_while_moving_inwards() {
        let bounds = default_test_bounds();
        let mut position = Vec3::new(-10.0, -10.0, -10.0);
        let mut velocity = Vec3::new(10.0, 10.0, 10.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(velocity, Vec3::new(10.0, 10.0, 10.0));

        position = Vec3::new(510.0, 510.0, 510.0);
        velocity = Vec3::new(-10.0, -10.0, -10.0);
        bounds.apply_bounce(&mut position, &mut velocity);
        assert_eq!(position, Vec3::new(500.0, 500.0, 500.0));
        assert_eq!(velocity, Vec3::new(-10.0, -10.0, -10.0));

    }

    // #[test]
    // fn test_contains_inside() {
    //     let bounds = default_test_bounds();
    //     assert!(bounds.contains(Vec3::new(250.0, 250.0, 250.0)));
    // }

    // #[test]
    // fn test_contains_on_boundaries() {
    //     let bounds = default_test_bounds();
    //     assert!(bounds.contains(Vec3::new(0.0, 0.0, 0.0)));
    //     assert!(bounds.contains(Vec3::new(500.0, 500.0, 500.0)));
    //     assert!(bounds.contains(Vec3::new(0.0, 500.0, 0.0)));
    //     assert!(bounds.contains(Vec3::new(500.0, 0.0, 500.0)));
    // }

    // #[test]
    // fn test_contains_outside() {
    //     let bounds = default_test_bounds();
    //     assert!(!bounds.contains(Vec3::new(-1.0, 250.0, 250.0)));
    //     assert!(!bounds.contains(Vec3::new(250.0, -1.0, 250.0)));
    //     assert!(!bounds.contains(Vec3::new(250.0, 250.0, -1.0)));
    //     assert!(!bounds.contains(Vec3::new(501.0, 250.0, 250.0)));
    //     assert!(!bounds.contains(Vec3::new(250.0, 501.0, 250.0)));
    //     assert!(!bounds.contains(Vec3::new(250.0, 250.0, 510.0)));
    // }
}
