use std::{collections::HashMap, mem::MaybeUninit};

use ux::*;

use crate::{LeBytes, SquashCursor, Vlq};

pub trait SquashObject {
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize>;
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
        where
            T: SquashCursor,
            Self: Sized;
}

macro_rules! impl_squash_objects {
    ($(($($t:ty),+), $size:expr);* $(;)?) => {
        $(
            $(
                impl $crate::SquashObject for $t {
                    fn pop_obj<T>(cursor: &mut T) -> $crate::Result<Self>
                    where
                        T: SquashCursor,
                        Self: Sized {
                        let mut buf = [0; $size];
                        cursor.pop_read(&mut buf)?;
                        Ok(<$t>::from_le_bytes(buf))
                    }
                    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> $crate::Result<usize> {
                        cursor.realloc($size);
                        Ok(cursor.write(&self.to_le_bytes()[..$size])?)
                    }
                }
            )*
        )*
    };
}

impl_squash_objects!(
    (u8, i8), 1;
    (u16, i16), 2;
    (u24, i24), 3;
    (u32, i32, f32), 4;
    (u40, i40), 5;
    (u48, i48), 6;
    (u56, i56), 7;
    (u64, i64, f64), 8;
);

impl SquashObject for bool {
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        let value = if self { 1_u8 } else { 0_u8 };
        cursor.push(value)
    }
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
            where
                T: SquashCursor,
                Self: Sized {
        let value = cursor.pop::<u8>()?;
        Ok(value == 1)
    }
}

impl<T> SquashObject for Option<T>
    where
        T: SquashObject
{
    fn pop_obj<U>(cursor: &mut U) -> crate::Result<Self>
            where
                U: SquashCursor,
                Self: Sized {
        let is_some = cursor.pop::<u8>()? == 1;
        if is_some {
            Ok(Some(T::pop_obj(cursor)?))
        } else {
            Ok(None)
        }
    }
    fn push_obj<U: SquashCursor>(self, cursor: &mut U) -> crate::Result<usize> {
        let mut count = 0;
        if let Some(x) = self {
            count += cursor.push(x)?;
        }
        count += cursor.push(1_u8)?;
        Ok(count)
    }
}

impl<T> SquashObject for Vec<T>
    where
        T: SquashObject
{
    fn pop_obj<U>(cursor: &mut U) -> crate::Result<Self>
            where
                U: SquashCursor,
                Self: Sized {
        let len = cursor.pop::<Vlq>()?.0;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            Vec::push(&mut vec, T::pop_obj(cursor)?);
        }
        Ok(vec)
    }
    fn push_obj<U: SquashCursor>(self, cursor: &mut U) -> crate::Result<usize> {
        let mut count = 0;
        count += cursor.push(Vlq(self.len() as u64))?;
        for item in self {
            count += cursor.push(item)?;
        }
        Ok(count)
    }
}

impl<K, V> SquashObject for HashMap<K, V>
where
    K: SquashObject + Eq + std::hash::Hash,
    V: SquashObject,
{
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
            where
                T: SquashCursor,
                Self: Sized {
        let len = cursor.pop::<Vlq>()?.0;
        let mut map = HashMap::with_capacity(len as usize);
        for _ in 0..len {
            let key = K::pop_obj(cursor)?;
            let value = V::pop_obj(cursor)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        let mut count = 0;
        count += cursor.push(Vlq(self.len() as u64))?;
        for (key, value) in self {
            count += cursor.push(key)?;
            count += cursor.push(value)?;
        }
        Ok(count)
    }
}

impl SquashObject for String {
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
            where
                T: SquashCursor,
                Self: Sized {
        let len = cursor.pop::<Vlq>()?;
        let mut buf = vec![0; len.0 as usize];
        cursor.pop_read(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        let len = Vlq(self.len() as u64);
        let mut count = 0;
        count += cursor.write(self.as_bytes())?;
        count += cursor.push(len)?;
        Ok(count)
    }
}

impl SquashObject for char {
    fn pop_obj<T>(cursor: &mut T) -> crate::Result<Self>
            where
                T: SquashCursor,
                Self: Sized {
        let str = String::pop_obj(cursor)?;
        str.chars().next().ok_or(crate::Error::CharMissing)
    }
    fn push_obj<T: SquashCursor>(self, cursor: &mut T) -> crate::Result<usize> {
        self.to_string().push_obj(cursor)
    }
}

impl<T: SquashObject, const N: usize> SquashObject for [T; N] {
    fn pop_obj<C>(cursor: &mut C) -> crate::Result<Self>
    where
        C: SquashCursor,
        Self: Sized,
    {
        let mut arr = Vec::with_capacity(N);
        for _ in 0..N {
            arr.push(T::pop_obj(cursor)?);
        }
        Ok(unsafe { arr.try_into().unwrap_unchecked() })
    }

    fn push_obj<C: SquashCursor>(self, cursor: &mut C) -> crate::Result<usize> {
        let mut count = 0;
        for item in self.into_iter() {
            count += item.push_obj(cursor)?;
        }
        Ok(count)
    }
}