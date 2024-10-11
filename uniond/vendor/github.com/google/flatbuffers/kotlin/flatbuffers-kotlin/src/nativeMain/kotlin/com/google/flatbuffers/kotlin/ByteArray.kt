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

/**
 * This implementation assumes that of native macOSX64 the byte order of the implementation is Little Endian.
 */

public actual inline fun ByteArray.getUByte(index: Int): UByte = getUByteAt(index)
public actual inline fun ByteArray.getShort(index: Int): Short = getShortAt(index)
public actual inline fun ByteArray.getUShort(index: Int): UShort = getUShortAt(index)
public actual inline fun ByteArray.getInt(index: Int): Int = getIntAt(index)
public actual inline fun ByteArray.getUInt(index: Int): UInt = getUIntAt(index)
public actual inline fun ByteArray.getLong(index: Int): Long = getLongAt(index)
public actual inline fun ByteArray.getULong(index: Int): ULong = getULongAt(index)

public actual inline fun ByteArray.setUByte(index: Int, value: UByte): Unit = setUByteAt(index, value)
public actual inline fun ByteArray.setShort(index: Int, value: Short): Unit = setShortAt(index, value)
public actual inline fun ByteArray.setUShort(index: Int, value: UShort): Unit = setUShortAt(index, value)
public actual inline fun ByteArray.setInt(index: Int, value: Int): Unit = setIntAt(index, value)
public actual inline fun ByteArray.setUInt(index: Int, value: UInt): Unit = setUIntAt(index, value)
public actual inline fun ByteArray.setLong(index: Int, value: Long): Unit = setLongAt(index, value)
public actual inline fun ByteArray.setULong(index: Int, value: ULong): Unit = setULongAt(index, value)
public actual inline fun ByteArray.setFloat(index: Int, value: Float): Unit = setFloatAt(index, value)
public actual inline fun ByteArray.setDouble(index: Int, value: Double): Unit = setDoubleAt(index, value)
public actual inline fun ByteArray.getFloat(index: Int): Float = Float.fromBits(getIntAt(index))
public actual inline fun ByteArray.getDouble(index: Int): Double = Double.fromBits(getLongAt(index))
