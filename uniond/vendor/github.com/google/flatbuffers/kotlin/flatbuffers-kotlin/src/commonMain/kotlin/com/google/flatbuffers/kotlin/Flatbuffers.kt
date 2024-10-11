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

import kotlin.jvm.JvmInline
import kotlin.math.min

// For now a typealias to guarantee type safety.
public typealias UnionOffset = Offset<Any>
public typealias UnionOffsetArray = OffsetArray<Any>
public typealias StringOffsetArray = OffsetArray<String>

public inline fun UnionOffsetArray(size: Int, crossinline call: (Int) -> Offset<Any>): UnionOffsetArray =
  UnionOffsetArray(IntArray(size) { call(it).value })
public inline fun StringOffsetArray(size: Int, crossinline call: (Int) -> Offset<String>): StringOffsetArray =
  StringOffsetArray(IntArray(size) { call(it).value })
/**
 * Represents a "pointer" to a pointer types (table, string, struct) within the buffer
 */
@JvmInline
public value class Offset<T>(public val value: Int) {
  public fun toUnion(): UnionOffset = UnionOffset(value)
}

/**
 * Represents an array of offsets. Used to avoid boxing
 * offset types.
 */
@JvmInline
public value class OffsetArray<T>(public val value: IntArray) {
  public inline val size: Int
    get() = value.size
  public inline operator fun get(index: Int): Offset<T> = Offset(value[index])
}

public inline fun <T> OffsetArray(size: Int, crossinline call: (Int) -> Offset<T>): OffsetArray<T> {
  return OffsetArray(IntArray(size) { call(it).value })
}


/**
 * Represents a "pointer" to a vector type with elements T
 */
@JvmInline
public value class VectorOffset<T>(public val value: Int)

public fun <T> Int.toOffset(): Offset<T> = Offset(this)

public operator fun <T> Offset<T>.minus(other: Int): Offset<T> = Offset(this.value - other)

public operator fun <T> Int.minus(other: Offset<T>): Int {
  return this - other.value
}
/**
 * All tables in the generated code derive from this class, and add their own accessors.
 */
public open class Table {

  /** Used to hold the position of the `bb` buffer.  */
  public var bufferPos: Int = 0

  /** The underlying ReadWriteBuffer to hold the data of the Table.  */
  public var bb: ReadWriteBuffer = emptyBuffer

  /** Used to hold the vtable position.  */
  public var vtableStart: Int = 0

  /** Used to hold the vtable size.  */
  public var vtableSize: Int = 0

  protected inline fun <reified T> Int.invalid(default: T, crossinline valid: (Int) -> T) : T =
    if (this != 0) valid(this) else default

  protected inline fun <reified T> lookupField(i: Int, default: T, crossinline found: (Int) -> T) : T =
    offset(i).invalid(default) { found(it) }

  /**
   * Look up a field in the vtable.
   *
   * @param vtableOffset An `int` offset to the vtable in the Table's ReadWriteBuffer.
   * @return Returns an offset into the object, or `0` if the field is not present.
   */
  public fun offset(vtableOffset: Int): Int =
    if (vtableOffset < vtableSize) bb.getShort(vtableStart + vtableOffset).toInt() else 0

  /**
   * Retrieve a relative offset.
   *
   * @param offset An `int` index into the Table's ReadWriteBuffer containing the relative offset.
   * @return Returns the relative offset stored at `offset`.
   */
  public fun indirect(offset: Int): Int = offset + bb.getInt(offset)

  /**
   * Create a Java `String` from UTF-8 data stored inside the FlatBuffer.
   *
   * This allocates a new string and converts to wide chars upon each access,
   * which is not very efficient. Instead, each FlatBuffer string also comes with an
   * accessor based on __vector_as_ReadWriteBuffer below, which is much more efficient,
   * assuming your Java program can handle UTF-8 data directly.
   *
   * @param offset An `int` index into the Table's ReadWriteBuffer.
   * @return Returns a `String` from the data stored inside the FlatBuffer at `offset`.
   */
  public fun string(offset: Int): String = string(offset, bb)

  /**
   * Get the length of a vector.
   *
   * @param offset An `int` index into the Table's ReadWriteBuffer.
   * @return Returns the length of the vector whose offset is stored at `offset`.
   */
  public fun vectorLength(offset: Int): Int {
    var newOffset = offset
    newOffset += bufferPos
    newOffset += bb.getInt(newOffset)
    return bb.getInt(newOffset)
  }

