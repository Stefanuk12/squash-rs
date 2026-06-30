use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Rect<T: SquashNumber> {
    pub max: Vector2<T>,
    pub min: Vector2<T>,
}
impl_squash_object_a!(Rect<T: SquashNumber>, max, min;min, max);
