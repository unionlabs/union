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
package com.google.flatbuffers.kotlin

import com.google.flatbuffers.kotlin.FlexBuffersBuilder.Companion.SHARE_NONE
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class FlexBuffersTest {

  @Test
  fun testWriteInt() {
    val values = listOf(
      Byte.MAX_VALUE.toLong() to 3,
      Short.MAX_VALUE.toLong() to 4,
      Int.MAX_VALUE.toLong() to 6,
      Long.MAX_VALUE to 10
    )
    val builder = FlexBuffersBuilder()
    values.forEach {
      builder.clear()
      builder.put(it.first)
      val data = builder.finish()
      val ref = getRoot(data)
      // although we put a long, it is shrink to a byte
      assertEquals(it.second, data.limit)
      assertEquals(it.first, ref.toLong())
    }
  }

  @Test
  fun testWriteUInt() {
    val values = listOf(
      UByte.MAX_VALUE.toULong() to 3,
      UShort.MAX_VALUE.toULong() to 4,
      UInt.MAX_VALUE.toULong() to 6,
      ULong.MAX_VALUE to 10
    )
    val builder = FlexBuffersBuilder()
    values.forEach {
      builder.clear()
      builder.put(it.first)
      val data = builder.finish()
      val ref = getRoot(data)
      // although we put a long, it is shrink to a byte
      assertEquals(it.second, data.limit)
      assertEquals(it.first, ref.toULong())
    }
  }

  @Test
  fun testWriteString() {
    val text = "Ḧ̵̘́ȩ̵̐l̶̿͜l̶͚͝o̷̦̚ ̷̫̊w̴̤͊ö̸̞́r̴͎̾l̷͚̐d̶̰̍"
    val builder = FlexBuffersBuilder()
    builder.put(text)
    val data = builder.finish()
    val ref = getRoot(data)
    assertEquals(text, ref.toString())
  }

  @Test
  fun testInt8Array() {
    val ary = intArrayOf(1, 2, 3, 4)
    val builder = FlexBuffersBuilder()
    builder.put(intArrayOf(1, 2, 3, 4))
    val data = builder.finish()
    val ref = getRoot(data)
    // although we put a long, it is shrink to a byte
    assertEquals(8, data.limit)
    assertArrayEquals(ary, ref.toIntArray())
  }

  @Test
  fun testShortArray() {
    val builder = FlexBuffersBuilder(ArrayReadWriteBuffer(20))
    val numbers = ShortArray(10) { it.toShort() }
    builder.put(numbers)
    val data = builder.finish()
    val ref = getRoot(data)
    assertArrayEquals(numbers, ref.toShortArray())
  }

  @Test
  fun testHugeArray() {
    val builder = FlexBuffersBuilder()
    val numbers = IntArray(1024) { it }
    builder.put(numbers)
    val data = builder.finish()
    val ref = getRoot(data)
    assertArrayEquals(numbers, ref.toIntArray())
  }

  @Test
  fun testFloatArray() {
    val builder = FlexBuffersBuilder()
    val numbers = FloatArray(1024) { it * 0.05f }
    builder.put(numbers)
    val data = builder.finish()
    val ref = getRoot(data)
    assertArrayEquals(numbers, ref.toFloatArray())
  }

  @Test
  fun testDoubleArray() {
    val builder = FlexBuffersBuilder()
    val numbers = DoubleArray(1024) { it * 0.0005 }
    builder.put(numbers)
    val data = builder.finish()
    val ref = getRoot(data)
    assertArrayEquals(numbers, ref.toDoubleArray())
  }

  @Test
  fun testLongArray() {
    val ary: LongArray = longArrayOf(0, Short.MIN_VALUE.toLong(), Int.MAX_VALUE.toLong(), Long.MAX_VALUE)
    val builder = FlexBuffersBuilder()
    builder.put(ary)
    val data = builder.finish()
    val ref = getRoot(data)
    // although we put a long, it is shrink to a byte
    assertArrayEquals(ary, ref.toLongArray())
  }

  @Test
  fun testStringArray() {
    val ary = Array(5) { "Hello world number: $it" }
    val builder = FlexBuffersBuilder(ArrayReadWriteBuffer(20), SHARE_NONE)
    builder.putVector {
      ary.forEach { put(it) }
    }
    val data = builder.finish()
    val vec = getRoot(data).toVector()
    // although we put a long, it is shrink to a byte
    assertEquals(5, vec.size)
    val stringAry = vec.map { it.toString() }.toTypedArray()
    // although we put a long, it is shrink to a byte
    assertArrayEquals(ary, stringAry)
  }

  @Test
  fun testBlobArray() {
    val ary = ByteArray(1000) { Random.nextInt().toByte() }
    val builder = FlexBuffersBuilder()
    builder.put(ary)
    val data = builder.finish()
    val blob = getRoot(data).toBlob()
    // although we put a long, it is shrink to a byte
    assertArrayEquals(ary, blob.toByteArray())
    for (i in 0 until blob.size) {
      assertEquals(ary[i], blob[i])
    }
  }

  @Test
  fun testArrays() {
    val builder = FlexBuffersBuilder()
    val ary: Array<String> = Array(5) { "Hello world number: $it" }
    val numbers = IntArray(10) { it }
    val doubles = DoubleArray(10) { it * 0.35 }

    // add 3 level array of arrays in the following way
    // [ [ "..", ...] [ "..", ..., [ "..", ...] ] ]
    val vec = builder.startVector()

    // [0, 1, 2, 3 ,4 ,5 ,6 ,7 ,8, 9]
    val vec1 = builder.startVector()
    numbers.forEach { builder.put(it) }
    builder.endTypedVector(vec1)

    // [0, 2, 4, 6 , 8, 10, 12, 14, 16, 18]
    builder.putTypedVector { doubles.forEach { put(it) } }

    // nested array
    // [ "He..", "He..", "He..", "He..", "He..", [ "He..", "He..", "He..", "He..", "He.." ] ]
    val vec3 = builder.startVector()
    ary.forEach { builder.put(it) }
    builder.putVector { ary.forEach { put("inner: $it") } }
    builder.endVector(vec3)

    builder.endVector(vec)

    val data = builder.finish()
    val ref = getRoot(data)
    val vecRef = getRoot(data).toVector()
    // although we put a long, it is shrink to a byte
    assertEquals(3, vecRef.size)

    assertArrayEquals(numbers, vecRef[0].toVector().map { it.toInt() }.toIntArray())
    assertArrayEquals(doubles, ref[1].toDoubleArray())
    assertEquals("Hello world number: 4", vecRef[2][4].toString())
    assertEquals("inner: Hello world number: 4", vecRef[2][5][4].toString())
    assertEquals("inner: Hello world number: 4", ref[2][5][4].toString())
  }

  @Test
  fun testMap() {
    val builder = FlexBuffersBuilder(shareFlag = FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    builder.putVector {
      put(10)
      putMap {
        this["chello"] = "world"
        this["aint"] = 10
        this["bfloat"] = 12.3
      }
      put("aString")
    }

    val ref = getRoot(builder.finish())
    val map = ref.toVector()
    assertEquals(3, map.size)
    assertEquals(10, map[0].toInt())
    assertEquals("aString", map[2].toString())
    assertEquals("world", map[1]["chello"].toString())
    assertEquals(10, map[1]["aint"].toInt())
    assertEquals(12.3, map[1]["bfloat"].toDouble())
  }

  @Test
  fun testMultiMap() {
    val builder = FlexBuffersBuilder(shareFlag = FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    builder.putMap {
      this["hello"] = "world"
      this["int"] = 10
      this["float"] = 12.3
      this["intarray"] = intArrayOf(1, 2, 3, 4, 5)
      this.putMap("myMap") {
        this["cool"] = "beans"
      }
    }

    val ref = getRoot(builder.finish())
    val map = ref.toMap()
    assertEquals(5, map.size)
    assertEquals("world", map["hello"].toString())
    assertEquals(10, map["int"].toInt())
    assertEquals(12.3, map["float"].toDouble())
    assertArrayEquals(intArrayOf(1, 2, 3, 4, 5), map["intarray"].toIntArray())
    assertEquals("beans", ref["myMap"]["cool"].toString())
    assertEquals(true, "myMap" in map)
    assertEquals(true, "cool" in map["myMap"].toMap())

    // testing null values
    assertEquals(true, ref["invalid_key"].isNull)

    val keys = map.keys.toTypedArray()
    arrayOf("hello", "int", "float", "intarray", "myMap").sortedArray().forEachIndexed { i: Int, it: String ->
      assertEquals(it, keys[i].toString())
    }
  }

  @Test
  fun testBigStringMap() {
    val builder = FlexBuffersBuilder(shareFlag = FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)

    val stringKey = Array(10000) { "Ḧ̵̘́ȩ̵̐myFairlyBigKey$it" }
    val stringValue = Array(10000) { "Ḧ̵̘́ȩ̵̐myFairlyBigValue$it" }
    val hashMap = mutableMapOf<String, String>()
    val pos = builder.startMap()
    for (i in stringKey.indices) {
      builder[stringKey[i]] = stringValue[i]
      hashMap[stringKey[i]] = stringValue[i]
    }
    builder.endMap(pos)
    val ref = getRoot(builder.finish())
    val map = ref.toMap()
    val sortedKeys = stringKey.sortedArray()
    val size = map.size
    for (i in 0 until size) {
      assertEquals(sortedKeys[i], map.keyAsString(i))
      assertEquals(sortedKeys[i], map.keyAt(i).toString())
      assertEquals(hashMap[sortedKeys[i]], map[map.keyAt(i)].toString())
    }
  }

  @Test
  fun testKeysAccess() {
    for (i in 1 until 1000) {
      val utf8String = "ሰማይ አይታረስ ንጉሥ አይከሰስ።$i"
      val bytes = ByteArray(Utf8.encodedLength(utf8String))
      val pos = Utf8.encodeUtf8Array(utf8String, bytes)
      val key = Key(ArrayReadWriteBuffer(bytes), 0, pos)
      assertEquals(utf8String.length, key.sizeInChars)
      for (j in utf8String.indices) {
        assertEquals(utf8String[j], key[j])
      }
    }
  }
}
