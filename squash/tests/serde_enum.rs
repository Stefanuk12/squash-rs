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
