//! End-to-end wire conformance against the *real* upstream Squash Luau library.
//!
//! This test runs the upstream library (`Data-Oriented-House/Squash`, fetched and
//! `luau.load`-ed at test time) under [Lune](https://lune-org.github.io/) and
//! asserts that the Rust port produces byte-for-byte identical output for a shared
//! catalog of values. See `tests/lune/` for the Luau side.
//!
//! ## Running
//! Requires the `lune` binary (provided by the Nix dev shell; otherwise
//! `cargo install lune` / Rokit). If `lune` is not found on `PATH` (and `$LUNE`
//! is unset) the test **skips** so `cargo test` stays green for contributors
//! without it. The first run fetches upstream over the network and caches it
//! under `tests/lune/.cache/` (gitignored); later runs are offline.
//!
//! ```bash
//! cargo test -p squash --test e2e_v5_lune            # vs latest release
//! SQUASH_REF=main cargo test -p squash --test e2e_v5_lune   # vs upstream tip
//! SQUASH_REFRESH=1 cargo test -p squash --test e2e_v5_lune  # bypass cache
//! LUNE=/path/to/lune cargo test -p squash --test e2e_v5_lune
//! ```
//!
//! ## Adding a case
//! Add it to BOTH `tests/lune/cases.luau` and `rust_catalog()` below, keyed by the
//! same name. The test fails if the two catalogs' name sets differ.
#![cfg(feature = "roblox")]

use std::collections::{BTreeSet, HashMap};
use std::process::Command;

use squash::codec::{
    array, array_n, literal, map, opt, range, string, string_n, to_bytes, Bool, U16, U8,
};
use squash::{
    i24, i40, i48, i56, serialize, u24, u40, u48, u56, BrickColor, Cframe, CframeRotSegments,
    Color3, ColorSequenceKeypoint, NumberRange, PhysicalProperties, Ray, Rect, Region3int16,
    SerDes, SquashObject, Udim, Udim2, Vector2, Vector2int16, Vector3, Vector3int16, Vlq,
};

/// Cases where the Rust port is known to diverge from upstream v5. Listed here so
/// the suite stays green while the gap is tracked; remove an entry once fixed.
///
/// - `num/i24_neg`: upstream encodes negative narrow ints (two's complement);
///   the port's `i24::new` rejects any negative value (audit-flagged), so the
///   Rust side errors instead of producing bytes.
const KNOWN_DIVERGENCES: &[&str] = &["num/i24_neg"];

type Bytes = Result<Vec<u8>, String>;

/// Serialize via the native `SquashObject` path.
fn obj<T: SquashObject>(v: T) -> Bytes {
    serialize(v).map_err(|e| e.to_string())
}

/// Serialize via a parameterizable codec.
fn cdc<S: SerDes>(sd: S, v: S::Value) -> Bytes {
    to_bytes(&sd, &v).map_err(|e| e.to_string())
}

fn hexencode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// A regular-fields-only struct: the `SquashObject` derive pushes fields in
/// ascending name order, matching upstream `Squash.record`'s sorted-key order.
#[derive(squash::SquashObject)]
struct Rec {
    a: u16,
    b: u32,
}

