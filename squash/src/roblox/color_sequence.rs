use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
)]
pub struct ColorSequence(pub Vec<ColorSequenceKeypoint>);
impl_squash!(ColorSequence);
