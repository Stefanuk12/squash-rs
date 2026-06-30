//! Round-trip + byte-order checks for the v5-corrected Roblox types.
#![cfg(feature = "roblox")]

use squash::{
    deserialize, serialize, BoolTuple3, CatalogSearchParams, Cframe, CframeRotSegments, Color3,
    ColorSequenceKeypoint, EnumItem, FloatCurveKey, Font, NumberRange, NumberSequenceKeypoint,
    OverlapParams, OverlapParamsBool, PathWaypoint, PhysicalProperties, Ray, RaycastParams,
    RaycastResult, Rect, Region3int16, RotationCurveKey, TweenInfo, Udim, Udim2, Vector2, Vector3,
    Vector3int16, Vlq,
};

fn rt<T>(v: T)
where
    T: squash::SquashObject + Clone + PartialEq + std::fmt::Debug,
{
    let back: T = deserialize(serialize(v.clone()).unwrap()).unwrap();
    assert_eq!(back, v);
}

#[test]
fn color3_byte_order_and_round_trip() {
    // v5 pushes b, g, r
    assert_eq!(serialize(Color3 { r: 10, g: 20, b: 30 }).unwrap(), vec![30, 20, 10]);
    rt(Color3 { r: 1, g: 2, b: 3 });
}

#[test]
fn vectors_round_trip() {
    rt(Vector3::<f32>::new(1.0, 2.0, 3.0));
    rt(Vector2::<f32>::new(-4.5, 9.0));
    rt(Vector3int16 { x: -1, y: 2, z: 3 });
    rt(NumberRange::<f32> { min: 0.0, max: 100.0 });
}

#[test]
fn region3int16_round_trips() {
    rt(Region3int16 {
        min: Vector3int16 { x: -10, y: -20, z: -30 },
        max: Vector3int16 { x: 10, y: 20, z: 30 },
    });
}

#[test]
fn raycast_params_round_trips() {
    rt(RaycastParams {
        bool_data: BoolTuple3(true, false, true),
        filter_type: EnumItem(Vlq(2)),
        collision_group: "Default".to_string(),
    });
}

#[test]
fn overlap_params_round_trips() {
    rt(OverlapParams {
        bool_data: OverlapParamsBool { brute_force_all_slow: true, respect_can_collide: false },
        max_parts: 12,
        raycast_filter_type: EnumItem(Vlq(1)),
        collision_group: "Default".to_string(),
    });
}

#[test]
fn cframe_special_and_general_rotation() {
    // special: rotation (0,0,0) is CFRAME_ROTS[1] -> single id byte
    let special = Cframe::<f32> {
        rotation: CframeRotSegments::new(0, 0, 0),
        position: Vector3::new(1.0, 2.0, 3.0),
    };
    rt(special);
    // general: a rotation not in the table -> 3xu16 + marker 0
    let general = Cframe::<f32> {
        rotation: CframeRotSegments::new(100, 200, 300),
        position: Vector3::new(-1.0, 0.5, 9.0),
    };
    rt(general);
}

#[test]
fn cframe_wire_layout_pins_both_branches() {
    // Round-trip alone can't catch a branch regression (e.g. always emitting the
    // general block, or the marker landing at the wrong offset). Pin the actual
    // bytes so the special-id vs 3xu16+marker layout is locked in.
    let special = Cframe::<f32> {
        rotation: CframeRotSegments::new(0, 0, 0), // CFRAME_ROTS[1]
        position: Vector3::new(1.0, 2.0, 3.0),
    };
    let sp = serialize(special).unwrap();
    // [special_id=1] ++ position (3 x f32). No 3xu16 block, no zero marker.
    assert_eq!(sp.len(), 1 + 12);
    assert_eq!(sp[0], 1);

    let general = Cframe::<f32> {
        rotation: CframeRotSegments::new(100, 200, 300), // x, y, z
        position: Vector3::new(-1.0, 0.5, 9.0),
    };
    let gn = serialize(general).unwrap();
    // [z as u16][y as u16][x as u16][marker=0] ++ position (3 x f32).
    assert_eq!(gn.len(), 6 + 1 + 12);
    assert_eq!(&gn[0..2], &300u16.to_le_bytes()); // z
    assert_eq!(&gn[2..4], &200u16.to_le_bytes()); // y
    assert_eq!(&gn[4..6], &100u16.to_le_bytes()); // x
    assert_eq!(gn[6], 0); // general-rotation marker
}

