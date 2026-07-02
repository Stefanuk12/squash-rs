use std::io::Cursor;

use serde::{
    de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess},
    Deserialize,
};

use crate::{Error, Result, SquashCursor, Vlq};

macro_rules! impl_deserialize {
    ($($fn_name:ident, $t:ty, $visit_method:ident),*) => {
        $(
            fn $fn_name<V>(self, visitor: V) -> Result<V::Value>
            where
                V: de::Visitor<'de> {
                visitor.$visit_method(self.input.pop::<$t>()?)
            }
        )*
    };
}

pub fn serde_deserialize<'de, T>(input: &'de mut Vec<u8>) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(input)?;
    let value = de::Deserialize::deserialize(&mut deserializer)?;
    Ok(value)
}

const MAX_DEPTH: usize = 128;

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: Cursor<&'de mut Vec<u8>>,
    depth: usize,
}
impl<'de> Deserializer<'de> {
    pub fn new(input: &'de mut Vec<u8>) -> Result<Self> {
        let mut input = Cursor::new(input);
        input.seek_end()?;
        Ok(Self { input, depth: 0 })
    }

    fn enter(&mut self) -> Result<()> {
        if self.depth == MAX_DEPTH {
            return Err(Error::RecursionDepthExceeded);
        }
        self.depth += 1;
        Ok(())
    }

    fn exit(&mut self) {
        self.depth -= 1;
    }
}
impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::DeserializeAnyNotImplemented)
    }

    impl_deserialize!(
        deserialize_bool,
        bool,
        visit_bool,
        deserialize_i8,
        i8,
        visit_i8,
        deserialize_i16,
        i16,
        visit_i16,
        deserialize_i32,
        i32,
        visit_i32,
        deserialize_i64,
        i64,
        visit_i64,
        deserialize_u8,
        u8,
        visit_u8,
        deserialize_u16,
        u16,
        visit_u16,
        deserialize_u32,
        u32,
        visit_u32,
        deserialize_u64,
        u64,
        visit_u64,
        deserialize_f32,
        f32,
        visit_f32,
        deserialize_f64,
        f64,
        visit_f64,
        deserialize_char,
        char,
        visit_char
    );

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(&self.input.pop::<String>()?)
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(&self.input.pop::<Vec<u8>>()?)
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.input.pop::<Vec<u8>>()?)
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let is_some = self.input.pop::<u8>()? == 1;
        if is_some {
            self.enter()?;
            let value = visitor.visit_some(&mut *self);
            self.exit();
            value
        } else {
            visitor.visit_none()
        }
    }
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.enter()?;
        let value = visitor.visit_newtype_struct(&mut *self);
        self.exit();
        value
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.input.pop::<Vlq>()?;
        self.enter()?;
        let value = visitor.visit_seq(VlqIncluded::new(&mut *self, *len, true));
        self.exit();
        value
    }
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.enter()?;
        let value = visitor.visit_seq(VlqIncluded::new(&mut *self, len as u64, false));
        self.exit();
        value
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.input.pop::<Vlq>()?;
        self.enter()?;
        let value = visitor.visit_map(VlqIncluded::new(&mut *self, *len, true));
        self.exit();
        value
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.enter()?;
        let value = visitor.visit_seq(VlqIncluded::new(&mut *self, fields.len() as u64, false));
        self.exit();
        value
    }
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.enter()?;
        let value = visitor.visit_enum(Enum::new(&mut *self));
        self.exit();
        value
    }
    // Identifiers exist on the wire only as single-byte enum variant tags;
    // names are never encoded.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.input.pop::<u8>()? as u64)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct VlqIncluded<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    size_hint: u64,
    /// Whether `size_hint` is an untrusted wire count (seq/map) rather than a
    /// schema-supplied length (tuple/struct). When set, a claimed element that
    /// consumes no bytes is rejected instead of looped over — this mirrors the
    /// native `read_elems` zero-width DoS guard (see `codec/array.rs`), without
    /// which a forged count over zero-byte elements (`Vec<()>`, unit structs,
    /// `PhantomData`) spins for up to `2^56 - 1` iterations.
    guarded: bool,
}
impl<'a, 'de> VlqIncluded<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, size_hint: u64, guarded: bool) -> Self {
        Self {
            de,
            size_hint,
            guarded,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for VlqIncluded<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.size_hint as usize)
    }
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if SeqAccess::size_hint(&self) == Some(0) {
            return Ok(None);
        }

        self.size_hint -= 1;
        let before = self.de.input.remaining()?;
        let value = seed.deserialize(&mut *self.de)?;
        if self.guarded && self.size_hint > 0 && self.de.input.remaining()? == before {
            return Err(Error::Custom(format!(
                "sequence claims {} more elements but this one consumed no bytes",
                self.size_hint
            )));
        }
        Ok(Some(value))
    }
}

impl<'de, 'a> MapAccess<'de> for VlqIncluded<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.size_hint as usize)
    }
    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        if MapAccess::size_hint(&self) == Some(0) {
            return Ok(None);
        }

        self.size_hint -= 1;
        let before = self.de.input.remaining()?;
        let key = kseed.deserialize(&mut *self.de)?;
        let value = vseed.deserialize(&mut *self.de)?;
        if self.guarded && self.size_hint > 0 && self.de.input.remaining()? == before {
            return Err(Error::Custom(format!(
                "map claims {} more entries but this one consumed no bytes",
                self.size_hint
            )));
        }
        Ok(Some((key, value)))
    }
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if MapAccess::size_hint(&self) == Some(0) {
            return Ok(None);
        }

        self.size_hint -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}
impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(VlqIncluded::new(self.de, len as u64, false))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(VlqIncluded::new(self.de, fields.len() as u64, false))
    }
}
