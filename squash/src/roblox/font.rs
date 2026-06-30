use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Font {
    pub family: String,
    pub bold: bool,
    pub weight: EnumItem,
    pub style: EnumItem,
}
impl_squash_object_a!(Font, family, bold, weight, style;style, weight, bold, family);
