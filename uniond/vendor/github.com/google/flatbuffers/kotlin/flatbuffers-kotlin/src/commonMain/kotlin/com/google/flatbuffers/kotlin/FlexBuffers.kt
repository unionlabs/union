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
@file:JvmName("FlexBuffers")
package com.google.flatbuffers.kotlin

import kotlin.jvm.JvmName

/**
 * Reads a FlexBuffer message in ReadBuf and returns [Reference] to
 * the root element.
 * @param buffer ReadBuf containing FlexBuffer message
 * @return [Reference] to the root object
 */
public fun getRoot(buffer: ReadBuffer): Reference {
  var end: Int = buffer.limit
  val byteWidth = buffer[--end].toInt()
  val packetType = buffer[--end].toInt()
  end -= byteWidth // The root data item.
  return Reference(buffer, end, ByteWidth(byteWidth), packetType)
}

/**
 * Represents an generic element in the buffer. It can be specialized into scalar types, using for example,
 * [Reference.toInt], or casted into Flexbuffer object types, like [Reference.toMap] or [Reference.toBlob].
 */
@Suppress("NOTHING_TO_INLINE")
public class Reference internal constructor(
  internal val buffer: ReadBuffer,
  internal val end: Int,
  internal val parentWidth: ByteWidth,
  internal val byteWidth: ByteWidth,
  public val type: FlexBufferType
) {

  internal constructor(bb: ReadBuffer, end: Int, parentWidth: ByteWidth, packedType: Int) :
    this(bb, end, parentWidth, ByteWidth(1 shl (packedType and 3)), FlexBufferType((packedType shr 2)))

  /**
   * Checks whether the element is null type
   * @return true if null type
   */
  public val isNull: Boolean get() = type == T_NULL

  /**
   * Checks whether the element is boolean type
   * @return true if boolean type
   */
  public val isBoolean: Boolean get() = type == T_BOOL

  /**
   * Checks whether the element type is numeric (signed/unsigned integers and floats)
   * @return true if numeric type
   */
  public val isNumeric: Boolean get() = isIntOrUInt || isFloat

  /**
   * Checks whether the element type is signed or unsigned integers
   * @return true if an integer type
   */
  public val isIntOrUInt: Boolean get() = isInt || isUInt

  /**
   * Checks whether the element type is float
   * @return true if a float type
   */
  public val isFloat: Boolean get() = type == T_FLOAT || type == T_INDIRECT_FLOAT

  /**
   * Checks whether the element type is signed integer
   * @return true if a signed integer type
   */
  public val isInt: Boolean get() = type == T_INT || type == T_INDIRECT_INT

  /**
   * Checks whether the element type is signed integer
   * @return true if a signed integer type
   */
  public val isUInt: Boolean get() = type == T_UINT || type == T_INDIRECT_UINT

  /**
   * Checks whether the element type is string
   * @return true if a string type
   */
  public val isString: Boolean get() = type == T_STRING

  /**
   * Checks whether the element type is key
   * @return true if a key type
   */
  public val isKey: Boolean get() = type == T_KEY

  /**
   * Checks whether the element type is vector or a map. [TypedVector] are considered different types and will return
   * false.
   * @return true if a vector type
   */
  public val isVector: Boolean get() = type == T_VECTOR || type == T_MAP

  /**
   * Checks whether the element type is typed vector
   * @return true if a typed vector type
   */
  public val isTypedVector: Boolean get() = type.isTypedVector()

  /**
   * Checks whether the element type is a map
   * @return true if a map type
   */
  public val isMap: Boolean get() = type == T_MAP

  /**
   * Checks whether the element type is a blob
   * @return true if a blob type
   */
  public val isBlob: Boolean get() = type == T_BLOB

  /**
   * Assumes [Reference] as a [Vector] and returns a [Reference] at index [index].
   */
  public operator fun get(index: Int): Reference = toVector()[index]

  /**
   * Assumes [Reference] as a [Map] and returns a [Reference] for the value at key [key].
   */
  public operator fun get(key: String): Reference = toMap()[key]

  /**
   * Returns element as a [Boolean].
   * If element type is not boolean, it will be casted to integer and compared against 0
   * @return element as [Boolean]
   */
  public fun toBoolean(): Boolean = if (isBoolean) buffer.getBoolean(end) else toUInt() != 0u

  /**
   * Returns element as [Byte].
   *  For vector types, it will return size of the vector.
   *  For String type, it will be parsed as integer.
   *  Unsigned elements will become signed (with possible overflow).
   *  Float elements will be casted to [Byte].
   * @return [Byte] or 0 if fail to convert element to integer.
   */
  public fun toByte(): Byte = toULong().toByte()

  /**
   * Returns element as [Short].
   *  For vector types, it will return size of the vector.
   *  For String type, it will type to be parsed as integer.
   *  Unsigned elements will become signed (with possible overflow).
   *  Float elements will be casted to [Short]
   * @return [Short] or 0 if fail to convert element to integer.
   */
  public fun toShort(): Short = toULong().toShort()

  /**
   * Returns element as [Int].
   *  For vector types, it will return size of the vector.
   *  For String type, it will type to be parsed as integer.
   *  Unsigned elements will become signed (with possible overflow).
   *  Float elements will be casted to [Int]
   * @return [Int] or 0 if fail to convert element to integer.
   */
  public fun toInt(): Int = toULong().toInt()

  /**
   * Returns element as [Long].
   * For vector types, it will return size of the vector
   * For String type, it will type to be parsed as integer
   * Unsigned elements will become negative
   * Float elements will be casted to integer
   * @return [Long] integer or 0 if fail to convert element to long.
   */
  public fun toLong(): Long = toULong().toLong()

  /**
   * Returns element as [UByte].
   *  For vector types, it will return size of the vector.
   *  For String type, it will type to be parsed as integer.
   *  Negative elements will become unsigned counterpart.
   *  Float elements will be casted to [UByte]
   * @return [UByte] or 0 if fail to convert element to integer.
   */
  public fun toUByte(): UByte = toULong().toUByte()

  /**
   * Returns element as [UShort].
   *  For vector types, it will return size of the vector.
   *  For String type, it will type to be parsed as integer.
   *  Negative elements will become unsigned counterpart.
   *  Float elements will be casted to [UShort]
   * @return [UShort] or 0 if fail to convert element to integer.
   */
  public fun toUShort(): UShort = toULong().toUShort()

  /**
   * Returns element as [UInt].
   *  For vector types, it will return size of the vector.
   *  For String type, it will type to be parsed as integer.
   *  Negative elements will become unsigned counterpart.
   *  Float elements will be casted to [UInt]
   * @return [UInt] or 0 if fail to convert element to integer.
   */
  public fun toUInt(): UInt = toULong().toUInt()

  /**
   * Returns element as  [ULong] integer.
   * For vector types, it will return size of the vector
   * For String type, it will type to be parsed as integer
   * Negative elements will become unsigned counterpart.
   * Float elements will be casted to integer
   * @return [ULong] integer or 0 if fail to convert element to long.
   */
  public fun toULong(): ULong = resolve { pos: Int, width: ByteWidth ->
    when (type) {
      T_INDIRECT_INT, T_INDIRECT_UINT, T_INT, T_BOOL, T_UINT -> buffer.readULong(pos, width)
      T_FLOAT, T_INDIRECT_FLOAT -> buffer.readFloat(pos, width).toULong()
      T_STRING -> toString().toULong()
      T_VECTOR -> toVector().size.toULong()
      else -> 0UL
    }
  }

  /**
   * Returns element as  [Float].
   * For vector types, it will return size of the vector
   * For String type, it will type to be parsed as [Float]
   * Float elements will be casted to integer
   * @return [Float] integer or 0 if fail to convert element to long.
   */
  public fun toFloat(): Float = resolve { pos: Int, width: ByteWidth ->
    when (type) {
      T_INDIRECT_FLOAT, T_FLOAT -> buffer.readFloat(pos, width).toFloat()
      T_INT -> buffer.readInt(end, parentWidth).toFloat()
      T_UINT, T_BOOL -> buffer.readUInt(end, parentWidth).toFloat()
      T_INDIRECT_INT -> buffer.readInt(pos, width).toFloat()
      T_INDIRECT_UINT -> buffer.readUInt(pos, width).toFloat()
      T_NULL -> 0.0f
      T_STRING -> toString().toFloat()
      T_VECTOR -> toVector().size.toFloat()
      else -> 0f
    }
  }

  /**
   * Returns element as  [Double].
   * For vector types, it will return size of the vector
   * For String type, it will type to be parsed as [Double]
   * @return [Float] integer or 0 if fail to convert element to long.
   */
  public fun toDouble(): Double = resolve { pos: Int, width: ByteWidth ->
    when (type) {
      T_INDIRECT_FLOAT, T_FLOAT -> buffer.readFloat(pos, width)
      T_INT -> buffer.readInt(pos, width).toDouble()
      T_UINT, T_BOOL -> buffer.readUInt(pos, width).toDouble()
      T_INDIRECT_INT -> buffer.readInt(pos, width).toDouble()
      T_INDIRECT_UINT -> buffer.readUInt(pos, width).toDouble()
      T_NULL -> 0.0
      T_STRING -> toString().toDouble()
      T_VECTOR -> toVector().size.toDouble()
      else -> 0.0
    }
  }

  /**
   * Returns element as [Key] or invalid key.
   */
  public fun toKey(): Key = when (type) {
    T_KEY -> Key(buffer, buffer.indirect(end, parentWidth))
    else -> nullKey()
  }
  /**
   * Returns element as a [String]
   * @return element as [String] or empty [String] if fail
   */
  override fun toString(): String = when (type) {
    T_STRING -> {
      val start = buffer.indirect(end, parentWidth)
      val size = buffer.readULong(start - byteWidth, byteWidth).toInt()
      buffer.getString(start, size)
    }
    T_KEY -> buffer.getKeyString(buffer.indirect(end, parentWidth))
    T_MAP -> "{ ${toMap().entries.joinToString(", ") { "${it.key}: ${it.value}"}} }"
    T_VECTOR, T_VECTOR_BOOL, T_VECTOR_FLOAT, T_VECTOR_INT,
    T_VECTOR_UINT, T_VECTOR_KEY, T_VECTOR_STRING_DEPRECATED ->
      "[ ${toVector().joinToString(", ") { it.toString() }} ]"
    T_INT -> toLong().toString()
    T_UINT -> toULong().toString()
    T_FLOAT -> toDouble().toString()
    else -> "${type.typeToString()}(end=$end)"
  }

  /**
   * Returns element as a [ByteArray], converting scalar types when possible.
   * @return element as [ByteArray] or empty [ByteArray] if fail.
   */
  public fun toByteArray(): ByteArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> ByteArray(vec.size) { vec.getInt(it).toByte() }
      T_VECTOR_UINT -> ByteArray(vec.size) { vec.getUInt(it).toByte() }
      T_VECTOR -> ByteArray(vec.size) { vec[it].toByte() }
      T_VECTOR_FLOAT -> ByteArray(vec.size) { vec.getFloat(it).toInt().toByte() }
      else -> ByteArray(0)
    }
  }

  /**
   * Returns element as a [ByteArray], converting scalar types when possible.
   * @return element as [ByteArray] or empty [ByteArray] if fail.
   */
  public fun toShortArray(): ShortArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> ShortArray(vec.size) { vec.getInt(it).toShort() }
      T_VECTOR_UINT -> ShortArray(vec.size) { vec.getUInt(it).toShort() }
      T_VECTOR -> ShortArray(vec.size) { vec[it].toShort() }
      T_VECTOR_FLOAT -> ShortArray(vec.size) { vec.getFloat(it).toInt().toShort() }
      else -> ShortArray(0)
    }
  }

  /**
   * Returns element as a [IntArray], converting scalar types when possible.
   * @return element as [IntArray] or empty [IntArray] if fail.
   */
  public fun toIntArray(): IntArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> IntArray(vec.size) { vec.getInt(it).toInt() }
      T_VECTOR_UINT -> IntArray(vec.size) { vec.getUInt(it).toInt() }
      T_VECTOR -> IntArray(vec.size) { vec[it].toInt() }
      T_VECTOR_FLOAT -> IntArray(vec.size) { vec.getFloat(it).toInt() }
      else -> IntArray(0)
    }
  }

  /**
   * Returns element as a [LongArray], converting scalar types when possible.
   * @return element as [LongArray] or empty [LongArray] if fail.
   */
  public fun toLongArray(): LongArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> LongArray(vec.size) { vec.getInt(it) }
      T_VECTOR_UINT -> LongArray(vec.size) { vec.getInt(it) }
      T_VECTOR -> LongArray(vec.size) { vec[it].toLong() }
      T_VECTOR_FLOAT -> LongArray(vec.size) { vec.getFloat(it).toLong() }
      else -> LongArray(0)
    }
  }

  /**
   * Returns element as a [UByteArray], converting scalar types when possible.
   * @return element as [UByteArray] or empty [UByteArray] if fail.
   */
  public fun toUByteArray(): UByteArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> UByteArray(vec.size) { vec.getInt(it).toUByte() }
      T_VECTOR_UINT -> UByteArray(vec.size) { vec.getUInt(it).toUByte() }
      T_VECTOR -> UByteArray(vec.size) { vec[it].toUByte() }
      T_VECTOR_FLOAT -> UByteArray(vec.size) { vec.getFloat(it).toInt().toUByte() }
      else -> UByteArray(0)
    }
  }

  /**
   * Returns element as a [UIntArray], converting scalar types when possible.
   * @return element as [UIntArray] or empty [UIntArray] if fail.
   */
  public fun toUShortArray(): UShortArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> UShortArray(vec.size) { vec.getInt(it).toUShort() }
      T_VECTOR_UINT -> UShortArray(vec.size) { vec.getUInt(it).toUShort() }
      T_VECTOR -> UShortArray(vec.size) { vec[it].toUShort() }
      T_VECTOR_FLOAT -> UShortArray(vec.size) { vec.getFloat(it).toUInt().toUShort() }
      else -> UShortArray(0)
    }
  }

  /**
   * Returns element as a [UIntArray], converting scalar types when possible.
   * @return element as [UIntArray] or empty [UIntArray] if fail.
   */
  public fun toUIntArray(): UIntArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> UIntArray(vec.size) { vec.getInt(it).toUInt() }
      T_VECTOR_UINT -> UIntArray(vec.size) { vec.getUInt(it).toUInt() }
      T_VECTOR -> UIntArray(vec.size) { vec[it].toUInt() }
      T_VECTOR_FLOAT -> UIntArray(vec.size) { vec.getFloat(it).toUInt() }
      else -> UIntArray(0)
    }
  }

  /**
   * Returns element as a [ULongArray], converting scalar types when possible.
   * @return element as [ULongArray] or empty [ULongArray] if fail.
   */
  public fun toULongArray(): ULongArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_INT -> ULongArray(vec.size) { vec.getUInt(it) }
      T_VECTOR_UINT -> ULongArray(vec.size) { vec.getUInt(it) }
      T_VECTOR -> ULongArray(vec.size) { vec[it].toULong() }
      T_VECTOR_FLOAT -> ULongArray(vec.size) { vec.getFloat(it).toULong() }
      else -> ULongArray(0)
    }
  }

  /**
   * Returns element as a [FloatArray], converting scalar types when possible.
   * @return element as [FloatArray] or empty [FloatArray] if fail.
   */
  public fun toFloatArray(): FloatArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_FLOAT -> FloatArray(vec.size) { vec.getFloat(it).toFloat() }
      T_VECTOR_INT -> FloatArray(vec.size) { vec.getInt(it).toFloat() }
      T_VECTOR_UINT -> FloatArray(vec.size) { vec.getUInt(it).toFloat() }
      T_VECTOR -> FloatArray(vec.size) { vec[it].toFloat() }
      else -> FloatArray(0)
    }
  }

  /**
   * Returns element as a [DoubleArray], converting scalar types when possible.
   * @return element as [DoubleArray] or empty [DoubleArray] if fail.
   */
  public fun toDoubleArray(): DoubleArray {
    val vec = TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
    return when (type) {
      T_VECTOR_FLOAT -> DoubleArray(vec.size) { vec[it].toDouble() }
      T_VECTOR_INT -> DoubleArray(vec.size) { vec[it].toDouble() }
      T_VECTOR_UINT -> DoubleArray(vec.size) { vec[it].toDouble() }
      T_VECTOR -> DoubleArray(vec.size) { vec[it].toDouble() }
      else -> DoubleArray(0)
    }
  }

  /**
   * Returns element as a [Vector]
   * @return element as [Vector] or empty [Vector] if fail
   */
  public fun toVector(): Vector {
    return when {
      isVector -> Vector(buffer, buffer.indirect(end, parentWidth), byteWidth)
      isTypedVector -> TypedVector(type.toElementTypedVector(), buffer, buffer.indirect(end, parentWidth), byteWidth)
      else -> emptyVector()
    }
  }

  /**
   * Returns element as a [Blob]
   * @return element as [Blob] or empty [Blob] if fail
   */
  public fun toBlob(): Blob {
    return when (type) {
      T_BLOB, T_STRING -> Blob(buffer, buffer.indirect(end, parentWidth), byteWidth)
      else -> emptyBlob()
    }
  }

  /**
   * Returns element as a [Map].
   * @return element as [Map] or empty [Map] if fail
   */
  public fun toMap(): Map = when (type) {
    T_MAP -> Map(buffer, buffer.indirect(end, parentWidth), byteWidth)
    else -> emptyMap()
  }

  private inline fun <T> resolve(crossinline block: (pos: Int, width: ByteWidth) -> T): T {
    return if (type.isIndirectScalar()) {
      block(buffer.indirect(end, byteWidth), byteWidth)
    } else {
      block(end, parentWidth)
    }
  }

  override fun equals(other: Any?): Boolean {
    if (this === other) return true
    if (other == null || this::class != other::class) return false
    other as Reference
    if (buffer != other.buffer ||
      end != other.end ||
      parentWidth != other.parentWidth ||
      byteWidth != other.byteWidth ||
      type != other.type
    ) return false
    return true
  }

  override fun hashCode(): Int {
    var result = buffer.hashCode()
    result = 31 * result + end
    result = 31 * result + parentWidth.value
    result = 31 * result + byteWidth.value
    result = 31 * result + type.hashCode()
    return result
  }
}

