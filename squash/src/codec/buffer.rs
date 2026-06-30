use super::SerDes;
use crate::{Result, SquashCursor, Vlq};

/// Variable-length buffer (v5 `buffer()`): raw bytes then VLQ length suffix.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Buf;
impl SerDes for Buf {
    type Value = Vec<u8>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<u8>) -> Result<usize> {
        let mut count = cursor.write(value)?;
        count += cursor.push(Vlq(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<u8>> {
        cursor.pop::<Vec<u8>>()
    }
}

/// Fixed-length buffer (v5 `buffer(n)`): exactly `n` raw bytes, no length.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BufN(pub usize);
impl SerDes for BufN {
    type Value = Vec<u8>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<u8>) -> Result<usize> {
        let mut buf = vec![0u8; self.0];
        let k = value.len().min(self.0);
        buf[..k].copy_from_slice(&value[..k]);
        Ok(cursor.write(&buf)?)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; self.0];
        cursor.pop_read(&mut buf)?;
        Ok(buf)
    }
}
