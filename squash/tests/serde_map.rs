//! Regression for the serde map serializer (ser.rs `MapSerializer`).
//!
//! Two bugs were fixed:
//! 1. A single shared `Serializer.counter` was corrupted by nesting: a map
//!    serialized as another map's value inflated/reset the outer count, so each
//!    level emitted the wrong `Vlq(len)` and `deserialize_map` read the wrong
//!    number of entries (silent data loss). `MapSerializer` now carries its own
//!    per-map `count`.
//! 2. The separate `serialize_key` + `serialize_value` API wrote key-then-value
//!    (reverse of `serialize_entry`'s value-then-key) and never incremented the
//!    count, so maps driven that way never round-tripped.
#![cfg(feature = "serde")]

use std::collections::HashMap;

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
fn flat_map_round_trips() {
    let mut m = HashMap::new();
    m.insert(1u8, 10u16);
    m.insert(2u8, 20u16);
    m.insert(3u8, 30u16);
    serde_rt(m);
}

#[test]
fn nested_map_round_trips() {
    // The shared-counter bug surfaced here: the inner map's `end()` emitted the
    // accumulated outer+inner count and reset the outer count to zero, so the
    // outer map decoded a wrong entry count and dropped data.
    let mut inner_a = HashMap::new();
    inner_a.insert(10u8, 100u16);
    let mut inner_b = HashMap::new();
    inner_b.insert(20u8, 200u16);
    inner_b.insert(21u8, 201u16);

    let mut outer: HashMap<u8, HashMap<u8, u16>> = HashMap::new();
    outer.insert(1, inner_a);
    outer.insert(2, inner_b);
    serde_rt(outer);
}

#[test]
fn map_value_is_seq_round_trips() {
    let mut m: HashMap<u8, Vec<u16>> = HashMap::new();
    m.insert(1, vec![1, 2, 3]);
    m.insert(2, vec![]);
    m.insert(3, vec![9]);
    serde_rt(m);
}
