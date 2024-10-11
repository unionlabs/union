local compat = require("flatbuffers.compat")
local string_unpack = compat.string_unpack


local m = {}
local mt = {}

local mt_name = "flatbuffers.view.mt"

local N = require("flatbuffers.numTypes")
local binaryarray = require("flatbuffers.binaryarray")

local function enforceOffset(off)
    if off < 0 or off > 42949672951 then
        error("Offset is not valid")
    end
end

local function unPackUoffset(bytes, off)
    return string_unpack("<I4", bytes.str, off + 1)
end

local function unPackVoffset(bytes, off)
    return string_unpack("<I2", bytes.str, off + 1)
end

function m.New(buf, pos)
    enforceOffset(pos)
    -- need to convert from a string buffer into
    -- a binary array

    local o = {
        bytes = type(buf) == "string" and binaryarray.New(buf) or buf,
        pos = pos,
    }
    setmetatable(o, {__index = mt, __metatable = mt_name})
    return o
end

function mt:Offset(vtableOffset)
    local vtable = self.vtable
    if not vtable then
        vtable = self.pos - self:Get(N.SOffsetT, self.pos)
        self.vtable = vtable
        self.vtableEnd = self:Get(N.VOffsetT, vtable)
    end
    if vtableOffset < self.vtableEnd then
        return unPackVoffset(self.bytes, vtable + vtableOffset)
    end
    return 0
end

function mt:Indirect(off)
    enforceOffset(off)
    return off + unPackUoffset(self.bytes, off)
end

function mt:String(off)
    enforceOffset(off)
    off = off + unPackUoffset(self.bytes, off)
    local start = off + 4
    local length = unPackUoffset(self.bytes, off)
    return self.bytes:Slice(start, start+length)
end

function mt:VectorLen(off)
    enforceOffset(off)
    off = off + self.pos
    off = off + unPackUoffset(self.bytes, off)
    return unPackUoffset(self.bytes, off)
end

function mt:Vector(off)
    enforceOffset(off)
    off = off + self.pos
    return off + self:Get(N.UOffsetT, off) + 4
end

function mt:VectorAsString(off, start, stop)
    local o = self:Offset(off)
    if o ~= 0 then
        start = start or 1
        stop = stop or self:VectorLen(o)
        local a = self:Vector(o) + start - 1
        return self.bytes:Slice(a, a + stop - start + 1)
    end
    return nil
end

function mt:Union(t2, off)
    assert(getmetatable(t2) == mt_name)
    enforceOffset(off)
    off = off + self.pos
    t2.pos = off + self:Get(N.UOffsetT, off)
    t2.bytes = self.bytes
end

function mt:Get(flags, off)
    enforceOffset(off)
    return flags:Unpack(self.bytes, off)
end

function mt:GetSlot(slot, d, validatorFlags)
    N.VOffsetT:EnforceNumber(slot)
    if validatorFlags then
        validatorFlags:EnforceNumber(d)
    end
    local off = self:Offset(slot)
    if off == 0 then
        return d
    end
    return self:Get(validatorFlags, self.pos + off)
end

function mt:GetVOffsetTSlot(slot, d)
    N.VOffsetT:EnforceNumbers(slot, d)
    local off = self:Offset(slot)
    if off == 0 then
        return d
    end
    return off
end

return m