use super::SerDes;
use crate::{Result, SquashCursor, Vlq};

/// Variable-length string (v5 `string()`): raw bytes then VLQ length suffix.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Str;
impl SerDes for Str {
    type Value = String;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &String) -> Result<usize> {
        let mut count = cursor.write(value.as_bytes())?;
        count += cursor.push(Vlq(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<String> {
        cursor.pop::<String>()
    }
}

/// Fixed-length string (v5 `string(n)`): exactly `n` raw bytes, no length.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StrN(pub usize);
impl SerDes for StrN {
    type Value = String;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &String) -> Result<usize> {
        let mut buf = vec![0u8; self.0];
        let bytes = value.as_bytes();
        let mut k = bytes.len().min(self.0);

        while k > 0 && !value.is_char_boundary(k) {
            k -= 1;
        }
        buf[..k].copy_from_slice(&bytes[..k]);
        Ok(cursor.write(&buf)?)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<String> {
        let mut buf = vec![0u8; self.0];
        cursor.pop_read(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}
