use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(
    From,
    Into,
    FromStr,
    AsRef,
    AsMut,
    derive_more::Debug,
    derive_more::Display,
    Index,
    Deref,
    Not,
    Add,
    Mul,
    Sum,
    IndexMut,
    DerefMut,
    AddAssign,
    MulAssign,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Default,
)]
pub struct EnumItem(pub Vlq);
impl_squash!(EnumItem);
