//! Verifies the `#[derive(SquashObject)]` record codegen matches v5 `record`
//! semantics: fields are sorted by name and bool/opt fields are bit-packed.
use squash::{deserialize, serialize, SquashObject};

#[derive(SquashObject, Clone, PartialEq, Debug, Default)]
struct Rec {
    name: String,
    age: u16,
    active: bool,
    nickname: Option<String>,
    verified: Option<bool>,
    score: u32,
}

#[test]
fn record_round_trips_all_present() {
    let r = Rec {
        name: "alice".into(),
        age: 30,
        active: true,
        nickname: Some("al".into()),
        verified: Some(false),
        score: 1000,
    };
    let back: Rec = deserialize(serialize(r.clone()).unwrap()).unwrap();
    assert_eq!(back, r);
}

#[test]
fn record_round_trips_options_none() {
    let r = Rec {
        name: "bob".into(),
        age: 0,
        active: false,
        nickname: None,
        verified: None,
        score: 42,
    };
    let back: Rec = deserialize(serialize(r.clone()).unwrap()).unwrap();
    assert_eq!(back, r);
}

#[test]
fn record_round_trips_mixed_option_presence() {
    // Guards the opt-vs-optbool flag-index boundary (`__flags[i + optbool_count]`
    // vs `__flags[i]`): exercise each Option present while the OTHER is absent,
    // so a wrong flag index would read the wrong presence bit and corrupt decode.
    let with_bool_only = Rec {
        name: "carol".into(),
        age: 7,
        active: true,
        nickname: None,            // opt absent
        verified: Some(true),      // optbool present
        score: 9,
    };
    let with_str_only = Rec {
        name: "dave".into(),
        age: 8,
        active: false,
        nickname: Some("d".into()), // opt present
        verified: None,             // optbool absent
        score: 11,
    };
    for r in [with_bool_only, with_str_only] {
        let back: Rec = deserialize(serialize(r.clone()).unwrap()).unwrap();
        assert_eq!(back, r);
    }
}

#[derive(SquashObject, Clone, PartialEq, Debug, Default)]
struct ManyBools {
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    e: bool,
    f: bool,
    g: bool,
    h: bool,
    i: bool,
    n: u8,
}

#[test]
fn record_packs_nine_bools_into_two_bytes() {
    let r = ManyBools {
        a: true, b: false, c: true, d: true, e: false, f: false, g: true, h: false, i: true,
        n: 7,
    };
    // 1 regular byte (n) + 2 bytes for 9 packed bools + 0 flag bytes (no opts)
    assert_eq!(serialize(r.clone()).unwrap().len(), 1 + 2);
    let back: ManyBools = deserialize(serialize(r.clone()).unwrap()).unwrap();
    assert_eq!(back, r);
}
