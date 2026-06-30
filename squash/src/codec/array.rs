use super::{vlq_to_usize, SerDes};
use crate::{Error, Result, SquashCursor, Vlq};

/// Serialize every element of `value` forward through `sd`.
fn write_elems<C: SquashCursor, S: SerDes>(
    cursor: &mut C,
    sd: &S,
    value: &[S::Value],
) -> Result<usize> {
    let mut count = 0;
    for x in value {
        count += sd.ser(cursor, x)?;
    }
    Ok(count)
}

/// Deserialize exactly `len` elements through `sd` and undo the LIFO order.
fn read_elems<C: SquashCursor, S: SerDes>(
    cursor: &mut C,
    sd: &S,
    len: usize,
    trusted: bool,
) -> Result<Vec<S::Value>> {
    let remaining = cursor.remaining()? as usize;
    let mut xs = Vec::with_capacity(len.min(remaining));
    for _ in 0..len {
        let guard = !trusted && len > remaining;
        let before = if guard { cursor.remaining()? } else { 0 };
        xs.push(sd.des(cursor)?);

        if guard && cursor.remaining()? == before {
            return Err(Error::Custom(format!(
                "array of {len} elements is unsatisfiable: the element codec consumed \
                 no bytes and only {remaining} were available"
            )));
        }
    }
    xs.reverse();
    Ok(xs)
}

/// Variable-length array (v5 `array(serdes)`): elements forward, VLQ count suffix.
#[derive(Clone, Copy, Debug, Default)]
pub struct Array<S>(pub S);
impl<S: SerDes> SerDes for Array<S> {
    type Value = Vec<S::Value>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<S::Value>) -> Result<usize> {
        let mut count = write_elems(cursor, &self.0, value)?;
        count += cursor.push(Vlq(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<S::Value>> {
        let len = vlq_to_usize(cursor.pop::<Vlq>()?.0)?;
        read_elems(cursor, &self.0, len, false)
    }
}

/// Fixed-length array (v5 `array(serdes, n)`): exactly `n` elements, no count.
#[derive(Clone, Copy, Debug, Default)]
pub struct ArrayN<S>(pub S, pub usize);
impl<S: SerDes> SerDes for ArrayN<S> {
    type Value = Vec<S::Value>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<S::Value>) -> Result<usize> {
        if value.len() != self.1 {
            return Err(Error::Custom(format!(
                "fixed-length array expects {} elements, got {}",
                self.1,
                value.len()
            )));
        }
        write_elems(cursor, &self.0, value)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<S::Value>> {
        read_elems(cursor, &self.0, self.1, true)
    }
}

/// Array whose count is encoded by a custom number codec (v5 `array(serdes, lengthSerDes)`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ArrayLen<S, L>(pub S, pub L);
impl<S: SerDes, L: SerDes<Value = u64>> SerDes for ArrayLen<S, L> {
    type Value = Vec<S::Value>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Vec<S::Value>) -> Result<usize> {
        let mut count = write_elems(cursor, &self.0, value)?;
        count += self.1.ser(cursor, &(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Vec<S::Value>> {
        let len = vlq_to_usize(self.1.des(cursor)?)?;
        read_elems(cursor, &self.0, len, false)
    }
}
