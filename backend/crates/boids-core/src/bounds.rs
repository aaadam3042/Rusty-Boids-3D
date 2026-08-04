use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum BoundsError {
    NonFinite,
    InvalidAxis,
}

pub enum BoundaryMode {
    Bounce,
    Wrap, 
}

pub struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    pub fn try_new(min: Vec3, max: Vec3) -> Result<Self, BoundsError> {
        if !min.is_finite() || !max.is_finite() {
            return Err(BoundsError::NonFinite);
        }

        if min.x > max.x || min.y > max.y || min.z > max.z {
            return Err(BoundsError::InvalidAxis);
        }

        Ok(Self { min, max })
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

impl Bounds {
     pub fn wrap_position(&self, position: Vec3) -> Vec3 {
        let mut wrapped = position;

        if wrapped.x < self.min.x {
            wrapped.x = self.max.x - (self.min.x - wrapped.x) % self.size().x;
        } else if wrapped.x > self.max.x {
            wrapped.x = self.min.x + (wrapped.x - self.max.x) % self.size().x;
        }

        if wrapped.y < self.min.y {
            wrapped.y = self.max.y - (self.min.y - wrapped.y) % self.size().y;
        } else if wrapped.y > self.max.y {
            wrapped.y = self.min.y + (wrapped.y - self.max.y) % self.size().y;
        }

        if wrapped.z < self.min.z {
            wrapped.z = self.max.z - (self.min.z - wrapped.z) % self.size().z;
        } else if wrapped.z > self.max.z {
            wrapped.z = self.min.z + (wrapped.z - self.max.z) % self.size().z;
        }

        wrapped
    }

    pub fn bounce_position(&self, position: Vec3, velocity: &mut Vec3) -> Vec3 {
        let mut bounced = position;

        if bounced.x < self.min.x {
            bounced.x = self.min.x;
            velocity.x = -velocity.x;
        } else if bounced.x > self.max.x {
            bounced.x = self.max.x;
            velocity.x = -velocity.x;
        }

        if bounced.y < self.min.y {
            bounced.y = self.min.y;
            velocity.y = -velocity.y;
        } else if bounced.y > self.max.y {
            bounced.y = self.max.y;
            velocity.y = -velocity.y;
        }

        if bounced.z < self.min.z {
            bounced.z = self.min.z;
            velocity.z = -velocity.z;
        } else if bounced.z > self.max.z {
            bounced.z = self.max.z;
            velocity.z = -velocity.z;
        }

        bounced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
