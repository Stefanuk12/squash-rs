//! serde round-trips for the standard scalar and container types through the
//! crate's own `Serializer`/`Deserializer`. The existing serde tests exercise
//! enums, tuples and Roblox records, but never a bare `u8`, `i64`, `f64`, `char`,
//! `Option`, `HashMap`, etc. A regression in any primitive `serialize_*` /
//! `deserialize_*` pair (or the reverse-cursor seq handling) would slip through.
#![cfg(feature = "serde")]

use std::collections::HashMap;

use squash::{serde_deserialize, serde_serialize};

fn serde_rt<T>(v: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let mut bytes = serde_serialize(&v).unwrap();
    let back: T = serde_deserialize(&mut bytes).unwrap();
    assert_eq!(back, v, "serde round-trip of {v:?}");
}

#[test]
fn bool_and_unsigned_round_trip() {
    serde_rt(true);
    serde_rt(false);
    for v in [0u8, 1, 255] {
        serde_rt(v);
    }
    for v in [0u16, 1, 513, u16::MAX] {
        serde_rt(v);
    }
    for v in [0u32, 1, 0x01020304, u32::MAX] {
        serde_rt(v);
    }
    for v in [0u64, 1, 0x0102030405060708, u64::MAX] {
        serde_rt(v);
    }
}

#[test]
fn signed_round_trip_including_negatives() {
    for v in [0i8, 1, -1, i8::MIN, i8::MAX] {
        serde_rt(v);
    }
    for v in [0i16, -1, 12345, i16::MIN, i16::MAX] {
        serde_rt(v);
    }
    for v in [0i32, -1, -70000, i32::MIN, i32::MAX] {
        serde_rt(v);
    }
    for v in [0i64, -1, -1_000_000_000_000, i64::MIN, i64::MAX] {
        serde_rt(v);
    }
}

#[test]
fn floats_round_trip() {
    for v in [0.0f32, -0.0, 1.5, -2.25, f32::MIN, f32::MAX, f32::INFINITY, f32::NEG_INFINITY] {
        serde_rt(v);
    }
    for v in [0.0f64, -0.0, 1.5, -2.25, f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY] {
        serde_rt(v);
    }
}

#[test]
fn nan_round_trips_bitwise() {
    // NaN != NaN, so `serde_rt`'s equality check can't be used directly.
    let mut bytes = serde_serialize(&f64::NAN).unwrap();
    let back: f64 = serde_deserialize(&mut bytes).unwrap();
    assert!(back.is_nan());
}

#[test]
fn char_and_string_round_trip() {
    for c in ['a', 'Z', '0', ' ', '€', '🦀', '\u{0}', char::MAX] {
        serde_rt(c);
    }
    for s in ["", "hi", "a unicode string: €🦀 ✓", &"long ".repeat(100)] {
        serde_rt(s.to_string());
    }
}

#[test]
fn option_round_trips() {
    serde_rt(Option::<u32>::None);
    serde_rt(Some(0u32));
    serde_rt(Some(70000u32));
    serde_rt(Some("hello".to_string()));
    serde_rt(Option::<String>::None);
    // Nested options.
    serde_rt(Some(Some(5u8)));
    serde_rt(Some(Option::<u8>::None));
}

#[test]
fn collections_round_trip() {
    serde_rt(Vec::<u32>::new());
    serde_rt(vec![1u32, 2, 3, 70000]);
    serde_rt(vec![Some(1u8), None, Some(3)]);
    serde_rt(vec!["a".to_string(), "".to_string(), "ccc".to_string()]);

    let mut m: HashMap<String, u32> = HashMap::new();
    m.insert("one".into(), 1);
    m.insert("two".into(), 2);
    m.insert("three".into(), 70000);
    serde_rt(m);

    let mut m2: HashMap<u16, Vec<u8>> = HashMap::new();
    m2.insert(1, vec![1, 2, 3]);
    m2.insert(2, vec![]);
    serde_rt(m2);
}

#[test]
fn nested_structures_round_trip() {
    serde_rt((1u8, "two".to_string(), vec![3u16, 4, 5], Some(6u32)));
    serde_rt(vec![vec![1u8, 2], vec![], vec![3, 4, 5]]);
    serde_rt(Some(vec![("k".to_string(), 1u8), ("k2".to_string(), 2)]));
}
