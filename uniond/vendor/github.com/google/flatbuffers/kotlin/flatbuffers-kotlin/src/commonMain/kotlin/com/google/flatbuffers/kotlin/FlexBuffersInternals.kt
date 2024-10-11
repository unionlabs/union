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

import kotlin.jvm.JvmInline

@JvmInline
public value class BitWidth(public val value: Int) {
  public inline fun max(other: BitWidth): BitWidth = if (this.value >= other.value) this else other
}

@JvmInline
public value class ByteWidth(public val value: Int)

@JvmInline
public value class FlexBufferType(public val value: Int) {
  public operator fun minus(other: FlexBufferType): FlexBufferType = FlexBufferType(this.value - other.value)
  public operator fun plus(other: FlexBufferType): FlexBufferType = FlexBufferType(this.value + other.value)
  public operator fun compareTo(other: FlexBufferType): Int = this.value - other.value
}

internal operator fun Int.times(width: ByteWidth): Int = this * width.value
internal operator fun Int.minus(width: ByteWidth): Int = this - width.value
internal operator fun Int.plus(width: ByteWidth): Int = this + width.value
internal operator fun Int.minus(type: FlexBufferType): Int = this - type.value

// Returns a Key string from the buffer starting at index [start]. Key Strings are stored as
// C-Strings, ending with '\0'. If zero byte not found returns empty string.
internal inline fun ReadBuffer.getKeyString(start: Int): String {
  val i = findFirst(0.toByte(), start)
  return if (i >= 0) getString(start, i - start) else ""
}

// read unsigned int with size byteWidth and return as a 64-bit integer
internal inline fun ReadBuffer.readULong(end: Int, byteWidth: ByteWidth): ULong {
  return when (byteWidth.value) {
    1 -> this.getUByte(end).toULong()
    2 -> this.getUShort(end).toULong()
    4 -> this.getUInt(end).toULong()
    8 -> this.getULong(end)
    else -> error("invalid byte width $byteWidth for scalar unsigned integer")
  }
}

internal inline fun ReadBuffer.readFloat(end: Int, byteWidth: ByteWidth): Double {
  return when (byteWidth.value) {
    4 -> this.getFloat(end).toDouble()
    8 -> this.getDouble(end)
    else -> error("invalid byte width $byteWidth for floating point scalar") // we should never reach here
  }
}
// return position on the [ReadBuffer] of the element that the offset is pointing to
// we assume all offset fits on a int, since ReadBuffer operates with that assumption
internal inline fun ReadBuffer.indirect(offset: Int, byteWidth: ByteWidth): Int = offset - readInt(offset, byteWidth)
// returns the size of an array-like element from [ReadBuffer].
internal inline fun ReadBuffer.readSize(end: Int, byteWidth: ByteWidth) = readInt(end - byteWidth, byteWidth)
internal inline fun ReadBuffer.readUInt(end: Int, byteWidth: ByteWidth): UInt = readULong(end, byteWidth).toUInt()
internal inline fun ReadBuffer.readInt(end: Int, byteWidth: ByteWidth): Int = readULong(end, byteWidth).toInt()
internal inline fun ReadBuffer.readLong(end: Int, byteWidth: ByteWidth): Long = readULong(end, byteWidth).toLong()

internal fun IntArray.widthInUBits(): BitWidth = arrayWidthInUBits(this.size) { this[it].toULong().widthInUBits() }
internal fun ShortArray.widthInUBits(): BitWidth = arrayWidthInUBits(this.size) { this[it].toULong().widthInUBits() }
internal fun LongArray.widthInUBits(): BitWidth = arrayWidthInUBits(this.size) { this[it].toULong().widthInUBits() }

private inline fun arrayWidthInUBits(size: Int, crossinline elemWidthBlock: (Int) -> BitWidth): BitWidth {
  // Figure out smallest bit width we can store this vector with.
  var bitWidth = W_8.max(size.toULong().widthInUBits())
  // Check bit widths and types for all elements.
  for (i in 0 until size) {
    // since we know its inline types we can just assume elmentWidth to be the value width in bits.
    bitWidth = bitWidth.max(elemWidthBlock(i))
  }
  return bitWidth
}

