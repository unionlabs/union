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
@file:OptIn(ExperimentalUnsignedTypes::class)

package com.google.flatbuffers.kotlin.benchmark
import com.google.flatbuffers.ArrayReadWriteBuf
import com.google.flatbuffers.FlexBuffers
import com.google.flatbuffers.FlexBuffersBuilder.BUILDER_FLAG_SHARE_ALL
import com.google.flatbuffers.kotlin.FlexBuffersBuilder
import com.google.flatbuffers.kotlin.getRoot
import kotlinx.benchmark.Blackhole
import org.openjdk.jmh.annotations.Benchmark
import org.openjdk.jmh.annotations.BenchmarkMode
import org.openjdk.jmh.annotations.Measurement
import org.openjdk.jmh.annotations.Mode
import org.openjdk.jmh.annotations.OutputTimeUnit
import org.openjdk.jmh.annotations.Scope
import org.openjdk.jmh.annotations.Setup
import org.openjdk.jmh.annotations.State
import java.util.concurrent.TimeUnit

@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.NANOSECONDS)
@Measurement(iterations = 20, time = 1, timeUnit = TimeUnit.NANOSECONDS)
open class FlexBuffersBenchmark {

  var initialCapacity = 1024
  var value: Double = 0.0
  val stringKey = Array(500) { "Ḧ̵̘́ȩ̵̐myFairlyBigKey$it" }
  val stringValue = Array(500) { "Ḧ̵̘́ȩ̵̐myFairlyBigValue$it" }
  val bigIntArray = IntArray(5000) { it }

  @Setup
  fun setUp() {
    value = 3.0
  }

