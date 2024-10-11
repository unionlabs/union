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

import kotlin.jvm.JvmOverloads


/**
 * Class that helps you build a FlatBuffer.  See the section
 * "Use in Kotlin" in the main FlatBuffers documentation.
 */
public class FlatBufferBuilder @JvmOverloads constructor(
  private val initialSize: Int = 1024,
  private var buffer: ReadWriteBuffer = ArrayReadWriteBuffer(initialSize)
) {
  // Remaining space in the ByteBuffer.
  private var space: Int = buffer.capacity

  // Minimum alignment encountered so far.
  private var minalign: Int = 1

  // The vtable for the current table.
  private var vtable: IntArray = IntArray(16)

  // The amount of fields we're actually using.
  private var vtableInUse: Int = 0

  // Whether we are currently serializing a table.
  private var nested: Boolean = false

  // Whether the buffer is finished.
  private var finished: Boolean = false

  // Starting offset of the current struct/table.
  private var objectStart: Int = 0

  // List of offsets of all vtables.
  private var vtables = IntArray(16)

  // Number of entries in `vtables` in use.
  private var numVtables = 0

  // For the current vector being built.
  private var vectorNumElems = 0

  // False omits default values from the serialized data.
  private var forceDefaults = false

  // map used to cache shared strings.
  private var stringPool: MutableMap<CharSequence, Offset<String>>? = null

  /**
   * Reset the FlatBufferBuilder by purging all data that it holds.
   */
  public fun clear() {
    space = buffer.capacity
    buffer.clear()
    minalign = 1
    vtable.fill(0, 0, vtableInUse)
    vtableInUse = 0
    nested = false
    finished = false
    objectStart = 0
    numVtables = 0
    vectorNumElems = 0
    stringPool?.clear()
  }

  /**
   * Offset relative to the end of the buffer.
   *
   * @return Offset relative to the end of the buffer.
   */
  public fun offset(): Int = buffer.capacity - space

  /**
   * Add zero valued bytes to prepare a new entry to be added.
   *
   * @param byteSize Number of bytes to add.
   */
  public fun pad(byteSize: Int) {
    for (i in 0 until byteSize) buffer[--space] = 0.toByte()
  }

  /**
   * Prepare to write an element of `size` after `additional_bytes`
   * have been written, e.g. if you write a string, you need to align such
   * the int length field is aligned to [com.google.flatbuffers.Int.SIZE_BYTES], and
   * the string data follows it directly.  If all you need to do is alignment, `additional_bytes`
   * will be 0.
   *
   * @param size This is the of the new element to write.
   * @param additionalBytes The padding size.
   */
  public fun prep(size: Int, additionalBytes: Int) {
    // Track the biggest thing we've ever aligned to.
    if (size > minalign) minalign = size
    // Find the amount of alignment needed such that `size` is properly
    // aligned after `additional_bytes`

    val alignSize: Int = ((buffer.capacity - space + additionalBytes).inv() + 1).and(size - 1)
    // Reallocate the buffer if needed.
    while (space < alignSize + size + additionalBytes) {
      val oldBufSize: Int = buffer.capacity
      val newBufSize = buffer.moveWrittenDataToEnd(oldBufSize + alignSize + size + additionalBytes)
      space += newBufSize - oldBufSize
    }
    if (alignSize > 0) {
      pad(alignSize)
    }
  }

  /**
   * Add a `boolean` to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A `boolean` to put into the buffer.
   */
  public fun put(x: Boolean) {
    space -= Byte.SIZE_BYTES
    buffer[space] = (if (x) 1 else 0).toByte()
  }

  /**
   * Add a [UByte] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [UByte] to put into the buffer.
   */
  public fun put(x: UByte): Unit = put(x.toByte())

  /**
   * Add a [Byte] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [Byte] to put into the buffer.
   */
  public fun put(x: Byte) {
    space -= Byte.SIZE_BYTES
    buffer[space] = x
  }

  /**
   * Add a [UShort] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [UShort] to put into the buffer.
   */
  public fun put(x: UShort): Unit = put(x.toShort())

  /**
   * Add a [Short] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [Short] to put into the buffer.
   */
  public fun put(x: Short) {
    space -= Short.SIZE_BYTES
    buffer.set(space, x)
  }

  /**
   * Add an [UInt] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x An [UInt] to put into the buffer.
   */
  public fun put(x: UInt): Unit = put(x.toInt())

  /**
   * Add an [Int] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x An [Int] to put into the buffer.
   */
  public fun put(x: Int){
    space -= Int.SIZE_BYTES
    buffer.set(space, x)
  }

  /**
   * Add a [ULong] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [ULong] to put into the buffer.
   */
  public fun put(x: ULong): Unit = put(x.toLong())

  /**
   * Add a [Long] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [Long] to put into the buffer.
   */
  public fun put(x: Long) {
    space -= Long.SIZE_BYTES
    buffer.set(space, x)
  }

  /**
   * Add a [Float] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [Float] to put into the buffer.
   */
  public fun put(x: Float) {
    space -= Float.SIZE_BYTES
    buffer.set(space, x)
  }

  /**
   * Add a [Double] to the buffer, backwards from the current location. Doesn't align nor
   * check for space.
   *
   * @param x A [Double] to put into the buffer.
   */
  public fun put(x: Double) {
    space -= Double.SIZE_BYTES
    buffer.set(space, x)
  }

  /**
   * Add a [Boolean] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [Boolean] to put into the buffer.
   */
  public fun add(x: Boolean) {
    prep(Byte.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add a [UByte] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [UByte] to put into the buffer.
   */
  public fun add(x: UByte): Unit = add(x.toByte())

  /**
   * Add a [Byte] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [Byte] to put into the buffer.
   */
  public fun add(x: Byte) {
    prep(Byte.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add a [UShort] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [UShort] to put into the buffer.
   */
  public fun add(x: UShort): Unit = add(x.toShort())

  /**
   * Add a [Short] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [Short] to put into the buffer.
   */
  public fun add(x: Short) {
    prep(Short.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add an [Unit] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x An [Unit] to put into the buffer.
   */
  public fun add(x: UInt): Unit = add(x.toInt())

  /**
   * Add an [Int] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x An [Int] to put into the buffer.
   */
  public fun add(x: Int) {
    prep(Int.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add a [ULong] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [ULong] to put into the buffer.
   */
  public fun add(x: ULong): Unit = add(x.toLong())

  /**
   * Add a `long` to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A `long` to put into the buffer.
   */
  public fun add(x: Long) {
    prep(Long.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add a [Float] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [Float] to put into the buffer.
   */
  public fun add(x: Float) {
    prep(Float.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Add a [Double] to the buffer, properly aligned, and grows the buffer (if necessary).
   *
   * @param x A [Double] to put into the buffer.
   */
  public fun add(x: Double) {
    prep(Double.SIZE_BYTES, 0)
    put(x)
  }

  /**
   * Adds on offset, relative to where it will be written.
   *
   * @param off The offset to add.
   */
  public fun add(off: Offset<*>): Unit = addOffset(off.value)
  public fun add(off: VectorOffset<*>): Unit = addOffset(off.value)
  private fun addOffset(off: Int) {
    prep(Int.SIZE_BYTES, 0) // Ensure alignment is already done.
    put(buffer.capacity - space - off + Int.SIZE_BYTES)
  }

  /**
   * Start a new array/vector of objects.  Users usually will not call
   * this directly.  The `FlatBuffers` compiler will create a start/end
   * method for vector types in generated code.
   *
   *
   * The expected sequence of calls is:
   *
   *  1. Start the array using this method.
   *  1. Call [.addOffset] `num_elems` number of times to set
   * the offset of each element in the array.
   *  1. Call [.endVector] to retrieve the offset of the array.
   *
   *
   *
   * For example, to create an array of strings, do:
   * <pre>`// Need 10 strings
   * FlatBufferBuilder builder = new FlatBufferBuilder(existingBuffer);
   * int[] offsets = new int[10];
   *
   * for (int i = 0; i < 10; i++) {
   * offsets[i] = fbb.createString(" " + i);
   * }
   *
   * // Have the strings in the buffer, but don't have a vector.
   * // Add a vector that references the newly created strings:
   * builder.startVector(4, offsets.length, 4);
   *
   * // Add each string to the newly created vector
   * // The strings are added in reverse order since the buffer
   * // is filled in back to front
   * for (int i = offsets.length - 1; i >= 0; i--) {
   * builder.addOffset(offsets[i]);
   * }
   *
   * // Finish off the vector
   * int offsetOfTheVector = fbb.endVector();
   `</pre> *
   *
   * @param elemSize The size of each element in the array.
   * @param numElems The number of elements in the array.
   * @param alignment The alignment of the array.
   */
  public fun startVector(elemSize: Int, numElems: Int, alignment: Int) {
    notNested()
    vectorNumElems = numElems
    prep(Int.SIZE_BYTES, elemSize * numElems)
    prep(alignment, elemSize * numElems) // Just in case alignment > int.
    nested = true
  }
  public fun startString(numElems: Int): Unit = startVector(1, numElems, 1)

  /**
   * Finish off the creation of an array and all its elements.  The array
   * must be created with [.startVector].
   *
   * @return The offset at which the newly created array starts.
   * @see .startVector
   */
  public fun <T> endVector(): VectorOffset<T> {
    if (!nested) throw AssertionError("FlatBuffers: endVector called without startVector")
    nested = false
    put(vectorNumElems)
    return VectorOffset(offset())
  }

  public fun endString(): Offset<String> {
    if (!nested) throw AssertionError("FlatBuffers: endString called without startString")
    nested = false
    put(vectorNumElems)
    return Offset(offset())
  }

  private fun endVector(): Int {
    if (!nested) throw AssertionError("FlatBuffers: endVector called without startVector")
    nested = false
    put(vectorNumElems)
    return offset()
  }

  /**
   * Create a new array/vector and return a ByteBuffer to be filled later.
   * Call [endVector] after this method to get an offset to the beginning
   * of vector.
   *
   * @param elemSize the size of each element in bytes.
   * @param numElems number of elements in the vector.
   * @param alignment byte alignment.
   * @return ByteBuffer with position and limit set to the space allocated for the array.
   */
  public fun createUnintializedVector(elemSize: Int, numElems: Int, alignment: Int): ReadWriteBuffer {
    val length = elemSize * numElems
    startVector(elemSize, numElems, alignment)
    space -= length
    buffer.writePosition = space
    return buffer.writeSlice(buffer.writePosition, length)
  }

  /**
   * Create a vector of tables.
   *
   * @param offsets Offsets of the tables.
   * @return Returns offset of the vector.
   */
  public fun <T> createVectorOfTables(offsets: Array<Offset<T>>): VectorOffset<T> {
    notNested()
    startVector(Int.SIZE_BYTES, offsets.size, Int.SIZE_BYTES)
    for (i in offsets.indices.reversed()) add(offsets[i])
    return VectorOffset(endVector())
  }

  /**
   * Create a vector of sorted by the key tables.
   *
   * @param obj Instance of the table subclass.
   * @param offsets Offsets of the tables.
   * @return Returns offset of the sorted vector.
   */
  public fun <T : Table> createSortedVectorOfTables(obj: T, offsets: Array<Offset<T>>): VectorOffset<T> {
    obj.sortTables(offsets, buffer)
    return createVectorOfTables(offsets)
  }

  /**
   * Encode the String `s` in the buffer using UTF-8. If a String with
   * this exact contents has already been serialized using this method,
   * instead simply returns the offset of the existing String.
   *
   * Usage of the method will incur into additional allocations,
   * so it is advisable to use it only when it is known upfront that
   * your message will have several repeated strings.
   *
   * @param s The String to encode.
   * @return The offset in the buffer where the encoded String starts.
   */
  public fun createSharedString(s: CharSequence): Offset<String> {
    if (stringPool == null) {
      stringPool = HashMap()
      val offset = createString(s)
      stringPool!![s] = offset
      return offset
    }
    var offset = stringPool!![s]
    if (offset == null) {
      offset = createString(s)
      stringPool?.put(s, offset)
    }
    return offset
  }

  /**
   * Encode the [CharSequence] `s` in the buffer using UTF-8.
   * @param s The [CharSequence] to encode.
   * @return The offset in the buffer where the encoded string starts.
   */
  public fun createString(s: CharSequence): Offset<String> {
    val length: Int = Utf8.encodedLength(s)
    add(0.toByte())
    startString(length)
    space -= length
    buffer.writePosition = space
    buffer.put(s, length)
    return endString()
  }

  /**
   * Create a string in the buffer from an already encoded UTF-8 string in a ByteBuffer.
   *
   * @param s An already encoded UTF-8 string as a `ByteBuffer`.
   * @return The offset in the buffer where the encoded string starts.
   */
 public fun createString(s: ReadBuffer): Offset<String> {
    val length: Int = s.limit
    add(0.toByte())
    startVector(1, length, 1)
    space -= length
    buffer.writePosition = space
    buffer.put(s)
    return endString()
  }

  /**
   * Create a byte array in the buffer.
   *
   * @param arr A source array with data
   * @return The offset in the buffer where the encoded array starts.
   */
  public fun createByteVector(arr: ByteArray): VectorOffset<Byte> {
    val length = arr.size
    startVector(1, length, 1)
    space -= length
    buffer.writePosition = space
    buffer.put(arr)
    return VectorOffset(endVector())
  }

  /**
   * Create a byte array in the buffer.
   *
   * @param arr a source array with data.
   * @param offset the offset in the source array to start copying from.
   * @param length the number of bytes to copy from the source array.
   * @return The offset in the buffer where the encoded array starts.
   */
  public fun createByteVector(arr: ByteArray, offset: Int, length: Int): VectorOffset<Byte> {
    startVector(1, length, 1)
    space -= length
    buffer.writePosition = space
    buffer.put(arr, offset, length)
    return VectorOffset(endVector())
  }

  /**
   * Create a byte array in the buffer.
   *
   * The source [ReadBuffer] position is advanced until [ReadBuffer.limit]
   * after this call.
   *
   * @param data A source [ReadBuffer] with data.
   * @return The offset in the buffer where the encoded array starts.
   */
 public fun createByteVector(data: ReadBuffer, from: Int = 0, until: Int = data.limit): VectorOffset<Byte> {
    val length: Int = until - from
    startVector(1, length, 1)
    space -= length
    buffer.writePosition = space
    buffer.put(data, from, until)
    return VectorOffset(endVector())
  }

  /**
   * Should not be accessing the final buffer before it is finished.
   */
  public fun finished() {
    if (!finished) throw AssertionError(
      "FlatBuffers: you can only access the serialized buffer after it has been" +
        " finished by FlatBufferBuilder.finish()."
    )
  }

  /**
   * Should not be creating any other object, string or vector
   * while an object is being constructed.
   */
  public fun notNested() {
    if (nested) throw AssertionError("FlatBuffers: object serialization must not be nested.")
  }

  /**
   * Structures are always stored inline, they need to be created right
   * where they're used.  You'll get this assertion failure if you
   * created it elsewhere.
   *
   * @param obj The offset of the created object.
   */
  public fun nested(obj: Int) {
    if (obj != offset()) throw AssertionError("FlatBuffers: struct must be serialized inline.")
  }

  /**
   * Start encoding a new object in the buffer.  Users will not usually need to
   * call this directly. The `FlatBuffers` compiler will generate helper methods
   * that call this method internally.
   *
   *
   * For example, using the "Monster" code found on the "landing page". An
   * object of type `Monster` can be created using the following code:
   *
   * <pre>`int testArrayOfString = Monster.createTestarrayofstringVector(fbb, new int[] {
   * fbb.createString("test1"),
   * fbb.createString("test2")
   * });
   *
   * Monster.startMonster(fbb);
   * Monster.addPos(fbb, Vec3.createVec3(fbb, 1.0f, 2.0f, 3.0f, 3.0,
   * Color.Green, (short)5, (byte)6));
   * Monster.addHp(fbb, (short)80);
   * Monster.addName(fbb, str);
   * Monster.addInventory(fbb, inv);
   * Monster.addTestType(fbb, (byte)Any.Monster);
   * Monster.addTest(fbb, mon2);
   * Monster.addTest4(fbb, test4);
   * Monster.addTestarrayofstring(fbb, testArrayOfString);
   * int mon = Monster.endMonster(fbb);
   `</pre> *
   *
   *
   * Here:
   *
   *  * The call to `Monster#startMonster(FlatBufferBuilder)` will call this
   * method with the right number of fields set.
   *  * `Monster#endMonster(FlatBufferBuilder)` will ensure [.endObject] is called.
   *
   *
   *
   * It's not recommended to call this method directly.  If it's called manually, you must ensure
   * to audit all calls to it whenever fields are added or removed from your schema.  This is
   * automatically done by the code generated by the `FlatBuffers` compiler.
   *
   * @param numFields The number of fields found in this object.
   */
  public fun startTable(numFields: Int) {
    notNested()
    if (vtable.size < numFields) {
      vtable = IntArray(numFields)
    }
    vtableInUse = numFields
    for (i in 0 until vtableInUse)
      vtable[i] = 0
    nested = true
    objectStart = offset()
  }

  /**
   * Add a [Boolean] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Boolean, d: Boolean?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Boolean, d: Boolean) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add a [UByte] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: UByte, d: UByte?): Unit = add(o, x.toByte(), d?.toByte())
  // unboxed specialization
  public fun add(o: Int, x: UByte, d: UByte): Unit = add(o, x.toByte(), d.toByte())

  /**
   * Add a [Byte] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Byte, d: Byte?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Byte, d: Byte) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  /**
   * Add a [UShort] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: UShort, d: UShort?): Unit = add(o, x.toShort(), d?.toShort())
  // unboxed specialization
  public fun add(o: Int, x: UShort, d: UShort): Unit = add(o, x.toShort(), d.toShort())


  /**
   * Add a [Short] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Short, d: Short?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Short, d: Short) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add a [UInt] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: UInt, d: UInt?): Unit = add(o, x.toInt(), d?.toInt())
  // unboxed specialization
  public fun add(o: Int, x: UInt, d: UInt): Unit = add(o, x.toInt(), d.toInt())

  /**
   * Add a [Int] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Int, d: Int?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Int, d: Int) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  /**
   * Add a [ULong] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: ULong, d: ULong?): Unit = add(o, x.toLong(), d?.toLong())
  // unboxed specialization
  public fun add(o: Int, x: ULong, d: ULong): Unit = add(o, x.toLong(), d.toLong())
  /**
   * Add a [Long] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Long, d: Long?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Long, d: Long) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add a [Float] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Float, d: Float?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Float, d: Float) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add a [Double] to a table at `o` into its vtable, with value `x` and default `d`.
   * If `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   */
  public fun add(o: Int, x: Double, d: Double?) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }
  // unboxed specialization
  public fun add(o: Int, x: Double, d: Double) {
    if (forceDefaults || x != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add an `offset` to a table at `o` into its vtable, with value `x` and default `d`.
   *
   * @param o The index into the vtable.
   * @param x An `offset` to put into the buffer, depending on how defaults are handled. If
   * `force_defaults` is `false`, compare `x` against the default value `d`. If `x` contains the
   * default value, it can be skipped.
   * @param d An `offset` default value to compare against when `force_defaults` is `false`.
   */
  public fun add(o: Int, x: Offset<*>, d: Int) {
    if (forceDefaults || x.value != d) {
      add(x)
      slot(o)
    }
  }
  public fun add(o: Int, x: VectorOffset<*>, d: Int) {
    if (forceDefaults || x.value != d) {
      add(x)
      slot(o)
    }
  }

  /**
   * Add a struct to the table. Structs are stored inline, so nothing additional is being added.
   *
   * @param vOffset The index into the vtable.
   * @param x The offset of the created struct.
   * @param d The default value is always `0`.
   */
  public fun addStruct(vOffset: Int, x: Offset<*>, d: Offset<*>?): Unit = addStruct(vOffset, x.value, d?.value)
  // unboxed specialization
  public fun addStruct(vOffset: Int, x: Offset<*>, d: Offset<*>): Unit = addStruct(vOffset, x.value, d.value)
  public fun addStruct(vOffset: Int, x: Int, d: Int?) {
    if (x != d) {
      nested(x)
      slot(vOffset)
    }
  }
  // unboxed specialization
  public fun addStruct(vOffset: Int, x: Int, d: Int) {
    if (x != d) {
      nested(x)
      slot(vOffset)
    }
  }

  /**
   * Set the current vtable at `voffset` to the current location in the buffer.
   *
   * @param vOffset The index into the vtable to store the offset relative to the end of the
   * buffer.
   */
  public fun slot(vOffset: Int) {
    vtable[vOffset] = offset()
  }

  /**
   * Finish off writing the object that is under construction.
   *
   * @return The offset to the object inside [.dataBuffer].
   * @see .startTable
   */
  public fun <T> endTable(): Offset<T> {
    if (!nested) throw AssertionError("FlatBuffers: endTable called without startTable")

    val vtable = this.vtable

    add(0)
    val vtableloc = offset()
    // Write out the current vtable.
    var i: Int = vtableInUse - 1
    // Trim trailing zeroes.
    while (i >= 0 && vtable[i] == 0) {
      i--
    }
    val trimmedSize = i + 1
    while (i >= 0) {
      // Offset relative to the start of the table.
      add((if (vtable[i] != 0) vtableloc - vtable[i] else 0).toShort())
      i--
    }

    add((vtableloc - objectStart).toShort())
    add(((trimmedSize + 2) * Short.SIZE_BYTES).toShort())

    // Search for an existing vtable that matches the current one.
    var existingVtable = 0
    i = 0
    outer_loop@ while (i < numVtables) {
      val vt1: Int = buffer.capacity - vtables[i]
      val vt2 = space
      val len: Short = buffer.getShort(vt1)
      if (len == buffer.getShort(vt2)) {
        var j: Int = Short.SIZE_BYTES
        while (j < len) {
          if (buffer.getShort(vt1 + j) != buffer.getShort(vt2 + j)) {
            i++
            continue@outer_loop
          }
          j += Short.SIZE_BYTES
        }
        existingVtable = vtables[i]
        break@outer_loop
      }
      i++
    }
    if (existingVtable != 0) {
      // Found a match:
      // Remove the current vtable.
      space = buffer.capacity - vtableloc
      // Point table to existing vtable.
      buffer.set(space, existingVtable - vtableloc)
    } else {
      // No match:
      // Add the location of the current vtable to the list of vtables.
      if (numVtables == vtables.size) vtables = vtables.copyOf(numVtables * 2)
      vtables[numVtables++] = offset()
      // Point table to current vtable.
      buffer.set(buffer.capacity - vtableloc, offset() - vtableloc)
    }
    nested = false
    return Offset(vtableloc)
  }

  /**
   * Checks that a required field has been set in a given table that has
   * just been constructed.
   *
   * @param table The offset to the start of the table from the `ByteBuffer` capacity.
   * @param field The offset to the field in the vtable.
   */
  public fun required(table: Offset<*>, field: Int, fileName: String? = null) {
    val tableStart: Int = buffer.capacity - table
    val vtableStart: Int = tableStart - buffer.getInt(tableStart)
    val ok = buffer.getShort(vtableStart + field).toInt() != 0
    // If this fails, the caller will show what field needs to be set.
    if (!ok) throw AssertionError("FlatBuffers: field ${fileName ?: field} must be set")
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`.
   *
   * @param rootTable An offset to be added to the buffer.
   * @param sizePrefix Whether to prefix the size to the buffer.
   */
  protected fun finish(rootTable: Offset<*>, sizePrefix: Boolean) {
    prep(minalign, Int.SIZE_BYTES + if (sizePrefix) Int.SIZE_BYTES else 0)
    add(rootTable)
    if (sizePrefix) {
      add(buffer.capacity - space)
    }
    buffer.writePosition = space
    finished = true
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`.
   *
   * @param rootTable An offset to be added to the buffer.
   */
  public fun finish(rootTable: Offset<*>) {
    finish(rootTable, false)
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`, with the size prefixed.
   *
   * @param rootTable An offset to be added to the buffer.
   */
  public fun finishSizePrefixed(rootTable: Offset<*>) {
    finish(rootTable, true)
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`.
   *
   * @param rootTable An offset to be added to the buffer.
   * @param fileIdentifier A FlatBuffer file identifier to be added to the buffer before
   * `root_table`.
   * @param sizePrefix Whether to prefix the size to the buffer.
   */
  protected fun finish(rootTable: Offset<*>, fileIdentifier: String, sizePrefix: Boolean) {
    val identifierSize = 4
    prep(minalign, Int.SIZE_BYTES + identifierSize + if (sizePrefix) Int.SIZE_BYTES else 0)
    if (fileIdentifier.length != identifierSize) throw AssertionError(
      "FlatBuffers: file identifier must be length " +
        identifierSize
    )
    for (i in identifierSize - 1 downTo 0) {
      add(fileIdentifier[i].code.toByte())
    }
    finish(rootTable, sizePrefix)
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`.
   *
   * @param rootTable An offset to be added to the buffer.
   * @param fileIdentifier A FlatBuffer file identifier to be added to the buffer before
   * `root_table`.
   */
  public fun finish(rootTable: Offset<*>, fileIdentifier: String) {
    finish(rootTable, fileIdentifier, false)
  }

  /**
   * Finalize a buffer, pointing to the given `root_table`, with the size prefixed.
   *
   * @param rootTable An offset to be added to the buffer.
   * @param fileIdentifier A FlatBuffer file identifier to be added to the buffer before
   * `root_table`.
   */
  public fun finishSizePrefixed(rootTable: Offset<*>, fileIdentifier: String) {
    finish(rootTable, fileIdentifier, true)
  }

  /**
   * In order to save space, fields that are set to their default value
   * don't get serialized into the buffer. Forcing defaults provides a
   * way to manually disable this optimization.
   *
   * @param forceDefaults When set to `true`, always serializes default values.
   * @return Returns `this`.
   */
  public fun forceDefaults(forceDefaults: Boolean): FlatBufferBuilder {
    this.forceDefaults = forceDefaults
    return this
  }

  /**
   * Get the ByteBuffer representing the FlatBuffer. Only call this after you've
   * called `finish()`. The actual data starts at the ByteBuffer's current position,
   * not necessarily at `0`.
   *
   * @return The [ReadBuffer] representing the FlatBuffer
   */
  public fun dataBuffer(): ReadWriteBuffer {
    finished()
    return buffer
  }

  /**
   * A utility function to copy and return the ByteBuffer data as a `byte[]`.
   *
   * @return A full copy of the [data buffer][.dataBuffer].
   */
 public fun sizedByteArray(start: Int = space, length: Int = buffer.capacity - space): ByteArray {
    finished()
    val array = ByteArray(length)
    buffer.getBytes(array, start)
    return array
  }

  /**
   * Helper function to test if a field is present in the table
   *
   * @param offset virtual table offset
   * @return true if the filed is present
  */
  public fun Table.isFieldPresent(offset: Int): Boolean = this.offset(offset) != 0
}

public fun Double.sign(): Double = when {
  this.isNaN() -> Double.NaN
  this > 0 -> 1.0
  this < 0 -> -1.0
  else -> this
}

public fun Float.sign(): Float = when {
  this.isNaN() -> Float.NaN
  this > 0 -> 1.0f
  this < 0 -> -1.0f
  else -> this
}

public fun Int.sign(): Int = when {
  this > 0 -> 1
  this < 0 -> -1
  else -> this
}