/**
 * Represents any element that has a size property to it, like: [Map], [Vector] and [TypedVector].
 */
public open class Sized internal constructor(
  public val buffer: ReadBuffer,
  public val end: Int,
  public val byteWidth: ByteWidth
) {
  public open val size: Int = buffer.readSize(end, byteWidth)
}

/**
 * Represent an array of bytes in the buffer.
 */
public open class Blob internal constructor(
  buffer: ReadBuffer,
  end: Int,
  byteWidth: ByteWidth
) : Sized(buffer, end, byteWidth) {
  /**
   * Return [Blob] as [ReadBuffer]
   * @return blob as [ReadBuffer]
   */
  public fun data(): ReadBuffer = buffer.slice(end, size)

  /**
   * Copy [Blob] into a [ByteArray]
   * @return A [ByteArray] containing the blob data.
   */
  public fun toByteArray(): ByteArray {
    val result = ByteArray(size)
    for (i in 0 until size) {
      result[i] = buffer[end + i]
    }
    return result
  }

  /**
   * Return individual byte at a given position
   * @param pos position of the byte to be read
   */
  public operator fun get(pos: Int): Byte {
    if (pos !in 0..size) error("$pos index out of bounds. Should be in range 0..$size")
    return buffer[end + pos]
  }

  override fun toString(): String = buffer.getString(end, size)
}

