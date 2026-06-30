use crate::BoolTuple3;

use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct RaycastParams {
    pub bool_data: BoolTuple3,
    pub filter_type: EnumItem,
    pub collision_group: String,
}
impl_squash_object_a!(RaycastParams, bool_data, collision_group, filter_type; filter_type, collision_group, bool_data);
