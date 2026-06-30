use crate::SquashObject;

import!(int, float, uint, ux, vlq);

pub trait Zero {
    const ZERO: Self;
}

macro_rules! impl_zero {
    ($($t:ty)*) => {
        $(
            impl Zero for $t {
                const ZERO: Self = 0;
            }
        )*
    }
}

impl_zero!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

#[cfg(feature = "serde")]
pub trait SquashNumber: serde::de::DeserializeOwned + SquashObject + Clone + Zero {}
#[cfg(not(feature = "serde"))]
pub trait SquashNumber: SquashObject + Clone + Zero {}

macro_rules! impl_squash_number {
    ($($t:ty),*) => {
        $(
            impl $crate::SquashNumber for $t {}
        )*
    };
}

impl_squash_number!(
    f32, f64, i8, i16, i24, i32, i40, i48, i56, i64, u8, u16, u24, u32, u40, u48, u56, u64
);
