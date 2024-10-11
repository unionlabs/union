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

import com.google.flatbuffers.kotlin.ArrayReadBuffer
import com.google.flatbuffers.kotlin.JSONParser
import com.google.flatbuffers.kotlin.Reference
import com.google.flatbuffers.kotlin.toJson
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import kotlinx.benchmark.Blackhole
import okio.Buffer
import org.openjdk.jmh.annotations.Benchmark
import org.openjdk.jmh.annotations.BenchmarkMode
import org.openjdk.jmh.annotations.Measurement
import org.openjdk.jmh.annotations.Mode
import org.openjdk.jmh.annotations.OutputTimeUnit
import org.openjdk.jmh.annotations.Scope
import org.openjdk.jmh.annotations.State
import java.io.ByteArrayInputStream
import java.io.InputStreamReader
import java.util.concurrent.TimeUnit

@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Measurement(iterations = 100, time = 1, timeUnit = TimeUnit.MICROSECONDS)
open class JsonBenchmark {

  final val moshi = Moshi.Builder()
    .addLast(KotlinJsonAdapterFactory())
    .build()
  final val moshiAdapter = moshi.adapter(Map::class.java)

  final val gson = Gson()
  final val gsonParser = JsonParser()

  val fbParser = JSONParser()

  final val classLoader = this.javaClass.classLoader
  final val twitterData = classLoader.getResourceAsStream("twitter.json")!!.readBytes()
  final val canadaData = classLoader.getResourceAsStream("canada.json")!!.readBytes()
  final val citmData = classLoader.getResourceAsStream("citm_catalog.json")!!.readBytes()

  val fbCitmRef = JSONParser().parse(ArrayReadBuffer(citmData))
  val moshiCitmRef = moshi.adapter(Map::class.java).fromJson(citmData.decodeToString())
  val gsonCitmRef = gsonParser.parse(citmData.decodeToString())

  fun readFlexBuffers(data: ByteArray): Reference = fbParser.parse(ArrayReadBuffer(data))

  fun readMoshi(data: ByteArray): Map<*, *>? {
    val buffer = Buffer().write(data)
    return moshiAdapter.fromJson(buffer)
  }

  fun readGson(data: ByteArray): JsonObject {
    val parser = JsonParser()
    val jsonReader = InputStreamReader(ByteArrayInputStream(data))
    return parser.parse(jsonReader).asJsonObject
  }

  // TWITTER
  @Benchmark
  open fun readTwitterFlexBuffers(hole: Blackhole? = null) = hole?.consume(readFlexBuffers(twitterData))
  @Benchmark
  open fun readTwitterMoshi(hole: Blackhole?) = hole?.consume(readMoshi(twitterData))
  @Benchmark
  open fun readTwitterGson(hole: Blackhole?) = hole?.consume(readGson(twitterData))

  @Benchmark
  open fun roundTripTwitterFlexBuffers(hole: Blackhole? = null) = hole?.consume(readFlexBuffers(twitterData).toJson())
  @Benchmark
  open fun roundTripTwitterMoshi(hole: Blackhole?) = hole?.consume(moshiAdapter.toJson(readMoshi(twitterData)))
  @Benchmark
  open fun roundTripTwitterGson(hole: Blackhole?) = hole?.consume(gson.toJson(readGson(twitterData)))

  // CITM
  @Benchmark
  open fun readCITMFlexBuffers(hole: Blackhole? = null) = hole?.consume(readFlexBuffers(citmData))
  @Benchmark
  open fun readCITMMoshi(hole: Blackhole?) = hole?.consume(moshiAdapter.toJson(readMoshi(citmData)))
  @Benchmark
  open fun readCITMGson(hole: Blackhole?) = hole?.consume(gson.toJson(readGson(citmData)))

  @Benchmark
  open fun roundTripCITMFlexBuffers(hole: Blackhole? = null) = hole?.consume(readFlexBuffers(citmData).toJson())
  @Benchmark
  open fun roundTripCITMMoshi(hole: Blackhole?) = hole?.consume(moshiAdapter.toJson(readMoshi(citmData)))
  @Benchmark
  open fun roundTripCITMGson(hole: Blackhole?) = hole?.consume(gson.toJson(readGson(citmData)))

  @Benchmark
  open fun writeCITMFlexBuffers(hole: Blackhole? = null) = hole?.consume(fbCitmRef.toJson())
  @Benchmark
  open fun writeCITMMoshi(hole: Blackhole?) = hole?.consume(moshiAdapter.toJson(moshiCitmRef))
  @Benchmark
  open fun writeCITMGson(hole: Blackhole?) = hole?.consume(gson.toJson(gsonCitmRef))

  // CANADA
  @Benchmark
  open fun readCanadaFlexBuffers(hole: Blackhole? = null) = hole?.consume(readFlexBuffers(canadaData))
  @Benchmark
  open fun readCanadaMoshi(hole: Blackhole?) = hole?.consume(readMoshi(canadaData))
  @Benchmark
  open fun readCanadaGson(hole: Blackhole?) = hole?.consume(readGson(canadaData))
}
