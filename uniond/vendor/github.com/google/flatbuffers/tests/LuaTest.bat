set buildtype=Release
if "%1"=="-b" set buildtype=%2

echo Run with LuaJIT:
luajit.exe luatest.lua
echo Run with Lua 5.3:
lua53.exe luatest.lua