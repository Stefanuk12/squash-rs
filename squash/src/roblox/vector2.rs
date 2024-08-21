use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Vector2<T: SquashNumber> {
    pub y: T,
    pub x: T,
}
impl_squash_object_a!(Vector2<T: SquashNumber>, x, y;y, x);
impl<T: SquashNumber> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Vector2int16 {
    pub y: i16,
    pub x: i16,
}
impl_squash_object_a!(Vector2int16, x, y;y, x);