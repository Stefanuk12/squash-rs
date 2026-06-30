use super::SerDes;
use crate::ux::{u24, u40, u48, u56};
use crate::{Result, SquashCursor, Vlq};

// Primitive number codecs (v5: u8()..u64(), i8()..i64(), f32(), f64())
macro_rules! prim_codec {
    ($($(#[$m:meta])* $name:ident => $t:ty),* $(,)?) => {
        $(
            $(#[$m])*
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
            pub struct $name;
            impl SerDes for $name {
                type Value = $t;
                fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &$t) -> Result<usize> {
                    cursor.push(*value)
                }
                fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<$t> {
                    cursor.pop::<$t>()
                }
            }
        )*
    };
}

prim_codec!(
    U8 => u8, U16 => u16, U24 => u24, U32 => u32,
    U40 => u40, U48 => u48, U56 => u56, U64 => u64,
    I8 => i8, I16 => i16, I24 => crate::ux::i24, I32 => i32,
    I40 => crate::ux::i40, I48 => crate::ux::i48, I56 => crate::ux::i56, I64 => i64,
    F32 => f32, F64 => f64,
    /// Single boolean as one byte (v5 `boolean()`).
    Bool => bool,
);

/// Variable-length quantity (v5 `vlq()`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VlqCodec;
impl SerDes for VlqCodec {
    type Value = u64;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &u64) -> Result<usize> {
        cursor.push(Vlq(*value))
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<u64> {
        Ok(cursor.pop::<Vlq>()?.0)
    }
}
