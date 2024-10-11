package com.google.flatbuffers.kotlin

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class BuffersTest {

  @Test
  fun readBufferStringTest() {
    val text = "Hello world!"
    val bytes = text.encodeToByteArray()
    val fullRead = ArrayReadBuffer(bytes)
    val helloRead = ArrayReadBuffer(bytes, limit = 5)
    val worldRead = fullRead.slice(6, 6)

    assertEquals(bytes.size, fullRead.limit)
    assertEquals(text, fullRead.getString(0, fullRead.limit))
    assertEquals("Hello" , helloRead.getString(0, helloRead.limit))
    assertEquals("world!" , worldRead.getString())
    assertEquals(fullRead.getString(0, 5) , helloRead.getString(0, helloRead.limit))
    assertEquals(fullRead.getString(6, 6) , worldRead.getString(0, worldRead.limit))

    for (i in 0 until helloRead.limit) {
      assertEquals(fullRead[i], helloRead[i])
    }
    for (i in 0 until worldRead.limit) {
      assertEquals(fullRead[6 + i], worldRead[i])
    }
  }

  @Test
  fun readWriteBufferPrimitivesTest() {
    val text = "Hello world!"
    val bytes = text.encodeToByteArray()
    val wt = ArrayReadWriteBuffer(bytes)
    wt.requestCapacity(4096)
    wt.put("Tests")
    val str1 = wt.writePosition
    assertEquals("Tests world!", wt.getString(0, bytes.size))
    assertEquals("Tests", wt.getString(0, str1))
    wt.put(Int.MAX_VALUE)
    assertEquals(Int.MAX_VALUE, wt.getInt(str1))

    val pos = wt.writePosition
    wt.put(Double.NEGATIVE_INFINITY)
    assertEquals(Double.NEGATIVE_INFINITY, wt.getDouble(pos))

    val jap = " are really すごい!".encodeToByteArray()
    wt.writePosition = str1
    wt.put(jap)
    assertEquals("Tests are really すごい!", wt.getString())
  }

  @Test
  fun readWriteBufferGrowthTest() {
    val a = ArrayReadWriteBuffer(1)
    assertEquals(1, a.capacity)
    a.put(0.toByte())
    assertEquals(1, a.capacity)
    assertFailsWith(IndexOutOfBoundsException::class) { a.put(0xFF.toShort()) }
    a.requestCapacity(8)
    a.writePosition = 0
    a.put(0xFF.toShort())
    assertEquals(8, a.capacity)
    assertEquals(0xFF, a.getShort(0))

    a.requestCapacity(8 + 12)
    a.put(ByteArray(12) { it.toByte() })

    // we grow as power or two, so 20 jumps to 32
    assertEquals(32, a.capacity)
    a.requestCapacity(513, false)
    assertEquals(1024, a.capacity)
    a.requestCapacity(234, false)
    assertEquals(1024, a.capacity)
  }
}
