macro_rules! impl_custom_int {
    ($name:ident, $int_type:ident, $byte_count:expr) => {
        #[cfg_attr(feature = "serde", derive(::serde::Serialize))]
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
        #[allow(non_camel_case_types)]
        pub struct $name([u8; $byte_count]);
        impl $name {
            pub fn new(val: $int_type) -> ::core::result::Result<Self, $crate::Error> {
                Self::try_from(val)
            }

            pub fn convert(self) -> $int_type {
                $int_type::from(self)
            }
        }

        impl From<$name> for $int_type {
            fn from(val: $name) -> $int_type {
                let mut bytes = [0u8; ::core::mem::size_of::<$int_type>()];
                bytes[..$byte_count].copy_from_slice(&val.0);
                $int_type::from_le_bytes(bytes)
            }
        }

        impl TryFrom<$int_type> for $name {
            type Error = $crate::Error;

            fn try_from(val: $int_type) -> $crate::Result<$name> {
                let bytes = val.to_le_bytes();
                if bytes[$byte_count..].iter().any(|&x| x != 0) {
                    return Err($crate::Error::ValueTooLarge);
                }
                let mut arr = [0u8; $byte_count];
                arr.copy_from_slice(&bytes[..$byte_count]);
                Ok($name(arr))
            }
        }

        impl From<$name> for ::ux::$name {
            fn from(val: $name) -> ::ux::$name {
                ::ux::$name::new($int_type::from(val))
            }
        }

        impl $crate::SquashObject for $name {
            fn pop_obj<T>(cursor: &mut T) -> $crate::Result<Self>
            where
                T: $crate::SquashCursor,
                Self: Sized,
            {
                let mut arr = [0u8; $byte_count];
                for i in (0..$byte_count).rev() {
                    arr[i] = cursor.pop()?;
                }
                Ok($name(arr))
            }

            fn push_obj<T: $crate::SquashCursor>(self, cursor: &mut T) -> $crate::Result<usize> {
                for i in 0..$byte_count {
                    cursor.push(self.0[i])?;
                }
                Ok($byte_count)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[doc(hidden)]
                struct Visitor<'de> {
                    marker: ::core::marker::PhantomData<$name>,
                    lifetime: ::core::marker::PhantomData<&'de ()>,
                }
                impl<'de> ::serde::de::Visitor<'de> for Visitor<'de> {
                    type Value = $name;
                    fn expecting(
                        &self,
                        formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(formatter, "tuple struct A")
                    }
                    #[inline]
                    fn visit_newtype_struct<E>(
                        self,
                        e: E,
                    ) -> ::core::result::Result<Self::Value, E::Error>
                    where
                        E: ::serde::Deserializer<'de>,
                    {
                        let x: [u8; $byte_count] =
                            <[u8; $byte_count] as ::serde::Deserialize>::deserialize(e)?;
                        Ok($name(x))
                    }
                    #[inline]
                    fn visit_seq<S>(
                        self,
                        mut seq: S,
                    ) -> ::core::result::Result<Self::Value, S::Error>
                    where
                        S: ::serde::de::SeqAccess<'de>,
                    {
                        ::serde::de::SeqAccess::next_element::<[u8; $byte_count]>(&mut seq)?
                            .ok_or(::serde::de::Error::invalid_length(
                                0usize,
                                &stringify!(
                                    "tuple struct ",
                                    $name,
                                    " with ",
                                    $byte_count,
                                    " element"
                                ),
                            ))
                            .map($name)
                    }
                }
                ::serde::Deserializer::deserialize_newtype_struct(
                    deserializer,
                    stringify!($name),
                    Visitor {
                        marker: ::core::marker::PhantomData::<$name>,
                        lifetime: ::core::marker::PhantomData,
                    },
                )
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", $int_type::from(*self))
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", $int_type::from(*self))
            }
        }

        impl $crate::Zero for $name {
            const ZERO: Self = $name([0u8; $byte_count]);
        }
    };
}

// Generate implementations for unsigned types
impl_custom_int!(u24, u32, 3);
impl_custom_int!(u40, u64, 5);
impl_custom_int!(u48, u64, 6);
impl_custom_int!(u56, u64, 7);

// Generate implementations for signed types
impl_custom_int!(i24, i32, 3);
impl_custom_int!(i40, i64, 5);
impl_custom_int!(i48, i64, 6);
impl_custom_int!(i56, i64, 7);
