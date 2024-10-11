package.path = string.format("../lua/?.lua;./?.lua;%s",package.path)
local compat = require("flatbuffers.compat")

local performBenchmarkTests = false

if #arg > 1 then
    print("usage: lua luatests [benchmark]");
    return
elseif #arg > 0 then 
    if(arg[1] == "benchmark") then
        performBenchmarkTests = true
    end 
end

local function checkReadBuffer(buf, offset, sizePrefix)
    offset = offset or 0
    
    if type(buf) == "string" then
        buf = flatbuffers.binaryArray.New(buf)
    end
    
    if sizePrefix then               
        local size = flatbuffers.N.Int32:Unpack(buf, offset)
        assert(size == buf.size - offset - 4)
        offset = offset + flatbuffers.N.Int32.bytewidth
    end    
    
    local mon = monster.GetRootAsMonster(buf, offset)
    assert(mon:Hp() == 80, "Monster Hp is not 80")
    assert(mon:Mana() == 150, "Monster Mana is not 150")
    assert(mon:Name() == "MyMonster", "Monster Name is not MyMonster")
    assert(mon:Testbool() == true)
    
    local vec = assert(mon:Pos(), "Monster Position is nil")
    assert(vec:X() == 1.0)
    assert(vec:Y() == 2.0)
    assert(vec:Z() == 3.0)
    assert(vec:Test1() == 3.0)
    assert(vec:Test2() == 2)
    
    local t = require("MyGame.Example.Test").New()
    t = assert(vec:Test3(t))
    
    assert(t:A() == 5)
    assert(t:B() == 6)
    
    local ut = require("MyGame.Example.Any")
    assert(mon:TestType() == ut.Monster)
    
    local table2 = mon:Test()
    assert(getmetatable(table2) == "flatbuffers.view.mt")
    
    local mon2 = monster.New()
    mon2:Init(table2.bytes, table2.pos)
    
    assert(mon2:Name() == "Fred")
    
    assert(mon:InventoryLength() == 5)
    local invsum = 0
    for i=1,mon:InventoryLength() do
        local v = mon:Inventory(i)
        invsum = invsum + v
    end
    assert(invsum == 10)
    
    for i=1,5 do
        assert(mon:VectorOfLongs(i) == 10^((i-1)*2))
    end
    
    local dbls = { -1.7976931348623157e+308, 0, 1.7976931348623157e+308}
    for i=1,mon:VectorOfDoublesLength() do
        assert(mon:VectorOfDoubles(i) == dbls[i])
    end
    
    assert(mon:Test4Length() == 2)
    
    local test0 = mon:Test4(1)
    local test1 = mon:Test4(2)
    
    local v0 = test0:A()
    local v1 = test0:B()
    local v2 = test1:A()
    local v3 = test1:B()
    
    local sumtest12 = v0 + v1 + v2 + v3
    assert(sumtest12 == 100)
    
    assert(mon:TestarrayofstringLength() == 2)
    assert(mon:Testarrayofstring(1) == "test1")
    assert(mon:Testarrayofstring(2) == "test2")
    
    assert(mon:TestarrayoftablesLength() == 0)
    assert(mon:TestnestedflatbufferLength() == 0)
    assert(mon:Testempty() == nil)
end

local function generateMonster(sizePrefix, b)
    if b then b:Clear() end
    b = b or flatbuffers.Builder(0)
    local str = b:CreateString("MyMonster")
    local test1 = b:CreateString("test1")
    local test2 = b:CreateString("test2")
    local fred = b:CreateString("Fred")
    
    monster.StartInventoryVector(b, 5)
    b:PrependByte(4)
    b:PrependByte(3)
    b:PrependByte(2)
    b:PrependByte(1)
    b:PrependByte(0)
    local inv = b:EndVector(5)
    
    monster.Start(b)
    monster.AddName(b, fred)
    local mon2 = monster.End(b)
    
    monster.StartTest4Vector(b, 2)
    test.CreateTest(b, 10, 20)
    test.CreateTest(b, 30, 40)
    local test4 = b:EndVector(2)
    
    monster.StartTestarrayofstringVector(b, 2)
    b:PrependUOffsetTRelative(test2)
    b:PrependUOffsetTRelative(test1)
    local testArrayOfString = b:EndVector(2)
    
    monster.StartVectorOfLongsVector(b, 5)
    b:PrependInt64(100000000)
    b:PrependInt64(1000000)
    b:PrependInt64(10000)
    b:PrependInt64(100)
    b:PrependInt64(1)
    local vectorOfLongs = b:EndVector(5)
    
    monster.StartVectorOfDoublesVector(b, 3)
    b:PrependFloat64(1.7976931348623157e+308)
    b:PrependFloat64(0)
    b:PrependFloat64(-1.7976931348623157e+308)
    local vectorOfDoubles = b:EndVector(3)
    
    monster.Start(b)
    local pos = vec3.CreateVec3(b, 1.0, 2.0, 3.0, 3.0, 2, 5, 6)
    monster.AddPos(b, pos)
    
    monster.AddHp(b, 80)
    monster.AddName(b, str)
    monster.AddInventory(b, inv)
    monster.AddTestType(b, 1)
    monster.AddTest(b, mon2)
    monster.AddTest4(b, test4)
    monster.AddTestbool(b, true)
    monster.AddTestbool(b, false)
    monster.AddTestbool(b, null)
    monster.AddTestbool(b,"true")
    monster.AddTestarrayofstring(b, testArrayOfString)
    monster.AddVectorOfLongs(b, vectorOfLongs)
    monster.AddVectorOfDoubles(b, vectorOfDoubles)
    local mon = monster.End(b)
    
    if sizePrefix then
        b:FinishSizePrefixed(mon)
    else
        b:Finish(mon)
    end
    return b:Output(true), b:Head()
