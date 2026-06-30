macro_rules! decl_bool_tuple {
    ($name:ident, $($idx:tt),* $(,)? ) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
        pub struct $name( $( pub decl_bool_tuple!( @decl $idx, bool) ),* );
        impl $crate::SquashObject for $name {
            fn push_obj<T: $crate::SquashCursor>(self, cursor: &mut T) -> $crate::Result<usize> {
                cursor.push(
                    0u8 $( | ((self.$idx as u8) << $idx) )*
                )
            }
            fn pop_obj<T>(cursor: &mut T) -> $crate::Result<Self>
            where
                T: $crate::SquashCursor,
                Self: Sized {
                let x = cursor.pop::<u8>()?;
                Ok($name( $( (x & (1 << $idx)) != 0 ),* ))
            }
        }
        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer
            {
                serializer.serialize_u8(
                    0u8 $( | ((self.$idx as u8) << $idx) )*
                )
            }
        }
        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>
            {
                let x = u8::deserialize(deserializer)?;
                Ok($name( $( (x & (1 << $idx)) != 0 ),* ))
            }
        }
    };
    // I need a way to expand to `pub bool` as many times as there are literals,
    // so I'm using a little helper that ignores the literal and just expands to something else.
    ( @decl $_:literal, $($expand:tt)* ) => { $($expand)* };
}

decl_bool_tuple!(BoolTuple2, 0, 1);
decl_bool_tuple!(BoolTuple3, 0, 1, 2);
decl_bool_tuple!(BoolTuple4, 0, 1, 2, 3);
decl_bool_tuple!(BoolTuple5, 0, 1, 2, 3, 4);
decl_bool_tuple!(BoolTuple6, 0, 1, 2, 3, 4, 5);
decl_bool_tuple!(BoolTuple7, 0, 1, 2, 3, 4, 5, 6);
decl_bool_tuple!(BoolTuple8, 0, 1, 2, 3, 4, 5, 6, 7);
