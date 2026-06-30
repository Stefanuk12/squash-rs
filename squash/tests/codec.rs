//! Per-codec isolation coverage for the `codec` (`SerDes`) API: each codec is
//! exercised once on its own for exact wire bytes plus a round-trip, and for the
//! edges that must error rather than silently truncate (out-of-range `Range`,
//! short fixed `ArrayN`, oversized `Literal`). `codec_composition.rs` builds on
//! this by nesting the same codecs inside one another.

use std::collections::HashMap;

use squash::codec::{
    array, array_n, from_bytes, literal, map, opt, range, string, string_n, to_bytes, ArrayLen,
    BitArray, BitArrayN, Uint, U16, U8,
};

#[test]
fn range_wire_bytes_and_round_trip() {
    // diff = 990 -> 2 bytes; value 1000 -> x = 990 = 0x03DE -> LE [DE, 03]
    let r = range(10, 1000);
    assert_eq!(to_bytes(&r, &1000).unwrap(), vec![0xDE, 0x03]);
    assert_eq!(to_bytes(&r, &10).unwrap(), vec![0x00, 0x00]);
    for v in [10i64, 11, 500, 999, 1000] {
        assert_eq!(from_bytes(&r, to_bytes(&r, &v).unwrap()).unwrap(), v);
    }
    // single-byte range, negative min
    let r2 = range(-5, 200);
    assert_eq!(to_bytes(&r2, &-5).unwrap(), vec![0x00]);
    assert_eq!(to_bytes(&r2, &0).unwrap(), vec![0x05]);
    for v in [-5i64, -1, 0, 100, 200] {
        assert_eq!(from_bytes(&r2, to_bytes(&r2, &v).unwrap()).unwrap(), v);
    }
}

#[test]
fn range_rejects_out_of_bounds_instead_of_truncating() {
    // diff = 1000 -> 2-byte width; 70000 would `as u16`-truncate to 4464,
    // and a value below min would wrap to a huge unsigned. Both must error.
    let r = range(0, 1000);
    assert!(to_bytes(&r, &70000).is_err());
    assert!(to_bytes(&r, &-1).is_err());
    // in-range values still encode
    assert_eq!(to_bytes(&r, &1000).unwrap(), vec![0xE8, 0x03]);
}

#[test]
fn fixed_array_rejects_short_input() {
    // ArrayN writes exactly n elements; a short Vec would under-write and
    // misalign everything after it, so ser must reject rather than corrupt.
    let an = array_n(U16, 3);
    assert!(to_bytes(&an, &vec![1u16, 2]).is_err());
    assert_eq!(to_bytes(&an, &vec![1u16, 2, 3]).unwrap(), vec![1, 0, 2, 0, 3, 0]);
}

#[test]
fn fixed_string_has_no_length_prefix() {
    let s = string_n(3);
    assert_eq!(to_bytes(&s, &"abc".to_string()).unwrap(), vec![0x61, 0x62, 0x63]);
    assert_eq!(from_bytes(&s, to_bytes(&s, &"abc".to_string()).unwrap()).unwrap(), "abc");
    // variable string still carries the vlq length
    assert_eq!(to_bytes(&string(), &"hi".to_string()).unwrap(), vec![0x68, 0x69, 0x02]);
}

#[test]
fn array_matches_native_and_preserves_order() {
    let a = array(U8);
    assert_eq!(to_bytes(&a, &vec![10u8, 20, 30]).unwrap(), vec![10, 20, 30, 0x03]);
    assert_eq!(
        from_bytes(&a, to_bytes(&a, &vec![1u8, 2, 3, 4]).unwrap()).unwrap(),
        vec![1u8, 2, 3, 4]
    );
    // fixed-length array: no count byte
    let an = array_n(U16, 3);
    assert_eq!(to_bytes(&an, &vec![1u16, 2, 3]).unwrap(), vec![1, 0, 2, 0, 3, 0]);
    assert_eq!(
        from_bytes(&an, to_bytes(&an, &vec![5u16, 6, 7]).unwrap()).unwrap(),
        vec![5u16, 6, 7]
    );
}

#[test]
fn opt_and_map_round_trip() {
    let o = opt(U16);
    assert_eq!(to_bytes(&o, &None).unwrap(), vec![0x00]);
    assert_eq!(to_bytes(&o, &Some(7u16)).unwrap(), vec![0x07, 0x00, 0x01]);
    assert_eq!(from_bytes(&o, to_bytes(&o, &Some(513u16)).unwrap()).unwrap(), Some(513));

    let m = map(U8, U16);
    let mut hm = HashMap::new();
    hm.insert(1u8, 100u16);
    hm.insert(2u8, 200u16);
    assert_eq!(from_bytes(&m, to_bytes(&m, &hm).unwrap()).unwrap(), hm);
}

#[test]
fn tuple_round_trip_and_order() {
    let t = (U8, U16);
    assert_eq!(to_bytes(&t, &(1u8, 0x0203u16)).unwrap(), vec![0x01, 0x03, 0x02]);
    assert_eq!(
        from_bytes(&t, to_bytes(&t, &(9u8, 0x1122u16)).unwrap()).unwrap(),
        (9, 0x1122)
    );
    let t3 = (U8, U8, U8);
    assert_eq!(from_bytes(&t3, to_bytes(&t3, &(1u8, 2, 3)).unwrap()).unwrap(), (1, 2, 3));
}

#[test]
fn bitarray_packs_and_round_trips() {
    let b = BitArray;
    // [true,false,true] -> partial byte 0b101 = 0x05, then vlq count 3
    assert_eq!(to_bytes(&b, &vec![true, false, true]).unwrap(), vec![0x05, 0x03]);
    let v = vec![true, true, false, false, true, false, true, true, true];
    assert_eq!(from_bytes(&b, to_bytes(&b, &v).unwrap()).unwrap(), v);
    let bn = BitArrayN(10);
    let v2 = vec![true, false, true, true, false, false, true, false, true, true];
    assert_eq!(from_bytes(&bn, to_bytes(&bn, &v2).unwrap()).unwrap(), v2);
}

#[test]
fn array_len_uses_custom_count_codec() {
    let a = ArrayLen(U8, Uint(2));
    assert_eq!(to_bytes(&a, &vec![10u8, 20]).unwrap(), vec![10, 20, 0x02, 0x00]);
    assert_eq!(
        from_bytes(&a, to_bytes(&a, &vec![1u8, 2, 3]).unwrap()).unwrap(),
        vec![1u8, 2, 3]
    );
}

#[test]
fn literal_encodes_one_byte_index() {
    let l = literal(vec!["red".to_string(), "green".to_string(), "blue".to_string()]);
    assert_eq!(to_bytes(&l, &"red".to_string()).unwrap(), vec![0x00]);
    assert_eq!(to_bytes(&l, &"blue".to_string()).unwrap(), vec![0x02]);
    assert_eq!(
        from_bytes(&l, to_bytes(&l, &"green".to_string()).unwrap()).unwrap(),
        "green"
    );
    assert!(to_bytes(&l, &"purple".to_string()).is_err());
}

#[test]
fn literal_with_more_than_256_entries_errors_instead_of_truncating() {
    // Index 256 would wrap to 0 in a single byte; ser must reject it rather
    // than silently round-trip to the wrong (0th) value.
    let l = literal((0..300u32).collect::<Vec<_>>());
    assert!(to_bytes(&l, &256u32).is_err());
    // indices that still fit in one byte keep working
    assert_eq!(to_bytes(&l, &255u32).unwrap(), vec![0xFF]);
}
