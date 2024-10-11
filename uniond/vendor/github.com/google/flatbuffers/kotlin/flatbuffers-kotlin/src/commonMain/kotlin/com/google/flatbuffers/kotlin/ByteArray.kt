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
@file:Suppress("NOTHING_TO_INLINE")

package com.google.flatbuffers.kotlin

import kotlin.experimental.and

internal fun ByteArray.getString(index: Int, size: Int): String = Utf8.decodeUtf8Array(this, index, size)

internal fun ByteArray.setCharSequence(index: Int, value: CharSequence): Int =
  Utf8.encodeUtf8Array(value, this, index, this.size - index)

// List of functions that needs to be implemented on all platforms.
internal expect inline fun ByteArray.getUByte(index: Int): UByte
internal expect inline fun ByteArray.getShort(index: Int): Short
internal expect inline fun ByteArray.getUShort(index: Int): UShort
internal expect inline fun ByteArray.getInt(index: Int): Int
internal expect inline fun ByteArray.getUInt(index: Int): UInt
internal expect inline fun ByteArray.getLong(index: Int): Long
internal expect inline fun ByteArray.getULong(index: Int): ULong
internal expect inline fun ByteArray.getFloat(index: Int): Float
internal expect inline fun ByteArray.getDouble(index: Int): Double

internal expect inline fun ByteArray.setUByte(index: Int, value: UByte)
public expect inline fun ByteArray.setShort(index: Int, value: Short)
internal expect inline fun ByteArray.setUShort(index: Int, value: UShort)
internal expect inline fun ByteArray.setInt(index: Int, value: Int)
internal expect inline fun ByteArray.setUInt(index: Int, value: UInt)
internal expect inline fun ByteArray.setLong(index: Int, value: Long)
internal expect inline fun ByteArray.setULong(index: Int, value: ULong)
internal expect inline fun ByteArray.setFloat(index: Int, value: Float)
internal expect inline fun ByteArray.setDouble(index: Int, value: Double)

/**
 * This implementation uses Little Endian order.
 */
public object ByteArrayOps {
  public inline fun getUByte(ary: ByteArray, index: Int): UByte = ary[index].toUByte()
  public inline fun getShort(ary: ByteArray, index: Int): Short {
    return (ary[index + 1].toInt() shl 8 or (ary[index].toInt() and 0xff)).toShort()
  }

  public inline fun getUShort(ary: ByteArray, index: Int): UShort = getShort(ary, index).toUShort()

  public inline fun getInt(ary: ByteArray, index: Int): Int {
    return (
      (ary[index + 3].toInt() shl 24) or
        ((ary[index + 2].toInt() and 0xff) shl 16) or
        ((ary[index + 1].toInt() and 0xff) shl 8) or
        ((ary[index].toInt() and 0xff))
      )
  }

  public inline fun getUInt(ary: ByteArray, index: Int): UInt = getInt(ary, index).toUInt()

  public inline fun getLong(ary: ByteArray, index: Int): Long {
    var idx = index
    return ary[idx++].toLong() and 0xff or
      (ary[idx++].toLong() and 0xff shl 8) or
      (ary[idx++].toLong() and 0xff shl 16) or
      (ary[idx++].toLong() and 0xff shl 24) or
      (ary[idx++].toLong() and 0xff shl 32) or
      (ary[idx++].toLong() and 0xff shl 40) or
      (ary[idx++].toLong() and 0xff shl 48) or
      (ary[idx].toLong() shl 56)
  }

  public inline fun getULong(ary: ByteArray, index: Int): ULong = getLong(ary, index).toULong()

  public inline fun setUByte(ary: ByteArray, index: Int, value: UByte) {
    ary[index] = value.toByte()
  }
  public inline fun setShort(ary: ByteArray, index: Int, value: Short) {
    var idx = index
    ary[idx++] = (value and 0xff).toByte()
    ary[idx] = (value.toInt() shr 8 and 0xff).toByte()
  }

  public inline fun setUShort(ary: ByteArray, index: Int, value: UShort): Unit = setShort(ary, index, value.toShort())

  public inline fun setInt(ary: ByteArray, index: Int, value: Int) {
    var idx = index
    ary[idx++] = (value and 0xff).toByte()
    ary[idx++] = (value shr 8 and 0xff).toByte()
    ary[idx++] = (value shr 16 and 0xff).toByte()
    ary[idx] = (value shr 24 and 0xff).toByte()
  }

  public inline fun setUInt(ary: ByteArray, index: Int, value: UInt): Unit = setInt(ary, index, value.toInt())

  public inline fun setLong(ary: ByteArray, index: Int, value: Long) {
    var i = value.toInt()
    setInt(ary, index, i)
    i = (value shr 32).toInt()
    setInt(ary, index + 4, i)
  }

  public inline fun setULong(ary: ByteArray, index: Int, value: ULong): Unit = setLong(ary, index, value.toLong())

  public inline fun setFloat(ary: ByteArray, index: Int, value: Float) {
    setInt(ary, index, value.toRawBits())
  }

  public inline fun setDouble(ary: ByteArray, index: Int, value: Double) {
    setLong(ary, index, value.toRawBits())
  }

  public inline fun getFloat(ary: ByteArray, index: Int): Float = Float.fromBits(getInt(ary, index))
  public inline fun getDouble(ary: ByteArray, index: Int): Double = Double.fromBits(getLong(ary, index))
}
