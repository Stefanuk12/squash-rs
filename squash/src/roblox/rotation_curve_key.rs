use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct RotationCurveKey<T: SquashNumber> {
    pub interpolation: EnumItem,
    pub value: T,
    pub time: f32,
}
impl_squash_object_a!(RotationCurveKey<T: SquashNumber>, interpolation, value, time;time, value, interpolation);
