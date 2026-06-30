use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct PhysicalProperties {
    pub elasticity_weight: f32,
    pub friction_weight: f32,
    pub elasticity: f32,
    pub friction: f32,
    pub density: f32,
}
impl_squash_object_a!(PhysicalProperties, elasticity_weight, friction_weight, elasticity, friction, density;density, friction, elasticity, friction_weight, elasticity_weight);
