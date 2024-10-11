local compats = {
    ["Lua 5.1"] = function()  
        -- Check if Lua JIT is installed first
        local ok = pcall(require, "jit")
        if not ok then
            return require("flatbuffers.compat_5_1")
        else
            return require("flatbuffers.compat_luajit")
        end
    end,
    ["Lua 5.2"] = function() return require("flatbuffers.compat_5_1") end,
    ["Lua 5.3"] = function() return require("flatbuffers.compat_5_3") end,
    ["Lua 5.4"] = function() return require("flatbuffers.compat_5_3") end,
}
return assert(compats[_VERSION], "Unsupported Lua Version: ".._VERSION)()