  /**
   * Get the start data of a vector.
   *
   * @param offset An `int` index into the Table's ReadWriteBuffer.
   * @return Returns the start of the vector data whose offset is stored at `offset`.
   */
  public fun vector(offset: Int): Int {
    var newOffset = offset
    newOffset += bufferPos
    return newOffset + bb.getInt(newOffset) + Int.SIZE_BYTES // data starts after the length
  }
    /**
     * Initialize vector as a ReadWriteBuffer.
     *
     * This is more efficient than using duplicate, since it doesn't copy the data
     * nor allocates a new [ReadBuffer], creating no garbage to be collected.
     *
     * @param buffer The [ReadBuffer] for the array
     * @param vectorOffset The position of the vector in the byte buffer
     * @param elemSize The size of each element in the array
     * @return The [ReadBuffer] for the array
     */
    public fun vectorAsBuffer(buffer: ReadWriteBuffer, vectorOffset: Int, elemSize: Int): ReadBuffer {
        val o = offset(vectorOffset)
        if (o == 0) return emptyBuffer
        val vectorStart = vector(o)
        return buffer.slice(vectorStart, vectorLength(o) * elemSize)
    }

  /**
   * Initialize any Table-derived type to point to the union at the given `offset`.
   *
   * @param t A `Table`-derived type that should point to the union at `offset`.
   * @param offset An `int` index into the Table's ReadWriteBuffer.
   * @return Returns the Table that points to the union at `offset`.
   */
  public fun union(t: Table, offset: Int): Table = union(t, offset, bb)

  /**
   * Sort tables by the key.
   *
   * @param offsets An 'int' indexes of the tables into the bb.
   * @param bb A `ReadWriteBuffer` to get the tables.
   */
  public fun <T> sortTables(offsets: Array<Offset<T>>, bb: ReadWriteBuffer) {
    val off = offsets.sortedWith { o1, o2 -> keysCompare(o1, o2, bb) }
    for (i in offsets.indices) offsets[i] = off[i]
  }

  /**
   * Compare two tables by the key.
   *
   * @param o1 An 'Integer' index of the first key into the bb.
   * @param o2 An 'Integer' index of the second key into the bb.
   * @param buffer A `ReadWriteBuffer` to get the keys.
   */
  public open fun keysCompare(o1: Offset<*>, o2: Offset<*>, buffer: ReadWriteBuffer): Int = 0

  /**
   * Re-init the internal state with an external buffer `ReadWriteBuffer` and an offset within.
   *
   * This method exists primarily to allow recycling Table instances without risking memory leaks
   * due to `ReadWriteBuffer` references.
   */
  public inline fun <reified T: Table> reset(i: Int, reuseBuffer: ReadWriteBuffer): T {
    bb = reuseBuffer
    if (bb != emptyBuffer) {
      bufferPos = i
      vtableStart = bufferPos - bb.getInt(bufferPos)
      vtableSize = bb.getShort(vtableStart).toInt()
    } else {
      bufferPos = 0
      vtableStart = 0
      vtableSize = 0
    }
    return this as T
  }

  /**
   * Resets the internal state with a null `ReadWriteBuffer` and a zero position.
   *
   * This method exists primarily to allow recycling Table instances without risking memory leaks
   * due to `ReadWriteBuffer` references. The instance will be unusable until it is assigned
   * again to a `ReadWriteBuffer`.
   */
  public inline fun <reified T: Table> reset(): T = reset(0, emptyBuffer)

