use super::{vlq_to_usize, SerDes};
use crate::{Error, Result, SquashCursor, Vlq};

/// Pack `bits` LSB-first, 8 per byte, full bytes first then a partial byte (v5 `serbitarr`).
pub fn push_bits<C: SquashCursor>(cursor: &mut C, bits: &[bool]) -> Result<usize> {
    let n = bits.len();
    if n == 0 {
        return Ok(0);
    }
    let bits_over = n % 8;
    let full = n - bits_over;
    let mut count = 0;
    let mut i = 0;
    while i < full {
        let mut byte = 0u8;
        for j in 0..8 {
            if bits[i + j] {
                byte |= 1 << j;
            }
        }
        count += cursor.push(byte)?;
        i += 8;
    }
    if bits_over > 0 {
        let mut byte = 0u8;
        for j in 0..bits_over {
            if bits[full + j] {
                byte |= 1 << j;
            }
        }
        count += cursor.push(byte)?;
    }
    Ok(count)
}

pub fn pop_bits<C: SquashCursor>(cursor: &mut C, n: usize) -> Result<Vec<bool>> {
    if n == 0 {
        return Ok(Vec::new());
    }

    let need = (n as u64 + 7) / 8;
    let remaining = cursor.remaining()?;
    if need > remaining {
        return Err(Error::Custom(format!(
            "bit array claims {n} bits ({need} bytes) but only {remaining} remain"
        )));
    }

    let mut arr = vec![false; n];
    let bytes = n / 8;
    let bits_over = n % 8;
    let base = bytes * 8;

    if bits_over > 0 {
        let byte = cursor.pop::<u8>()?;
        for j in 0..bits_over {
            arr[base + j] = (byte & (1 << j)) != 0;
        }
    }
    
    for i in (0..bytes).rev() {
        let byte = cursor.pop::<u8>()?;
        let b = i * 8;
        for j in 0..8 {
            arr[b + j] = (byte & (1 << j)) != 0;
        }
    }
    Ok(arr)
}

/// Variable-length packed bool array (v5 `array(boolean())`): bits then VLQ count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BitArray;
impl SerDes for BitArray {
    type Value = Vec<bool>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<bool>) -> Result<usize> {
        let mut count = push_bits(cursor, value)?;
        count += cursor.push(Vlq(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<bool>> {
        let n = vlq_to_usize(cursor.pop::<Vlq>()?.0)?;
        pop_bits(cursor, n)
    }
}

/// Fixed-length packed bool array (v5 `array(boolean(), n)`): exactly `n` bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BitArrayN(pub usize);
impl SerDes for BitArrayN {
    type Value = Vec<bool>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<bool>) -> Result<usize> {
        let mut bits = vec![false; self.0];
        let k = value.len().min(self.0);
        bits[..k].copy_from_slice(&value[..k]);
        push_bits(cursor, &bits)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<bool>> {
        pop_bits(cursor, self.0)
    }
}
