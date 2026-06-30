use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "T: SquashNumber")))]
#[derive(
    From,
    Into,
    IntoIterator,
    AsRef,
    AsMut,
    Index,
    Deref,
    Mul,
    IndexMut,
    DerefMut,
    MulAssign,
    Clone,
    PartialEq,
    PartialOrd,
    Debug,
    Default,
)]
pub struct NumberSequence<T: SquashNumber>(pub Vec<NumberSequenceKeypoint<T>>);
impl_squash!(NumberSequence<T: SquashNumber>);