  public companion object {

    public fun offset(vtableOffset: Int, offset: Offset<*>, bb: ReadWriteBuffer): Int {
      val vtable: Int = bb.capacity - offset.value
      return bb.getShort(vtable + vtableOffset - bb.getInt(vtable)) + vtable
    }

    /**
     * Retrieve a relative offset.
     *
     * @param offset An `int` index into a ReadWriteBuffer containing the relative offset.
     * @param bb from which the relative offset will be retrieved.
     * @return Returns the relative offset stored at `offset`.
     */
    public fun indirect(offset: Int, bb: ReadWriteBuffer): Int {
      return offset + bb.getInt(offset)
    }

    /**
     * Create a Java `String` from UTF-8 data stored inside the FlatBuffer.
     *
     * This allocates a new string and converts to wide chars upon each access,
     * which is not very efficient. Instead, each FlatBuffer string also comes with an
     * accessor based on __vector_as_ReadWriteBuffer below, which is much more efficient,
     * assuming your Java program can handle UTF-8 data directly.
     *
     * @param offset An `int` index into the Table's ReadWriteBuffer.
     * @param bb Table ReadWriteBuffer used to read a string at given offset.
     * @return Returns a `String` from the data stored inside the FlatBuffer at `offset`.
     */
    public fun string(offset: Int, bb: ReadWriteBuffer): String {
      var newOffset = offset
      newOffset += bb.getInt(newOffset)
      val length: Int = bb.getInt(newOffset)
      return bb.getString(newOffset + Int.SIZE_BYTES, length)
    }

    /**
     * Initialize any Table-derived type to point to the union at the given `offset`.
     *
     * @param t A `Table`-derived type that should point to the union at `offset`.
     * @param offset An `int` index into the Table's ReadWriteBuffer.
     * @param bb Table ReadWriteBuffer used to initialize the object Table-derived type.
     * @return Returns the Table that points to the union at `offset`.
     */
    public fun union(t: Table, offset: Int, bb: ReadWriteBuffer): Table =
      t.reset(indirect(offset, bb), bb)

    /**
     * Check if a [ReadWriteBuffer] contains a file identifier.
     *
     * @param bb A `ReadWriteBuffer` to check if it contains the identifier
     * `ident`.
     * @param ident A `String` identifier of the FlatBuffer file.
     * @return True if the buffer contains the file identifier
     */
    public fun hasIdentifier(bb: ReadWriteBuffer?, ident: String): Boolean {
      val identifierLength = 4
      if (ident.length != identifierLength)
        throw AssertionError("FlatBuffers: file identifier must be length $identifierLength")
      for (i in 0 until identifierLength) {
        if (ident[i].code.toByte() != bb!![bb.limit + Int.SIZE_BYTES + i]) return false
      }
      return true
    }

    /**
     * Compare two strings in the buffer.
     *
     * @param offsetA An 'int' index of the first string into the bb.
     * @param offsetB An 'int' index of the second string into the bb.
     * @param bb A `ReadWriteBuffer` to get the strings.
     */
    public fun compareStrings(offsetA: Int, offsetB: Int, bb: ReadWriteBuffer): Int {
      var offset1 = offsetA
      var offset2 = offsetB
      offset1 += bb.getInt(offset1)
      offset2 += bb.getInt(offset2)
      val len1: Int = bb.getInt(offset1)
      val len2: Int = bb.getInt(offset2)
      val startPos1: Int = offset1 + Int.SIZE_BYTES
      val startPos2: Int = offset2 + Int.SIZE_BYTES
      val len: Int = min(len1, len2)
      for (i in 0 until len) {
        if (bb[i + startPos1] != bb[i + startPos2]) {
          return bb[i + startPos1] - bb[i + startPos2]
        }
      }
      return len1 - len2
    }

    /**
     * Compare string from the buffer with the 'String' object.
     *
     * @param offset An 'int' index of the first string into the bb.
     * @param key Second string as a byte array.
     * @param bb A `ReadWriteBuffer` to get the first string.
     */
    public fun compareStrings(offset: Int, key: ByteArray, bb: ReadWriteBuffer): Int {
      var offset1 = offset
      offset1 += bb.getInt(offset1)
      val len1: Int = bb.getInt(offset1)
      val len2 = key.size
      val startPos: Int = offset1 + Int.SIZE_BYTES
      val len: Int = min(len1, len2)
      for (i in 0 until len) {
        if (bb[i + startPos] != key[i]) return bb[i + startPos] - key[i]
      }
      return len1 - len2
    }
  }
}

/**
 * All structs in the generated code derive from this class, and add their own accessors.
 */
public open class Struct {
  /** Used to hold the position of the `bb` buffer.  */
  protected var bufferPos: Int = 0

  /** The underlying ByteBuffer to hold the data of the Struct.  */
  protected var bb: ReadWriteBuffer = emptyBuffer

  /**
   * Re-init the internal state with an external buffer `ByteBuffer` and an offset within.
   *
   * This method exists primarily to allow recycling Table instances without risking memory leaks
   * due to `ByteBuffer` references.
   */
  protected inline fun <reified T: Struct> reset(i: Int, reuseBuffer: ReadWriteBuffer): T {
    bb = reuseBuffer
    bufferPos = if (bb != emptyBuffer) i else 0
    return this as T
  }

  /**
   * Resets internal state with a null `ByteBuffer` and a zero position.
   *
   * This method exists primarily to allow recycling Struct instances without risking memory leaks
   * due to `ByteBuffer` references. The instance will be unusable until it is assigned
   * again to a `ByteBuffer`.
   */
  private inline fun <reified T: Struct> reset(): T = reset(0, emptyBuffer)
}

public inline val <T> T.value: T get() = this

public const val VERSION_2_0_8: Int = 1
