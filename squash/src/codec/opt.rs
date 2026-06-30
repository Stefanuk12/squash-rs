use super::SerDes;
use crate::{Result, SquashCursor};

/// Optional value (v5 `opt(serdes)`): `None` -> `0x00`; `Some` -> payload then `0x01`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Opt<S>(pub S);
impl<S: SerDes> SerDes for Opt<S> {
    type Value = Option<S::Value>;
    fn ser<C: SquashCursor>(&self, cursor: &mut C, value: &Option<S::Value>) -> Result<usize> {
        let mut count = 0;
        match value {
            Some(x) => {
                count += self.0.ser(cursor, x)?;
                count += cursor.push(1_u8)?;
            }
            None => {
                count += cursor.push(0_u8)?;
            }
        }
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<Option<S::Value>> {
        if cursor.pop::<u8>()? == 1 {
            Ok(Some(self.0.des(cursor)?))
        } else {
            Ok(None)
        }
    }
}
