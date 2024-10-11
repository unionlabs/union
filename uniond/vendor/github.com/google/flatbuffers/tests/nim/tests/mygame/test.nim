discard """
  action:   "run"
  exitcode: 0
  timeout:  60.0
"""
import std/unittest
import std/options
import flatbuffers
import ../../../MyGame/Example/Test
import ../../../MyGame/Example/Monster
import ../../../MyGame/Example/Vec3
import ../../../MyGame/Example/Color as ColorMod
import ../../../MyGame/Example/Any as AnyMod

proc verifyMonster(monster: var Monster) =
  check(monster.hp == 80)
  check(monster.mana == 150)
  check(monster.name == "MyMonster")
  check(monster.pos.isSome)
  let pos = monster.pos.get()
  check(pos.x == 1)
  check(pos.y == 2)
  check(pos.z == 3)
  check(pos.test1 == 3)
  check(pos.test2 == Color.Green)
  check(pos.test3.a == 5)
  check(pos.test3.b == 6)
  check(monster.testType == Any.Monster)
  check(monster.test.isSome)
  let monster2 = Monster(tab: monster.test.get())
  check(monster2.name == "Fred")
  check((monster.mana = 10) == false)
  check(monster.mana == 150)
  check(monster.inventoryLength == 5)
  var sum: uint8 = 0
  for item in monster.inventory:
    sum += item
  check(sum == 10)
  check(monster.test4Length == 2)

  let test0 = monster.test4(0)
  let test1 = monster.test4(1)
  var sum0 = test0.a + test0.b
  var sum1 = test1.a + test1.b
  check(sum0 + sum1 == 100)

  check(monster.testarrayofstringLength == 2)
  check(monster.testarrayofstring(0) == "test1")
  check(monster.testarrayofstring(1) == "test2")
  check(monster.testbool == true)


