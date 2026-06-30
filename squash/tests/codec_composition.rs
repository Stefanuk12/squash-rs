//! Composition and edge coverage for the `codec` (`SerDes`) API. `codec.rs`
//! covers each codec once in isolation; this exercises them nested
//! inside one another (the real v5 usage), plus the thinner spots: `buffer()`,
//! fixed-width truncation/padding for `string(n)`/`buffer(n)`, and the `Range`
//! byte-width boundaries (1/2/3/4/8 bytes).

use std::collections::HashMap;

use squash::codec::{
    array, array_n, buffer, buffer_n, from_bytes, literal, map, opt, range, string, string_n,
    to_bytes, ArrayLen, BitArray, BitArrayN, Range, SerDes, Uint, VlqCodec, F64, I32, U16, U32, U8,
};

fn codec_rt<S: SerDes>(sd: &S, v: S::Value)
where
    S::Value: PartialEq + std::fmt::Debug,
{
    let bytes = to_bytes(sd, &v).unwrap();
    let back = from_bytes(sd, bytes).unwrap();
    assert_eq!(back, v, "codec round-trip");
}

#[test]
fn nested_codecs_round_trip() {
    // array of optionals
    codec_rt(&array(opt(U16)), vec![Some(1u16), None, Some(0xABCD)]);
    // optional array
    codec_rt(&opt(array(U8)), Some(vec![1u8, 2, 3]));
    codec_rt(&opt(array(U8)), None);
    // map of string -> array
    let mut m: HashMap<String, Vec<u16>> = HashMap::new();
    m.insert("a".into(), vec![1, 2]);
    m.insert("bb".into(), vec![]);
    codec_rt(&map(string(), array(U16)), m);
    // array of arrays
    codec_rt(&array(array(U8)), vec![vec![1u8, 2], vec![], vec![3, 4, 5]]);
}

#[test]
fn tuple_of_codecs_preserves_order_and_types() {
    let t = (string(), array(U8), opt(U32));
    let v = ("hello".to_string(), vec![1u8, 2, 3], Some(70000u32));
    codec_rt(&t, v);

    // heterogeneous numeric tuple
    let t2 = (U8, I32, F64);
    codec_rt(&t2, (9u8, -123456i32, 2.5f64));
}

#[test]
fn buffer_wire_and_round_trip() {
    // variable buffer: raw bytes then VLQ length suffix
    assert_eq!(to_bytes(&buffer(), &vec![1u8, 2, 3]).unwrap(), vec![1, 2, 3, 0x03]);
    codec_rt(&buffer(), vec![0u8, 255, 7, 42]);
    codec_rt(&buffer(), Vec::<u8>::new());
}

#[test]
fn fixed_buffer_truncates_and_pads() {
    let b = buffer_n(4);
    // exact length round-trips
    codec_rt(&b, vec![1u8, 2, 3, 4]);
    // shorter input is zero-padded to n on the wire
    assert_eq!(to_bytes(&b, &vec![1u8, 2]).unwrap(), vec![1, 2, 0, 0]);
    assert_eq!(from_bytes(&b, to_bytes(&b, &vec![1u8, 2]).unwrap()).unwrap(), vec![1, 2, 0, 0]);
    // longer input is truncated to n
    assert_eq!(to_bytes(&b, &vec![1u8, 2, 3, 4, 5, 6]).unwrap(), vec![1, 2, 3, 4]);
}

#[test]
fn fixed_string_truncates_and_pads() {
    let s = string_n(4);
    codec_rt(&s, "abcd".to_string());
    // shorter string keeps trailing NULs after round-trip (no length on the wire)
    assert_eq!(to_bytes(&s, &"ab".to_string()).unwrap(), vec![0x61, 0x62, 0, 0]);
    assert_eq!(from_bytes(&s, to_bytes(&s, &"ab".to_string()).unwrap()).unwrap(), "ab\0\0");
    // longer string is truncated
    assert_eq!(from_bytes(&s, to_bytes(&s, &"abcdef".to_string()).unwrap()).unwrap(), "abcd");
}

#[test]
fn fixed_string_truncates_multibyte_at_char_boundary() {
    // Regression: truncating mid-codepoint produced bytes that StrN's own `des`
    // (String::from_utf8) could not decode, so ser succeeded but de of that same
    // output errored. Truncation must fall back to a char boundary.
    let s = string_n(2);
    // "é" is 2 bytes (0xC3 0xA9). "aé" is 3 bytes; truncating to 2 would split 'é'.
    let out = to_bytes(&s, &"aé".to_string()).unwrap();
    assert_eq!(out.len(), 2);
    // round-trip of the codec's own output must not error; the split char is dropped.
    assert_eq!(from_bytes(&s, out).unwrap(), "a\0");
    // a string that exactly fills n with a whole codepoint is preserved.
    assert_eq!(from_bytes(&s, to_bytes(&s, &"é".to_string()).unwrap()).unwrap(), "é");
}