  @Benchmark
  open fun mapKotlin(blackhole: Blackhole) {
    val kBuilder = FlexBuffersBuilder(initialCapacity, FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    kBuilder.putMap {
      this["hello"] = "world"
      this["int"] = 10
      this["float"] = 12.3
      this["intarray"] = bigIntArray
      this.putMap("myMap") {
        this["cool"] = "beans"
      }
    }
    val ref = getRoot(kBuilder.finish())
    val map = ref.toMap()
    blackhole.consume(map.size)
    blackhole.consume(map["hello"].toString())
    blackhole.consume(map["int"].toInt())
    blackhole.consume(map["float"].toDouble())
    blackhole.consume(map["intarray"].toIntArray())
    blackhole.consume(ref["myMap"]["cool"].toString())
    blackhole.consume(ref["invalid_key"].isNull)
  }

  @Benchmark
  open fun mapJava(blackhole: Blackhole) {
    val jBuilder = com.google.flatbuffers.FlexBuffersBuilder(ArrayReadWriteBuf(initialCapacity), BUILDER_FLAG_SHARE_ALL)
    val startMap = jBuilder.startMap()
    jBuilder.putString("hello", "world")
    jBuilder.putInt("int", 10)
    jBuilder.putFloat("float", 12.3)

    val startVec = jBuilder.startVector()
    bigIntArray.forEach { jBuilder.putInt(it) }
    jBuilder.endVector("intarray", startVec, true, false)

    val startInnerMap = jBuilder.startMap()
    jBuilder.putString("cool", "beans")
    jBuilder.endMap("myMap", startInnerMap)

    jBuilder.endMap(null, startMap)
    val ref = FlexBuffers.getRoot(jBuilder.finish())
    val map = ref.asMap()
    blackhole.consume(map.size())
    blackhole.consume(map.get("hello").toString())
    blackhole.consume(map.get("int").asInt())
    blackhole.consume(map.get("float").asFloat())
    val vec = map.get("intarray").asVector()
    blackhole.consume(IntArray(vec.size()) { vec.get(it).asInt() })

    blackhole.consume(ref.asMap()["myMap"].asMap()["cool"].toString())
    blackhole.consume(ref.asMap()["invalid_key"].isNull)
  }

  @Benchmark
  open fun intArrayKotlin(blackhole: Blackhole) {
    val kBuilder = FlexBuffersBuilder(initialCapacity, FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    kBuilder.put(bigIntArray)
    val root = getRoot(kBuilder.finish())
    blackhole.consume(root.toIntArray())
  }

  @Benchmark
  open fun intArrayJava(blackhole: Blackhole) {
    val jBuilder = com.google.flatbuffers.FlexBuffersBuilder(ArrayReadWriteBuf(initialCapacity), BUILDER_FLAG_SHARE_ALL)
    val v = jBuilder.startVector()
    bigIntArray.forEach { jBuilder.putInt(it) }
    jBuilder.endVector(null, v, true, false)
    jBuilder.finish()
    val root = FlexBuffers.getRoot(jBuilder.buffer)
    val vec = root.asVector()
    blackhole.consume(
      IntArray(vec.size()) {
        vec[it].asInt()
      }
    )
  }

  @Benchmark
  open fun stringArrayKotlin(blackhole: Blackhole) {
    val kBuilder = FlexBuffersBuilder(initialCapacity, FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    kBuilder.putVector { stringValue.forEach { kBuilder.put(it) } }
    kBuilder.finish()
    val root = getRoot(kBuilder.buffer)
    val vec = root.toVector()
    blackhole.consume(Array(vec.size) { vec[it].toString() })
  }

  @Benchmark
  open fun stringArrayJava(blackhole: Blackhole) {
    val jBuilder = com.google.flatbuffers.FlexBuffersBuilder(ArrayReadWriteBuf(initialCapacity), BUILDER_FLAG_SHARE_ALL)
    val v = jBuilder.startVector()
    stringValue.forEach { jBuilder.putString(it) }
    jBuilder.endVector(null, v, false, false)
    jBuilder.finish()
    val root = FlexBuffers.getRoot(jBuilder.buffer)
    val vec = root.asVector()
    blackhole.consume(Array(vec.size()) { vec[it].toString() })
  }

  @Benchmark
  open fun stringMapKotlin(blackhole: Blackhole) {
    val kBuilder = FlexBuffersBuilder(initialCapacity, FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    val pos = kBuilder.startMap()
    for (i in stringKey.indices) {
      kBuilder[stringKey[i]] = stringValue[i]
    }
    kBuilder.endMap(pos)
    val ref = getRoot(kBuilder.finish())
    val map = ref.toMap()
    val keys = map.keys

    for (key in keys) {
      blackhole.consume(map[key.toString()].toString())
    }
  }

  @Benchmark
  open fun stringMapBytIndexKotlin(blackhole: Blackhole) {
    val kBuilder = FlexBuffersBuilder(initialCapacity, FlexBuffersBuilder.SHARE_KEYS_AND_STRINGS)
    val pos = kBuilder.startMap()
    for (i in stringKey.indices) {
      kBuilder[stringKey[i]] = stringValue[i]
    }
    kBuilder.endMap(pos)
    val ref = getRoot(kBuilder.finish())
    val map = ref.toMap()
    for (index in 0 until map.size) {
      blackhole.consume(map[index].toString())
    }
  }

  @Benchmark
  open fun stringMapJava(blackhole: Blackhole) {
    val jBuilder = com.google.flatbuffers.FlexBuffersBuilder(ArrayReadWriteBuf(initialCapacity), BUILDER_FLAG_SHARE_ALL)
    val v = jBuilder.startMap()
    for (i in stringKey.indices) {
      jBuilder.putString(stringKey[i], stringValue[i])
    }
    jBuilder.endMap(null, v)
    val ref = FlexBuffers.getRoot(jBuilder.finish())
    val map = ref.asMap()
    val keyVec = map.keys()
    for (i in 0 until keyVec.size()) {
      val s = keyVec[i].toString()
      blackhole.consume(map[s].toString())
    }
  }
}
