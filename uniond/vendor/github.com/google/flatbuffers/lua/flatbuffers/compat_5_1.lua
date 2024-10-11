local m = {}
local ok, bit = pcall(require, "bit32")
assert(ok, "The Bit32 library must be installed")
assert(pcall(require, "compat53"), "The Compat 5.3 library must be installed")

m.GetAlignSize = function(k, size)
    return bit.band(bit.bnot(k) + 1,(size - 1))
end

if not table.unpack then
    table.unpack = unpack
end

if not table.pack then
    table.pack = pack
end

m.string_pack = string.pack
m.string_unpack = string.unpack

return m