internal fun ULong.widthInUBits(): BitWidth = when {
  this <= MAX_UBYTE_ULONG -> W_8
  this <= UShort.MAX_VALUE -> W_16
  this <= UInt.MAX_VALUE -> W_32
  else -> W_64
}

// returns the number of bytes needed for padding the scalar of size scalarSize.
internal inline fun paddingBytes(bufSize: Int, scalarSize: Int): Int = bufSize.inv() + 1 and scalarSize - 1

internal inline fun FlexBufferType.isInline(): Boolean = this.value <= T_FLOAT.value || this == T_BOOL

internal fun FlexBufferType.isScalar(): Boolean = when (this) {
  T_INT, T_UINT, T_FLOAT, T_BOOL -> true
  else -> false
}

internal fun FlexBufferType.isIndirectScalar(): Boolean = when (this) {
  T_INDIRECT_INT, T_INDIRECT_UINT, T_INDIRECT_FLOAT -> true
  else -> false
}

internal fun FlexBufferType.isTypedVector(): Boolean =
  this >= T_VECTOR_INT && this <= T_VECTOR_STRING_DEPRECATED || this == T_VECTOR_BOOL

internal fun FlexBufferType.isTypedVectorElementType(): Boolean =
  (this.value in T_INT.value..T_KEY.value) || this == T_BOOL

// returns the typed vector of a given scalar type.
internal fun FlexBufferType.toTypedVector(): FlexBufferType = (this - T_INT) + T_VECTOR_INT
// returns the element type of given typed vector.
internal fun FlexBufferType.toElementTypedVector(): FlexBufferType = this - T_VECTOR_INT + T_INT

// Holds information about the elements inserted on the buffer.
internal data class Value(
  var type: FlexBufferType = T_INT,
  var key: Int = -1,
  var minBitWidth: BitWidth = W_8,
  var iValue: ULong = 0UL, // integer value
  var dValue: Double = 0.0 // TODO(paulovap): maybe we can keep floating type on iValue as well.
) { // float value

  inline fun storedPackedType(parentBitWidth: BitWidth = W_8): Byte = packedType(storedWidth(parentBitWidth), type)

  private inline fun packedType(bitWidth: BitWidth, type: FlexBufferType): Byte =
    (bitWidth.value or (type.value shl 2)).toByte()

  private inline fun storedWidth(parentBitWidth: BitWidth): BitWidth =
    if (type.isInline()) minBitWidth.max(parentBitWidth) else minBitWidth

  fun elemWidth(bufSize: Int, elemIndex: Int): BitWidth =
    elemWidth(type, minBitWidth, iValue.toLong(), bufSize, elemIndex)
}

internal fun elemWidth(
  type: FlexBufferType,
  minBitWidth: BitWidth,
  iValue: Long,
  bufSize: Int,
  elemIndex: Int
): BitWidth {
  if (type.isInline()) return minBitWidth

  // We have an absolute offset, but want to store a relative offset
  // elem_index elements beyond the current buffer end. Since whether
  // the relative offset fits in a certain byte_width depends on
  // the size of the elements before it (and their alignment), we have
  // to test for each size in turn.
  // Original implementation checks for largest scalar
  // which is long unsigned int
  var byteWidth = 1
  while (byteWidth <= 32) {
    // Where are we going to write this offset?
    val offsetLoc: Int = bufSize + paddingBytes(bufSize, byteWidth) + elemIndex * byteWidth
    // Compute relative offset.
    val offset: Int = offsetLoc - iValue.toInt()
    // Does it fit?
    val bitWidth = offset.toULong().widthInUBits()
    if (1 shl bitWidth.value == byteWidth) return bitWidth
    byteWidth *= 2
  }
  return W_64
}