#[test]
fn range_byte_width_boundaries() {
    // 1 byte: diff 255
    let r1 = range(0, 255);
    assert_eq!(to_bytes(&r1, &255).unwrap().len(), 1);
    // 2 bytes: diff 256
    let r2 = range(0, 256);
    assert_eq!(to_bytes(&r2, &256).unwrap().len(), 2);
    // 3 bytes: diff fits in 24 bits
    let r3 = range(0, 0xFFFFFF);
    assert_eq!(to_bytes(&r3, &0xFFFFFF).unwrap().len(), 3);
    // 4 bytes
    let r4 = range(0, 0xFFFF_FFFF);
    assert_eq!(to_bytes(&r4, &0xFFFF_FFFF).unwrap().len(), 4);
    // 8 bytes: huge diff
    let r8 = range(0, i64::MAX);
    assert_eq!(to_bytes(&r8, &i64::MAX).unwrap().len(), 8);

    for r in [r1, r2, r3, r4, r8] {
        for v in [r.min, r.min + 1, (r.min + r.max) / 2, r.max] {
            codec_rt(&r, v);
        }
    }
}

#[test]
fn range_spanning_zero_round_trips() {
    let r = range(-1000, 1000);
    for v in [-1000i64, -1, 0, 1, 1000] {
        codec_rt(&r, v);
    }
    // min < 0 < max with a diff that exceeds i64 range arithmetic safely
    let wide = Range { min: i64::MIN, max: i64::MAX };
    codec_rt(&wide, 0i64);
    codec_rt(&wide, i64::MIN);
    codec_rt(&wide, i64::MAX);
}

#[test]
fn uint_fixed_widths_round_trip() {
    for bytes in 1u8..=8 {
        let u = Uint(bytes);
        let max = if bytes == 8 { u64::MAX } else { (1u64 << (8 * bytes as u32)) - 1 };
        for v in [0u64, 1, max] {
            assert_eq!(to_bytes(&u, &v).unwrap().len(), bytes as usize);
            codec_rt(&u, v);
        }
    }
}

#[test]
fn array_len_with_custom_count_codecs() {
    // VLQ-counted array
    codec_rt(&ArrayLen(U8, VlqCodec), vec![1u8, 2, 3, 4, 5]);
    // fixed-width-counted array
    codec_rt(&ArrayLen(U16, Uint(2)), vec![10u16, 20, 30]);
    codec_rt(&ArrayLen(U8, Uint(1)), Vec::<u8>::new());
}

#[test]
fn literal_of_various_types() {
    let colors = literal(vec!["red".to_string(), "green".to_string(), "blue".to_string()]);
    codec_rt(&colors, "green".to_string());
    let nums = literal(vec![10u32, 20, 30, 40]);
    codec_rt(&nums, 30u32);
}

#[test]
fn bit_arrays_round_trip_large() {
    codec_rt(&BitArray, Vec::<bool>::new());
    let v: Vec<bool> = (0..37).map(|i| i % 3 == 0).collect();
    codec_rt(&BitArray, v.clone());
    codec_rt(&BitArrayN(37), v);
    codec_rt(&array_n(U16, 3), vec![1u16, 2, 3]);
}

#[test]
fn fixed_array_of_zero_width_elements_round_trips() {
    // Regression: the zero-width DoS guard in `read_elems` is for hostile WIRE
    // counts only. A fixed `ArrayN` length is schema-supplied (trusted), so a
    // fixed array of zero-width elements must round-trip even though `ser`
    // writes zero bytes for it — otherwise the codec rejects its own output.
    let s = array_n(string_n(0), 3);
    assert_eq!(to_bytes(&s, &vec![String::new(), String::new(), String::new()]).unwrap(), Vec::<u8>::new());
    codec_rt(&s, vec![String::new(), String::new(), String::new()]);
    codec_rt(&array_n(buffer_n(0), 2), vec![Vec::<u8>::new(), Vec::<u8>::new()]);
    // nested: a fixed array of fixed-zero-width arrays serializes to zero bytes.
    codec_rt(&array_n(array_n(U8, 0), 3), vec![Vec::<u8>::new(), Vec::<u8>::new(), Vec::<u8>::new()]);
    // a zero-length fixed array is the trivial empty case.
    codec_rt(&array_n(U8, 0), Vec::<u8>::new());
}
