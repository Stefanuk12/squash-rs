use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct NumberRange<T: SquashNumber> {
    pub min: T,
    pub max: T
}
impl_squash!(NumberRange<T: SquashNumber>, min, max;min, max);