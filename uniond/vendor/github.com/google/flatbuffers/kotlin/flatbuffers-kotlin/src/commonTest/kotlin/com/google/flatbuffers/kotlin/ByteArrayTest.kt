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

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ByteArrayTest {

  @Test
  fun testByte() {
    val testSet = arrayOf(
      67.toByte() to byteArrayOf(67),
      Byte.MIN_VALUE to byteArrayOf(-128),
      Byte.MAX_VALUE to byteArrayOf(127),
      0.toByte() to byteArrayOf(0)
    )
    val data = ByteArray(1)
    testSet.forEach {
      data[0] = it.first
      assertArrayEquals(data, it.second)
      assertEquals(it.first, data[0])
    }
  }

  @Test
  fun testShort() {
    val testSet = arrayOf(
      6712.toShort() to byteArrayOf(56, 26),
      Short.MIN_VALUE to byteArrayOf(0, -128),
      Short.MAX_VALUE to byteArrayOf(-1, 127),
      0.toShort() to byteArrayOf(0, 0,)
    )

    val data = ByteArray(Short.SIZE_BYTES)
    testSet.forEach {
      data.setShort(0, it.first)
      assertArrayEquals(data, it.second)
      assertEquals(it.first, data.getShort(0))
    }
  }

  @Test
  fun testInt() {
    val testSet = arrayOf(
      33333500 to byteArrayOf(-4, -96, -4, 1),
      Int.MIN_VALUE to byteArrayOf(0, 0, 0, -128),
      Int.MAX_VALUE to byteArrayOf(-1, -1, -1, 127),
      0 to byteArrayOf(0, 0, 0, 0)
    )
    val data = ByteArray(Int.SIZE_BYTES)
    testSet.forEach {
      data.setInt(0, it.first)
      assertArrayEquals(data, it.second)
      assertEquals(it.first, data.getInt(0))
    }
  }

  @Test
  fun testLong() {
    val testSet = arrayOf(
      1234567123122890123L to byteArrayOf(-117, -91, 29, -23, 65, 16, 34, 17),
      -1L to byteArrayOf(-1, -1, -1, -1, -1, -1, -1, -1),
      Long.MIN_VALUE to byteArrayOf(0, 0, 0, 0, 0, 0, 0, -128),
      Long.MAX_VALUE to byteArrayOf(-1, -1, -1, -1, -1, -1, -1, 127),
      0L to byteArrayOf(0, 0, 0, 0, 0, 0, 0, 0)
    )
    val data = ByteArray(Long.SIZE_BYTES)
    testSet.forEach {
      data.setLong(0, it.first)
      assertArrayEquals(data, it.second)
      assertEquals(it.first, data.getLong(0))
    }
  }

  @Test
  fun testULong() {
    val testSet = arrayOf(
      1234567123122890123UL to byteArrayOf(-117, -91, 29, -23, 65, 16, 34, 17),
      ULong.MIN_VALUE to byteArrayOf(0, 0, 0, 0, 0, 0, 0, 0),
      (-1L).toULong() to byteArrayOf(-1, -1, -1, -1, -1, -1, -1, -1),
      0UL to byteArrayOf(0, 0, 0, 0, 0, 0, 0, 0)
    )
    val data = ByteArray(ULong.SIZE_BYTES)
    testSet.forEach {
      data.setULong(0, it.first)
      assertArrayEquals(it.second, data)
      assertEquals(it.first, data.getULong(0))
    }
  }

  @Test
  fun testFloat() {
    val testSet = arrayOf(
      3545.56337f to byteArrayOf(4, -103, 93, 69),
      Float.MIN_VALUE to byteArrayOf(1, 0, 0, 0),
      Float.MAX_VALUE to byteArrayOf(-1, -1, 127, 127),
      0f to byteArrayOf(0, 0, 0, 0)
    )
    val data = ByteArray(Float.SIZE_BYTES)
    testSet.forEach {
      data.setFloat(0, it.first)
      assertArrayEquals(data, it.second)
    }
  }

  @Test
  fun testDouble() {
    val testSet = arrayOf(
      123456.523423423412 to byteArrayOf(88, 61, -15, 95, 8, 36, -2, 64),
      Double.MIN_VALUE to byteArrayOf(1, 0, 0, 0, 0, 0, 0, 0),
      Double.MAX_VALUE to byteArrayOf(-1, -1, -1, -1, -1, -1, -17, 127),
      0.0 to byteArrayOf(0, 0, 0, 0, 0, 0, 0, 0)
    )
    val data = ByteArray(Long.SIZE_BYTES)
    testSet.forEach {
      data.setDouble(0, it.first)
      assertArrayEquals(data, it.second)
      assertEquals(it.first, data.getDouble(0))
    }
  }

  @Test
  fun testString() {
    val testSet = "∮ E⋅da = Q"
    val encoded = testSet.encodeToByteArray()
    val data = ByteArray(encoded.size)
    data.setCharSequence(0, testSet)
    assertArrayEquals(encoded, data)
    assertEquals(testSet, data.getString(0, encoded.size))
  }
}

