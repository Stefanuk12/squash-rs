//! Coverage for the custom-width integers (`u24/u40/u48/u56`, `i24/i40/i48/i56`).
//!
//! These all come from the one `impl_custom_int!` macro, so a bug in it hits
//! every width at once. In particular the serde `Deserialize` (`visit_newtype_struct`)
//! had an extra `x.reverse()` that made a `u24` of `4` decode as `262144`; only
//! `u24` was guarded afterwards. This file round-trips *every* width through both
//! the native and serde paths, plus checks wire bytes and the value-too-large /
//! negative-rejection edges.
#![cfg(all(feature = "serde", feature = "roblox"))]

use squash::{
    deserialize, i24, i40, i48, i56, serde_deserialize, serde_serialize, serialize, u24, u40, u48,
    u56,
};

fn native_rt<T>(v: T)
where
    T: squash::SquashObject + Clone + PartialEq + std::fmt::Debug,
{
    let back: T = deserialize(serialize(v.clone()).unwrap()).unwrap();
    assert_eq!(back, v, "native round-trip");
}

fn serde_rt<T>(v: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let mut bytes = serde_serialize(&v).unwrap();
    let back: T = serde_deserialize(&mut bytes).unwrap();
    assert_eq!(back, v, "serde round-trip");
}

#[test]
fn unsigned_widths_round_trip_both_paths() {
    for v in [0u32, 1, 0xFF, 0x100, 0x010203, 0xFFFFFF] {
        let x = u24::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0u64, 1, 0xFF, 0x0102030405, 0xFF_FFFF_FFFF] {
        let x = u40::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0u64, 1, 0x010203040506, 0xFFFF_FFFF_FFFF] {
        let x = u48::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0u64, 1, 0x01020304050607, 0xFF_FFFF_FFFF_FFFF] {
        let x = u56::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
}

#[test]
fn signed_widths_round_trip_both_paths() {
    // NOTE: the signed narrow ints currently reject negatives (see
    // `signed_narrow_ints_reject_negatives`), so only non-negative values here.
    for v in [0i32, 1, 0x7F, 0x010203, 0x7FFFFF] {
        let x = i24::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0i64, 1, 0x0102030405, 0x7F_FFFF_FFFF] {
        let x = i40::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0i64, 1, 0x010203040506] {
        let x = i48::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
    for v in [0i64, 1, 0x01020304050607] {
        let x = i56::new(v).unwrap();
        native_rt(x);
        serde_rt(x);
    }
}

#[test]
fn native_wire_is_little_endian_truncated() {
    // Mirrors `ux_wire.rs::ux_little_endian_wire` for the remaining widths.
    assert_eq!(serialize(u48::new(0x010203040506).unwrap()).unwrap(), vec![6, 5, 4, 3, 2, 1]);
    assert_eq!(
        serialize(u56::new(0x01020304050607).unwrap()).unwrap(),
        vec![7, 6, 5, 4, 3, 2, 1]
    );
    assert_eq!(serialize(i24::new(0x010203).unwrap()).unwrap(), vec![3, 2, 1]);
}

#[test]
fn value_too_large_is_rejected() {
    assert!(u24::new(0x1000000).is_err());
    assert!(u40::new(1u64 << 40).is_err());
    assert!(u48::new(1u64 << 48).is_err());
    assert!(u56::new(1u64 << 56).is_err());
    // boundary just below the limit constructs fine
    assert!(u24::new(0xFFFFFF).is_ok());
    assert!(u56::new((1u64 << 56) - 1).is_ok());
}

#[test]
fn signed_narrow_ints_reject_negatives() {
    // Documents a known limitation: the `to_le_bytes` high-byte check treats the
    // sign-extension bytes of a negative as overflow, so negatives don't fit.
    assert!(i24::new(-1).is_err());
    assert!(i40::new(-1).is_err());
    assert!(i48::new(-1).is_err());
    assert!(i56::new(-1).is_err());
}

#[test]
fn ux_value_conversions_round_trip() {
    // `new(x).convert() == x` for representable values.
    assert_eq!(u24::new(0x123456).unwrap().convert(), 0x123456);
    assert_eq!(u40::new(0x0102030405).unwrap().convert(), 0x0102030405);
    assert_eq!(i24::new(0x7FFFFF).unwrap().convert(), 0x7FFFFF);
}
