use std::collections::HashMap;
use std::hash::Hash;

use super::SerDes;
use crate::{Error, Result, SquashCursor, Vlq};

/// Map (v5 `map(k, v)`): per entry value-then-key, VLQ count suffix.
#[derive(Clone, Copy, Debug, Default)]
pub struct Map<K, V>(pub K, pub V);
impl<K: SerDes, V: SerDes> SerDes for Map<K, V>
where
    K::Value: Eq + Hash,
{
    type Value = HashMap<K::Value, V::Value>;
    fn ser<C: SquashCursor>(
        &self,
        cursor: &mut C,
        value: &HashMap<K::Value, V::Value>,
    ) -> Result<usize> {
        let mut count = 0;
        
        for (k, v) in value {
            count += self.1.ser(cursor, v)?;
            count += self.0.ser(cursor, k)?;
        }

        count += cursor.push(Vlq(value.len() as u64))?;
        Ok(count)
    }
    fn des<C: SquashCursor>(&self, cursor: &mut C) -> Result<HashMap<K::Value, V::Value>> {
        let len = cursor.pop::<Vlq>()?.0;
        let remaining = cursor.remaining()?;
        let mut map = HashMap::with_capacity(len.min(remaining / 2) as usize);
        for _ in 0..len {
            let before = if len > remaining {
                cursor.remaining()?
            } else {
                0
            };

            let key = self.0.des(cursor)?;
            let val = self.1.des(cursor)?;
            map.insert(key, val);

            if len > remaining && cursor.remaining()? == before {
                return Err(Error::Custom(format!(
                    "map of {len} entries is unsatisfiable: the entry codecs consumed \
                     no bytes and only {remaining} were available"
                )));
            }
        }
        Ok(map)
    }
}