/**
 * [Vector] represents an array of elements in the buffer. The element can be of any type.
 */
public open class Vector internal constructor(
  buffer: ReadBuffer,
  end: Int,
  byteWidth: ByteWidth
) : Collection<Reference>,
  Sized(buffer, end, byteWidth) {

  /**
   * Returns a [Reference] from the [Vector] at position [index]. Returns a null reference
   * @param index position in the vector.
   * @return [Reference] for a key or a null [Reference] if not found.
   */
  public open operator fun get(index: Int): Reference {
    if (index >= size) return nullReference()
    val packedType = buffer[(end + size * byteWidth.value + index)].toInt()
    val objEnd = end + index * byteWidth
    return Reference(buffer, objEnd, byteWidth, packedType)
  }

  // overrides from Collection<Reference>

  override fun contains(element: Reference): Boolean = find { it == element } != null

  override fun containsAll(elements: Collection<Reference>): Boolean {
    elements.forEach { if (!contains(it)) return false }
    return true
  }

  override fun isEmpty(): Boolean = size == 0

  override fun iterator(): Iterator<Reference> = object : Iterator<Reference> {
    var position = 0
    override fun hasNext(): Boolean = position != size
    override fun next(): Reference = get(position++)
  }
}

/**
 * [TypedVector] represents an array of scalar elements of the same type in the buffer.
 */
