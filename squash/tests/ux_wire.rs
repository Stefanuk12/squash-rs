//! Native wire bytes for the custom-width integers. These live outside any
//! feature gate (unlike `ux_types.rs`, which also drives the serde path) because
//! the little-endian, truncated layout is a property of the `SquashObject` path
//! alone and must hold even with `--no-default-features`.

use squash::{deserialize, serialize, u24, u40, u48, u56};

#[test]
fn ux_little_endian_wire() {
    assert_eq!(
        serialize(u24::try_from(0x010203u32).unwrap()).unwrap(),
        vec![0x03, 0x02, 0x01]
    );
    assert_eq!(
        serialize(u40::try_from(0x0102030405u64).unwrap()).unwrap(),
        vec![0x05, 0x04, 0x03, 0x02, 0x01]
    );
}

#[test]
fn ux_round_trips() {
    let a = u48::try_from(0x010203040506u64).unwrap();
    assert_eq!(deserialize::<u48>(serialize(a).unwrap()).unwrap(), a);
    let b = u56::try_from(0x01020304050607u64).unwrap();
    assert_eq!(deserialize::<u56>(serialize(b).unwrap()).unwrap(), b);
    let c = u24::try_from(0u32).unwrap();
    assert_eq!(deserialize::<u24>(serialize(c).unwrap()).unwrap(), c);
}