/// serde round-trip through the crate's own Serializer/Deserializer. This path
/// is separate from the native `serialize`/`deserialize` above: the serde
/// `Deserialize` (derived via `ReverseDeserialize`) must read fields in the
/// reverse of declaration order to undo the LIFO cursor. A regression here would
/// silently swap fields (e.g. Vector3 x<->z) without touching the native path.
#[cfg(feature = "serde")]
fn serde_rt<T>(v: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + PartialEq + std::fmt::Debug,
{
    let mut bytes = squash::serde_serialize(&v).unwrap();
    let back: T = squash::serde_deserialize(&mut bytes).unwrap();
    assert_eq!(back, v);
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_preserve_fields() {
    serde_rt(Color3 { r: 10, g: 20, b: 30 });
    serde_rt(Vector3::<f32>::new(1.0, 2.0, 3.0));
    serde_rt(Vector3int16 { x: -1, y: 2, z: 3 });
    serde_rt(NumberRange::<f32> { min: 0.0, max: 100.0 });
    serde_rt(Region3int16 {
        min: Vector3int16 { x: -10, y: -20, z: -30 },
        max: Vector3int16 { x: 10, y: 20, z: 30 },
    });
    serde_rt(RaycastParams {
        bool_data: BoolTuple3(true, false, true),
        filter_type: EnumItem(Vlq(2)),
        collision_group: "Default".to_string(),
    });
    // Regression: OverlapParamsBool's serde Serialize previously emitted two
    // bytes while its Deserialize read one, corrupting OverlapParams round-trip.
    serde_rt(OverlapParams {
        bool_data: OverlapParamsBool { brute_force_all_slow: true, respect_can_collide: false },
        max_parts: 12,
        raycast_filter_type: EnumItem(Vlq(1)),
        collision_group: "Default".to_string(),
    });
    serde_rt(TweenInfo {
        delay_time: 0.5,
        reverses: true,
        repeat_count: Vlq(3),
        easing_direction: EnumItem(Vlq(1)),
        easing_style: EnumItem(Vlq(2)),
        time: 1.5,
    });
    // Cframe has a fully hand-written serde Serialize+Deserialize pair (special-id
    // branch vs general rotation block); exercise both arms through the serde path.
    serde_rt(Cframe::<f32> {
        rotation: CframeRotSegments::new(0, 0, 0), // special id
        position: Vector3::new(1.0, 2.0, 3.0),
    });
    serde_rt(Cframe::<f32> {
        rotation: CframeRotSegments::new(100, 200, 300), // general rotation
        position: Vector3::new(-1.0, 0.5, 9.0),
    });
}

#[test]
fn cframe_invalid_special_id_errors_without_panicking() {
    // Wire: [marker, <12 bytes position>] — pop reads position first, then the
    // marker. A marker of 25 is out of range (valid ids are 1..=24) and must
    // produce an error, not an out-of-bounds panic.
    let bytes = vec![25u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(deserialize::<Cframe<f32>>(bytes).is_err());
}

/// The remaining `impl_squash!` Roblox records were migrated to
/// `impl_squash_object_a!` + `#[derive(ReverseDeserialize)]` so their serde
/// `Deserialize` reads fields in reverse-declaration order (undoing the LIFO
/// cursor) instead of in the v5 wire-order list. Previously the macro reused the
/// wire-order forward list, so serde round-trip silently swapped/corrupted fields
/// whenever declaration order != wire order. Round-trip each here.
#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_migrated_records() {
    serde_rt(Udim::<f32> { offset: 1.5, scale: 0.25 });
    serde_rt(Udim2::<f32> {
        x: Udim { offset: 1.0, scale: 0.5 },
        y: Udim { offset: -2.0, scale: 0.75 },
    });
    serde_rt(Rect::<f32> {
        min: Vector2::new(-1.0, -2.0),
        max: Vector2::new(3.0, 4.0),
    });
    serde_rt(Ray::<f32> {
        origin: Vector3::new(0.0, 1.0, 2.0),
        direction: Vector3::new(1.0, 0.0, 0.0),
    });
    serde_rt(PathWaypoint::<f32> {
        label: "spawn".to_string(),
        action: EnumItem(Vlq(1)),
        position: Vector3::new(5.0, 6.0, 7.0),
    });
    serde_rt(RaycastResult::<f32> {
        distance: 12.5,
        position: Vector3::new(1.0, 2.0, 3.0),
        normal: Vector3::new(0.0, 1.0, 0.0),
        material: EnumItem(Vlq(4)),
    });
    serde_rt(ColorSequenceKeypoint { value: Color3 { r: 10, g: 20, b: 30 }, time: 128 });
    serde_rt(Font {
        family: "Gotham".to_string(),
        bold: true,
        weight: EnumItem(Vlq(2)),
        style: EnumItem(Vlq(1)),
    });
    serde_rt(PhysicalProperties {
        elasticity_weight: 1.0,
        friction_weight: 2.0,
        elasticity: 0.5,
        friction: 0.25,
        density: 7.0,
    });
    serde_rt(FloatCurveKey { interpolation: EnumItem(Vlq(0)), value: 9.0, time: 3.0 });
    serde_rt(NumberSequenceKeypoint::<f32> { value: 1.0, envelope: 0.1, time: 200 });
    serde_rt(RotationCurveKey::<f32> { interpolation: EnumItem(Vlq(1)), value: 2.0, time: 4.0 });
    serde_rt(CatalogSearchParams {
        include_off_sale: true,
        limit: 50,
        min_price: 10,
        max_price: 1000,
        creator_name: "builder".to_string(),
        search_keyworld: "sword".to_string(),
        sort_type: EnumItem(Vlq(1)),
        sort_aggregration: EnumItem(Vlq(2)),
        category_filter: EnumItem(Vlq(3)),
        sales_type_filter: EnumItem(Vlq(4)),
        // Multi-element, distinct values: exercises serde sequence ORDER through
        // the LIFO cursor (regression guard for the SeqSerializer reverse).
        asset_types: vec![EnumItem(Vlq(7)), EnumItem(Vlq(8)), EnumItem(Vlq(9))],
    });
}

/// Direct regression for the serde sequence-order fix (SeqSerializer in ser.rs):
/// a `Vec` serialized then deserialized through the crate's serde path must
/// preserve element order, not reverse it.
#[cfg(feature = "serde")]
#[test]
fn serde_vec_preserves_order() {
    serde_rt(vec![1u16, 2, 3, 4, 5]);
    serde_rt(vec!["a".to_string(), "bb".to_string(), "ccc".to_string()]);
    // Nested: a vec of vecs must preserve order at both levels.
    serde_rt(vec![vec![1u8, 2], vec![3u8, 4, 5], vec![6u8]]);
}

/// `Vlq`'s serde `Deserialize` previously read its bytes via `Vec::<u8>::deserialize`
/// (which routes through `deserialize_seq` and reverses), corrupting any multi-byte
/// VLQ — e.g. `Vlq(200)` round-tripped to `Vlq(72)`. It now reads via
/// `deserialize_bytes`. `EnumItem` wraps `Vlq`, so this hits every enum-typed field.
#[cfg(feature = "serde")]
#[test]
fn serde_multibyte_vlq_round_trips() {
    for v in [0u64, 1, 127, 128, 200, 300, 16383, 16384, 70000] {
        serde_rt(Vlq(v));
        serde_rt(EnumItem(Vlq(v)));
    }
    serde_rt(vec![EnumItem(Vlq(200)), EnumItem(Vlq(16384)), EnumItem(Vlq(5))]);
}
