use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct RotationCurveKey<T: SquashNumber> {
    pub interpolation: EnumItem,
    pub value: T,
    pub time: f32,
}
impl_squash!(RotationCurveKey<T: SquashNumber>, interpolation, value, time;time, value, interpolation);