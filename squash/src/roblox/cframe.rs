use std::num::NonZeroU8;

use super::prelude::*;

const CFRAME_ROTS: [CframeRotSegments; 25] = [
    // 0 is not a valid special_id, used as a placeholder. needed since luau is 1-indexed.
    CframeRotSegments::new(0, 0, 0),
    CframeRotSegments::new(0, 0, 0),
    CframeRotSegments::new(1608, 0, 0),
    CframeRotSegments::new(3216, -1, -1),
    CframeRotSegments::new(-1609, 0, 0),
    CframeRotSegments::new(2274, 2274, -1),
    CframeRotSegments::new(1238, 1238, 1238),
    CframeRotSegments::new(0, 0, 1608),
    CframeRotSegments::new(-1239, -1239, 1238),
    CframeRotSegments::new(-1239, -1239, 1238),
    CframeRotSegments::new(0, -1609, 0),
    CframeRotSegments::new(1238, -1239, -1239),
    CframeRotSegments::new(2274, -1, 2274),
    CframeRotSegments::new(2274, -1, -2275),
    CframeRotSegments::new(0, 3216, 0),
    CframeRotSegments::new(0, -2275, 2274),
    CframeRotSegments::new(0, 0, 3216),
    CframeRotSegments::new(-1, 2274, 2274),
    CframeRotSegments::new(0, 0, -1609),
    CframeRotSegments::new(1238, -1239, -1239),
    CframeRotSegments::new(2274, -2275, 0),
    CframeRotSegments::new(-1239, 1238, -1239),
    CframeRotSegments::new(1238, 1238, 1238),
    CframeRotSegments::new(0, 1608, 0),
    CframeRotSegments::new(-1239, 1238, -1239),
];

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct CframeRotSegments {
    pub z: i16,
    pub y: i16,
    pub x: i16,
}
impl_squash_object_a!(CframeRotSegments, x, y, z;z, y, x);
impl CframeRotSegments {
    pub const fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }

    pub const fn from_special_id(id: NonZeroU8) -> Option<Self> {
        let Some(twenty_five) = NonZeroU8::new(25) else {
            return None;
        };
        if id.get() > twenty_five.get() {
            None
        } else {
            Some(CFRAME_ROTS[id.get() as usize])
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Cframe<T: SquashNumber> {
    pub rotation: CframeRotSegments,
    pub position: Vector3<T>,
}
impl_squash_object_a!(Cframe<T: SquashNumber>, rotation, position;position, rotation);

impl<T: SquashNumber> Cframe<T> {
    pub const fn special_id(&self) -> Option<u8> {
        let mut i = 1;
        while i < CFRAME_ROTS.len() {
            let rots = CFRAME_ROTS[i];
            if rots.x == self.rotation.x && rots.y == self.rotation.y && rots.z == self.rotation.z {
                return Some(i as u8);
            }
            i += 1;
        }
        None
    }

    pub const fn from_special_id(id: NonZeroU8) -> Option<Self> {
        let Some(rotation) = CframeRotSegments::from_special_id(id) else {
            return None;
        };

        Some(Cframe {
            rotation,
            position: Vector3::ZERO,
        })
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Cframe<T>
where
    T: SquashNumber,
    Vector3<T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> CoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Cframe", 2)?;
        if let Some(special_id) = self.special_id() {
            state.serialize_field("special_id", &special_id)?;
        } else {
            state.serialize_field("rotation", &self.rotation)?;
            state.serialize_field("special_id", &0_u8)?;
        }
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}
#[cfg(feature = "serde")]
impl<'de, T: SquashNumber> Deserialize<'de> for Cframe<T> {
    fn deserialize<D>(deserializer: D) -> CoreResult<Self, D::Error>
        where
            D: Deserializer<'de> {
        struct CframeVisitor<T>(core::marker::PhantomData<T>);

        impl<'de, T: SquashNumber> serde::de::Visitor<'de> for CframeVisitor<T> {
            type Value = Cframe<T>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a CFrame")
            }

            fn visit_seq<A>(self, mut seq: A) -> CoreResult<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>, {
                let position = seq.next_element::<Vector3<T>>()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let special_id = seq.next_element::<u8>()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let rotation = if special_id == 0 {
                    seq.next_element::<CframeRotSegments>()?.ok_or_else(|| serde::de::Error::invalid_length(2, &self))?
                } else {
                    CframeRotSegments::from_special_id(NonZeroU8::new(special_id).unwrap()).ok_or_else(|| serde::de::Error::custom("invalid special_id"))?
                };

                Ok(Cframe { rotation, position })
            }
        }

        deserializer.deserialize_struct("Cframe", &["rotation", "special_id", "position"], CframeVisitor(core::marker::PhantomData))
    }
}