end

local function sizePrefix(sizePrefix)
    local buf,offset = generateMonster(sizePrefix)
    checkReadBuffer(buf, offset, sizePrefix)
end

local function fbbClear()
    -- Generate a builder that will be 'cleared' and reused to create two different objects.
    local fbb = flatbuffers.Builder(0)

    -- First use the builder to read the normal monster data and verify it works
    local buf, offset = generateMonster(false, fbb)
    checkReadBuffer(buf, offset, false)

    -- Then clear the builder to be used again
    fbb:Clear()

    -- Storage for the built monsters
    local monsters = {}
    local lastBuf

    -- Make another builder that will be use identically to the 'cleared' one so outputs can be compared. Build both the
    -- Cleared builder and new builder in the exact same way, so we can compare their results
    for i, builder in ipairs({fbb, flatbuffers.Builder(0)}) do
        local strOffset = builder:CreateString("Hi there")
        monster.Start(builder)
        monster.AddPos(builder, vec3.CreateVec3(builder, 3.0, 2.0, 1.0, 17.0, 3, 100, 123))
        monster.AddName(builder, strOffset)
        monster.AddMana(builder, 123)
        builder:Finish(monster.End(builder))
        local buf = builder:Output(false)
        if not lastBuf then
            lastBuf = buf
        else
            -- the output, sized-buffer should be identical
            assert(lastBuf == buf, "Monster output buffers are not identical")
        end
        monsters[i] = monster.GetRootAsMonster(flatbuffers.binaryArray.New(buf), 0)
    end

    -- Check that all the fields for the generated monsters are as we expect
    for i, monster in ipairs(monsters) do
        assert(monster:Name() == "Hi there", "Monster Name is not 'Hi There' for monster "..i)
        -- HP is default to 100 in the schema, but we change it in generateMonster to 80, so this is a good test to
        -- see if the cleared builder really clears the data.
        assert(monster:Hp() == 100, "HP doesn't equal the default value for monster "..i)
        assert(monster:Mana() == 123, "Monster Mana is not '123' for monster "..i)
        assert(monster:Pos():X() == 3.0, "Monster vec3.X is not '3' for monster "..i)
    end
end

local function testCanonicalData()
    local f = assert(io.open('monsterdata_test.mon', 'rb'))
    local wireData = f:read("*a")
    f:close()    
    checkReadBuffer(wireData)  
end    
    
local function testCreateEmptyString()
    local b = flatbuffers.Builder(0)
    local str = b:CreateString("")
    monster.Start(b)
    monster.AddName(b, str)
    b:Finish(monster.End(b))
    local s = b:Output()
    local data = flatbuffers.binaryArray.New(s)
    local mon = monster.GetRootAsMonster(data, 0)
    assert(mon:Name() == "")
end

local function benchmarkMakeMonster(count, reuseBuilder)
    local fbb = reuseBuilder and flatbuffers.Builder(0)
    local length = #(generateMonster(false, fbb))

    local s = os.clock()
    for i=1,count do
        generateMonster(false, fbb)
    end
    local e = os.clock()    

    local dur = (e - s)
    local rate = count / (dur * 1000)
    local data = (length * count) / (1024 * 1024)
    local dataRate = data / dur
    
    print(string.format('built %d %d-byte flatbuffers in %.2fsec: %.2f/msec, %.2fMB/sec',
        count, length, dur, rate, dataRate))
end

