use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct PathWaypoint<T: SquashNumber> {
    pub label: String,
    pub action: EnumItem,
    pub position: Vector3<T>,
}
impl_squash!(PathWaypoint<T: SquashNumber>, label, action, position;position, action, label);