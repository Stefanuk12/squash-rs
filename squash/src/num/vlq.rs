use crate::prelude::*;

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
pub struct Vlq(pub u64);
impl SquashObject for Vlq {
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
    where
        T: SquashCursor,
        Self: Sized,
    {
        let mut x: u64 = 0;
        let mut terminated = false;

        for _ in 0..8 {
            let b = cursor.pop::<u8>()? as u64;
            x = x * 128 + (b % 128);

            if b < 128 {
                terminated = true;
                break;
            }
        }

        if !terminated {
            return Err(crate::Error::InvalidVlq(x));
        }
        Ok(Self(x))
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        let value = self.0;
        if value >= 1 << 56 {
            return Err(crate::Error::InvalidVlq(value));
        }

        let count: u32 = if value >= 1 << 49 {
            8
        } else if value >= 1 << 42 {
            7
        } else if value >= 1 << 35 {
            6
        } else if value >= 1 << 28 {
            5
        } else if value >= 1 << 21 {
            4
        } else if value >= 1 << 14 {
            3
        } else if value >= 1 << 7 {
            2
        } else {
            1
        };

        let mut written = 0;
        for i in 0..count {
            let group = ((value >> (7 * i)) & 0x7F) as u8;
            let byte = if i == 0 { group } else { group | 0x80 };
            written += cursor.push(byte)?;
        }
        Ok(written)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Vlq {
    fn serialize<S>(&self, serializer: S) -> CoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new(Vec::new());
        cursor.push(*self).map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(cursor.into_inner().as_slice())
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Vlq {
    fn deserialize<D>(deserializer: D) -> CoreResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VlqVisitor;
        impl<'de> serde::de::Visitor<'de> for VlqVisitor {
            type Value = Vlq;
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("the bytes of a VLQ")
            }
            fn visit_bytes<E>(self, v: &[u8]) -> CoreResult<Vlq, E>
            where
                E: serde::de::Error,
            {
                let mut cursor = std::io::Cursor::new(v.to_vec());
                cursor.seek_end().map_err(serde::de::Error::custom)?;
                cursor.pop::<Vlq>().map_err(serde::de::Error::custom)
            }
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> CoreResult<Vlq, E>
            where
                E: serde::de::Error,
            {
                self.visit_bytes(&v)
            }
        }
        deserializer.deserialize_bytes(VlqVisitor)
    }
}
