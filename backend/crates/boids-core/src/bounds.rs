use crate::math::Vec3;

pub enum WrapMode {
    Bounce,
    Wrap, 
}

pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
    pub wrap_mode: WrapMode,
}
