//! Regression for the serde tuple/tuple-struct element ordering fix (ser.rs):
//! `serialize_tuple` must reverse elements on write (like the sequence path) so
//! the LIFO `deserialize_tuple` reads them back in declaration order. Before the
//! fix a heterogeneous tuple round-tripped to scrambled, wrongly-typed values
//! (e.g. `(7u8, 0x0102u16, "hi")` -> `(2, 26984, "\u{2}")`).
#![cfg(feature = "serde")]

use squash::{serde_deserialize, serde_serialize};

fn serde_rt<T>(v: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + PartialEq + std::fmt::Debug,
{
    let mut bytes = serde_serialize(&v).unwrap();
    let back: T = serde_deserialize(&mut bytes).unwrap();
    assert_eq!(back, v);
}

#[test]
fn heterogeneous_tuple_round_trips() {
    serde_rt((7u8, 0x0102u16, "hi".to_string()));
    serde_rt((1u8, 2u8, 3u8));
    serde_rt((true, -5i32, 9.5f32));
}

#[test]
fn nested_and_tuple_of_seq_round_trip() {
    serde_rt((vec![1u16, 2, 3], "tail".to_string()));
    serde_rt(((1u8, 2u16), (3u32, 4u64)));
}
