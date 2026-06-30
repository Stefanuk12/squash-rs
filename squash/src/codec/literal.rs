use super::SerDes;
use crate::{Error, Result, SquashCursor};

/// One of a fixed set of values, encoded as its 0-based index in a single byte (v5 `literal(...)`).
#[derive(Clone, Debug, Default)]
pub struct Literal<T>(pub Vec<T>);
impl<T: PartialEq + Clone> SerDes for Literal<T> {
    type Value = T;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &T) -> Result<usize> {
        let idx = self
            .0
            .iter()
            .position(|x| x == value)
            .ok_or_else(|| Error::Custom("value is not in the literal set".into()))?;

        if idx > u8::MAX as usize {
            return Err(Error::Custom(format!(
                "literal set has {} entries; index {idx} does not fit in one byte",
                self.0.len()
            )));
        }
        
        cursor.push(idx as u8)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<T> {
        let idx = cursor.pop::<u8>()? as usize;
        self.0
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::Custom(format!("literal index {idx} out of range")))
    }
}