// For debugging purposes, convert type to a human-readable string.
internal fun FlexBufferType.typeToString(): String = when (this) {
  T_NULL -> "Null"
  T_INT -> "Int"
  T_UINT -> "UInt"
  T_FLOAT -> "Float"
  T_KEY -> "Key"
  T_STRING -> "String"
  T_INDIRECT_INT -> "IndirectInt"
  T_INDIRECT_UINT -> "IndirectUInt"
  T_INDIRECT_FLOAT -> "IndirectFloat"
  T_MAP -> "Map"
  T_VECTOR -> "Vector"
  T_VECTOR_INT -> "IntVector"
  T_VECTOR_UINT -> "UIntVector"
  T_VECTOR_FLOAT -> "FloatVector"
  T_VECTOR_KEY -> "KeyVector"
  T_VECTOR_STRING_DEPRECATED -> "StringVectorDeprecated"
  T_VECTOR_INT2 -> "Int2Vector"
  T_VECTOR_UINT2 -> "UInt2Vector"
  T_VECTOR_FLOAT2 -> "Float2Vector"
  T_VECTOR_INT3 -> "Int3Vector"
  T_VECTOR_UINT3 -> "UInt3Vector"
  T_VECTOR_FLOAT3 -> "Float3Vector"
  T_VECTOR_INT4 -> "Int4Vector"
  T_VECTOR_UINT4 -> "UInt4Vector"
  T_VECTOR_FLOAT4 -> "Float4Vector"
  T_BLOB -> "BlobVector"
  T_BOOL -> "BoolVector"
  T_VECTOR_BOOL -> "BoolVector"
  else -> "UnknownType"
}

// Few repeated values used in hot path is cached here
internal fun emptyBlob() = Blob(emptyBuffer, 1, ByteWidth(1))
internal fun emptyVector() = Vector(emptyBuffer, 1, ByteWidth(1))
internal fun emptyMap() = Map(ArrayReadWriteBuffer(3), 3, ByteWidth(1))
internal fun nullReference() = Reference(emptyBuffer, 1, ByteWidth(0), T_NULL.value)
internal fun nullKey() = Key(emptyBuffer, 1)

internal const val ZeroByte = 0.toByte()
internal const val MAX_UBYTE_ULONG = 255UL
internal const val MAX_UBYTE = 255
internal const val MAX_USHORT = 65535

// value bit width possible sizes
internal val W_8 = BitWidth(0)
internal val W_16 = BitWidth(1)
internal val W_32 = BitWidth(2)
internal val W_64 = BitWidth(3)

// These are used as the upper 6 bits of a type field to indicate the actual type.
internal val T_INVALID = FlexBufferType(-1)
internal val T_NULL = FlexBufferType(0)
internal val T_INT = FlexBufferType(1)
internal val T_UINT = FlexBufferType(2)
internal val T_FLOAT = FlexBufferType(3) // Types above stored inline, types below are stored in an offset.
internal val T_KEY = FlexBufferType(4)
internal val T_STRING = FlexBufferType(5)
internal val T_INDIRECT_INT = FlexBufferType(6)
internal val T_INDIRECT_UINT = FlexBufferType(7)
internal val T_INDIRECT_FLOAT = FlexBufferType(8)
internal val T_MAP = FlexBufferType(9)
internal val T_VECTOR = FlexBufferType(10) // Untyped.
internal val T_VECTOR_INT = FlexBufferType(11) // Typed any size  = stores no type table).
internal val T_VECTOR_UINT = FlexBufferType(12)
internal val T_VECTOR_FLOAT = FlexBufferType(13)
internal val T_VECTOR_KEY = FlexBufferType(14)
// DEPRECATED, use FBT_VECTOR or FBT_VECTOR_KEY instead.
// more info on https://github.com/google/flatbuffers/issues/5627.
internal val T_VECTOR_STRING_DEPRECATED = FlexBufferType(15)
internal val T_VECTOR_INT2 = FlexBufferType(16) // Typed tuple  = no type table; no size field).
internal val T_VECTOR_UINT2 = FlexBufferType(17)
internal val T_VECTOR_FLOAT2 = FlexBufferType(18)
internal val T_VECTOR_INT3 = FlexBufferType(19) // Typed triple  = no type table; no size field).
internal val T_VECTOR_UINT3 = FlexBufferType(20)
internal val T_VECTOR_FLOAT3 = FlexBufferType(21)
internal val T_VECTOR_INT4 = FlexBufferType(22) // Typed quad  = no type table; no size field).
internal val T_VECTOR_UINT4 = FlexBufferType(23)
internal val T_VECTOR_FLOAT4 = FlexBufferType(24)
internal val T_BLOB = FlexBufferType(25)
internal val T_BOOL = FlexBufferType(26)
internal val T_VECTOR_BOOL = FlexBufferType(36) // To Allow the same type of conversion of type to vector type
