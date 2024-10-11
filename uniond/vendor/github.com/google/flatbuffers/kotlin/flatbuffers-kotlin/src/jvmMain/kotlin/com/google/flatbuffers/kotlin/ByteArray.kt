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

@file:JvmName("JVMByteArray")
@file:Suppress("NOTHING_TO_INLINE")

package com.google.flatbuffers.kotlin

/**
 * This implementation uses Little Endian order.
 */
public actual inline fun ByteArray.getUByte(index: Int): UByte = ByteArrayOps.getUByte(this, index)
public actual inline fun ByteArray.getShort(index: Int): Short = ByteArrayOps.getShort(this, index)
public actual inline fun ByteArray.getUShort(index: Int): UShort = ByteArrayOps.getUShort(this, index)
public actual inline fun ByteArray.getInt(index: Int): Int = ByteArrayOps.getInt(this, index)
public actual inline fun ByteArray.getUInt(index: Int): UInt = ByteArrayOps.getUInt(this, index)
public actual inline fun ByteArray.getLong(index: Int): Long = ByteArrayOps.getLong(this, index)
public actual inline fun ByteArray.getULong(index: Int): ULong = ByteArrayOps.getULong(this, index)
public actual inline fun ByteArray.getFloat(index: Int): Float = ByteArrayOps.getFloat(this, index)
public actual inline fun ByteArray.getDouble(index: Int): Double = ByteArrayOps.getDouble(this, index)

public actual inline fun ByteArray.setUByte(index: Int, value: UByte): Unit = ByteArrayOps.setUByte(this, index, value)
public actual inline fun ByteArray.setShort(index: Int, value: Short): Unit = ByteArrayOps.setShort(this, index, value)
public actual inline fun ByteArray.setUShort(index: Int, value: UShort): Unit = ByteArrayOps.setUShort(this, index, value)
public actual inline fun ByteArray.setInt(index: Int, value: Int): Unit = ByteArrayOps.setInt(this, index, value)
public actual inline fun ByteArray.setUInt(index: Int, value: UInt): Unit = ByteArrayOps.setUInt(this, index, value)
public actual inline fun ByteArray.setLong(index: Int, value: Long): Unit = ByteArrayOps.setLong(this, index, value)
public actual inline fun ByteArray.setULong(index: Int, value: ULong): Unit = ByteArrayOps.setULong(this, index, value)
public actual inline fun ByteArray.setFloat(index: Int, value: Float): Unit = ByteArrayOps.setFloat(this, index, value)
public actual inline fun ByteArray.setDouble(index: Int, value: Double): Unit = ByteArrayOps.setDouble(this, index, value)
