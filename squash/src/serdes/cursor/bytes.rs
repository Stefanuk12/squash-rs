use ux::*;

pub trait LeBytes<const N: usize> {
    fn to_le_bytes(self) -> [u8; N];
    fn from_le_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_le_bytes {
    ($($type:ty, $size:expr, $from_ty:ty),*) => {
        $(
            impl LeBytes<$size> for $type {
                fn to_le_bytes(self) -> [u8; $size] {
                    <$from_ty>::from(self).to_le_bytes()[..$size].try_into().unwrap()
                }
                fn from_le_bytes(bytes: [u8; $size]) -> Self {
                    <$from_ty>::from_le_bytes({
                        let mut arr = [0; std::mem::size_of::<$from_ty>()];
                        arr[..$size].copy_from_slice(&bytes);
                        arr
                    }).try_into().unwrap()
                }
            }
        )*
    };
}
impl_le_bytes!(
    u24, 3, u32, u40, 5, u64, u48, 6, u64, u56, 7, u64, i24, 3, i32, i40, 5, i64, i48, 6, i64, i56,
    7, i64
);
