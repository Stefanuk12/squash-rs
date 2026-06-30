use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Region3<T: SquashNumber> {
    pub size: Vector3<T>,
    pub position: Vector3<T>,
}
impl_squash_object_a!(Region3<T: SquashNumber>, size, position;position, size);

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Region3int16 {
    pub min: Vector3int16,
    pub max: Vector3int16,
}
impl_squash_object_a!(Region3int16, max, min; min, max);
