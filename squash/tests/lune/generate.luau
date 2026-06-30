--!strict
-- Entry point run by `squash/tests/e2e_v5_lune.rs` via `lune run generate.luau`.
-- Loads upstream Squash (see reference.luau), runs every case (see cases.luau),
-- and prints one tab-separated line per case to stdout:
--
--   <name>\tOK\t<hex>     case serialized successfully; <hex> is the wire bytes
--   <name>\tERR\t<msg>    case errored (e.g. datatype missing in Lune) — skipped
--
-- Plus two header lines:
--   REF\t<ref>            the upstream ref that was loaded
--   SKIP\t<reason>        upstream could not be loaded at all (network, etc.)
--
-- The Rust side ignores any other stdout line, so stray upstream prints are safe.

local function tohex(buf: buffer): string
	local n = buffer.len(buf)
	local out = table.create(n)
	for i = 0, n - 1 do
		out[i + 1] = string.format("%02x", buffer.readu8(buf, i))
	end
	return table.concat(out)
end

local ok, ref = pcall(require, "./reference")
if not ok then
	print(`SKIP\t{tostring(ref)}`)
	return
end

local Squash = ref.Squash
print(`REF\t{ref.ref}`)

local buildCases = require("./cases")
local cases = buildCases(Squash, ref.roblox)

for _, case in cases do
	local runOk, result = pcall(function(): string
		local cursor = Squash.cursor()
		case.run(cursor)
		return tohex(Squash.tobuffer(cursor))
	end)
	if runOk then
		print(`{case.name}\tOK\t{result}`)
	else
		-- Collapse any newlines in the error so it stays one TSV line.
		local msg = string.gsub(tostring(result), "%s+", " ")
		print(`{case.name}\tERR\t{msg}`)
	end
end
