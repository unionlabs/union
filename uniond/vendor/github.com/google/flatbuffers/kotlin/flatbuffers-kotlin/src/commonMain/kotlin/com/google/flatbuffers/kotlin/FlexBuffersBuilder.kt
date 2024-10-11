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

@ExperimentalUnsignedTypes
public class FlexBuffersBuilder(
  public val buffer: ReadWriteBuffer,
  private val shareFlag: Int = SHARE_KEYS
) {

  public constructor(initialCapacity: Int = 1024, shareFlag: Int = SHARE_KEYS) :
    this(ArrayReadWriteBuffer(initialCapacity), shareFlag)

  private val stringValuePool: HashMap<String, Value> = HashMap()
  private val stringKeyPool: HashMap<String, Int> = HashMap()
  private val stack: MutableList<Value> = mutableListOf()
  private var finished: Boolean = false

  /**
   * Reset the FlexBuffersBuilder by purging all data that it holds. Buffer might
   * keep its capacity after a reset.
   */
  public fun clear() {
    buffer.clear()
    stringValuePool.clear()
    stringKeyPool.clear()
    stack.clear()
    finished = false
  }

  /**
   * Finish writing the message into the buffer. After that no other element must
   * be inserted into the buffer. Also, you must call this function before start using the
   * FlexBuffer message
   * @return [ReadBuffer] containing the FlexBuffer message
   */
  public fun finish(): ReadBuffer {
    // If you hit this, you likely have objects that were never included
    // in a parent. You need to have exactly one root to finish a buffer.
    // Check your Start/End calls are matched, and all objects are inside
    // some other object.
    if (stack.size != 1) error("There is must be only on object as root. Current ${stack.size}.")
    // Write root value.
    val byteWidth = align(stack[0].elemWidth(buffer.writePosition, 0))
    buffer.requestAdditionalCapacity(byteWidth.value + 2)
    writeAny(stack[0], byteWidth)
    // Write root type.
    buffer.put(stack[0].storedPackedType())
    // Write root size. Normally determined by parent, but root has no parent :)
    buffer.put(byteWidth.value.toByte())
    this.finished = true
    return buffer // TODO: make a read-only shallow copy
  }

  /**
   * Insert a single [Boolean] into the buffer
   * @param value true or false
   */
  public fun put(value: Boolean): Unit = run { this[null] = value }

  /**
   * Insert a null reference into the buffer. A key must be present if element is inserted into a map.
   */
  public fun putNull(key: String? = null): Unit =
    run { stack.add(Value(T_NULL, putKey(key), W_8, 0UL)) }

  /**
   * Insert a single [Boolean] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: Boolean): Unit =
    run { stack.add(Value(T_BOOL, putKey(key), W_8, if (value) 1UL else 0UL)) }

  /**
   * Insert a single [Byte] into the buffer
   */
  public fun put(value: Byte): Unit = set(null, value.toLong())

  /**
   * Insert a single [Byte] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: Byte): Unit = set(key, value.toLong())

  /**
   * Insert a single [Short] into the buffer.
   */
  public fun put(value: Short): Unit = set(null, value.toLong())

  /**
   * Insert a single [Short] into the buffer. A key must be present if element is inserted into a map.
   */
  public inline operator fun set(key: String? = null, value: Short): Unit = set(key, value.toLong())

  /**
   * Insert a single [Int] into the buffer.
   */
  public fun put(value: Int): Unit = set(null, value.toLong())

  /**
   * Insert a single [Int] into the buffer. A key must be present if element is inserted into a map.
   */
  public inline operator fun set(key: String? = null, value: Int): Unit = set(key, value.toLong())

  /**
   * Insert a single [Long] into the buffer.
   */
  public fun put(value: Long): Unit = set(null, value)

  /**
   * Insert a single [Long] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: Long): Unit =
    run { stack.add(Value(T_INT, putKey(key), value.toULong().widthInUBits(), value.toULong())) }

  /**
   * Insert a single [UByte] into the buffer
   */
  public fun put(value: UByte): Unit = set(null, value.toULong())

  /**
   * Insert a single [UByte] into the buffer. A key must be present if element is inserted into a map.
   */
  public inline operator fun set(key: String? = null, value: UByte): Unit = set(key, value.toULong())

  /**
   * Insert a single [UShort] into the buffer.
   */
  public fun put(value: UShort): Unit = set(null, value.toULong())

  /**
   * Insert a single [UShort] into the buffer. A key must be present if element is inserted into a map.
   */
  private inline operator fun set(key: String? = null, value: UShort): Unit = set(key, value.toULong())

  /**
   * Insert a single [UInt] into the buffer.
   */
  public fun put(value: UInt): Unit = set(null, value.toULong())

  /**
   * Insert a single [UInt] into the buffer. A key must be present if element is inserted into a map.
   */
  private inline operator fun set(key: String? = null, value: UInt): Unit = set(key, value.toULong())

  /**
   * Insert a single [ULong] into the buffer.
   */
  public fun put(value: ULong): Unit = set(null, value)

  /**
   * Insert a single [ULong] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: ULong): Unit =
    run { stack.add(Value(T_UINT, putKey(key), value.widthInUBits(), value)) }

  /**
   * Insert a single [Float] into the buffer.
   */
  public fun put(value: Float): Unit = run { this[null] = value }

  /**
   * Insert a single [Float] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: Float): Unit =
    run { stack.add(Value(T_FLOAT, putKey(key), W_32, dValue = value.toDouble())) }

  /**
   * Insert a single [Double] into the buffer.
   */
  public fun put(value: Double): Unit = run { this[null] = value }

  /**
   * Insert a single [Double] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: Double): Unit =
    run { stack.add(Value(T_FLOAT, putKey(key), W_64, dValue = value)) }

  /**
   * Insert a single [String] into the buffer.
   */
  public fun put(value: String): Int = set(null, value)

  /**
   * Insert a single [String] into the buffer. A key must be present if element is inserted into a map.
   */
  public operator fun set(key: String? = null, value: String): Int {
    val iKey = putKey(key)
    val holder = if (shareFlag and SHARE_STRINGS != 0) {
      stringValuePool.getOrPut(value) {
        writeString(iKey, value).also { stringValuePool[value] = it }
      }.copy(key = iKey)
    } else {
      writeString(iKey, value)
    }
    stack.add(holder)
    return holder.iValue.toInt()
  }

  /**
   * Adds a [ByteArray] into the message as a [Blob].
   * @param value byte array
   * @return position in buffer as the start of byte array
   */
  public fun put(value: ByteArray): Int = set(null, value)

  /**
   * Adds a [ByteArray] into the message as a [Blob]. A key must be present if element is inserted into a map.
   * @param value byte array
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: ByteArray): Int {
    val element = writeBlob(putKey(key), value, T_BLOB, false)
    stack.add(element)
    return element.iValue.toInt()
  }

  /**
   * Adds a [IntArray] into the message as a typed vector of fixed size.
   * @param value [IntArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: IntArray): Int = set(null, value)

  /**
   * Adds a [IntArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [IntArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: IntArray): Int =
    setTypedVector(key, value.size, T_VECTOR_INT, value.widthInUBits()) { writeIntArray(value, it) }

  /**
   * Adds a [ShortArray] into the message as a typed vector of fixed size.
   * @param value [ShortArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: ShortArray): Int = set(null, value)

  /**
   * Adds a [ShortArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [ShortArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: ShortArray): Int =
    setTypedVector(key, value.size, T_VECTOR_INT, value.widthInUBits()) { writeIntArray(value, it) }

  /**
   * Adds a [LongArray] into the message as a typed vector of fixed size.
   * @param value [LongArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: LongArray): Int = set(null, value)

  /**
   * Adds a [LongArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [LongArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: LongArray): Int =
    setTypedVector(key, value.size, T_VECTOR_INT, value.widthInUBits()) { writeIntArray(value, it) }

  /**
   * Adds a [FloatArray] into the message as a typed vector of fixed size.
   * @param value [FloatArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: FloatArray): Int = set(null, value)

  /**
   * Adds a [FloatArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [FloatArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: FloatArray): Int =
    setTypedVector(key, value.size, T_VECTOR_FLOAT, W_32) { writeFloatArray(value) }

  /**
   * Adds a [DoubleArray] into the message as a typed vector of fixed size.
   * @param value [DoubleArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: DoubleArray): Int = set(null, value)

  /**
   * Adds a [DoubleArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [DoubleArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: DoubleArray): Int =
    setTypedVector(key, value.size, T_VECTOR_FLOAT, W_64) { writeFloatArray(value) }

  /**
   * Adds a [UByteArray] into the message as a typed vector of fixed size.
   * @param value [UByteArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: UByteArray): Int = set(null, value)

  /**
   * Adds a [UByteArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [UByteArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: UByteArray): Int =
    setTypedVec(key) { value.forEach { put(it) } }

  /**
   * Adds a [UShortArray] into the message as a typed vector of fixed size.
   * @param value [UShortArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: UShortArray): Int = set(null, value)

  /**
   * Adds a [UShortArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [UShortArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: UShortArray): Int =
    setTypedVec(key) { value.forEach { put(it) } }

  /**
   * Adds a [UIntArray] into the message as a typed vector of fixed size.
   * @param value [UIntArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: UIntArray): Int = set(null, value)

  /**
   * Adds a [UIntArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [UIntArray]
   * @return position in buffer as the start of byte array
   */
  public fun set(key: String? = null, value: UIntArray): Int =
    setTypedVec(key) { value.forEach { put(it) } }

  /**
   * Adds a [ULongArray] into the message as a typed vector of fixed size.
   * @param value [ULongArray]
   * @return position in buffer as the start of byte array
   */
  public fun put(value: ULongArray): Int = set(null, value)

  /**
   * Adds a [ULongArray] into the message as a typed vector of fixed size.
   * A key must be present if element is inserted into a map.
   * @param value [ULongArray]
   * @return position in buffer as the start of byte array
   */
  public operator fun set(key: String? = null, value: ULongArray): Int =
    setTypedVec(key) { value.forEach { put(it) } }

  /**
   * Creates a new vector will all elements inserted in [block].
   * @param block where elements will be inserted
   * @return position in buffer as the start of byte array
   */
  public inline fun putVector(crossinline block: FlexBuffersBuilder.() -> Unit): Int {
    val pos = startVector()
    this.block()
    return endVector(pos)
  }

  /**
   * Creates a new typed vector will all elements inserted in [block].
   * @param block where elements will be inserted
   * @return position in buffer as the start of byte array
   */
  public inline fun putTypedVector(crossinline block: FlexBuffersBuilder.() -> Unit): Int {
    val pos = startVector()
    this.block()
    return endTypedVector(pos)
  }

  /**
   * Helper function to return position for starting a new vector.
   */
  public fun startVector(): Int = stack.size

  /**
   * Finishes a vector element. The initial position of the vector must be passed
   * @param position position at the start of the vector
   */
  public fun endVector(position: Int): Int = endVector(null, position)

  /**
   * Finishes a vector element. The initial position of the vector must be passed
   * @param position position at the start of the vector
   */
  public fun endVector(key: String? = null, position: Int): Int =
    endAnyVector(position) { createVector(putKey(key), position, stack.size - position) }
  /**
   * Finishes a typed vector element. The initial position of the vector must be passed
   * @param position position at the start of the vector
   */
  public fun endTypedVector(position: Int): Int = endTypedVector(position, null)

  /**
   * Helper function to return position for starting a new vector.
   */
  public fun startMap(): Int = stack.size

  /**
   * Creates a new map will all elements inserted in [block].
   * @param block where elements will be inserted
   * @return position in buffer as the start of byte array
   */
  public inline fun putMap(key: String? = null, crossinline block: FlexBuffersBuilder.() -> Unit): Int {
    val pos = startMap()
    this.block()
    return endMap(pos, key)
  }

  /**
   * Finishes a map, but writing the information in the buffer
   * @param key   key used to store element in map
   * @return Reference to the map
   */
  public fun endMap(start: Int, key: String? = null): Int {
    stack.subList(start, stack.size).sortWith(keyComparator)
    val length = stack.size - start
    val keys = createKeyVector(start, length)
    val vec = putMap(putKey(key), start, length, keys)
    // Remove temp elements and return map.
    while (stack.size > start) {
      stack.removeAt(stack.size - 1)
    }
    stack.add(vec)
    return vec.iValue.toInt()
  }

  private inline fun setTypedVector(
    key: String? = null,
    length: Int,
    vecType: FlexBufferType,
    bitWidth: BitWidth,
    crossinline writeBlock: (ByteWidth) -> Unit
  ): Int {
    val keyPos = putKey(key)
    val byteWidth = align(bitWidth)
    // Write vector. First the keys width/offset if available, and size.
    // write the size
    writeInt(length, byteWidth)

    // Then the actual data.
    val vloc: Int = buffer.writePosition
    writeBlock(byteWidth)
    stack.add(Value(vecType, keyPos, bitWidth, vloc.toULong()))
    return vloc
  }

  private inline fun setTypedVec(key: String? = null, crossinline block: FlexBuffersBuilder.() -> Unit): Int {
    val pos = startVector()
    this.block()
    return endTypedVector(pos, key)
  }

  public fun endTypedVector(position: Int, key: String? = null): Int =
    endAnyVector(position) { createTypedVector(putKey(key), position, stack.size - position) }

  private inline fun endAnyVector(start: Int, crossinline creationBlock: () -> Value): Int {
    val vec = creationBlock()
    // Remove temp elements and return vector.
    while (stack.size > start) {
      stack.removeLast()
    }
    stack.add(vec)
    return vec.iValue.toInt()
  }

  private inline fun putKey(key: String? = null): Int {
    if (key == null) return -1
    return if ((shareFlag and SHARE_KEYS) != 0) {
      stringKeyPool.getOrPut(key) {
        val pos: Int = buffer.writePosition
        val encodedKeySize = Utf8.encodedLength(key)
        buffer.requestAdditionalCapacity(encodedKeySize + 1)
        buffer.put(key, encodedKeySize)
        buffer.put(ZeroByte)
        pos
      }
    } else {
      val pos: Int = buffer.writePosition
      val encodedKeySize = Utf8.encodedLength(key)
      buffer.requestAdditionalCapacity(encodedKeySize + 1)
      buffer.put(key, encodedKeySize)
      buffer.put(ZeroByte)
      pos
    }
  }

  private fun writeAny(toWrite: Value, byteWidth: ByteWidth) = when (toWrite.type) {
    T_NULL, T_BOOL, T_INT, T_UINT -> writeInt(toWrite.iValue, byteWidth)
    T_FLOAT -> writeDouble(toWrite.dValue, byteWidth)
    else -> writeOffset(toWrite.iValue.toInt(), byteWidth)
  }

  private fun writeString(key: Int, s: String): Value {
    val encodedSize = Utf8.encodedLength(s)
    val bitWidth = encodedSize.toULong().widthInUBits()
    val byteWidth = align(bitWidth)

    writeInt(encodedSize, byteWidth)

    buffer.requestAdditionalCapacity(encodedSize + 1)
    val sloc: Int = buffer.writePosition
    if (encodedSize > 0)
      buffer.put(s, encodedSize)
    buffer.put(ZeroByte)
    return Value(T_STRING, key, bitWidth, sloc.toULong())
  }

  private fun writeDouble(toWrite: Double, byteWidth: ByteWidth) {
    buffer.requestAdditionalCapacity(byteWidth.value)
    when (byteWidth.value) {
      4 -> buffer.put(toWrite.toFloat())
      8 -> buffer.put(toWrite)
      else -> Unit
    }
  }

  private fun writeOffset(toWrite: Int, byteWidth: ByteWidth) {
    buffer.requestAdditionalCapacity(byteWidth.value)
    val relativeOffset = (buffer.writePosition - toWrite)
    if (byteWidth.value != 8 && relativeOffset >= 1L shl byteWidth.value * 8) error("invalid offset $relativeOffset, writer pos ${buffer.writePosition}")
    writeInt(relativeOffset, byteWidth)
  }

  private inline fun writeBlob(key: Int, blob: ByteArray, type: FlexBufferType, trailing: Boolean): Value {
    val bitWidth = blob.size.toULong().widthInUBits()
    val byteWidth = align(bitWidth)

    writeInt(blob.size, byteWidth)

    val sloc: Int = buffer.writePosition
    buffer.requestAdditionalCapacity(blob.size + trailing.compareTo(false))
    buffer.put(blob, 0, blob.size)
    if (trailing) {
      buffer.put(ZeroByte)
    }
    return Value(type, key, bitWidth, sloc.toULong())
  }

  private fun writeIntArray(value: IntArray, byteWidth: ByteWidth) =
    writeIntegerArray(0, value.size, byteWidth) { value[it].toULong() }

  private fun writeIntArray(value: ShortArray, byteWidth: ByteWidth) =
    writeIntegerArray(0, value.size, byteWidth) { value[it].toULong() }

  private fun writeIntArray(value: LongArray, byteWidth: ByteWidth) =
    writeIntegerArray(0, value.size, byteWidth) { value[it].toULong() }

  private fun writeFloatArray(value: FloatArray) {
    buffer.requestAdditionalCapacity(Float.SIZE_BYTES * value.size)
    value.forEach { buffer.put(it) }
  }

  private fun writeFloatArray(value: DoubleArray) {
    buffer.requestAdditionalCapacity(Double.SIZE_BYTES * value.size)
    value.forEach { buffer.put(it) }
  }

  private inline fun writeIntegerArray(
    start: Int,
    size: Int,
    byteWidth: ByteWidth,
    crossinline valueBlock: (Int) -> ULong
  ) {
    buffer.requestAdditionalCapacity(size * byteWidth.value)
    return when (byteWidth.value) {
      1 -> for (i in start until start + size) {
        buffer.put(valueBlock(i).toUByte())
      }
      2 -> for (i in start until start + size) {
        buffer.put(valueBlock(i).toUShort())
      }
      4 -> for (i in start until start + size) {
        buffer.put(valueBlock(i).toUInt())
      }
      8 -> for (i in start until start + size) {
        buffer.put(valueBlock(i))
      }
      else -> Unit
    }
  }

  private fun writeInt(value: Int, byteWidth: ByteWidth) {
    buffer.requestAdditionalCapacity(byteWidth.value)
    when (byteWidth.value) {
      1 -> buffer.put(value.toUByte())
      2 -> buffer.put(value.toUShort())
      4 -> buffer.put(value.toUInt())
      8 -> buffer.put(value.toULong())
      else -> Unit
    }
  }

  private fun writeInt(value: ULong, byteWidth: ByteWidth) {
    buffer.requestAdditionalCapacity(byteWidth.value)
    when(byteWidth.value) {
      1 -> buffer.put(value.toUByte())
      2 -> buffer.put(value.toUShort())
      4 -> buffer.put(value.toUInt())
      8 -> buffer.put(value)
      else -> Unit
    }
  }

  // Align to prepare for writing a scalar with a certain size.
  // returns the amounts of bytes needed to be written.
  private fun align(alignment: BitWidth): ByteWidth {
    val byteWidth = 1 shl alignment.value
    var padBytes = paddingBytes(buffer.writePosition, byteWidth)
    buffer.requestCapacity(buffer.capacity + padBytes)
    while (padBytes-- != 0) {
      buffer.put(ZeroByte)
    }
    return ByteWidth(byteWidth)
  }

  private fun calculateKeyVectorBitWidth(start: Int, length: Int): BitWidth {
    val bitWidth = length.toULong().widthInUBits()
    var width = bitWidth
    val prefixElems = 1
    // Check bit widths and types for all elements.
    for (i in start until stack.size) {
      val elemWidth = elemWidth(T_KEY, W_8, stack[i].key.toLong(), buffer.writePosition, i + prefixElems)
      width = width.max(elemWidth)
    }
    return width
  }

  private fun createKeyVector(start: Int, length: Int): Value {
    // Figure out smallest bit width we can store this vector with.
    val bitWidth = calculateKeyVectorBitWidth(start, length)
    val byteWidth = align(bitWidth)
    // Write vector. First the keys width/offset if available, and size.
    writeInt(length, byteWidth)
    // Then the actual data.
    val vloc = buffer.writePosition.toULong()
    for (i in start until stack.size) {
      val pos = stack[i].key
      if (pos == -1) error("invalid position $pos for key")
      writeOffset(stack[i].key, byteWidth)
    }
    // Then the types.
    return Value(T_VECTOR_KEY, -1, bitWidth, vloc)
  }

  private inline fun createVector(key: Int, start: Int, length: Int, keys: Value? = null): Value {
    return createAnyVector(key, start, length, T_VECTOR, keys) {
      // add types since we are not creating a typed vector.
      buffer.requestAdditionalCapacity(stack.size)
      for (i in start until stack.size) {
        buffer.put(stack[i].storedPackedType(it))
      }
    }
  }

  private fun putMap(key: Int, start: Int, length: Int, keys: Value? = null): Value {
    return createAnyVector(key, start, length, T_MAP, keys) {
      // add types since we are not creating a typed vector.
      buffer.requestAdditionalCapacity(stack.size)
      for (i in start until stack.size) {
        buffer.put(stack[i].storedPackedType(it))
      }
    }
  }

  private inline fun createTypedVector(key: Int, start: Int, length: Int, keys: Value? = null): Value {
    // We assume the callers of this method guarantees all elements are of the same type.
    val elementType: FlexBufferType = stack[start].type
    for (i in start + 1 until length) {
      if (elementType != stack[i].type) error("TypedVector does not support array of different element types")
    }
    if (!elementType.isTypedVectorElementType()) error("TypedVector does not support this element type")
    return createAnyVector(key, start, length, elementType.toTypedVector(), keys)
  }

  private inline fun createAnyVector(
    key: Int,
    start: Int,
    length: Int,
    type: FlexBufferType,
    keys: Value? = null,
    crossinline typeBlock: (BitWidth) -> Unit = {}
  ): Value {
    // Figure out the smallest bit width we can store this vector with.
    var bitWidth = W_8.max(length.toULong().widthInUBits())
    var prefixElems = 1
    if (keys != null) {
      // If this vector is part of a map, we will pre-fix an offset to the keys
      // to this vector.
      bitWidth = bitWidth.max(keys.elemWidth(buffer.writePosition, 0))
      prefixElems += 2
    }
    // Check bit widths and types for all elements.
    for (i in start until stack.size) {
      val elemWidth = stack[i].elemWidth(buffer.writePosition, i + prefixElems)
      bitWidth = bitWidth.max(elemWidth)
    }
    val byteWidth = align(bitWidth)
    // Write vector. First the keys width/offset if available, and size.
    if (keys != null) {
      writeOffset(keys.iValue.toInt(), byteWidth)
      writeInt(1 shl keys.minBitWidth.value, byteWidth)
    }
    // write the size
    writeInt(length, byteWidth)

    // Then the actual data.
    val vloc: Int = buffer.writePosition
    for (i in start until stack.size) {
      writeAny(stack[i], byteWidth)
    }

    // Optionally you can introduce the types for non-typed vector
    typeBlock(bitWidth)
    return Value(type, key, bitWidth, vloc.toULong())
  }

  // A lambda to sort map keys
  internal val keyComparator = object : Comparator<Value> {
    override fun compare(a: Value, b: Value): Int {
      var ia: Int = a.key
      var io: Int = b.key
      var c1: Byte
      var c2: Byte
      do {
        c1 = buffer[ia]
        c2 = buffer[io]
        if (c1.toInt() == 0) return c1 - c2
        ia++
        io++
      } while (c1 == c2)
      return c1 - c2
    }
  }

  public companion object {
    /**
     * No keys or strings will be shared
     */
    public const val SHARE_NONE: Int = 0

    /**
     * Keys will be shared between elements. Identical keys will only be serialized once, thus possibly saving space.
     * But serialization performance might be slower and consumes more memory.
     */
    public const val SHARE_KEYS: Int = 1

    /**
     * Strings will be shared between elements. Identical strings will only be serialized once, thus possibly saving space.
     * But serialization performance might be slower and consumes more memory. This is ideal if you expect many repeated
     * strings on the message.
     */
    public const val SHARE_STRINGS: Int = 2

    /**
     * Strings and keys will be shared between elements.
     */
    public const val SHARE_KEYS_AND_STRINGS: Int = 3
  }
}
