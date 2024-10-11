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
package com.google.flatbuffers.kotlin.benchmark

import com.google.flatbuffers.kotlin.ArrayReadWriteBuffer
import com.google.flatbuffers.kotlin.Key
import com.google.flatbuffers.kotlin.Utf8
import kotlinx.benchmark.Blackhole
import org.openjdk.jmh.annotations.Benchmark
import org.openjdk.jmh.annotations.BenchmarkMode
import org.openjdk.jmh.annotations.Measurement
import org.openjdk.jmh.annotations.Mode
import org.openjdk.jmh.annotations.OutputTimeUnit
import org.openjdk.jmh.annotations.Scope
import org.openjdk.jmh.annotations.Setup
import org.openjdk.jmh.annotations.State
import java.nio.ByteBuffer
import java.util.concurrent.TimeUnit
import kotlin.random.Random

@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Measurement(iterations = 100, time = 1, timeUnit = TimeUnit.MICROSECONDS)
open class UTF8Benchmark {

  private val sampleSize = 5000
  private val stringSize = 25
  private var sampleSmallUtf8 = (0..sampleSize).map { populateUTF8(stringSize) }.toList()
  private var sampleSmallUtf8Decoded = sampleSmallUtf8.map { it.encodeToByteArray() }.toList()
  private var sampleSmallAscii = (0..sampleSize).map { populateAscii(stringSize) }.toList()
  private var sampleSmallAsciiDecoded = sampleSmallAscii.map { it.encodeToByteArray() }.toList()

  @Setup
  fun setUp() {
  }

  @Benchmark
  fun encodeUtf8KotlinStandard(blackhole: Blackhole) {
    for (i in sampleSmallUtf8) {
      blackhole.consume(i.encodeToByteArray())
    }
  }
  @Benchmark
  fun encodeUtf8KotlinFlatbuffers(blackhole: Blackhole) {
    for (i in sampleSmallUtf8) {
      val byteArray = ByteArray((i.length * 4))
      blackhole.consume(Utf8.encodeUtf8Array(i, byteArray, 0, byteArray.size))
    }
  }
  @Benchmark
  fun encodeUtf8JavaFlatbuffers(blackhole: Blackhole) {
    val javaUtf8 = com.google.flatbuffers.Utf8.getDefault()
    for (i in sampleSmallUtf8) {
      val byteBuffer = ByteBuffer.wrap(ByteArray(i.length * 4))
      blackhole.consume(javaUtf8.encodeUtf8(i, byteBuffer))
    }
  }

  @Benchmark
  fun decodeUtf8KotlinStandard(blackhole: Blackhole) {
    for (ary in sampleSmallUtf8Decoded) {
      blackhole.consume(ary.decodeToString())
    }
  }

  @Benchmark
  fun decodeUtf8KotlinFlatbuffers(blackhole: Blackhole) {
    for (ary in sampleSmallUtf8Decoded) {
      blackhole.consume(Utf8.decodeUtf8Array(ary, 0, ary.size))
    }
  }

  @Benchmark
  fun decodeUtf8JavaFlatbuffers(blackhole: Blackhole) {
    val javaUtf8 = com.google.flatbuffers.Utf8.getDefault()
    for (ary in sampleSmallUtf8Decoded) {
      val byteBuffer = ByteBuffer.wrap(ary)
      blackhole.consume(javaUtf8.decodeUtf8(byteBuffer, 0, ary.size))
    }
  }

  // ASCII TESTS