public open class TypedVector(
  private val elementType: FlexBufferType,
  buffer: ReadBuffer,
  end: Int,
  byteWidth: ByteWidth
) : Vector(buffer, end, byteWidth) {

  /**
   * Returns a [Reference] from the [TypedVector] at position [index]. Returns a null reference
   * @param index position in the vector.
   * @return [Reference] for a key or a null [Reference] if not found.
   */
  override operator fun get(index: Int): Reference {
    if (index >= size) return nullReference()
    val childPos: Int = end + index * byteWidth
    return Reference(buffer, childPos, byteWidth, ByteWidth(1), elementType)
  }

  private inline fun <T> resolveAt(index: Int, crossinline block: (Int, ByteWidth) -> T): T {
    val childPos: Int = end + index * byteWidth
    return block(childPos, byteWidth)
  }

  internal fun getBoolean(index: Int): Boolean = resolveAt(index) {
      pos: Int, _: ByteWidth -> buffer.getBoolean(pos)
  }
  internal fun getInt(index: Int): Long = resolveAt(index) {
      pos: Int, width: ByteWidth -> buffer.readLong(pos, width)
  }
  internal fun getUInt(index: Int): ULong = resolveAt(index) {
      pos: Int, width: ByteWidth -> buffer.readULong(pos, width)
  }
  internal fun getFloat(index: Int): Double = resolveAt(index) {
      pos: Int, width: ByteWidth -> buffer.readFloat(pos, width)
  }
}

