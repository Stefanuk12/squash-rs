use crate::u48;

use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(
    From,
    Into,
    AsRef,
    AsMut,
    derive_more::Debug,
    derive_more::Display,
    Index,
    Deref,
    Mul,
    IndexMut,
    DerefMut,
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
pub struct DateTime(pub u48);
impl_squash!(DateTime);
