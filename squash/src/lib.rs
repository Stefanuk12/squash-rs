macro_rules! import {
    ($($module:ident),*) => {
        $(
            pub mod $module;
            pub use $module::*;
        )*
    };
}

import!(ds, num, error, serdes);

pub mod codec;
pub use codec::SerDes;

pub use squash_derive::*;

#[cfg(feature = "roblox")]
import!(roblox);

#[macro_export]
macro_rules! impl_number {
    ($aa:ident, $t:ty, $serialize_fn:ident) => {
        impl ::serde::Serialize for $aa<$t> {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.$serialize_fn(self.0)
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $aa<$t> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                <$t>::deserialize(deserializer).map($aa)
            }
        }
        impl $crate::SquashObject for $aa<$t> {
            fn pop_obj<T>(cursor: &mut T) -> $crate::Result<Self>
            where
                T: $crate::SquashCursor,
                Self: Sized,
            {
                Ok(cursor.pop::<$t>().map($aa)?)
            }
            fn push_obj<T: $crate::SquashCursor>(self, cursor: &mut T) -> $crate::Result<usize> {
                Ok(cursor.push(self.0)?)
            }
        }
        impl From<$t> for $aa<$t> {
            fn from(t: $t) -> Self {
                $aa(t)
            }
        }
        impl From<$aa<$t>> for $t {
            fn from(t: $aa<$t>) -> Self {
                t.0
            }
        }
        impl ::core::ops::Deref for $aa<$t> {
            type Target = $t;
            fn deref(&self) -> &$t {
                &self.0
            }
        }
        impl ::core::ops::DerefMut for $aa<$t> {
            fn deref_mut(&mut self) -> &mut $t {
                &mut self.0
            }
        }
    };
    ($aa:ident, $t:ty) => {
        impl ::serde::Serialize for $aa<$t> {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                let bytes = self.0.to_le_bytes();
                serializer.serialize_bytes(&bytes)
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $aa<$t> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let bytes = Vec::<u8>::deserialize(deserializer)?;
                Ok($aa(<$t>::from_le_bytes(bytes.try_into().unwrap())))
            }
        }
        impl $crate::SquashObject for $aa<$t> {
            fn pop_obj<T>(cursor: &mut T) -> $crate::Result<Self>
            where
                T: $crate::SquashCursor,
                Self: Sized,
            {
                Ok(cursor.pop::<$t>().map(|x| $aa(x as $t))?)
            }
            fn push_obj<T: $crate::SquashCursor>(self, cursor: &mut T) -> $crate::Result<usize> {
                Ok(cursor.push(self.0)?)
            }
        }
        impl From<$t> for $aa<$t> {
            fn from(t: $t) -> Self {
                $aa(t)
            }
        }
        impl From<$aa<$t>> for $t {
            fn from(t: $aa<$t>) -> Self {
                t.0
            }
        }
        impl ::core::ops::Deref for $aa<$t> {
            type Target = $t;
            fn deref(&self) -> &$t {
                &self.0
            }
        }
        impl ::core::ops::DerefMut for $aa<$t> {
            fn deref_mut(&mut self) -> &mut $t {
                &mut self.0
            }
        }
    };
}