/**
 * Represents a key element in the buffer. Keys are
 * used to reference objects in a [Map]
 */
public data class Key(
  public val buffer: ReadBuffer,
  public val start: Int,
  public val end: Int = buffer.findFirst(ZeroByte, start)
) {

  val sizeInBytes: Int = end - start

  private val codePoint = CharArray(2)

  val sizeInChars: Int
    get() {
      var count = 0
      var i = start
      while (i < end) {
        val size = codePointSizeInBytes(i)
        i += size
        count += if (size == 4) 2 else 1
      }
      return count
    }

  public operator fun get(index: Int): Char {
    var count = 0
    var i = start
    var size = 0
    // we loop over the bytes to find the right position for the "char" at index i
    while (i < end && count < index) {
      size = codePointSizeInBytes(i)
      i += size
      // 4 bytes utf8 are 2 chars wide, the rest is on char.
      count += if (size == 4) 2 else 1
    }
    return when {
      count == index -> {
        Utf8.decodeUtf8CodePoint(buffer, i, codePoint)
        codePoint[0]
      }
      count == index + 1 && size == 4 -> {
        Utf8.decodeUtf8CodePoint(buffer, i - size, codePoint)
        codePoint[1]
      }
      else -> error("Invalid count=$count, index=$index")
    }
  }

  private inline fun codePointSizeInBytes(pos: Int): Int {
    val b = buffer[pos]
    return when {
      Utf8.isOneByte(b) -> 1
      Utf8.isTwoBytes(b) -> 2
      Utf8.isThreeBytes(b) -> 3
      else -> 4
    }
  }

  override fun toString(): String = if (sizeInBytes > 0) buffer.getString(start, sizeInBytes) else ""

  /**
   * Checks whether Key is invalid or not.
   */
  public fun isInvalid(): Boolean = sizeInBytes <= 0
}

