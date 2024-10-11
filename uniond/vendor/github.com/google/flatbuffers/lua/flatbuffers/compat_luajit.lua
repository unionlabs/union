local bit = require("bit")
local ffi = require("ffi")
local band = bit.band
local bnot = bit.bnot


local m = {}
local Uint8Bound = 256 -- bound is the max uintN + 1
local Uint16Bound = 65536
local Uint32Bound = 4294967296

if not table.unpack then
    table.unpack = unpack
end

if not table.pack then
    table.pack = pack
end

m.GetAlignSize = function(k, size)
    return band((bnot(k) + 1), (size - 1))
end


local function pack_I1(n)
    return string.char(n)
end
local function pack_i1(n)
    if n < 0 then
        n = Uint8Bound + n
    end
    return pack_I1(n)
end

local function unpack_I1(n, pos)
    return string.byte(n, pos)
end
local function unpack_i1(n, pos)
    local res = unpack_I1(n, pos)
    if res >= Uint8Bound / 2 then
        return res - Uint8Bound
    end
    return res
end

local b2 = ffi.new("unsigned char[2]")
local function pack_I2(n)
    for i = 0, 1 do
        b2[i] = bit.band(n, 255)
        n = bit.rshift(n, 8)
    end
    return ffi.string(b2, 2)
end
local function pack_i2(n)
    if n < 0 then
        n = Uint16Bound + n
    end
    return pack_I2(n)
end

local function unpack_I2(n, pos)
    local a, b = string.byte(n, pos, pos + 1)
    return b * Uint8Bound + a
end
local function unpack_i2(n, pos)
    local res = unpack_I2(n, pos)
    if res >= Uint16Bound / 2 then
        return res - Uint16Bound
    end
    return res
end

local b4 = ffi.new("unsigned char[4]")
local function pack_I4(n)
    for i = 0, 3 do
        b4[i] = bit.band(n, 255)
        n = bit.rshift(n, 8)
    end
    return ffi.string(b4, 4)
end
local function pack_i4(n)
    if n < 0 then
        n = Uint32Bound + n
    end
    return pack_I4(n)
end

local function unpack_I4(n, pos)
    local a, b, c, d = string.byte(n, pos, pos + 3)
    return Uint8Bound * (Uint8Bound * ((Uint8Bound * d) + c) + b) + a
end
local function unpack_i4(n, pos)
    local res = unpack_I4(n, pos)
    if res >= Uint32Bound / 2 then
        return res - Uint32Bound
    end
    return res
end

local b8 = ffi.new("unsigned char[8]")
local function pack_I8(n)
    n = ffi.cast("unsigned long long", n)
    local hi = math.floor(tonumber(n / Uint32Bound))
    local li = n % Uint32Bound
    for i = 0, 3 do
        b8[i] = bit.band(li, 255)
        li = bit.rshift(li, 8)
    end
    for i = 4, 7 do
        b8[i] = bit.band(hi, 255)
        hi = bit.rshift(hi, 8)
    end
    return ffi.string(b8, 8)
end
local function pack_i8(n)
    n = ffi.cast("signed long long", n)
    return pack_I8(n)
end

local function unpack_I8(n, pos)
    local a, b, c, d = string.byte(n, pos, pos + 3)
    local li = Uint8Bound * (Uint8Bound * ((Uint8Bound * d) + c) + b) + a
    local a, b, c, d = string.byte(n, pos + 4, pos + 7)
    local hi = Uint8Bound * (Uint8Bound * ((Uint8Bound * d) + c) + b) + a
    return ffi.cast("unsigned long long", hi) * Uint32Bound + li
end
local function unpack_i8(n, pos)
    local res = unpack_I8(n, pos)
    return ffi.cast("signed long long", res)
end

local bf = ffi.new("float[1]")
local function pack_f(n)
    bf[0] = n
    return ffi.string(bf, 4)
end

local function unpack_f(n, pos)
    ffi.copy(bf, ffi.cast("char *", n) + pos - 1, 4)
    return tonumber(bf[0])
end

local bd = ffi.new("double[1]")
local function pack_d(n)
    bd[0] = n
    return ffi.string(bd, 8)
end

local function unpack_d(n, pos)
    ffi.copy(bd, ffi.cast("char *", n) + pos - 1, 8)
    return tonumber(bd[0])
end


m.string_pack = function(fmt, i, ...)
    if fmt == "<I1" then
        return pack_I1(i)
    elseif fmt == "<I2" then
        return pack_I2(i)
    elseif fmt == "<I4" then
        return pack_I4(i)
    elseif fmt == "<I8" then
        return pack_I8(i)
    elseif fmt == "<i1" then
        return pack_i1(i)
    elseif fmt == "<i2" then
        return pack_i2(i)
    elseif fmt == "<i4" then
        return pack_i4(i)
    elseif fmt == "<i8" then
        return pack_i8(i)
    elseif fmt == "<f" then
        return pack_f(i)
    elseif fmt == "<d" then
        return pack_d(i)
    else
        error(string.format("FIXME: support fmt %s", fmt))
    end
end


m.string_unpack = function(fmt, s, pos)
    if not pos then
        pos = 1
    end

    if fmt == "<I1" then
        return unpack_I1(s, pos)
    elseif fmt == "<I2" then
        return unpack_I2(s, pos)
    elseif fmt == "<I4" then
        return unpack_I4(s, pos)
    elseif fmt == "<I8" then
        return unpack_I8(s, pos)
    elseif fmt == "<i1" then
        return unpack_i1(s, pos)
    elseif fmt == "<i2" then
        return unpack_i2(s, pos)
    elseif fmt == "<i4" then
        return unpack_i4(s, pos)
    elseif fmt == "<i8" then
        return unpack_i8(s, pos)
    elseif fmt == "<f" then
        return unpack_f(s, pos)
    elseif fmt == "<d" then
        return unpack_d(s, pos)
    else
        error(string.format("FIXME: support fmt %s", fmt))
    end
end


return m
