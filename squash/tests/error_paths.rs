//! Error-path coverage. `codec.rs` checks a few rejections (range
//! out-of-bounds, oversized literal); this collects the rest so that the "this
//! must error, not silently corrupt / panic" contracts are pinned down: invalid
//! UTF-8, VLQ overflow and non-termination, custom-int overflow, an empty-string
//! decoded as `char`, and the codec-level out-of-range / length mismatches.

use squash::codec::{
    array, array_n, from_bytes, literal, range, to_bytes, ArrayLen, BitArray, Uint, U16, U8,
};
use squash::{deserialize, serialize, u24, u40, u48, u56, Vlq};

#[test]
fn string_with_invalid_utf8_errors() {
    // wire = [raw bytes..., vlq len]; 0xFF 0xFE is not valid UTF-8
    let bytes = vec![0xFFu8, 0xFE, 0x02];
    assert!(deserialize::<String>(bytes).is_err());
}

#[test]
fn vlq_push_overflow_errors() {
    // values >= 1<<56 are not representable and must be rejected, not truncated
    assert!(serialize(Vlq(1u64 << 56)).is_err());
    assert!(serialize(Vlq(u64::MAX)).is_err());
    // largest representable still encodes
    assert!(serialize(Vlq((1u64 << 56) - 1)).is_ok());
}

#[test]
fn vlq_non_terminated_errors() {
    // 8 continuation bytes (high bit set) with no terminator is a malformed VLQ
    assert!(deserialize::<Vlq>(vec![0x80u8; 8]).is_err());
}

#[test]
fn custom_int_overflow_errors() {
    assert!(u24::new(1u32 << 24).is_err());
    assert!(u40::new(1u64 << 40).is_err());
    assert!(u48::new(1u64 << 48).is_err());
    assert!(u56::new(1u64 << 56).is_err());
}

#[test]
fn empty_string_decoded_as_char_errors() {
    // serialize("") -> [0x00]; decoding a char needs at least one char
    let empty = serialize(String::from("")).unwrap();
    assert!(deserialize::<char>(empty).is_err());
}

#[test]
fn range_out_of_bounds_errors() {
    let r = range(0, 1000);
    assert!(to_bytes(&r, &70000).is_err()); // above max
    assert!(to_bytes(&r, &-1).is_err()); // below min
    assert!(to_bytes(&r, &1000).is_ok()); // boundary ok
    assert!(to_bytes(&r, &0).is_ok());
}

#[test]
fn fixed_array_length_mismatch_errors() {
    let an = array_n(U16, 3);
    assert!(to_bytes(&an, &vec![1u16, 2]).is_err()); // too few under-writes the stream
    assert!(to_bytes(&an, &vec![1u16, 2, 3]).is_ok());
    assert!(to_bytes(&an, &vec![1u16, 2, 3, 4]).is_err()); // too many would silently drop the tail
}

#[test]
fn uint_width_out_of_range_errors() {
    // Uint is documented as a fixed 1..=8 byte width; a width of 0 or >8 must
    // error rather than silently fall back to an 8-byte u64 and misalign the stream.
    assert!(to_bytes(&Uint(0), &5u64).is_err());
    assert!(to_bytes(&Uint(9), &5u64).is_err());
    // a valid width still round-trips at exactly that many bytes.
    assert_eq!(to_bytes(&Uint(3), &5u64).unwrap().len(), 3);
}

#[test]
fn uint_value_too_large_for_width_errors() {
    // A valid width but an out-of-width VALUE must error, not silently `as u8`/`as u16`
    // truncate (which would emit 44 for 300 and misalign the stream on decode). Mirrors
    // the `Range` out-of-range contract, since both share `write_uint`.
    assert!(to_bytes(&Uint(1), &300u64).is_err());
    assert!(to_bytes(&Uint(2), &70000u64).is_err());
    assert!(to_bytes(&Uint(4), &(1u64 << 32)).is_err());
    // boundary values that exactly fill the width still encode
    assert_eq!(to_bytes(&Uint(1), &255u64).unwrap(), vec![0xFF]);
    assert_eq!(to_bytes(&Uint(2), &0xFFFFu64).unwrap(), vec![0xFF, 0xFF]);
    // a too-small fixed-width count codec rejects an over-long array rather than
    // truncating the element count and stranding the tail bytes.
    let big: Vec<u16> = (0..300).collect();
    assert!(to_bytes(&ArrayLen(U16, Uint(1)), &big).is_err());
}

#[test]
fn malformed_length_prefix_errors_without_huge_alloc() {
    // A message that is ONLY a length prefix claiming ~2^40 elements/bytes/bits must
    // error gracefully — not pre-allocate (and OOM) for a count that the remaining
    // bytes cannot possibly back. The guard rejects before allocating.
    let huge_len = serialize(Vlq(1u64 << 40)).unwrap();
    assert!(deserialize::<String>(huge_len.clone()).is_err());
    assert!(deserialize::<Vec<u8>>(huge_len.clone()).is_err());
    assert!(from_bytes(&array(U8), huge_len.clone()).is_err());
    assert!(from_bytes(&BitArray, huge_len).is_err());
}

#[test]
fn range_decode_rejects_out_of_bounds_value() {
    // A forged/corrupted 1-byte payload for range(0,10) can store 200, which `ser`
    // would never produce; `des` must reject it rather than return a value outside
    // the declared bounds.
    let r = range(0, 10);
    assert!(from_bytes(&r, vec![200u8]).is_err());
    // a valid in-range payload still decodes
    assert_eq!(from_bytes(&r, vec![7u8]).unwrap(), 7);
}

#[test]
fn literal_unknown_value_and_bad_index_error() {
    let l = literal(vec!["a".to_string(), "b".to_string()]);
    // value not in the set cannot be encoded
    assert!(to_bytes(&l, &"z".to_string()).is_err());
    // index past the end of the set cannot be decoded
    assert!(squash::codec::from_bytes(&l, vec![0x05]).is_err());
}

#[test]
fn oversized_literal_index_errors() {
    // index 256 doesn't fit in the one-byte tag; must reject rather than wrap to 0
    let big = literal((0..300u32).collect::<Vec<_>>());
    assert!(to_bytes(&big, &256u32).is_err());
    assert!(to_bytes(&big, &255u32).is_ok());
}