/**
 * A Map class that provide support to access Key-Value data from Flexbuffers.
 */
public class Map
  internal constructor(buffer: ReadBuffer, end: Int, byteWidth: ByteWidth):
  Sized(buffer, end, byteWidth),
  kotlin.collections.Map<Key, Reference> {

  // used for accessing the key vector elements
  private var keyVectorEnd: Int
  private var keyVectorByteWidth: ByteWidth
  init {
    val keysOffset = end - (3 * byteWidth) // 3 is number of prefixed fields
    keyVectorEnd = buffer.indirect(keysOffset, byteWidth)
    keyVectorByteWidth = ByteWidth(buffer.readInt(keysOffset + byteWidth, byteWidth))
  }

  /**
   * Returns a [Reference] from the [Map] at position [index]. Returns a null reference
   * @param index position in the map
   * @return [Reference] for a key or a null [Reference] if not found.
   */
  public operator fun get(index: Int): Reference {
    if (index >= size) return nullReference()
    val packedPos = end + size * byteWidth + index
    val packedType = buffer[packedPos].toInt()
    val objEnd = end + index * byteWidth
    return Reference(buffer, objEnd, byteWidth, packedType)
  }

  /**
   * Returns a [Reference] from the [Map] for a given [String] [key].
   * @param key access key to element on map
   * @return [Reference] for a key or a null [Reference] if not found.
   */
  public operator fun get(key: String): Reference {
    val index: Int = binarySearch(key)
    return if (index in 0 until size) {
      get(index)
    } else nullReference()
  }

  /**
   * Returns a [Reference] from the [Map] for a given [Key] [key].
   * @param key access key to element on map
   * @return [Reference] for a key or a null [Reference] if not found.
   */
  override operator fun get(key: Key): Reference {
    val index = binarySearch(key)
    return if (index in 0 until size) {
      get(index)
    } else nullReference()
  }

  /**
   * Checks whether the map contains a [key].
   * @param key [String]
   * @return true if key is found in the map, otherwise false.
   */
  public operator fun contains(key: String): Boolean = binarySearch(key) >= 0

  /**
   * Returns a [Key] for a given position [index] in the [Map].
   * @param index of the key in the map
   * @return a Key for the given index. Out of bounds indexes returns invalid keys.
   */
  public fun keyAt(index: Int): Key {
    val childPos: Int = keyVectorEnd + index * keyVectorByteWidth
    return Key(buffer, buffer.indirect(childPos, keyVectorByteWidth))
  }

  /**
   * Returns a [Key] as [String] for a given position [index] in the [Map].
   * @param index of the key in the map
   * @return a Key for the given index. Out of bounds indexes returns empty string.
   */
  public fun keyAsString(index: Int): String {
    val childPos: Int = keyVectorEnd + index * keyVectorByteWidth
    val start = buffer.indirect(childPos, keyVectorByteWidth)
    val end = buffer.findFirst(ZeroByte, start)
    return if (end > start) buffer.getString(start, end - start) else ""
  }

  // Overrides from kotlin.collections.Map<Key, Reference>

  public data class Entry(override val key: Key, override val value: Reference) :
    kotlin.collections.Map.Entry<Key, Reference>

  override val entries: Set<kotlin.collections.Map.Entry<Key, Reference>>
    get() = keys.map { Entry(it, get(it.toString())) }.toSet()

  override val keys: Set<Key>
    get() {
      val set = LinkedHashSet<Key>(size)
      for (i in 0 until size) {
        val key = keyAt(i)
        set.add(key)
      }
      return set
    }

  /**
   * Returns a [Vector] for accessing all values in the [Map].
   * @return [Vector] of values.
   */
  override val values: Collection<Reference>
    get() = Vector(buffer, end, byteWidth)

  override fun containsKey(key: Key): Boolean {
    for (i in 0 until size) {
      if (key == keyAt(i))
        return true
    }
    return false
  }

  override fun containsValue(value: Reference): Boolean = values.contains(value)

  override fun isEmpty(): Boolean = size == 0

  // Performs a binary search on a key vector and return index of the key in key vector
  private fun binarySearch(searchedKey: String) = binarySearch { compareCharSequence(it, searchedKey) }
  // Performs a binary search on a key vector and return index of the key in key vector
  private fun binarySearch(key: Key): Int = binarySearch { compareKeys(it, key.start) }

  private inline fun binarySearch(crossinline comparisonBlock: (Int) -> Int): Int {
    var low = 0
    var high = size - 1
    while (low <= high) {
      val mid = low + high ushr 1
      val keyPos: Int = buffer.indirect(keyVectorEnd + mid * keyVectorByteWidth, keyVectorByteWidth)
      val cmp: Int = comparisonBlock(keyPos)
      if (cmp < 0) low = mid + 1 else if (cmp > 0) high = mid - 1 else return mid // key found
    }
    return -(low + 1) // key not found
  }

  // compares a CharSequence against a T_KEY
  private fun compareKeys(start: Int, other: Int): Int {
    var bufferPos = start
    var otherPos = other
    val limit: Int = buffer.limit
    var c1: Byte = ZeroByte
    var c2: Byte = ZeroByte
    while (otherPos < limit) {
      c1 = buffer[bufferPos++]
      c2 = buffer[otherPos++]
      when {
        c1 == ZeroByte -> return c1 - c2
        c1 != c2 -> return c1 - c2
      }
    }
    return c1 - c2
  }

  // compares a CharSequence against a [CharSequence]
  private fun compareCharSequence(start: Int, other: CharSequence): Int {
    var bufferPos = start
    var otherPos = 0
    val limit: Int = buffer.limit
    val otherLimit = other.length
    // special loop for ASCII characters. Most of keys should be ASCII only, so this
    // loop should be optimized for that.
    // breaks if a multi-byte character is found
    while (otherPos < otherLimit) {
      val c2 = other[otherPos]
      // not a single byte codepoint
      if (c2.code >= 0x80) {
        break
      }
      val b: Byte = buffer[bufferPos]
      when {
        b == ZeroByte -> return -c2.code
        b < 0 -> break
        b != c2.code.toByte() -> return b - c2.code.toByte()
      }
      ++bufferPos
      ++otherPos
    }
    if (bufferPos < limit)
      return 0

    val comparisonBuffer = ByteArray(4)
    while (bufferPos < limit) {
      val sizeInBuff = Utf8.encodeUtf8CodePoint(other, otherPos, comparisonBuffer)
      if (sizeInBuff == 0) {
        return buffer[bufferPos].toInt()
      }
      for (i in 0 until sizeInBuff) {
        val bufferByte: Byte = buffer[bufferPos++]
        val otherByte: Byte = comparisonBuffer[i]
        when {
          bufferByte == ZeroByte -> return -otherByte
          bufferByte != otherByte -> return bufferByte - otherByte
        }
      }
      otherPos += if (sizeInBuff == 4) 2 else 1
    }
    return 0
  }
}
