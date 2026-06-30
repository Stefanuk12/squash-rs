use super::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Axes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
    pub back: bool,
    pub front: bool,
}
impl From<u16> for Axes {
    fn from(value: u16) -> Self {
        Axes {
            x: (value & 256) != 0,
            y: (value & 512) != 0,
            z: (value & 1024) != 0,
            top: (value & 32) != 0,
            bottom: (value & 4) != 0,
            left: (value & 8) != 0,
            right: (value & 16) != 0,
            back: (value & 1) != 0,
            front: (value & 2) != 0,
        }
    }
}
impl From<Axes> for u16 {
    fn from(value: Axes) -> Self {
        (value.back as u16)
            | (value.bottom as u16) << 1
            | (value.front as u16) << 2
            | (value.left as u16) << 3
            | (value.right as u16) << 4
            | (value.top as u16) << 5
            | (value.x as u16) << 8
            | (value.y as u16) << 9
            | (value.z as u16) << 10
    }
}
impl SquashObject for Axes {
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
    where
        T: SquashCursor,
        Self: Sized,
    {
        u16::pop_obj(cursor).map(Self::from)
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        cursor.push(u16::from(self))
    }
}
#[cfg(feature = "serde")]
impl Serialize for Axes {
    fn serialize<S>(&self, serializer: S) -> CoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(u16::from(*self))
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Axes {
    fn deserialize<D>(deserializer: D) -> CoreResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x = u16::deserialize(deserializer)?;
        Ok(Self::from(x))
    }
}