#[cfg(feature = "serde")]
#[macro_export]
macro_rules! impl_reverse_deserialize {
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>, $($field:tt),*) => {
        impl<'de, $($gen $(: $bound)?),*> ::serde::Deserialize<'de> for $ident<$($gen),*> {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                #[allow(non_camel_case_types)]
                enum Field {
                    $($field,)*
                }
                struct MainVisitor<$($gen $(: $bound)?),*>(::core::marker::PhantomData<($($gen,)*)>);

                impl<'de, $($gen $(: $bound)?),*> ::serde::de::Visitor<'de> for MainVisitor<$($gen),*> {
                    type Value = $ident<$($gen),*>;
                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(stringify!("struct {}", $ident))
                    }
                    fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        $(
                            let $field = seq
                                .next_element()?
                                .ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        )*
                        Ok(Self::Value { $($field,)* })
                    }
                }
                const FIELDS: &[&str] = &[$(stringify!($field),)*];
                deserializer.deserialize_struct(
                    stringify!($ident),
                    FIELDS,
                    MainVisitor(::core::marker::PhantomData),
                )
            }
        }
    };
    ($ident:ident, $($field:tt),*) => {
        impl<'de> ::serde::Deserialize<'de> for $ident {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                #[allow(non_camel_case_types)]
                enum Field {
                    $($field,)*
                }
                struct MainVisitor(::core::marker::PhantomData<()>);

                impl<'de> ::serde::de::Visitor<'de> for MainVisitor {
                    type Value = $ident;
                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(stringify!("struct {}", $ident))
                    }
                    fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        $(
                            let $field = seq
                                .next_element()?
                                .ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        )*
                        Ok(Self::Value { $($field,)* })
                    }
                }
                const FIELDS: &[&str] = &[$(stringify!($field),)*];
                deserializer.deserialize_struct(
                    stringify!($ident),
                    FIELDS,
                    MainVisitor(::core::marker::PhantomData),
                )
            }
        }
    };
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>) => {
        impl<'de, $($gen $(: $bound)?),*> ::serde::Deserialize<'de> for $ident<$($gen),*> {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                #[allow(non_camel_case_types)]
                enum Field {
                    _0,
                }
                struct MainVisitor<$($gen $(: $bound)?),*>(::core::marker::PhantomData<($($gen,)*)>);

                impl<'de, $($gen $(: $bound)?),*> ::serde::de::Visitor<'de> for MainVisitor<$($gen),*> {
                    type Value = $ident<$($gen),*>;
                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(stringify!("struct {}", $ident))
                    }
                    fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        let _0 = seq
                            .next_element()?
                            .ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        Ok(Self(_0))
                    }
                }
                const FIELDS: &[&str] = &["_0"];
                deserializer.deserialize_struct(
                    stringify!($ident),
                    FIELDS,
                    MainVisitor(::core::marker::PhantomData),
                )
            }
        }
    };
    ($ident:ident) => {
        impl<'de> ::serde::Deserialize<'de> for $ident {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                #[allow(non_camel_case_types)]
                enum Field {
                    _0,
                }
                struct MainVisitor(::core::marker::PhantomData<()>);

                impl<'de> ::serde::de::Visitor<'de> for MainVisitor {
                    type Value = $ident;
                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(stringify!("struct {}", $ident))
                    }
                    fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        let _0 = seq
                            .next_element()?
                            .ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        Ok(Self::Value(_0))
                    }
                }
                const FIELDS: &[&str] = &["_0"];
                deserializer.deserialize_struct(
                    stringify!($ident),
                    FIELDS,
                    MainVisitor(::core::marker::PhantomData),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_squash_object_a {
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>, $($field:tt),*; $($backward_field:tt),*) => {
        impl<$($gen $(: $bound)?),*> $crate::SquashObject for $ident<$($gen),*> {
            fn pop_obj<Obj>(cursor: &mut Obj) -> $crate::Result<Self>
            where
                Obj: $crate::SquashCursor,
                Self: Sized,
            {
                Ok($ident {
                    $(
                        $backward_field: cursor.pop()?,
                    )*
                })
            }
            fn push_obj<Obj: $crate::SquashCursor>(self, cursor: &mut Obj) -> $crate::Result<usize> {
                let mut count = 0;
                $(
                    count += cursor.push(self.$field)?;
                )*
                Ok(count)
            }
        }
    };
    ($ident:ident, $($field:tt),*; $($backward_field:tt),*) => {
        impl $crate::SquashObject for $ident {
            fn pop_obj<Obj>(cursor: &mut Obj) -> $crate::Result<Self>
            where
                Obj: $crate::SquashCursor,
                Self: Sized,
            {
                Ok($ident {
                    $(
                        $backward_field: cursor.pop()?,
                    )*
                })
            }
            fn push_obj<Obj: $crate::SquashCursor>(self, cursor: &mut Obj) -> $crate::Result<usize> {
                let mut count = 0;
                $(
                    count += cursor.push(self.$field)?;
                )*
                Ok(count)
            }
        }
    };
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>) => {
        impl<$($gen $(: $bound)?),*> $crate::SquashObject for $ident<$($gen),*> {
            fn pop_obj<Obj>(cursor: &mut Obj) -> $crate::Result<Self>
            where
                Obj: $crate::SquashCursor,
                Self: Sized,
            {
                Ok($ident(cursor.pop()?))
            }
            fn push_obj<Obj: $crate::SquashCursor>(self, cursor: &mut Obj) -> $crate::Result<usize> {
                Ok(cursor.push(self.0)?)
            }
        }
    };
    ($ident:ident) => {
        impl $crate::SquashObject for $ident {
            fn pop_obj<Obj>(cursor: &mut Obj) -> $crate::Result<Self>
            where
                Obj: $crate::SquashCursor,
                Self: Sized,
            {
                Ok($ident(cursor.pop()?))
            }
            fn push_obj<Obj: $crate::SquashCursor>(self, cursor: &mut Obj) -> $crate::Result<usize> {
                Ok(cursor.push(self.0)?)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_squash {
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>, $($field:tt),*; $($backward_field:tt),*) => {
        #[cfg(feature = "serde")]
        $crate::impl_reverse_deserialize!($ident<$($gen $(: $bound)?),*>, $($field),*);
        $crate::impl_squash_object_a!($ident<$($gen $(: $bound)?),*>, $($field),*; $($backward_field),*);
    };
    ($ident:ident, $($field:tt),*; $($backward_field:tt),*) => {
        #[cfg(feature = "serde")]
        $crate::impl_reverse_deserialize!($ident, $($field),*);
        $crate::impl_squash_object_a!($ident, $($field),*; $($backward_field),*);
    };
    ($ident:ident<$($gen:ident $(: $bound:path)?),*>) => {
        $crate::impl_squash_object_a!($ident<$($gen $(: $bound)?),*>);
    };
    ($ident:ident) => {
        $crate::impl_squash_object_a!($ident);
    };
}

#[cfg(feature = "serde")]
#[macro_export]
macro_rules! impl_serde_for_enum {
    ($enum_name:ident, $($variant:ident = $index:literal),*) => {
        impl ::serde::Serialize for $enum_name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeStruct;

                match self {
                    $(
                        $enum_name::$variant(v) => {
                            let mut seq = serializer.serialize_struct(stringify!($enum_name), 2)?;
                            seq.serialize_field("0", v)?;
                            seq.serialize_field("1", &($index as u8))?;
                            seq.end()
                        }
                    )*
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $enum_name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                enum Field {
                    C,
                    T,
                }

                struct MainVisitor;
                impl<'de> ::serde::de::Visitor<'de> for MainVisitor {
                    type Value = $enum_name;

                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(stringify!("enum ", $enum_name))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let tag = seq
                            .next_element::<u8>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                        match tag {
                            $(
                                $index => Ok($enum_name::$variant(
                                    seq.next_element()?
                                        .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?,
                                )),
                            )*
                            _ => Err(serde::de::Error::invalid_length(0, &self)),
                        }
                    }
                }

                const FIELDS: &[&str] = &["t", "c"];
                ::serde::Deserializer::deserialize_struct(deserializer, stringify!($enum_name), FIELDS, MainVisitor)
            }
        }
    };
}
