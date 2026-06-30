//! VLQ wire format. The exact bytes must match Squash Luau v5.0.0's
//! `pushvlqrealloc`, since the length prefix of every `array`/`string`/`map` is a
//! VLQ — a single off-by-one here corrupts every container in the format.

use squash::{deserialize, serialize, Vlq};

/// Exact wire bytes must match Squash Luau v5.0.0 `pushvlqrealloc`.
#[test]
fn vlq_wire_bytes_match_v5() {
    let cases: &[(u64, &[u8])] = &[
        (0, &[0x00]),
        (5, &[0x05]),
        (127, &[0x7F]),
        (128, &[0x00, 0x81]),
        (300, &[0x2C, 0x82]),
        (16383, &[0x7F, 0xFF]),
        (16384, &[0x00, 0x80, 0x81]),
        (70000, &[0x70, 0xA2, 0x84]),
    ];
    for (value, expected) in cases {
        let got = serialize(Vlq(*value)).unwrap();
        assert_eq!(&got, expected, "Vlq({value}) wire bytes");
    }
}

#[test]
fn vlq_round_trips() {
    let values: &[u64] = &[
        0,
        1,
        127,
        128,
        255,
        256,
        300,
        16383,
        16384,
        70000,
        1 << 21,
        1 << 28,
        1 << 35,
        1 << 42,
        1 << 49,
        (1 << 49) + 12345,
        (1 << 55) + 777,
    ];
    for &v in values {
        let bytes = serialize(Vlq(v)).unwrap();
        let back: Vlq = deserialize(bytes).unwrap();
        assert_eq!(back.0, v, "round-trip Vlq({v})");
    }
}