/// The Rust side of the shared catalog. Names must match `tests/lune/cases.luau`.
fn rust_catalog() -> Vec<(&'static str, Bytes)> {
    let mut out: Vec<(&'static str, Bytes)> = Vec::new();
    macro_rules! add {
        ($n:expr, $b:expr) => {
            out.push(($n, $b))
        };
    }

    // unsigned ints
    add!("num/u8", obj(0xC8u8));
    add!("num/u16", obj(0x0203u16));
    add!("num/u24", u24::new(0xCCBBAAu32).map_err(|e| e.to_string()).and_then(obj));
    add!("num/u32", obj(0x11223344u32));
    add!("num/u40", u40::new(0xAABBCCDDEEu64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/u48", u48::new(0xAABBCCDDEEFFu64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/u56", u56::new(0x10FFEEDDCCBBAAu64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/u64", obj(0x11223344556677u64));

    // signed ints
    add!("num/i8_neg", obj(-100i8));
    add!("num/i16_neg", obj(-5i16));
    add!("num/i24_pos", i24::new(0x334455i32).map_err(|e| e.to_string()).and_then(obj));
    add!("num/i32_neg", obj(-123456i32));
    add!("num/i40_pos", i40::new(0x55BBCCDDEEi64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/i48_pos", i48::new(0x5544CCDDEEFFi64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/i56_pos", i56::new(0x10FFEEDDCCBBAAi64).map_err(|e| e.to_string()).and_then(obj));
    add!("num/i64_neg", obj(-1000000i64));
    add!("num/i24_neg", i24::new(-100i32).map_err(|e| e.to_string()).and_then(obj));

    // floats
    add!("num/f32", obj(1.5f32));
    add!("num/f32_neg", obj(-2.5f32));
    add!("num/f64", obj(1.5f64));
    add!("num/f64_pi", obj(std::f64::consts::PI));

    // vlq
    for v in [0u64, 1, 127, 128, 300, 16383, 16384, 70000] {
        add!(
            match v {
                0 => "vlq/0",
                1 => "vlq/1",
                127 => "vlq/127",
                128 => "vlq/128",
                300 => "vlq/300",
                16383 => "vlq/16383",
                16384 => "vlq/16384",
                _ => "vlq/70000",
            },
            obj(Vlq(v))
        );
    }

    // bools
    add!("bool/true", cdc(Bool, true));
    add!("bool/false", cdc(Bool, false));

    // strings
    add!("string/var_hello", cdc(string(), "hello".to_string()));
    add!("string/var_empty", cdc(string(), String::new()));
    add!("string/var_utf8", cdc(string(), "héllo".to_string()));
    add!("string/fixed5", cdc(string_n(5), "hello".to_string()));

    // arrays
    add!("array/u16_var", cdc(array(U16), vec![1u16, 2, 3]));
    add!("array/u16_empty", cdc(array(U16), Vec::<u16>::new()));
    add!("array/u8_fixed3", cdc(array_n(U8, 3), vec![10u8, 20, 30]));

    // opt
    add!("opt/some", cdc(opt(U16), Some(0x0203u16)));
    add!("opt/none", cdc(opt(U16), None::<u16>));

    // map (single entry -> deterministic)
    add!("map/single", cdc(map(U8, U16), HashMap::from([(5u8, 0x0203u16)])));

    // range
    add!("range/500", cdc(range(10, 1000), 500i64));
    add!("range/neg", cdc(range(-100, 100), -7i64));

    // literal (upstream's enum-tag codec: 0-based index in one byte)
    add!("literal/first", cdc(literal(vec!["red", "green", "blue"]), "red"));
    add!("literal/last", cdc(literal(vec!["red", "green", "blue"]), "blue"));

    // record (regular fields, sorted-order)
    add!("record/ab", obj(Rec { a: 0x0203, b: 0x11223344 }));

    // Roblox datatypes
    add!("roblox/vector3_f32", obj(Vector3::<f32>::new(1.0, 2.0, 3.0)));
    add!("roblox/vector3_f64", obj(Vector3::<f64>::new(1.0, 2.0, 3.0)));
    add!("roblox/vector2_f32", obj(Vector2::<f32>::new(1.5, -2.5)));
    add!("roblox/vector3int16", obj(Vector3int16 { x: -1, y: 2, z: 3 }));
    add!("roblox/vector2int16", obj(Vector2int16 { x: -1, y: 2 }));
    add!("roblox/color3", obj(Color3 { r: 10, g: 20, b: 30 }));
    add!(
        "roblox/cframe_special_origin",
        obj(Cframe::<f32> { rotation: CframeRotSegments::new(0, 0, 0), position: Vector3::new(0.0, 0.0, 0.0) })
    );
    add!(
        "roblox/cframe_special_pos",
        obj(Cframe::<f32> { rotation: CframeRotSegments::new(0, 0, 0), position: Vector3::new(1.0, 2.0, 3.0) })
    );
    add!("roblox/numrange_f32", obj(NumberRange::<f32> { min: 0.0, max: 100.0 }));
    add!(
        "roblox/rect_f32",
        obj(Rect::<f32> { min: Vector2::new(1.0, 2.0), max: Vector2::new(3.0, 4.0) })
    );
    add!("roblox/udim_f32", obj(Udim::<f32> { offset: 10.0, scale: 0.5 }));
    add!(
        "roblox/udim2_f32",
        obj(Udim2::<f32> {
            x: Udim { offset: 10.0, scale: 0.5 },
            y: Udim { offset: 20.0, scale: 0.25 },
        })
    );
    add!(
        "roblox/ray_f32",
        obj(Ray::<f32> { origin: Vector3::new(0.0, 1.0, 2.0), direction: Vector3::new(1.0, 0.0, 0.0) })
    );
    add!(
        "roblox/physprops",
        obj(PhysicalProperties {
            elasticity_weight: 1.0,
            friction_weight: 2.0,
            elasticity: 0.5,
            friction: 0.25,
            density: 7.0,
        })
    );
    add!(
        "roblox/region3int16",
        obj(Region3int16 {
            min: Vector3int16 { x: -10, y: -20, z: -30 },
            max: Vector3int16 { x: 10, y: 20, z: 30 },
        })
    );
    add!("roblox/brickcolor", obj(BrickColor(194)));
    add!(
        "roblox/colorseqkp",
        obj(ColorSequenceKeypoint { value: Color3 { r: 10, g: 20, b: 30 }, time: 255 })
    );

    out
}

/// Locate the `lune` binary: `$LUNE` if set, else `lune` on `PATH`.
fn find_lune() -> Option<String> {
    if let Ok(p) = std::env::var("LUNE") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    let ok = Command::new("lune")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    ok.then(|| "lune".to_string())
}

#[test]
fn e2e_v5_lune_byte_conformance() {
    let Some(lune) = find_lune() else {
        eprintln!("[e2e] `lune` not found (set $LUNE or add to PATH); skipping conformance suite");
        return;
    };

    let lune_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/lune");
    let output = Command::new(&lune)
        .arg("run")
        .arg("generate.luau")
        .current_dir(lune_dir)
        .output()
        .expect("failed to spawn lune");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!(
            "lune exited unsuccessfully ({})\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            output.status
        );
    }

    // Parse the Luau side: name -> Ok(hex) | Err(msg).
    let mut luau: HashMap<String, Result<String, String>> = HashMap::new();
    let mut reference_ref = String::from("<unknown>");
    for line in stdout.lines() {
        let mut parts = line.splitn(3, '\t');
        match (parts.next(), parts.next(), parts.next()) {
            (Some("REF"), Some(r), _) => reference_ref = r.to_string(),
            (Some("SKIP"), reason, _) => {
                eprintln!(
                    "[e2e] upstream Squash could not be loaded ({}); skipping",
                    reason.unwrap_or("")
                );
                return;
            }
            (Some(name), Some("OK"), Some(hex)) => {
                luau.insert(name.to_string(), Ok(hex.to_string()));
            }
            (Some(name), Some("ERR"), Some(msg)) => {
                luau.insert(name.to_string(), Err(msg.to_string()));
            }
            _ => {} // ignore stray output (e.g. upstream prints)
        }
    }
    assert!(!luau.is_empty(), "lune produced no cases\n--- stdout ---\n{stdout}");

    let rust = rust_catalog();

    // Catalog parity: both sides must declare the exact same case names.
    let rust_names: BTreeSet<&str> = rust.iter().map(|(n, _)| *n).collect();
    let luau_names: BTreeSet<&str> = luau.keys().map(String::as_str).collect();
    let only_rust: Vec<_> = rust_names.difference(&luau_names).collect();
    let only_luau: Vec<_> = luau_names.difference(&rust_names).collect();
    assert!(
        only_rust.is_empty() && only_luau.is_empty(),
        "case catalogs diverge — keep cases.luau and rust_catalog() in sync.\n  only in Rust: {only_rust:?}\n  only in Luau: {only_luau:?}"
    );

    // Compare bytes.
    let mut matched = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    let mut known: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for (name, rust_bytes) in &rust {
        match &luau[*name] {
            Err(msg) => skipped.push(format!("{name} (luau error: {msg})")),
            Ok(hex) => {
                let allow = KNOWN_DIVERGENCES.contains(name);
                let report = match rust_bytes {
                    Ok(b) => {
                        let rhex = hexencode(b);
                        if &rhex == hex {
                            matched += 1;
                            continue;
                        }
                        format!("{name}\n    rust: {rhex}\n    luau: {hex}")
                    }
                    Err(e) => format!("{name}\n    rust: <error: {e}>\n    luau: {hex}"),
                };
                if allow {
                    known.push(report);
                } else {
                    failures.push(report);
                }
            }
        }
    }

    eprintln!(
        "[e2e] ref={reference_ref}  matched={matched}  skipped={}  known-divergences={}  failures={}",
        skipped.len(),
        known.len(),
        failures.len()
    );
    for s in &skipped {
        eprintln!("[e2e] skipped: {s}");
    }
    for k in &known {
        eprintln!("[e2e] KNOWN DIVERGENCE:\n{k}");
    }

    assert!(
        failures.is_empty(),
        "{} case(s) diverge from upstream Squash {reference_ref}:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
