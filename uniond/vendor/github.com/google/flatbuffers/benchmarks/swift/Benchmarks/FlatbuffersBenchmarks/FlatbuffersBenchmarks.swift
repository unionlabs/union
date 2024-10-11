/*
 * Copyright 2023 Google Inc. All rights reserved.
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

import Benchmark
import CoreFoundation
import FlatBuffers

@usableFromInline
struct AA: NativeStruct {
  public init(a: Double, b: Double) {
    self.a = a
    self.b = b
  }
  var a: Double
  var b: Double
}

let benchmarks = {
  let ints: [Int] = Array(repeating: 42, count: 100)
  let bytes: [UInt8] = Array(repeating: 42, count: 100)
  let str10 = (0...9).map { _ -> String in "x" }.joined()
  let str100 = (0...99).map { _ -> String in "x" }.joined()
  let array: [AA] = [
    AA(a: 2.4, b: 2.4),
    AA(a: 2.4, b: 2.4),
    AA(a: 2.4, b: 2.4),
    AA(a: 2.4, b: 2.4),
    AA(a: 2.4, b: 2.4),
  ]

  let metrics: [BenchmarkMetric] = [
    .cpuTotal,
    .wallClock,
    .mallocCountTotal,
    .releaseCount,
    .peakMemoryResident,
  ]
  let maxIterations = 1_000_000
  let maxDuration: Duration = .seconds(3)
  let singleConfiguration: Benchmark.Configuration = .init(
    metrics: metrics,
    warmupIterations: 1,
    scalingFactor: .one,
    maxDuration: maxDuration,
    maxIterations: maxIterations)
  let kiloConfiguration: Benchmark.Configuration = .init(
    metrics: metrics,
    warmupIterations: 1,
    scalingFactor: .kilo,
    maxDuration: maxDuration,
    maxIterations: maxIterations)
  let megaConfiguration: Benchmark.Configuration = .init(
    metrics: metrics,
    warmupIterations: 1,
    scalingFactor: .mega,
    maxDuration: maxDuration,
    maxIterations: maxIterations)

  Benchmark.defaultConfiguration = megaConfiguration

  Benchmark("Allocating 1GB", configuration: singleConfiguration) { benchmark in
    for _ in benchmark.scaledIterations {
      blackHole(FlatBufferBuilder(initialSize: 1_024_000_000))
    }
  }

  Benchmark("Clearing 1GB", configuration: singleConfiguration) { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1_024_000_000)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      blackHole(fb.clear())
    }
  }

  Benchmark("Strings 10") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      blackHole(fb.create(string: str10))
    }
  }

  Benchmark("Strings 100") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      blackHole(fb.create(string: str100))
    }
  }

  Benchmark("Vector 1 Bytes") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      blackHole(fb.createVector(bytes: bytes))
    }
  }

  Benchmark("Vector 1 Ints") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      blackHole(fb.createVector(ints))
    }
  }

  Benchmark("Vector 100 Ints") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for i in benchmark.scaledIterations {
      blackHole(fb.createVector(ints))
    }
  }

  Benchmark("Vector 100 Bytes") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for i in benchmark.scaledIterations {
      blackHole(fb.createVector(bytes))
    }
  }

  Benchmark("Vector 100 ContiguousBytes") { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1<<20)
    benchmark.startMeasurement()
    for i in benchmark.scaledIterations {
      blackHole(fb.createVector(bytes: bytes))
    }
  }

  Benchmark(
    "FlatBufferBuilder Add",
    configuration: kiloConfiguration)
  { benchmark in
    var fb = FlatBufferBuilder(initialSize: 1024 * 1024 * 32)
    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      let off = fb.create(string: "T")
      let s = fb.startTable(with: 4)
      fb.add(element: 3.2, def: 0, at: 2)
      fb.add(element: 4.2, def: 0, at: 4)
      fb.add(element: 5.2, def: 0, at: 6)
      fb.add(offset: off, at: 8)
      blackHole(fb.endTable(at: s))
    }
  }

  Benchmark("Structs") { benchmark in
    let rawSize = ((16 * 5) * benchmark.scaledIterations.count) / 1024
    var fb = FlatBufferBuilder(initialSize: Int32(rawSize * 1600))
    var offsets: [Offset] = []

    benchmark.startMeasurement()
    for _ in benchmark.scaledIterations {
      let vector = fb.createVector(
        ofStructs: array)
      let start = fb.startTable(with: 1)
      fb.add(offset: vector, at: 4)
      offsets.append(Offset(offset: fb.endTable(at: start)))
    }

    let vector = fb.createVector(ofOffsets: offsets)
    let start = fb.startTable(with: 1)
    fb.add(offset: vector, at: 4)
    let root = Offset(offset: fb.endTable(at: start))
    fb.finish(offset: root)
  }
}
