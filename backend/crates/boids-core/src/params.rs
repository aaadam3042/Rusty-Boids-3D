#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamsError {
    NonFinite(&'static str),
    MustBePositive(&'static str),
    MustBeNonNegative(&'static str),
    InvalidRadiusRelationship,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationParams {
    perception_radius: f32,
    separation_radius: f32,

    cohesion_weight: f32,
    alignment_weight: f32,
    separation_weight: f32,

    max_speed: f32,
    max_acceleration: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            perception_radius: 15.0,
            separation_radius: 2.0,

            cohesion_weight: 18.0,
            alignment_weight: 3.0,
            separation_weight: 180.0,

            max_speed: 200.0,
            max_acceleration: 100.0,
        }
    }
}

// Constructors
impl SimulationParams {
    pub fn try_new(
        perception_radius: f32,
        separation_radius: f32,
        cohesion_weight: f32,
        alignment_weight: f32,
        separation_weight: f32,
        max_speed: f32,
        max_acceleration: f32,
    ) -> Result<Self, ParamsError> {
        let params = Self {
            perception_radius,
            separation_radius,

            cohesion_weight,
            alignment_weight,
            separation_weight,

            max_speed,
            max_acceleration,
        };

        params.validate()?;
        Ok(params)
    }
}

// Getters
impl SimulationParams {
    pub const fn perception_radius(&self) -> f32 {
        self.perception_radius
    }

    pub const fn separation_radius(&self) -> f32 {
        self.separation_radius
    }

    pub const fn cohesion_weight(&self) -> f32 {
        self.cohesion_weight
    }

    pub const fn alignment_weight(&self) -> f32 {
        self.alignment_weight
    }

    pub const fn separation_weight(&self) -> f32 {
        self.separation_weight
    }

    pub const fn max_speed(&self) -> f32 {
        self.max_speed
    }

    pub const fn max_acceleration(&self) -> f32 {
        self.max_acceleration
    }
}

// Private helpers
impl SimulationParams {
    fn validate(&self) -> Result<(), ParamsError> {
        if !self.perception_radius.is_finite() {
            return Err(ParamsError::NonFinite("perception_radius"));
        }
        if !self.separation_radius.is_finite() {
            return Err(ParamsError::NonFinite("separation_radius"));
        }
        if !self.cohesion_weight.is_finite() {
            return Err(ParamsError::NonFinite("cohesion_weight"));
        }
        if !self.alignment_weight.is_finite() {
            return Err(ParamsError::NonFinite("alignment_weight"));
        }
        if !self.separation_weight.is_finite() {
            return Err(ParamsError::NonFinite("separation_weight"));
        }
        if !self.max_speed.is_finite() {
            return Err(ParamsError::NonFinite("max_speed"));
        }
        if !self.max_acceleration.is_finite() {
            return Err(ParamsError::NonFinite("max_acceleration"));
        }

        if self.perception_radius <= 0.0 {
            return Err(ParamsError::MustBePositive("perception_radius"));
        }
        if self.separation_radius <= 0.0 {
            return Err(ParamsError::MustBePositive("separation_radius"));
        }
        if self.cohesion_weight < 0.0 {
            return Err(ParamsError::MustBeNonNegative("cohesion_weight"));
        }
        if self.alignment_weight < 0.0 {
            return Err(ParamsError::MustBeNonNegative("alignment_weight"));
        }
        if self.separation_weight < 0.0 {
            return Err(ParamsError::MustBeNonNegative("separation_weight"));
        }
        if self.max_speed <= 0.0 {
            return Err(ParamsError::MustBePositive("max_speed"));
        }
        if self.max_acceleration <= 0.0 {
            return Err(ParamsError::MustBePositive("max_acceleration"));
        }

        if self.separation_radius >= self.perception_radius {
            return Err(ParamsError::InvalidRadiusRelationship);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct(params: SimulationParams) -> Result<SimulationParams, ParamsError> {
        SimulationParams::try_new(
            params.perception_radius,
            params.separation_radius,
            params.cohesion_weight,
            params.alignment_weight,
            params.separation_weight,
            params.max_speed,
            params.max_acceleration,
        )
    }

    fn assert_invalid(params: SimulationParams, expected_error: ParamsError) {
        assert_eq!(construct(params), Err(expected_error));
    }

    #[test]
    fn test_default_params() {
        let params = SimulationParams::default();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_valid_params() {
        let params = SimulationParams::try_new(20.0, 5.0, 1.0, 1.0, 1.0, 10.0, 5.0);
        assert!(params.is_ok());
    }

    #[test]
    fn test_valid_zero_weights() {
        let params = SimulationParams::try_new(20.0, 5.0, 0.0, 0.0, 0.0, 10.0, 5.0);
        assert!(params.is_ok());
    }

    #[test]
    fn test_invalid_params_radii() {
        let mut params = SimulationParams::default();
        params.separation_radius = params.perception_radius; // Invalid: separation_radius == perception_radius
        assert_invalid(params, ParamsError::InvalidRadiusRelationship);

        params.separation_radius += params.perception_radius; // Invalid: separation_radius >= perception_radius
        assert_invalid(params, ParamsError::InvalidRadiusRelationship);
    }

    #[test]
    fn test_invalid_params_non_finite() {
        let mut params = SimulationParams::default();

        params.perception_radius = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("perception_radius"));

        params = SimulationParams::default();
        params.separation_radius = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("separation_radius"));

        params = SimulationParams::default();
        params.cohesion_weight = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("cohesion_weight"));

        params = SimulationParams::default();
        params.alignment_weight = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("alignment_weight"));

        params = SimulationParams::default();
        params.separation_weight = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("separation_weight"));

        params = SimulationParams::default();
        params.max_speed = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("max_speed"));

        params = SimulationParams::default();
        params.max_acceleration = f32::NAN;
        assert_invalid(params, ParamsError::NonFinite("max_acceleration"));

        params = SimulationParams::default();
        params.perception_radius = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("perception_radius"));

        params = SimulationParams::default();
        params.separation_radius = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("separation_radius"));

        params = SimulationParams::default();
        params.cohesion_weight = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("cohesion_weight"));

        params = SimulationParams::default();
        params.alignment_weight = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("alignment_weight"));

        params = SimulationParams::default();
        params.separation_weight = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("separation_weight"));

        params = SimulationParams::default();
        params.max_speed = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("max_speed"));

        params = SimulationParams::default();
        params.max_acceleration = f32::INFINITY;
        assert_invalid(params, ParamsError::NonFinite("max_acceleration"));
    }

    #[test]
    fn test_invalid_params_non_negative() {
        let mut params = SimulationParams::default();
        params.cohesion_weight = -1.0;
        assert_invalid(params, ParamsError::MustBeNonNegative("cohesion_weight"));

        params = SimulationParams::default();
        params.alignment_weight = -1.0;
        assert_invalid(params, ParamsError::MustBeNonNegative("alignment_weight"));

        params = SimulationParams::default();
        params.separation_weight = -1.0;
        assert_invalid(params, ParamsError::MustBeNonNegative("separation_weight"));
    }

    #[test]
    fn test_invalid_params_positive() {
        let mut params = SimulationParams::default();
        params.perception_radius = -1.0;
        assert_invalid(params, ParamsError::MustBePositive("perception_radius"));

        params = SimulationParams::default();
        params.separation_radius = -0.5;
        assert_invalid(params, ParamsError::MustBePositive("separation_radius"));

        params = SimulationParams::default();
        params.max_speed = -1.0;
        assert_invalid(params, ParamsError::MustBePositive("max_speed"));

        params = SimulationParams::default();
        params.max_acceleration = -1.0;
        assert_invalid(params, ParamsError::MustBePositive("max_acceleration"));
    }

    #[test]
    fn test_invalid_params_zero() {
        let mut params = SimulationParams::default();
        params.perception_radius = 0.0;
        assert_invalid(params, ParamsError::MustBePositive("perception_radius"));

        params = SimulationParams::default();
        params.separation_radius = 0.0;
        assert_invalid(params, ParamsError::MustBePositive("separation_radius"));

        params = SimulationParams::default();
        params.max_speed = 0.0;
        assert_invalid(params, ParamsError::MustBePositive("max_speed"));

        params = SimulationParams::default();
        params.max_acceleration = 0.0;
        assert_invalid(params, ParamsError::MustBePositive("max_acceleration"));
    }
}
