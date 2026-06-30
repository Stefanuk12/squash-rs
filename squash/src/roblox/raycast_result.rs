use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct RaycastResult<T: SquashNumber> {
    pub distance: f32,
    pub position: Vector3<T>,
    pub normal: Vector3<T>,
    pub material: EnumItem,
}
impl_squash_object_a!(RaycastResult<T: SquashNumber>, distance, position, normal, material;material, normal, position, distance);