  @Benchmark
  fun encodeAsciiKotlinStandard(blackhole: Blackhole) {
    for (i in sampleSmallAscii) {
      blackhole.consume(i.encodeToByteArray())
    }
  }
  @Benchmark
  fun encodeAsciiKotlinFlatbuffers(blackhole: Blackhole) {
    for (i in sampleSmallAscii) {
      val byteArray = ByteArray(i.length) // Utf8.encodedLength(i))
      blackhole.consume(Utf8.encodeUtf8Array(i, byteArray, 0, byteArray.size))
    }
  }
  @Benchmark
  fun encodeAsciiJavaFlatbuffers(blackhole: Blackhole) {
    val javaUtf8 = com.google.flatbuffers.Utf8.getDefault()
    for (i in sampleSmallAscii) {
      val byteBuffer = ByteBuffer.wrap(ByteArray(i.length))
      blackhole.consume(javaUtf8.encodeUtf8(i, byteBuffer))
    }
  }

  @Benchmark
  fun decodeAsciiKotlinStandard(blackhole: Blackhole) {

    for (ary in sampleSmallAsciiDecoded) {
      String(ary)
      blackhole.consume(ary.decodeToString())
    }
  }

  @Benchmark
  fun decodeAsciiKotlinFlatbuffers(blackhole: Blackhole) {
    for (ary in sampleSmallAsciiDecoded) {
      blackhole.consume(Utf8.decodeUtf8Array(ary, 0, ary.size))
    }
  }

  @Benchmark
  fun decodeAsciiJavaFlatbuffers(blackhole: Blackhole) {
    val javaUtf8 = com.google.flatbuffers.Utf8.getDefault()
    for (ary in sampleSmallAsciiDecoded) {
      val byteBuffer = ByteBuffer.wrap(ary)
      blackhole.consume(javaUtf8.decodeUtf8(byteBuffer, 0, ary.size))
    }
  }

  @Benchmark
  fun readAllCharsString(blackhole: Blackhole) {
    for (ary in sampleSmallAsciiDecoded) {
      val key = Utf8.decodeUtf8Array(ary, 0, ary.size)
      for (i in key.indices) {
        blackhole.consume(key[i])
      }
    }
  }

  @Benchmark
  fun readAllCharsCharSequence(blackhole: Blackhole) {
    for (ary in sampleSmallAsciiDecoded) {
      val key = Key(ArrayReadWriteBuffer(ary), 0, ary.size)
      for (i in 0 until key.sizeInChars) {
        blackhole.consume(key[i])
      }
    }
  }

  fun populateAscii(size: Int): String {
    val data = ByteArray(size)
    for (i in data.indices) {
      data[i] = Random.nextInt(0, 127).toByte()
    }

    return String(data, 0, data.size)
  }

  // generate a string having at least length N
  // can exceed by up to 3 chars, returns the actual length
  fun populateUTF8(size: Int): String {
    val data = ByteArray(size + 3)
    var i = 0
    while (i < size) {
      val w = Random.nextInt() and 0xFF
      when {
        w < 0x80 -> data[i++] = 0x20; // w;
        w < 0xE0 -> {
          data[i++] = (0xC2 + Random.nextInt() % (0xDF - 0xC2 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w == 0xE0 -> {
          data[i++] = w.toByte()
          data[i++] = (0xA0 + Random.nextInt() % (0xBF - 0xA0 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w <= 0xEC -> {
          data[i++] = w.toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w == 0xED -> {
          data[i++] = w.toByte()
          data[i++] = (0x80 + Random.nextInt() % (0x9F - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w <= 0xEF -> {
          data[i++] = w.toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w < 0xF0 -> {
          data[i++] = (0xF1 + Random.nextInt() % (0xF3 - 0xF1 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w == 0xF0 -> {
          data[i++] = w.toByte()
          data[i++] = (0x90 + Random.nextInt() % (0xBF - 0x90 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w <= 0xF3 -> {
          data[i++] = (0xF1 + Random.nextInt() % (0xF3 - 0xF1 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
        w == 0xF4 -> {
          data[i++] = w.toByte()
          data[i++] = (0x80 + Random.nextInt() % (0x8F - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
          data[i++] = (0x80 + Random.nextInt() % (0xBF - 0x80 + 1)).toByte()
        }
      }
    }
    return String(data, 0, i)
  }
}