local function benchmarkReadBuffer(count)
    local f = assert(io.open('monsterdata_test.mon', 'rb'))
    local buf = f:read("*a")
    f:close()    
        
    local s = os.clock()
    for i=1,count do
        checkReadBuffer(buf)
    end
    local e = os.clock()
        
    local dur = (e - s)
    local rate = count / (dur * 1000)
    local data = (#buf * count) / (1024 * 1024)
    local dataRate = data / dur
    
    print(string.format('traversed %d %d-byte flatbuffers in %.2fsec: %.2f/msec, %.2fMB/sec',
        count, #buf, dur, rate, dataRate))
end

local function getRootAs_canAcceptString()
    local f = assert(io.open('monsterdata_test.mon', 'rb'))
    local wireData = f:read("*a")
    f:close() 
    assert(type(wireData) == "string", "Data is not a string");
    local mon = monster.GetRootAsMonster(wireData, 0)
    assert(mon:Hp() == 80, "Monster Hp is not 80")
end
    
local function testAccessByteVectorAsString()
    local f = assert(io.open('monsterdata_test.mon', 'rb'))
    local wireData = f:read("*a")
    f:close()
    local mon = monster.GetRootAsMonster(wireData, 0)
    -- the data of byte array Inventory is [0, 1, 2, 3, 4]
    local s = mon:InventoryAsString(1, 3)
    assert(#s == 3)
    for i = 1, #s do
        assert(string.byte(s, i) == i - 1)
    end

    local s = mon:InventoryAsString(2, 5)
    assert(#s == 4)
    for i = 1, #s do
        assert(string.byte(s, i) == i)
    end

    local s = mon:InventoryAsString(5, 5)
    assert(#s == 1)
    assert(string.byte(s, 1) == 4)

    local s = mon:InventoryAsString(2)
    assert(#s == 4)
    for i = 1, #s do
        assert(string.byte(s, i) == i)
    end

    local s = mon:InventoryAsString()
    assert(#s == 5)
    for i = 1, #s do
        assert(string.byte(s, i) == i - 1)
    end
end

local tests = 
{ 
    {   
        f = sizePrefix, 
        d = "Test size prefix",
        args = {{true}, {false}}
    },
    {
        f = fbbClear,
        d = "FlatBufferBuilder Clear",
    },
    {   
        f = testCanonicalData, 
        d = "Tests Canonical flatbuffer file included in repo"       
    },
    {
        f = testCreateEmptyString,
        d = "Avoid infinite loop when creating empty string"
    },
    {
        f = getRootAs_canAcceptString,
        d = "Tests that GetRootAs<type>() generated methods accept strings"
    },
    {
        f = testAccessByteVectorAsString,
        d = "Access byte vector as string"
    },
}

local benchmarks = 
{
    {
        f = benchmarkMakeMonster,
        d = "Benchmark making monsters",
        args = {
            {100}, 
            {1000},
            {10000},
            {10000, true}
        }
    },   
    {
        f = benchmarkReadBuffer,
        d = "Benchmark reading monsters",
        args = {
            {100}, 
            {1000},
            {10000},
            -- uncomment following to run 1 million to compare. 
            -- Took ~141 seconds on my machine
            --{1000000},
        }
    }, 
}

local result, err = xpcall(function()
    flatbuffers = assert(require("flatbuffers"))
    monster = assert(require("MyGame.Example.Monster"))  
    test = assert(require("MyGame.Example.Test"))
    vec3 = assert(require("MyGame.Example.Vec3"))
    
    local function buildArgList(tbl)
        local s = ""
        for _,item in ipairs(tbl) do
            s = s .. tostring(item) .. ","          
        end
        return s:sub(1,-2)
    end
    
    if performBenchmarkTests then
        for _,benchmark in ipairs(benchmarks) do
            table.insert(tests, benchmark)
        end
    end

    local testsPassed, testsFailed = 0,0
    for _,test in ipairs(tests) do
        local allargs = test.args or {{}}
        for _,args in ipairs(allargs) do
            local results, err = xpcall(test.f,debug.traceback, table.unpack(args))        
            if results then
                testsPassed = testsPassed + 1
            else
                testsFailed = testsFailed + 1
                print(string.format(" Test [%s](%s) failed: \n\t%s", 
                        test.d or "", 
                        buildArgList(args),
                        err)) 
            end
        end
    end
    
    local totalTests = testsPassed + testsFailed
    print(string.format("# of test passed: %d / %d (%.2f%%)",
        testsPassed, 
        totalTests, 
        totalTests ~= 0 
            and 100 * (testsPassed / totalTests) 
            or 0)
        )
    
    return 0
end, debug.traceback)

if not result then
    print("Unable to run tests due to test framework error: ",err)
end

os.exit(result and 0 or -1)
