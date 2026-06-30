use super::SerDes;
use crate::ux::{u24, u40, u48, u56};
use crate::{Error, Result, SquashCursor};


/// Integer in `[min, max]` stored as `value - min` in the minimum number of bytes (v5 `range(min, max)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub min: i64,
    pub max: i64,
}
impl Range {
    pub fn new(min: i64, max: i64) -> Self {
        debug_assert!(min <= max, "range min must be <= max");
        Self { min, max }
    }
    fn bytes(&self) -> u8 {
        uint_bytes((self.max as i128 - self.min as i128) as u64)
    }
}
impl SerDes for Range {
    type Value = i64;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &i64) -> Result<usize> {
        if *value < self.min || *value > self.max {
            return Err(Error::Custom(format!(
                "value {value} is outside range [{}, {}]",
                self.min, self.max
            )));
        }
        write_uint(
            cursor,
            (*value as i128 - self.min as i128) as u64,
            self.bytes(),
        )
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<i64> {
        let value = self.min as i128 + read_uint(cursor, self.bytes())? as i128;
        if value < self.min as i128 || value > self.max as i128 {
            return Err(Error::Custom(format!(
                "decoded value {value} is outside range [{}, {}]",
                self.min, self.max
            )));
        }
        Ok(value as i64)
    }
}

/// Number of bytes to hold `0..=diff` (v5: `difference < 256^k`).
fn uint_bytes(diff: u64) -> u8 {
    if diff < 1 << 8 {
        1
    } else if diff < 1 << 16 {
        2
    } else if diff < 1 << 24 {
        3
    } else if diff < 1 << 32 {
        4
    } else if diff < 1 << 40 {
        5
    } else if diff < 1 << 48 {
        6
    } else if diff < 1 << 56 {
        7
    } else {
        8
    }
}

fn write_uint<C: SquashCursor>(cursor: &mut C, x: u64, bytes: u8) -> Result<usize> {
    if (1..=7).contains(&bytes) && x >= (1u64 << (bytes as u32 * 8)) {
        return Err(Error::Custom(format!(
            "value {x} does not fit in {bytes} byte(s)"
        )));
    }

    match bytes {
        1 => cursor.push(x as u8),
        2 => cursor.push(x as u16),
        3 => cursor.push(u24::try_from(x as u32)?),
        4 => cursor.push(x as u32),
        5 => cursor.push(u40::try_from(x)?),
        6 => cursor.push(u48::try_from(x)?),
        7 => cursor.push(u56::try_from(x)?),
        8 => cursor.push(x),
        _ => Err(Error::Custom(format!(
            "uint width {bytes} out of range 1..=8"
        ))),
    }
}

fn read_uint<C: SquashCursor>(cursor: &mut C, bytes: u8) -> Result<u64> {
    match bytes {
        1 => Ok(cursor.pop::<u8>()? as u64),
        2 => Ok(cursor.pop::<u16>()? as u64),
        3 => Ok(u32::from(cursor.pop::<u24>()?) as u64),
        4 => Ok(cursor.pop::<u32>()? as u64),
        5 => Ok(u64::from(cursor.pop::<u40>()?)),
        6 => Ok(u64::from(cursor.pop::<u48>()?)),
        7 => Ok(u64::from(cursor.pop::<u56>()?)),
        8 => Ok(cursor.pop::<u64>()?),
        _ => Err(Error::Custom(format!(
            "uint width {bytes} out of range 1..=8"
        ))),
    }
}

/// Unsigned integer in a fixed number of bytes 1..=8 (v5 `uint(bytes)`), little-endian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uint(pub u8);
impl SerDes for Uint {
    type Value = u64;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &u64) -> Result<usize> {
        write_uint(cursor, *value, self.0)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<u64> {
        read_uint(cursor, self.0)
    }
}
