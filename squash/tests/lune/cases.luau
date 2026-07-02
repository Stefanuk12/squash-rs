--!strict
-- The conformance case catalog (Luau / upstream-Squash side).
--
-- Each case has a unique `name` and a `run(cursor)` that serializes one value
-- with the upstream serdes. `squash/tests/e2e_v5_lune.rs` holds a Rust catalog
-- keyed by the SAME names; the two are asserted to match byte-for-byte. When you
-- add a case here, add the matching Rust case (and vice versa) — the test fails
-- if the name sets differ.
--
-- Excluded on purpose (non-deterministic or float-derived bytes that can't be
-- byte-compared): multi-entry maps, general-rotation CFrames, NumberSequence*
-- (v5 stores keypoint time as f32, the Rust port uses u8).

return function(S: any, rbx: any)
	local cases: { { name: string, run: (any) -> () } } = {}
	local function add(name: string, run: (any) -> ())
		table.insert(cases, { name = name, run = run })
	end

	---------------------------------------------------------------- unsigned ints
	add("num/u8", function(c) S.u8().ser(c, 0xC8) end)
	add("num/u16", function(c) S.u16().ser(c, 0x0203) end)
	add("num/u24", function(c) S.u24().ser(c, 0xCCBBAA) end)
	add("num/u32", function(c) S.u32().ser(c, 0x11223344) end)
	add("num/u40", function(c) S.u40().ser(c, 0xAABBCCDDEE) end)
	add("num/u48", function(c) S.u48().ser(c, 0xAABBCCDDEEFF) end)
	add("num/u56", function(c) S.u56().ser(c, 0x10FFEEDDCCBBAA) end)
	add("num/u64", function(c) S.u64().ser(c, 0x11223344556677) end)

	------------------------------------------------------------------ signed ints
	add("num/i8_neg", function(c) S.i8().ser(c, -100) end)
	add("num/i16_neg", function(c) S.i16().ser(c, -5) end)
	add("num/i24_pos", function(c) S.i24().ser(c, 0x334455) end)
	add("num/i32_neg", function(c) S.i32().ser(c, -123456) end)
	add("num/i40_pos", function(c) S.i40().ser(c, 0x55BBCCDDEE) end)
	add("num/i48_pos", function(c) S.i48().ser(c, 0x5544CCDDEEFF) end)
	add("num/i56_pos", function(c) S.i56().ser(c, 0x10FFEEDDCCBBAA) end)
	add("num/i64_neg", function(c) S.i64().ser(c, -1000000) end)
	-- Probe: upstream encodes a negative narrow int fine; the Rust port's
	-- i24::new rejects negatives (audit-flagged). Listed in KNOWN_DIVERGENCES.
	add("num/i24_neg", function(c) S.i24().ser(c, -100) end)

	----------------------------------------------------------------------- floats
	add("num/f32", function(c) S.f32().ser(c, 1.5) end)
	add("num/f32_neg", function(c) S.f32().ser(c, -2.5) end)
	add("num/f64", function(c) S.f64().ser(c, 1.5) end)
	add("num/f64_pi", function(c) S.f64().ser(c, 3.141592653589793) end)

	-------------------------------------------------------------------------- vlq
	for _, v in { 0, 1, 127, 128, 300, 16383, 16384, 70000 } do
		add(`vlq/{v}`, function(c) S.vlq().ser(c, v) end)
	end

	------------------------------------------------------------------------ bools
	add("bool/true", function(c) S.boolean().ser(c, true) end)
	add("bool/false", function(c) S.boolean().ser(c, false) end)

	---------------------------------------------------------------------- strings
	add("string/var_hello", function(c) S.string().ser(c, "hello") end)
	add("string/var_empty", function(c) S.string().ser(c, "") end)
	add("string/var_utf8", function(c) S.string().ser(c, "héllo") end)
	add("string/fixed5", function(c) S.string(5).ser(c, "hello") end)

	----------------------------------------------------------------------- arrays
	add("array/u16_var", function(c) S.array(S.u16()).ser(c, { 1, 2, 3 }) end)
	add("array/u16_empty", function(c) S.array(S.u16()).ser(c, {}) end)
	add("array/u8_fixed3", function(c) S.array(S.u8(), 3).ser(c, { 10, 20, 30 }) end)

	------------------------------------------------------------------------ opt
	add("opt/some", function(c) S.opt(S.u16()).ser(c, 0x0203) end)
	add("opt/none", function(c) S.opt(S.u16()).ser(c, nil) end)

	------------------------------------------------------------ map (single entry)
	add("map/single", function(c) S.map(S.u8(), S.u16()).ser(c, { [5] = 0x0203 }) end)

	------------------------------------------------------------------------ range
	add("range/500", function(c) S.range(10, 1000).ser(c, 500) end)
	add("range/neg", function(c) S.range(-100, 100).ser(c, -7) end)

	---------------------------------------------------------------------- literal
	add("literal/first", function(c) S.literal("red", "green", "blue").ser(c, "red") end)
	add("literal/last", function(c) S.literal("red", "green", "blue").ser(c, "blue") end)

	----------------------------------------------------------- record (regular fields)
	add("record/ab", function(c)
		S.record({ a = S.u16(), b = S.u32() }).ser(c, { a = 0x0203, b = 0x11223344 })
	end)

	-------------------------------------------------------------- Roblox datatypes
	add("roblox/vector3_f32", function(c) S.Vector3(S.f32()).ser(c, rbx.Vector3.new(1, 2, 3)) end)
	add("roblox/vector3_f64", function(c) S.Vector3(S.f64()).ser(c, rbx.Vector3.new(1, 2, 3)) end)
	add("roblox/vector2_f32", function(c) S.Vector2(S.f32()).ser(c, rbx.Vector2.new(1.5, -2.5)) end)
	add("roblox/vector3int16", function(c) S.Vector3int16().ser(c, rbx.Vector3int16.new(-1, 2, 3)) end)
	add("roblox/vector2int16", function(c) S.Vector2int16().ser(c, rbx.Vector2int16.new(-1, 2)) end)
	add("roblox/color3", function(c) S.Color3().ser(c, rbx.Color3.fromRGB(10, 20, 30)) end)
	add("roblox/cframe_special_origin", function(c) S.CFrame(S.f32()).ser(c, rbx.CFrame.new(0, 0, 0)) end)
	add("roblox/cframe_special_pos", function(c) S.CFrame(S.f32()).ser(c, rbx.CFrame.new(1, 2, 3)) end)
	add("roblox/numrange_f32", function(c) S.NumberRange(S.f32()).ser(c, rbx.NumberRange.new(0, 100)) end)
	add("roblox/rect_f32", function(c) S.Rect(S.f32()).ser(c, rbx.Rect.new(1, 2, 3, 4)) end)
	add("roblox/udim_f32", function(c) S.UDim(S.f32()).ser(c, rbx.UDim.new(0.5, 10)) end)
	add("roblox/udim2_f32", function(c) S.UDim2(S.f32()).ser(c, rbx.UDim2.new(0.5, 10, 0.25, 20)) end)
	add("roblox/ray_f32", function(c)
		S.Ray(S.f32()).ser(c, rbx.Ray.new(rbx.Vector3.new(0, 1, 2), rbx.Vector3.new(1, 0, 0)))
	end)
	add("roblox/physprops", function(c)
		S.PhysicalProperties().ser(c, rbx.PhysicalProperties.new(7, 0.25, 0.5, 2, 1))
	end)
	add("roblox/region3int16", function(c)
		S.Region3int16().ser(c, rbx.Region3int16.new(rbx.Vector3int16.new(-10, -20, -30), rbx.Vector3int16.new(10, 20, 30)))
	end)
	add("roblox/brickcolor", function(c) S.BrickColor().ser(c, rbx.BrickColor.new(194)) end)
	add("roblox/colorseqkp", function(c)
		S.ColorSequenceKeypoint().ser(c, rbx.ColorSequenceKeypoint.new(1, rbx.Color3.fromRGB(10, 20, 30)))
	end)

	return cases
end
