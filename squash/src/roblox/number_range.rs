use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct NumberRange<T: SquashNumber> {
    pub min: T,
    pub max: T,
}
impl_squash_object_a!(NumberRange<T: SquashNumber>, max, min; min, max);
