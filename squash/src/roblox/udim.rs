use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Udim<T: SquashNumber> {
    pub offset: T,
    pub scale: T,
}
impl_squash_object_a!(Udim<T: SquashNumber>, offset, scale;scale, offset);

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Udim2<T: SquashNumber> {
    pub y: Udim<T>,
    pub x: Udim<T>,
}
impl_squash_object_a!(Udim2<T: SquashNumber>, y, x;x, y);
