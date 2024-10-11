/*
 * Copyright 2021 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
@file:Suppress("UNCHECKED_CAST")

package com.google.flatbuffers.kotlin

import Attacker
import AttackerOffsetArray
import CharacterEArray
import dictionaryLookup.LongFloatEntry
import dictionaryLookup.LongFloatMap
import Movie
import dictionaryLookup.LongFloatEntryOffsetArray
import myGame.example.*
import myGame.example.Test.Companion.createTest
import optionalScalars.OptionalByte
import optionalScalars.ScalarStuff
import kotlin.test.Test
import kotlin.test.assertEquals


@ExperimentalUnsignedTypes
class FlatBufferBuilderTest {

  @Test
  fun testSingleTable() {
    val fbb = FlatBufferBuilder()
    val name = fbb.createString("Frodo")
    val invValues = ubyteArrayOf(10u, 11u, 12u, 13u, 14u)
    val inv = Monster.createInventoryVector(fbb, invValues)
    Monster.startMonster(fbb)
    Monster.addPos(
      fbb, Vec3.createVec3(
        fbb, 1.0f, 2.0f, 3.0f, 3.0,
        Color.Green, 5.toShort(), 6.toByte()
      )
    )
    Monster.addHp(fbb, 80.toShort())
    Monster.addName(fbb, name)
    Monster.addMana(fbb, 150)
    Monster.addInventory(fbb, inv)
    Monster.addTestType(fbb, AnyE.Monster)
    Monster.addTestbool(fbb, true)
    Monster.addTesthashu32Fnv1(fbb, (Int.MAX_VALUE + 1L).toUInt())
    val root = Monster.endMonster(fbb)
    fbb.finish(root)

    val monster = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monster.name, "Frodo")
    assertEquals(monster.mana, 150.toShort())
    assertEquals(monster.hp, 80)

    val pos = monster.pos!!
    assertEquals(monster.inventory(0), invValues[0])
    assertEquals(monster.inventory(1), invValues[1])
    assertEquals(monster.inventory(2), invValues[2])
    assertEquals(monster.inventory(3), invValues[3])
    assertEquals(pos.x, 1.0f)
    assertEquals(pos.y, 2.0f)
    assertEquals(pos.z, 3.0f)
    assertEquals(pos.test1, 3.0)
    assertEquals(pos.test2, Color.Green)
    assertEquals(pos.test3!!.a, 5.toShort())
    assertEquals(pos.test3!!.b, 6.toByte())

    val inventoryBuffer = monster.inventoryAsBuffer()
    assertEquals(invValues.size, inventoryBuffer.limit)
    for (i in invValues.indices) {
      assertEquals(invValues[i], inventoryBuffer.getUByte(i))
    }
  }

  @Test
  fun testSortedVector() {
    val fbb = FlatBufferBuilder()
    val names = arrayOf(fbb.createString("Frodo"), fbb.createString("Barney"), fbb.createString("Wilma"))
    val monsters = MonsterOffsetArray(3) {
      Monster.startMonster(fbb)
      Monster.addName(fbb, names[it])
      Monster.endMonster(fbb)
    }
    val ary = Monster.createTestarrayoftablesVector(fbb, monsters)
    Monster.startMonster(fbb)
    Monster.addName(fbb, names[0])
    Monster.addTestarrayoftables(fbb, ary)
    val root = Monster.endMonster(fbb)
    fbb.finish(root)
    val a = Monster.asRoot(fbb.dataBuffer())
    assertEquals(a.name, "Frodo")
    assertEquals(a.testarrayoftablesLength, 3)
    val monster0 = a.testarrayoftables(0)!!
    val monster1 = a.testarrayoftables(1)!!
    val monster2 = a.testarrayoftables(2)!!
    assertEquals(monster0.name, "Frodo")
    assertEquals(monster1.name, "Barney")
    assertEquals(monster2.name, "Wilma")

    // test AsBuffer feature

  }

  @Test
  fun testCreateBufferVector() {
    val fbb = FlatBufferBuilder(16)
    val str = fbb.createString("MyMonster")
    val inventory = ubyteArrayOf(0u, 1u, 2u, 3u, 4u, 5u, 6u, 88u, 99u, 122u, 1u)
    val vec = Monster.createInventoryVector(fbb, inventory)
    Monster.startMonster(fbb)
    Monster.addInventory(fbb, vec)
    Monster.addName(fbb, str)
    val monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)
    val monsterObject: Monster = Monster.asRoot(fbb.dataBuffer())
    val iBuffer = monsterObject.inventoryAsBuffer()

    assertEquals(monsterObject.inventoryLength, inventory.size)
    assertEquals(iBuffer.limit, inventory.size)

    for (i in inventory.indices) {
      assertEquals(inventory[i], monsterObject.inventory(i))
      assertEquals(inventory[i], iBuffer.getUByte(i))
    }
  }

  @Test
  fun testCreateUninitializedVector() {
    val fbb = FlatBufferBuilder(16)
    val str = fbb.createString("MyMonster")
    val inventory = byteArrayOf(10, 11, 12, 13, 14)
    val uninitializedBuffer = fbb.createUnintializedVector(1, inventory.size, 1)
    for (i in inventory) {
      uninitializedBuffer.put(i)
    }
    val vec = fbb.endVector<UByte>()
    Monster.startMonster(fbb)
    Monster.addInventory(fbb, vec)
    Monster.addName(fbb, str)
    val monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)
    val monsterObject: Monster = Monster.asRoot(fbb.dataBuffer())
    assertEquals(inventory[1].toUByte(), monsterObject.inventory(1))
    assertEquals(inventory.size, monsterObject.inventoryLength)
    val inventoryBuffer = monsterObject.inventoryAsBuffer()
    assertEquals(inventory[1].toInt().toUByte(), inventoryBuffer.getUByte(1))
    assertEquals(inventory.size, inventoryBuffer.limit)
  }

  @Test
  fun testBuilderBasics() {
    val fbb = FlatBufferBuilder()
    val names = arrayOf(fbb.createString("Frodo"), fbb.createString("Barney"), fbb.createString("Wilma"))
    val off = Array<Offset<Monster>>(3) { Offset(0) }
    Monster.startMonster(fbb)
    Monster.addName(fbb, names[0])
    off[0] = Monster.endMonster(fbb)
    Monster.startMonster(fbb)
    Monster.addName(fbb, names[1])
    off[1] = Monster.endMonster(fbb)
    Monster.startMonster(fbb)
    Monster.addName(fbb, names[2])
    off[2] = Monster.endMonster(fbb)
    val sortMons = fbb.createSortedVectorOfTables(Monster(), off)

    // We set up the same values as monsterdata.json:

    val inv = Monster.createInventoryVector(fbb, byteArrayOf(0,1,2,3,4).toUByteArray())

    val fred = fbb.createString("Fred")
    Monster.startMonster(fbb)
    Monster.addName(fbb, fred)
    val mon2 = Monster.endMonster(fbb)

    Monster.startTest4Vector(fbb, 2)
    createTest(fbb, 10.toShort(), 20.toByte())
    createTest(fbb, 30.toShort(), 40.toByte())
    val test4 = fbb.endVector<myGame.example.Test>()

    val strings = StringOffsetArray(2) { fbb.createString("test$it") }
    val testArrayOfString =
      Monster.createTestarrayofstringVector(fbb, strings)

    Monster.startMonster(fbb)
    Monster.addName(fbb, names[0])
    Monster.addPos(fbb, Vec3.createVec3(
      fbb, 1.0f, 2.0f, 3.0f, 3.0,
      Color.Green, 5.toShort(), 6.toByte()
    ))
    Monster.addHp(fbb, 80)
    Monster.addMana(fbb, 150)
    Monster.addInventory(fbb, inv)
    Monster.addTestType(fbb, AnyE.Monster)
    Monster.addTest(fbb, mon2.toUnion())
    Monster.addTest4(fbb, test4)
    Monster.addTestarrayofstring(fbb, testArrayOfString)
    Monster.addTestbool(fbb, true)
    Monster.addTesthashu32Fnv1(fbb, (Int.MAX_VALUE + 1L).toUInt())
    Monster.addTestarrayoftables(fbb, sortMons)
    val mon = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, mon)
    //Attempt to mutate Monster fields and check whether the buffer has been mutated properly
    // revert to original values after testing
    val monster = Monster.asRoot(fbb.dataBuffer())

    // mana is optional and does not exist in the buffer so the mutation should fail
    // the mana field should retain its default value
    assertEquals(monster.mana, 150.toShort())
    assertEquals(monster.hp, 80)

    // Accessing a vector of sorted by the key tables
    assertEquals(monster.testarrayoftables(0)!!.name, "Barney")
    assertEquals(monster.testarrayoftables(1)!!.name, "Frodo")
    assertEquals(monster.testarrayoftables(2)!!.name, "Wilma")

    // Example of searching for a table by the key
    assertEquals(monster.testarrayoftablesByKey("Frodo")!!.name, "Frodo")
    assertEquals(monster.testarrayoftablesByKey("Barney")!!.name, "Barney")
    assertEquals(monster.testarrayoftablesByKey("Wilma")!!.name, "Wilma")

    for (i in 0 until monster.inventoryLength) {
      assertEquals(monster.inventory(i), (i).toUByte())
    }

    // get a struct field and edit one of its fields
    val pos2 = monster.pos!!
    assertEquals(pos2.x, 1.0f)
    assertEquals(pos2.test2, Color.Green)
  }

  @Test
  fun testVectorOfUnions() {
    val fbb = FlatBufferBuilder()
    val swordAttackDamage = 1
    val attacker = Attacker.createAttacker(fbb, swordAttackDamage).toUnion()
    val attackers = UnionOffsetArray(1) { attacker }
    val characters = CharacterEArray(1)
    characters[0] = CharacterE.MuLan.value

    Movie.finishMovieBuffer(
      fbb,
      Movie.createMovie(
        fbb,
        CharacterE.MuLan,
        attacker,
        Movie.createCharactersTypeVector(fbb, characters),
        Movie.createCharactersVector(fbb, attackers)
      )
    )

    val movie: Movie = Movie.asRoot(fbb.dataBuffer())



    assertEquals(movie.charactersTypeLength, 1)
    assertEquals(movie.charactersLength, 1)

    assertEquals(movie.charactersType(0), CharacterE.MuLan)
    assertEquals((movie.characters(Attacker(), 0) as Attacker).swordAttackDamage, swordAttackDamage)
  }

  @Test
  fun TestVectorOfBytes() {
    val fbb = FlatBufferBuilder(16)
    var str = fbb.createString("ByteMonster")
    val data = ubyteArrayOf(0u, 1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u, 9u)
    var offset = Monster.createInventoryVector(fbb, data)
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    var monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject = Monster.asRoot(fbb.dataBuffer())
    assertEquals("ByteMonster", monsterObject.name)
    assertEquals(data.size, monsterObject.inventoryLength)
    assertEquals(monsterObject.inventory(4), data[4])
    offset = fbb.createByteVector(data.toByteArray()) as VectorOffset<UByte> // TODO: fix me
    str = fbb.createString("ByteMonster")
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject2 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject2.inventoryLength, data.size)
    for (i in data.indices) {
      assertEquals(monsterObject2.inventory(i), data[i])
    }
    fbb.clear()
    offset = fbb.createByteVector(data.toByteArray(), 3, 4) as VectorOffset<UByte>
    str = fbb.createString("ByteMonster")
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject3 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject3.inventoryLength, 4)
    assertEquals(monsterObject3.inventory(0), data[3])
    fbb.clear()
    offset = Monster.createInventoryVector(fbb, data)
    str = fbb.createString("ByteMonster")
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject4 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject4.inventoryLength, data.size)
    assertEquals(monsterObject4.inventory(8), 8u)
    fbb.clear()

    val largeData = ByteArray(1024)
    offset = fbb.createByteVector(largeData) as VectorOffset<UByte> //TODO: fix me
    str = fbb.createString("ByteMonster")
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject5 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject5.inventoryLength, largeData.size)
    assertEquals(monsterObject5.inventory(25), largeData[25].toUByte())
    fbb.clear()

    var bb = ArrayReadBuffer(largeData, 512)
    offset = fbb.createByteVector(bb) as VectorOffset<UByte> //TODO: fix me
    str = fbb.createString("ByteMonster")
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)
    val monsterObject6 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject6.inventoryLength, 512)
    assertEquals(monsterObject6.inventory(0), largeData[0].toUByte())
    fbb.clear()

    bb = ArrayReadBuffer(largeData, largeData.size - 216)
    val stringBuffer = ArrayReadBuffer("AlreadyBufferedString".encodeToByteArray())
    offset = fbb.createByteVector(bb) as VectorOffset<UByte> //TODO: fix me
    str = fbb.createString(stringBuffer)
    Monster.startMonster(fbb)
    Monster.addName(fbb, str)
    Monster.addInventory(fbb, offset)
    monster1 = Monster.endMonster(fbb)
    Monster.finishMonsterBuffer(fbb, monster1)

    val monsterObject7 = Monster.asRoot(fbb.dataBuffer())
    assertEquals(monsterObject7.inventoryLength, 216)
    assertEquals("AlreadyBufferedString", monsterObject7.name)
  }

  @Test
  fun testEnums() {
    assertEquals(Color.name(Color.Red), "Red")
    assertEquals(Color.name(Color.Blue), "Blue")
    assertEquals(AnyE.name(AnyE.None), "NONE")
    assertEquals(AnyE.name(AnyE.Monster), "Monster")
  }

  @Test
  fun testSharedStringPool() {
    val fb = FlatBufferBuilder(1)
    val testString = "My string"
    val offset = fb.createSharedString(testString)
    for (i in 0..9) {
      assertEquals(offset, fb.createSharedString(testString))
    }
  }

  @Test
  fun testScalarOptional() {
    val fbb = FlatBufferBuilder(1)
    ScalarStuff.startScalarStuff(fbb)
    var pos = ScalarStuff.endScalarStuff(fbb)
    fbb.finish(pos)
    var scalarStuff: ScalarStuff = ScalarStuff.asRoot(fbb.dataBuffer())
    assertEquals(scalarStuff.justI8, 0.toByte())
    assertEquals(scalarStuff.maybeI8, null)
    assertEquals(scalarStuff.defaultI8, 42.toByte())
    assertEquals(scalarStuff.justU8, 0u)
    assertEquals(scalarStuff.maybeU8, null)
    assertEquals(scalarStuff.defaultU8, 42u)
    assertEquals(scalarStuff.justI16, 0.toShort())
    assertEquals(scalarStuff.maybeI16, null)
    assertEquals(scalarStuff.defaultI16, 42.toShort())
    assertEquals(scalarStuff.justU16, 0u)
    assertEquals(scalarStuff.maybeU16, null)
    assertEquals(scalarStuff.defaultU16, 42u)
    assertEquals(scalarStuff.justI32, 0)
    assertEquals(scalarStuff.maybeI32, null)
    assertEquals(scalarStuff.defaultI32, 42)
    assertEquals(scalarStuff.justU32, 0u)
    assertEquals(scalarStuff.maybeU32, null)
    assertEquals(scalarStuff.defaultU32, 42u)
    assertEquals(scalarStuff.justI64, 0L)
    assertEquals(scalarStuff.maybeI64, null)
    assertEquals(scalarStuff.defaultI64, 42L)
    assertEquals(scalarStuff.justU64, 0UL)
    assertEquals(scalarStuff.maybeU64, null)
    assertEquals(scalarStuff.defaultU64, 42UL)
    assertEquals(scalarStuff.justF32, 0.0f)
    assertEquals(scalarStuff.maybeF32, null)
    assertEquals(scalarStuff.defaultF32, 42.0f)
    assertEquals(scalarStuff.justF64, 0.0)
    assertEquals(scalarStuff.maybeF64, null)
    assertEquals(scalarStuff.defaultF64, 42.0)
    assertEquals(scalarStuff.justBool, false)
    assertEquals(scalarStuff.maybeBool, null)
    assertEquals(scalarStuff.defaultBool, true)
    assertEquals(scalarStuff.justEnum, OptionalByte.None)
    assertEquals(scalarStuff.maybeEnum, null)
    assertEquals(scalarStuff.defaultEnum, OptionalByte.One)
    fbb.clear()
    ScalarStuff.startScalarStuff(fbb)
    ScalarStuff.addJustI8(fbb, 5.toByte())
    ScalarStuff.addMaybeI8(fbb, 5.toByte())
    ScalarStuff.addDefaultI8(fbb, 5.toByte())
    ScalarStuff.addJustU8(fbb, 6u)
    ScalarStuff.addMaybeU8(fbb, 6u)
    ScalarStuff.addDefaultU8(fbb, 6u)
    ScalarStuff.addJustI16(fbb, 7.toShort())
    ScalarStuff.addMaybeI16(fbb, 7.toShort())
    ScalarStuff.addDefaultI16(fbb, 7.toShort())
    ScalarStuff.addJustU16(fbb, 8u)
    ScalarStuff.addMaybeU16(fbb, 8u)
    ScalarStuff.addDefaultU16(fbb, 8u)
    ScalarStuff.addJustI32(fbb, 9)
    ScalarStuff.addMaybeI32(fbb, 9)
    ScalarStuff.addDefaultI32(fbb, 9)
    ScalarStuff.addJustU32(fbb, 10u)
    ScalarStuff.addMaybeU32(fbb, 10u)
    ScalarStuff.addDefaultU32(fbb, 10u)
    ScalarStuff.addJustI64(fbb, 11L)
    ScalarStuff.addMaybeI64(fbb, 11L)
    ScalarStuff.addDefaultI64(fbb, 11L)
    ScalarStuff.addJustU64(fbb, 12UL)
    ScalarStuff.addMaybeU64(fbb, 12UL)
    ScalarStuff.addDefaultU64(fbb, 12UL)
    ScalarStuff.addJustF32(fbb, 13.0f)
    ScalarStuff.addMaybeF32(fbb, 13.0f)
    ScalarStuff.addDefaultF32(fbb, 13.0f)
    ScalarStuff.addJustF64(fbb, 14.0)
    ScalarStuff.addMaybeF64(fbb, 14.0)
    ScalarStuff.addDefaultF64(fbb, 14.0)
    ScalarStuff.addJustBool(fbb, true)
    ScalarStuff.addMaybeBool(fbb, true)
    ScalarStuff.addDefaultBool(fbb, true)
    ScalarStuff.addJustEnum(fbb, OptionalByte.Two)
    ScalarStuff.addMaybeEnum(fbb, OptionalByte.Two)
    ScalarStuff.addDefaultEnum(fbb, OptionalByte.Two)
    pos = ScalarStuff.endScalarStuff(fbb)
    fbb.finish(pos)
    scalarStuff = ScalarStuff.asRoot(fbb.dataBuffer())
    assertEquals(scalarStuff.justI8, 5.toByte())
    assertEquals(scalarStuff.maybeI8, 5.toByte())
    assertEquals(scalarStuff.defaultI8, 5.toByte())
    assertEquals(scalarStuff.justU8, 6u)
    assertEquals(scalarStuff.maybeU8, 6u)
    assertEquals(scalarStuff.defaultU8, 6u)
    assertEquals(scalarStuff.justI16, 7.toShort())
    assertEquals(scalarStuff.maybeI16, 7.toShort())
    assertEquals(scalarStuff.defaultI16, 7.toShort())
    assertEquals(scalarStuff.justU16, 8u)
    assertEquals(scalarStuff.maybeU16, 8u)
    assertEquals(scalarStuff.defaultU16, 8u)
    assertEquals(scalarStuff.justI32, 9)
    assertEquals(scalarStuff.maybeI32, 9)
    assertEquals(scalarStuff.defaultI32, 9)
    assertEquals(scalarStuff.justU32, 10u)
    assertEquals(scalarStuff.maybeU32, 10u)
    assertEquals(scalarStuff.defaultU32, 10u)
    assertEquals(scalarStuff.justI64, 11L)
    assertEquals(scalarStuff.maybeI64, 11L)
    assertEquals(scalarStuff.defaultI64, 11L)
    assertEquals(scalarStuff.justU64, 12UL)
    assertEquals(scalarStuff.maybeU64, 12UL)
    assertEquals(scalarStuff.defaultU64, 12UL)
    assertEquals(scalarStuff.justF32, 13.0f)
    assertEquals(scalarStuff.maybeF32, 13.0f)
    assertEquals(scalarStuff.defaultF32, 13.0f)
    assertEquals(scalarStuff.justF64, 14.0)
    assertEquals(scalarStuff.maybeF64, 14.0)
    assertEquals(scalarStuff.defaultF64, 14.0)
    assertEquals(scalarStuff.justBool, true)
    assertEquals(scalarStuff.maybeBool, true)
    assertEquals(scalarStuff.defaultBool, true)
    assertEquals(scalarStuff.justEnum, OptionalByte.Two)
    assertEquals(scalarStuff.maybeEnum, OptionalByte.Two)
    assertEquals(scalarStuff.defaultEnum, OptionalByte.Two)
  }

// @todo Seems like nesting code generation is broken for all generators.
// disabling test for now.
//  @Test
//  fun testNamespaceNesting() {
//    // reference / manipulate these to verify compilation
//    val fbb = FlatBufferBuilder(1)
//    TableInNestedNS.startTableInNestedNS(fbb)
//    TableInNestedNS.addFoo(fbb, 1234)
//    val nestedTableOff = TableInNestedNS.endTableInNestedNs(fbb)
//    TableInFirstNS.startTableInFirstNS(fbb)
//    TableInFirstNS.addFooTable(fbb, nestedTableOff)
//    TableInFirstNS.endTableInFirstNs(fbb)
//  }

  @Test
  fun testNestedFlatBuffer() {
    val nestedMonsterName = "NestedMonsterName"
    val nestedMonsterHp: Short = 600
    val nestedMonsterMana: Short = 1024
    val fbb1 = FlatBufferBuilder(16)
    val str1 = fbb1.createString(nestedMonsterName)
    Monster.startMonster(fbb1)
    Monster.addName(fbb1, str1)
    Monster.addHp(fbb1, nestedMonsterHp)
    Monster.addMana(fbb1, nestedMonsterMana)
    val monster1 = Monster.endMonster(fbb1)
    Monster.finishMonsterBuffer(fbb1, monster1)
    val fbb1Bytes: ByteArray = fbb1.sizedByteArray()
    val fbb2 = FlatBufferBuilder(16)
    val str2 = fbb2.createString("My Monster")
    val nestedBuffer = Monster.createTestnestedflatbufferVector(fbb2, fbb1Bytes.toUByteArray())
    Monster.startMonster(fbb2)
    Monster.addName(fbb2, str2)
    Monster.addHp(fbb2, 50.toShort())
    Monster.addMana(fbb2, 32.toShort())
    Monster.addTestnestedflatbuffer(fbb2, nestedBuffer)
    val monster = Monster.endMonster(fbb2)
    Monster.finishMonsterBuffer(fbb2, monster)

    // Now test the data extracted from the nested buffer
    val mons = Monster.asRoot(fbb2.dataBuffer())
    val nestedMonster = mons.testnestedflatbufferAsMonster
    assertEquals(nestedMonsterMana, nestedMonster!!.mana)
    assertEquals(nestedMonsterHp, nestedMonster.hp)
    assertEquals(nestedMonsterName, nestedMonster.name)
  }

  @Test
  fun testDictionaryLookup() {
    val fbb = FlatBufferBuilder(16)
    val lfIndex = LongFloatEntry.createLongFloatEntry(fbb, 0, 99.0f)
    val vectorEntriesIdx = LongFloatMap.createEntriesVector(fbb, LongFloatEntryOffsetArray(1) { lfIndex })
    val rootIdx = LongFloatMap.createLongFloatMap(fbb, vectorEntriesIdx)
    LongFloatMap.finishLongFloatMapBuffer(fbb, rootIdx)
    val map: LongFloatMap = LongFloatMap.asRoot(fbb.dataBuffer())

    assertEquals(1, map.entriesLength)

    val e: LongFloatEntry = map.entries(0)!!
    assertEquals(0L, e.key)
    assertEquals(99.0f, e.value)
    val e2: LongFloatEntry = map.entriesByKey(0)!!
    assertEquals(0L, e2.key)
    assertEquals(99.0f, e2.value)
  }
}
