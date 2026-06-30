use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct NumberSequenceKeypoint<T: SquashNumber> {
    pub value: T,
    pub envelope: T,
    pub time: u8,
}
impl_squash_object_a!(NumberSequenceKeypoint<T: SquashNumber>, value, envelope, time;time, envelope, value);
