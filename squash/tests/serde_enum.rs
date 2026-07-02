//! Regressions for two serde round-trip bugs surfaced by the `enum_serde` example:
//!
//! 1. `impl_serde_for_enum!` wrote the variant tag as a bare integer literal,
//!    which defaults to `i32` (4 bytes), while the deserializer reads the tag as
//!    a `u8` (1 byte). The 3 stray bytes shifted every field read, so e.g.
//!    `Bar { a: 2, b: 3, c: 4, d: 5 }` decoded as `Bar { a: 0, b: 1024, c: 5, d: 0 }`.
//!
//! 2. `u24`/`u40`/... `visit_newtype_struct` reversed the byte array an extra
//!    time. The tuple serializer already reverses on write and the LIFO cursor
//!    reverses again on read (those cancel), so the manual `x.reverse()` was a
//!    third reversal: a `u24` of `4` round-tripped to `262144` (`4 << 16`).
#![cfg(feature = "serde")]

use squash::{impl_serde_for_enum, serde_deserialize, serde_serialize, u24, ReverseDeserialize};

fn serde_rt<T>(v: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let mut bytes = serde_serialize(&v).unwrap();
    let back: T = serde_deserialize(&mut bytes).unwrap();
    assert_eq!(back, v);
}

#[derive(Debug, PartialEq, serde::Serialize, ReverseDeserialize)]
struct Bar {
    a: u8,
    b: u16,
    c: u24,
    d: u64,
}

#[derive(Debug, PartialEq, serde::Serialize, ReverseDeserialize)]
struct Baz {
    a: f32,
    b: f64,
}

#[derive(Debug, PartialEq)]
enum Foo {
    Bar(Bar),
    Baz(Baz),
}
impl_serde_for_enum!(Foo, Bar = 0, Baz = 1);

#[test]
fn u24_serde_round_trips() {
    serde_rt(u24::new(4).unwrap());
    serde_rt(u24::new(0).unwrap());
    serde_rt(u24::new(0x010203).unwrap());
    serde_rt(u24::new(0xFFFFFF).unwrap());
}

#[test]
fn enum_first_variant_round_trips() {
    serde_rt(Foo::Bar(Bar {
        a: 2,
        b: 3,
        c: u24::new(4).unwrap(),
        d: 5,
    }));
}

#[test]
fn enum_second_variant_round_trips() {
    serde_rt(Foo::Baz(Baz {
        a: 1.5,
        b: -2.25,
    }));
}

/// Native serde derives on enums: every variant kind is framed as payload
/// bytes plus a single trailing declaration-order tag byte, the same layout as
/// `impl_serde_for_enum!` (pinned by `macro_and_native_derive_layouts_match`).
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Native {
    Unit,
    Newtype(u16),
    Tuple(u8, u16),
    Struct { a: u8, b: String },
}

#[test]
fn native_derive_wire_bytes() {
    // Unit variant is nothing but its tag.
    assert_eq!(serde_serialize(&Native::Unit).unwrap(), vec![0x00]);
    // Payload first (u16 little-endian), tag last.
    assert_eq!(
        serde_serialize(&Native::Newtype(7)).unwrap(),
        vec![0x07, 0x00, 0x01]
    );
}

#[test]
fn native_derive_variants_round_trip() {
    serde_rt(Native::Unit);
    serde_rt(Native::Newtype(513));
    serde_rt(Native::Tuple(7, 65535));
    serde_rt(Native::Struct {
        a: 1,
        b: "hi".to_string(),
    });
}

#[test]
fn unknown_tag_is_rejected() {
    // Native has 4 variants, so tag 9 must not decode.
    let mut bytes = vec![0x09];
    assert!(serde_deserialize::<Native>(&mut bytes).is_err());
}

#[test]
fn variant_index_above_255_is_rejected() {
    use serde::ser::Serializer as _;

    let mut ser = squash::Serializer::new(std::io::Cursor::new(Vec::new()));
    let err = (&mut ser).serialize_unit_variant("E", 256, "x").unwrap_err();
    assert!(matches!(err, squash::Error::VariantIndexTooLarge(256)));
}

#[test]
fn variant_index_error_leaves_no_partial_bytes() {
    use serde::ser::Serializer as _;

    let mut ser = squash::Serializer::new(std::io::Cursor::new(Vec::new()));
    let err = (&mut ser)
        .serialize_newtype_variant("E", 300, "x", &0xAABBu16)
        .unwrap_err();
    assert!(matches!(err, squash::Error::VariantIndexTooLarge(300)));
    assert!(ser.get_ref().is_empty());
}

/// `impl_serde_for_enum!` and the native derive must stay wire-compatible:
/// same payload and variant index -> same bytes, decodable by either side.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum FooTwin {
    Bar(Bar),
    Baz(Baz),
}

#[test]
fn macro_and_native_derive_layouts_match() {
    let baz = || Baz { a: 1.5, b: -2.25 };
    let macro_bytes = serde_serialize(&Foo::Baz(baz())).unwrap();
    let derive_bytes = serde_serialize(&FooTwin::Baz(baz())).unwrap();
    assert_eq!(macro_bytes, derive_bytes);

    let mut bytes = macro_bytes;
    assert_eq!(
        serde_deserialize::<FooTwin>(&mut bytes).unwrap(),
        FooTwin::Baz(baz())
    );
    let mut bytes = derive_bytes;
    assert_eq!(
        serde_deserialize::<Foo>(&mut bytes).unwrap(),
        Foo::Baz(baz())
    );
}

/// A recursive enum decodes one nesting level per tag byte, so a small forged
/// input must hit the depth limit as an `Err` instead of overflowing the
/// stack and aborting the process.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Rec {
    Node(Box<Rec>),
    Leaf,
}

#[test]
fn recursion_depth_is_bounded() {
    // [Leaf tag, N Node tags]: the LIFO cursor pops a Node tag per level.
    let deep = |n: usize| {
        let mut bytes = vec![0x00; n + 1];
        bytes[0] = 0x01;
        bytes
    };

    let mut ok = deep(100);
    assert!(serde_deserialize::<Rec>(&mut ok).is_ok());

    let mut too_deep = deep(100_000);
    assert!(matches!(
        serde_deserialize::<Rec>(&mut too_deep),
        Err(squash::Error::RecursionDepthExceeded)
    ));
}
