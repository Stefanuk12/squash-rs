use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Ray<T: SquashNumber> {
    pub direction: Vector3<T>,
    pub origin: Vector3<T>,
}
impl_squash_object_a!(Ray<T: SquashNumber>, direction, origin;origin, direction);
