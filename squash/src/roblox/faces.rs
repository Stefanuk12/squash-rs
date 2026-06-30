use super::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Faces {
    pub back: bool,
    pub bottom: bool,
    pub front: bool,
    pub left: bool,
    pub right: bool,
    pub top: bool,
}
impl From<u8> for Faces {
    fn from(x: u8) -> Self {
        Faces {
            back: (x & 1) != 0,
            bottom: (x & 2) != 0,
            front: (x & 4) != 0,
            left: (x & 8) != 0,
            right: (x & 16) != 0,
            top: (x & 32) != 0,
        }
    }
}
impl From<Faces> for u8 {
    fn from(x: Faces) -> Self {
        (x.back as u8)
            | (x.bottom as u8) << 1
            | (x.front as u8) << 2
            | (x.left as u8) << 3
            | (x.right as u8) << 4
            | (x.top as u8) << 5
    }
}
impl SquashObject for Faces {
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
    where
        T: SquashCursor,
        Self: Sized,
    {
        u8::pop_obj(cursor).map(Self::from)
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        cursor.push(u8::from(self))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Faces {
    fn serialize<S>(&self, serializer: S) -> CoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(u8::from(*self))
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Faces {
    fn deserialize<D>(deserializer: D) -> CoreResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x = u8::deserialize(deserializer)?;
        Ok(Self::from(x))
    }
}
