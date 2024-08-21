use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Region3<T: SquashNumber> {
    pub size: Vector3<T>,
    pub position: Vector3<T>,
}
impl_squash!(Region3<T: SquashNumber>, size, position;position, size);

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Region3int16 {
    pub size: Vector3int16,
    pub position: Vector3int16,
}
impl_squash!(Region3int16, size, position;position, size);