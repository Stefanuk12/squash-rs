//! Native `serialize`/`deserialize` (the `SquashObject` path) round-trips for the
//! built-in scalar and container impls. The Roblox tests cover these indirectly
//! through composite types, but the standalone primitives — signed integers at
//! their extremes, float specials, `char`, and nested `Option`/`Vec`/`HashMap` —
//! aren't directly exercised anywhere.

use std::collections::HashMap;

use squash::{deserialize, serialize, SquashObject};

fn rt<T>(v: T)
where
    T: SquashObject + Clone + PartialEq + std::fmt::Debug,
{
    let back: T = deserialize(serialize(v.clone()).unwrap()).unwrap();
    assert_eq!(back, v, "native round-trip of {v:?}");
}

#[test]
fn unsigned_extremes_round_trip() {
    for v in [0u8, 1, u8::MAX] {
        rt(v);
    }
    for v in [0u16, u16::MAX] {
        rt(v);
    }
    for v in [0u32, u32::MAX] {
        rt(v);
    }
    for v in [0u64, u64::MAX] {
        rt(v);
    }
}

#[test]
fn signed_extremes_round_trip() {
    for v in [0i8, -1, 1, i8::MIN, i8::MAX] {
        rt(v);
    }
    for v in [0i16, -1, i16::MIN, i16::MAX] {
        rt(v);
    }
    for v in [0i32, -1, -70000, i32::MIN, i32::MAX] {
        rt(v);
    }
    for v in [0i64, -1, i64::MIN, i64::MAX] {
        rt(v);
    }
}

#[test]
fn floats_round_trip_including_specials() {
    for v in [0.0f32, -0.0, 3.5, -2.5, f32::MIN, f32::MAX, f32::INFINITY, f32::NEG_INFINITY] {
        rt(v);
    }
    for v in [0.0f64, -0.0, 3.5, -2.5, f64::MIN, f64::MAX, f64::INFINITY, f64::NEG_INFINITY] {
        rt(v);
    }
    // NaN needs a bitwise check (NaN != NaN).
    let back: f32 = deserialize(serialize(f32::NAN).unwrap()).unwrap();
    assert!(back.is_nan());
}

#[test]
fn char_round_trips() {
    for c in ['a', 'Z', '9', '€', '🦀', '\u{0}', char::MAX] {
        rt(c);
    }
}

#[test]
fn strings_round_trip() {
    for s in ["", "hi", "unicode: €🦀✓", &"x".repeat(500)] {
        rt(s.to_string());
    }
}

#[test]
fn empty_and_nested_collections_round_trip() {
    rt(Vec::<u8>::new());
    rt(Vec::<String>::new());
    rt(vec![vec![1u16, 2], Vec::new(), vec![3, 4, 5]]);
    rt(vec![Some(1u8), None, Some(255)]);
    rt(Some(vec![1u32, 2, 3]));
    rt(Option::<Vec<u32>>::None);
    rt(vec!["alpha".to_string(), "".to_string(), "gamma".to_string()]);
}

#[test]
fn fixed_arrays_round_trip() {
    rt([1u8, 2, 3]);
    rt([0i32, -1, 70000, i32::MIN]);
    rt([[1u8, 2], [3, 4], [5, 6]]);
}

#[test]
fn hashmap_with_string_keys_round_trips() {
    let mut m: HashMap<String, Vec<u16>> = HashMap::new();
    m.insert("a".into(), vec![1, 2, 3]);
    m.insert("bb".into(), vec![]);
    m.insert("ccc".into(), vec![70, 80]);
    rt(m);
}
