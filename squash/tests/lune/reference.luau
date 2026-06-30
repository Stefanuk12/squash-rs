--!strict
-- Fetches the upstream Squash Luau library and loads it under a Roblox-datatype
-- environment so it can run inside Lune (which has no global Roblox datatypes).
--
-- Driven by `squash/tests/e2e_v5_lune.rs`. Environment variables:
--   SQUASH_REF      which upstream ref to test against. Default "latest" (resolves
--                   the newest GitHub release tag; falls back to "main"). May be
--                   any tag/branch/sha, e.g. "v5.0.0" or "main".
--   SQUASH_REFRESH  if set (non-empty), bypass the on-disk cache and refetch.
--
-- The loaded source is cached under `.cache/squash-<ref>.luau` (gitignored) so
-- repeat runs are offline and reproducible.

local net = require("@lune/net")
local fs = require("@lune/fs")
local luau = require("@lune/luau")
local serde = require("@lune/serde")
local process = require("@lune/process")
local roblox = require("@lune/roblox")

local REPO = "Data-Oriented-House/Squash"
local CACHE_DIR = ".cache"

local function envOr(name: string, default: string?): string?
	local v = process.env[name]
	if v == nil or v == "" then
		return default
	end
	return v
end

-- Resolve the configured ref to a concrete one. "latest" -> newest release tag.
local function resolveRef(): string
	local ref = envOr("SQUASH_REF", "latest") :: string
	if ref ~= "latest" then
		return ref
	end
	-- Call inside a closure (not `pcall(net.request, ...)`) so the type solver keeps
	-- the FetchParams overload instead of erasing it across the pcall boundary.
	local ok, res = pcall(function()
		return net.request({
			url = `https://api.github.com/repos/{REPO}/releases/latest`,
			headers = { ["User-Agent"] = "squash-e2e" },
		})
	end)
	if ok and res.ok then
		local decoded_ok, decoded = pcall(function()
			return serde.decode("json", res.body)
		end)
		if decoded_ok and type(decoded) == "table" and type(decoded.tag_name) == "string" then
			return decoded.tag_name
		end
	end
	return "main" -- network/API failure: fall back to the branch tip.
end

local function fetchSource(ref: string): string
	local cachePath = `{CACHE_DIR}/squash-{ref}.luau`
	local refresh = envOr("SQUASH_REFRESH", nil) ~= nil
	if not refresh and fs.isFile(cachePath) then
		return fs.readFile(cachePath)
	end
	local res = net.request(`https://raw.githubusercontent.com/{REPO}/{ref}/src/init.luau`)
	if not res.ok then
		error(`failed to fetch {ref}: {res.statusCode} {res.statusMessage}`)
	end
	if not fs.isDir(CACHE_DIR) then
		fs.writeDir(CACHE_DIR)
	end
	fs.writeFile(cachePath, res.body)
	return res.body
end

-- Roblox datatypes that Lune's `@lune/roblox` provides. Squash references these
-- as globals; each upstream block is guarded by `if <Type> ~= nil then ... end`,
-- so any datatype Lune lacks simply doesn't register its serializer (safe).
local DATATYPES = {
	"Axes", "BrickColor", "CFrame", "Color3", "ColorSequence", "ColorSequenceKeypoint",
	"Enum", "Faces", "Font", "NumberRange", "NumberSequence", "NumberSequenceKeypoint",
	"PhysicalProperties", "Ray", "Rect", "Region3", "Region3int16", "UDim", "UDim2",
	"Vector2", "Vector2int16", "Vector3", "Vector3int16",
}

local function buildEnv()
	local e: { [string]: any } = {}
	for _, name in DATATYPES do
		e[name] = (roblox :: any)[name]
	end
	-- Fall back to this script's globals (buffer/string/table/math/vector/...).
	return setmetatable(e, { __index = _G })
end

local ref = resolveRef()
local source = fetchSource(ref)
local Squash = luau.load(source, { debugName = `squash@{ref}`, environment = buildEnv() })()

return {
	Squash = Squash,
	roblox = roblox,
	ref = ref,
}
