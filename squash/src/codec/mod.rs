use crate::{Error, Result, SquashCursor};

mod array;
mod bits;
mod buffer;
mod literal;
mod map;
mod opt;
mod prim;
mod range;
mod string;
mod tuple;

pub use array::{Array, ArrayLen, ArrayN};
pub use bits::{pop_bits, push_bits, BitArray, BitArrayN};
pub use buffer::{Buf, BufN};
pub use literal::Literal;
pub use map::Map;
pub use opt::Opt;
pub use prim::{
    Bool, VlqCodec, F32, F64, I16, I24, I32, I40, I48, I56, I64, I8, U16, U24, U32, U40, U48, U56,
    U64, U8,
};
pub use range::{Range, Uint};
pub use string::{Str, StrN};

/// A parameterizable serializer/deserializer pair for a single value type.
pub trait SerDes {
    type Value;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Self::Value) -> Result<usize>;
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Self::Value>;
}

/// Serialize a value with the given codec.
pub fn to_bytes<S: SerDes>(sd: &S, value: &S::Value) -> Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    sd.ser(&mut cursor, value)?;
    Ok(cursor.into_inner())
}

/// Deserialize a value with the given codec.
pub fn from_bytes<S: SerDes>(sd: &S, data: Vec<u8>) -> Result<S::Value> {
    let mut cursor = std::io::Cursor::new(data);
    cursor.seek_end()?;
    sd.des(&mut cursor)
}

/// Narrow a wire-decoded `u64` count to `usize`.
pub(crate) fn vlq_to_usize(len: u64) -> Result<usize> {
    usize::try_from(len)
        .map_err(|_| Error::Custom(format!("length {len} exceeds this platform's usize")))
}

pub fn opt<S: SerDes>(inner: S) -> Opt<S> {
    Opt(inner)
}
pub fn array<S: SerDes>(inner: S) -> Array<S> {
    Array(inner)
}
pub fn array_n<S: SerDes>(inner: S, n: usize) -> ArrayN<S> {
    ArrayN(inner, n)
}
pub fn map<K: SerDes, V: SerDes>(key: K, value: V) -> Map<K, V> {
    Map(key, value)
}
pub fn string() -> Str {
    Str
}
pub fn string_n(n: usize) -> StrN {
    StrN(n)
}
pub fn buffer() -> Buf {
    Buf
}
pub fn buffer_n(n: usize) -> BufN {
    BufN(n)
}
pub fn range(min: i64, max: i64) -> Range {
    Range::new(min, max)
}
pub fn vlq() -> VlqCodec {
    VlqCodec
}
pub fn literal<T: PartialEq + Clone>(values: Vec<T>) -> Literal<T> {
    Literal(values)
}
