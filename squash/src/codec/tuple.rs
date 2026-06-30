use super::SerDes;
use crate::{Result, SquashCursor};

macro_rules! tuple_codec {
    ( $( $t:ident . $i:tt ),+ ; $( $rt:ident . $ri:tt ),+ ) => {
        impl<$($t: SerDes),+> SerDes for ($($t,)+) {
            type Value = ($($t::Value,)+);
            fn ser<Cur: SquashCursor>(&self, cursor: &mut Cur, value: &Self::Value) -> Result<usize> {
                let mut count = 0;
                $( count += self.$i.ser(cursor, &value.$i)?; )+
                Ok(count)
            }
            #[allow(non_snake_case)]
            fn des<Cur: SquashCursor>(&self, cursor: &mut Cur) -> Result<Self::Value> {
                // Read back-to-front (LIFO), bind by index, then assemble in order.
                $( let $rt = self.$ri.des(cursor)?; )+
                Ok(( $($t,)+ ))
            }
        }
    };
}

tuple_codec!(A.0, B.1; B.1, A.0);
tuple_codec!(A.0, B.1, C.2; C.2, B.1, A.0);
tuple_codec!(A.0, B.1, C.2, D.3; D.3, C.2, B.1, A.0);
tuple_codec!(A.0, B.1, C.2, D.3, E.4; E.4, D.3, C.2, B.1, A.0);
tuple_codec!(A.0, B.1, C.2, D.3, E.4, F.5; F.5, E.4, D.3, C.2, B.1, A.0);
