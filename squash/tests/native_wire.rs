//! Exact v5 wire bytes for the built-in `SquashObject` container impls
//! (`Option`/`String`/`Vec`/`[T; N]`/`HashMap`). `native_roundtrip.rs` proves
//! these survive a round-trip; this pins the actual bytes so a regression in the
//! LIFO ordering or the VLQ-suffix layout fails loudly against upstream v5.0.0.

use squash::{deserialize, serialize};

/// Squash v5 `opt`: None -> [0x00]; Some(v) -> [<v bytes>, 0x01].
#[test]
fn option_wire_bytes_match_v5() {
    assert_eq!(serialize::<Option<u8>>(None).unwrap(), vec![0x00]);
    assert_eq!(serialize::<Option<u8>>(Some(7)).unwrap(), vec![0x07, 0x01]);
}

#[test]
fn option_round_trips() {
    for v in [None, Some(0u8), Some(7u8), Some(255u8)] {
        let bytes = serialize::<Option<u8>>(v).unwrap();
        let back: Option<u8> = deserialize(bytes).unwrap();
        assert_eq!(back, v, "round-trip {v:?}");
    }
}

/// Cascade of the VLQ fix: v5 `string` writes raw bytes then a VLQ length
/// suffix, so "hi" -> [0x68, 0x69, 0x02].
#[test]
fn string_matches_v5_and_round_trips() {
    assert_eq!(serialize(String::from("hi")).unwrap(), vec![0x68, 0x69, 0x02]);
    for s in ["", "hi", "a longer string that exceeds a single vlq byte ".repeat(4).as_str()] {
        let bytes = serialize(String::from(s)).unwrap();
        let back: String = deserialize(bytes).unwrap();
        assert_eq!(back, s, "round-trip string");
    }
}

/// v5 `array`: elements forward, count as a VLQ suffix.
#[test]
fn vec_matches_v5_and_preserves_order() {
    assert_eq!(serialize(vec![10u8, 20, 30]).unwrap(), vec![10, 20, 30, 0x03]);
    let v = vec![1u16, 2, 3, 4];
    let back: Vec<u16> = deserialize(serialize(v.clone()).unwrap()).unwrap();
    assert_eq!(back, v, "Vec order preserved");
    let empty: Vec<u8> = vec![];
    assert_eq!(serialize(empty.clone()).unwrap(), vec![0x00]);
    assert_eq!(deserialize::<Vec<u8>>(serialize(empty.clone()).unwrap()).unwrap(), empty);
}

#[test]
fn fixed_array_preserves_order() {
    assert_eq!(serialize([1u8, 2, 3]).unwrap(), vec![1, 2, 3]);
    let a = [1u16, 2, 3, 4, 5];
    let back: [u16; 5] = deserialize(serialize(a).unwrap()).unwrap();
    assert_eq!(back, a, "[T; N] order preserved");
}

#[test]
fn hashmap_round_trips() {
    let mut m = std::collections::HashMap::new();
    m.insert(1u8, 100u16);
    m.insert(2u8, 200u16);
    m.insert(3u8, 300u16);
    let back: std::collections::HashMap<u8, u16> =
        deserialize(serialize(m.clone()).unwrap()).unwrap();
    assert_eq!(back, m, "HashMap round-trip");
}
