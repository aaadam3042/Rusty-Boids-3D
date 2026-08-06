use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn limit_length(self, max_length: f32) -> Self {
        debug_assert!(max_length.is_finite() && max_length >= 0.0);

        let length_sq = self.length_squared();
        let max_length_sq = max_length * max_length;
        if length_sq > max_length_sq {
            self * (max_length / length_sq.sqrt())
        } else {
            self
        }
    }

    pub fn normalise_or_zero(self) -> Self {
        let length = self.length();
        if length > 0.0 && length.is_finite() {
            self / length
        } else {
            Self::ZERO
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() <= f32::EPSILON);
        assert!((actual.y - expected.y).abs() <= f32::EPSILON);
        assert!((actual.z - expected.z).abs() <= f32::EPSILON);
    }

    #[test]
    fn test_zero_vector() {
        let v = Vec3::ZERO;
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn test_partial_eq() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        let v3 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_addition() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;
        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_addition_assign() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        v1 += v2;
        assert_eq!(v1, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_addition_with_zero_vector() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::ZERO;
        let result = v1 + v2;
        assert_eq!(result, v1);
    }

    #[test]
    fn test_addition_with_negative_vector() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(-1.0, -2.0, -3.0);
        let result = v1 + v2;
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_subtraction() {
        let v1 = Vec3::new(4.0, 5.0, 6.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        let result = v1 - v2;
        assert_eq!(result, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_subtraction_assign() {
        let mut v1 = Vec3::new(4.0, 5.0, 6.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        v1 -= v2;
        assert_eq!(v1, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_subtraction_with_zero_vector() {
        let v1 = Vec3::new(4.0, 5.0, 6.0);
        let v2 = Vec3::ZERO;
        let result = v1 - v2;
        assert_eq!(result, v1);
    }

    #[test]
    fn test_subtraction_with_negative_vector() {
        let v1 = Vec3::new(4.0, 5.0, 6.0);
        let v2 = Vec3::new(-1.0, -2.0, -3.0);
        let result = v1 - v2;
        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        let result = v * scalar;
        assert_eq!(result, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_scalar_multiplication_assign() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        v *= scalar;
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_scalar_multiplication_by_zero() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 0.0;
        let result = v * scalar;
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_scalar_multiplication_by_non_finite() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = f32::NAN;
        let result = v * scalar;
        assert!(result.x.is_nan());
        assert!(result.y.is_nan());
        assert!(result.z.is_nan());
    }

    #[test]
    fn test_scalar_multiplication_by_negative() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = -2.0;
        let result = v * scalar;
        assert_eq!(result, Vec3::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn test_scalar_division() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = 2.0;
        let result = v / scalar;
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scalar_division_assign() {
        let mut v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = 2.0;
        v /= scalar;
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scalar_division_by_zero() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = 0.0;
        let result = v / scalar;
        assert!(result.x.is_infinite());
        assert!(result.y.is_infinite());
        assert!(result.z.is_infinite());
    }

    #[test]
    fn test_scalar_division_by_negative() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = -2.0;
        let result = v / scalar;
        assert_eq!(result, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_scalar_division_by_non_finite() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = f32::NAN;
        let result = v / scalar;
        assert!(result.x.is_nan());
        assert!(result.y.is_nan());
        assert!(result.z.is_nan());
    }

    #[test]
    fn test_negation() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        let result = -v;
        assert_eq!(result, Vec3::new(-1.0, 2.0, -3.0));
    }

    #[test]
    fn test_is_finite() {
        let v1 = Vec3::new(10.0, 10.0, 10.0);
        let v2 = Vec3::new(10.0, -10.0, 10.0);
        let v3 = Vec3::new(0.0, 0.0, 0.0);
        assert!(v1.is_finite());
        assert!(v2.is_finite());
        assert!(v3.is_finite());
    }

    #[test]
    fn test_is_infinite() {
        let v1 = Vec3::new(f32::NAN, 1.0, 1.0);
        let v2 = Vec3::new(1.0, f32::NAN, 1.0);
        let v3 = Vec3::new(1.0, 1.0, f32::NAN);
        let v4 = Vec3::new(f32::NAN, f32::NAN, f32::NAN);
        let v5 = Vec3::new(f32::INFINITY, 1.0, 1.0);
        let v6 = Vec3::new(1.0, f32::INFINITY, 1.0);
        let v7 = Vec3::new(1.0, 1.0, f32::INFINITY);
        let v8 = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        assert!(!v1.is_finite());
        assert!(!v2.is_finite());
        assert!(!v3.is_finite());
        assert!(!v4.is_finite());
        assert!(!v5.is_finite());
        assert!(!v6.is_finite());
        assert!(!v7.is_finite());
        assert!(!v8.is_finite());
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, -5.0, 6.0);
        let result = v1.dot(v2);
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_length_squared() {
        let v = Vec3::new(1.0, 2.0, 2.0);
        let result = v.length_squared();
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_length() {
        let v = Vec3::new(1.0, 2.0, 2.0);
        let result = v.length();
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_normalise_or_zero() {
        let v = Vec3::new(3.0, 0.0, 4.0);
        let result = v.normalise_or_zero();
        assert_eq!(result, Vec3::new(0.6, 0.0, 0.8));
    }

    #[test]
    fn test_normalise_or_zero_zero_vector() {
        let v = Vec3::ZERO;
        let result = v.normalise_or_zero();
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_normalise_or_zero_non_finite_vector() {
        let v = Vec3::new(f32::NAN, 0.0, 0.0);
        let result = v.normalise_or_zero();
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_distance() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 6.0, 3.0);
        let result = v1.distance(v2);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_distance_squared() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 6.0, 3.0);
        let result = v1.distance_squared(v2);
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_limit_length_exceeds_limit() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let result = v.limit_length(3.0);
        assert_eq!(result.length(), 3.0);
        assert_vec3_approx_eq(v.normalise_or_zero(), result.normalise_or_zero());
    }

    #[test]
    fn test_limit_length_within_limit() {
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = v.limit_length(5.0);
        assert_eq!(result, v);
    }

    #[test]
    fn test_limit_length_at_limit() {
        let v = Vec3::new(0.0, 3.0, 0.0);
        let result = v.limit_length(3.0);
        assert_eq!(result, v);
    }

    #[test]
    fn test_limit_length_zero_vector() {
        let v = Vec3::ZERO;
        let result = v.limit_length(5.0);
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_limit_length_negative_components() {
        let v = Vec3::new(-3.0, -4.0, 0.0);
        let result = v.limit_length(3.0);
        assert_eq!(result.length(), 3.0);
        assert_vec3_approx_eq(v.normalise_or_zero(), result.normalise_or_zero());
    }

    #[test]
    fn test_limit_length_small_max_length() {
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = v.limit_length(0.5);
        assert!(result.length() <= 0.5);
    }

    #[test]
    fn test_limit_length_zero_max_length() {
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = v.limit_length(0.0);
        assert_eq!(result.length(), 0.0);
    }
}