suite "TestMyGame":

  test "testData":
    let data: seq[byte] = @[byte(48), 0, 0, 0, 77, 79, 78, 83, 0, 0, 0, 0, 36,
        0, 72, 0, 40, 0, 0, 0, 38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0,
        16, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 164, 0, 0, 0,
        0, 0, 0, 1, 60, 0, 0, 0, 68, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0,
        0, 120, 0, 0, 0, 0, 0, 80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 2, 0, 0, 0,
        64, 0, 0, 0, 48, 0, 0, 0, 2, 0, 0, 0, 30, 0, 40, 0, 10, 0, 20, 0, 152,
        255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0, 70, 114, 101, 100, 0, 0, 0, 0, 5,
        0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 50, 0,
        0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 0, 0, 0, 9, 0, 0, 0, 77, 121,
        77, 111, 110, 115, 116, 101, 114, 0, 0, 0, 3, 0, 0, 0, 20, 0, 0, 0, 36,
        0, 0, 0, 4, 0, 0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255,
        255, 36, 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28,
        0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97,
        114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 111, 0, 0, 0]
    var monster: Monster
    GetRootAs(monster, data, 0)
    verifyMonster(monster)

  test "testCreateString":
    var fbb = newBuilder(0)
    let name = fbb.Create("Frodo")
    fbb.Finish(name)
    check(fbb.FinishedBytes() == @[byte(4), 0, 0, 0, 5, 0, 0, 0, 70, 114, 111,
        100, 111, 0, 0, 0])

  test "testCreateVector":
    var fbb = newBuilder(0)
    let vec = fbb.Create(@[byte(0), 1, 2, 3, 4])
    fbb.Finish(vec)
    check(fbb.FinishedBytes() == @[byte(4), 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4,
        0, 0, 0])

  test "createSimpleMonster":
    var fbb = newBuilder(0)
    let names = [
      fbb.Create("Frodo"),
      fbb.Create("Barney"),
      fbb.Create("Wilma"),
    ]
    fbb.MonsterStart()
    fbb.MonsterAddName(names[0])
    let monster = fbb.MonsterEnd()
    fbb.Finish(monster)
    check(fbb.FinishedBytes() == @[byte(16), 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0,
        0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97,
        0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70,
        114, 111, 100, 111, 0, 0, 0])

  test "testCreateTestVector":
    var fbb = newBuilder(0)
    fbb.MonsterStartTest4Vector(2)
    discard fbb.TestCreate(a = 30, b = 40)
    discard fbb.TestCreate(a = 10, b = 20)
    let test4 = fbb.EndVector()
    fbb.Finish(test4)
    check(fbb.FinishedBytes() == @[byte(4), 0, 0, 0, 2, 0, 0, 0, 10, 0, 20, 0,
        30, 0, 40, 0])

  test "testTableWithStruct":
    var fbb = newBuilder(0)
    fbb.MonsterStart()
    fbb.MonsterAddPos(fbb.Vec3Create(x = 1,
        y = 2,
        z = 3,
        test1 = 3,
        test2 = Color.Green,
        test3_a = 5, test3_b = 6))

    let monster_end = fbb.MonsterEnd()
    fbb.Finish(monster_end)
    check(fbb.FinishedBytes() == @[byte(12), 0, 0, 0, 0, 0, 6, 0, 36, 0, 4, 0,
        6, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0])

  test "testCreateMonster":
    var fbb = newBuilder(0)
    let names = [
      fbb.Create("Frodo"),
      fbb.Create("Barney"),
      fbb.Create("Wilma"),
    ]

    var offsets: seq[uoffset] = @[]
    fbb.MonsterStart()
    fbb.MonsterAddName(names[0])
    offsets.add(fbb.MonsterEnd())
    fbb.MonsterStart()
    fbb.MonsterAddName(names[1])
    offsets.add(fbb.MonsterEnd())
    fbb.MonsterStart()
    fbb.MonsterAddName(names[2])
    offsets.add(fbb.MonsterEnd())

    let str = fbb.Create("MyMonster")
    let test1 = fbb.Create("test1")
    let test2 = fbb.Create("test2")
    let inv = fbb.Create(@[byte(0), 1, 2, 3, 4])
    let fred = fbb.Create("Fred")
    fbb.MonsterStart()
    fbb.MonsterAddName(fred)
    let mon2 = fbb.MonsterEnd()

    fbb.MonsterStartTest4Vector(2)
    discard fbb.TestCreate(a = 30, b = 40)
    discard fbb.TestCreate(a = 10, b = 20)
    let test4 = fbb.EndVector()

    fbb.MonsterStartTestarrayofstringVector(2)
    fbb.PrependOffsetRelative(test1)
    fbb.PrependOffsetRelative(test2)
    let stringTestVector = fbb.EndVector()

    fbb.MonsterStartTestarrayoftablesVector(3)
    fbb.PrependOffsetRelative(offsets[0])
    fbb.PrependOffsetRelative(offsets[1])
    fbb.PrependOffsetRelative(offsets[2])
    let tableTestVector = fbb.EndVector()

    fbb.MonsterStart()
    fbb.MonsterAddPos(fbb.Vec3Create(x = 1,
        y = 2,
        z = 3,
        test1 = 3,
        test2 = Color.Green,
        test3_a = 5, test3_b = 6))
    fbb.MonsterAddHp(80)
    fbb.MonsterAddName(str)
    fbb.MonsterAddInventory(inv)
    fbb.MonsterAddTestType(Any.Monster.uint8)
    fbb.MonsterAddTest(mon2)
    fbb.MonsterAddTest4(test4)
    fbb.MonsterAddTestarrayofstring(stringTestVector)
    fbb.MonsterAddTestbool(true)
    fbb.MonsterAddTestarrayoftables(tableTestVector)
    let monster_end = fbb.MonsterEnd()
    fbb.Finish(monster_end)
    check(fbb.FinishedBytes() == @[byte(40), 0, 0, 0, 36, 0, 72, 0, 40, 0, 0, 0,
        38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0, 16, 0, 12, 0, 4, 0, 0, 0,
        0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 68, 0, 0, 0, 0, 0, 0, 1, 76, 0, 0, 0,
        84, 0, 0, 0, 92, 0, 0, 0, 0, 0, 0, 1, 104, 0, 0, 0, 136, 0, 0, 0, 0, 0,
        80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 3, 0, 0, 0, 108, 0, 0, 0, 112, 0,
        0, 0, 128, 0, 0, 0, 2, 0, 0, 0, 52, 0, 0, 0, 60, 0, 0, 0, 2, 0, 0, 0,
        10, 0, 20, 0, 30, 0, 40, 0, 168, 255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0,
        70, 114, 101, 100, 0, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 5, 0,
        0, 0, 116, 101, 115, 116, 50, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116,
        49, 0, 0, 0, 9, 0, 0, 0, 77, 121, 77, 111, 110, 115, 116, 101, 114, 0,
        0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255, 255, 36, 0, 0, 0,
        12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0,
        0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101,
        121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 111, 0, 0, 0])
