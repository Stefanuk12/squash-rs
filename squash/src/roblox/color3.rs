use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Color3 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}
impl_squash_object_a!(Color3, b, g, r; r, g, b);
