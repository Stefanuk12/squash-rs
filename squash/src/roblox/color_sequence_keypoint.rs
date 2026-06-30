use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ColorSequenceKeypoint {
    pub value: Color3,
    pub time: u8,
}
impl_squash_object_a!(ColorSequenceKeypoint, value, time;time, value);
