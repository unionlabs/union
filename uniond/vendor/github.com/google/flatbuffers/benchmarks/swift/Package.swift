// swift-tools-version:5.8
/*
 * Copyright 2020 Google Inc. All rights reserved.
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

import PackageDescription

let package = Package(
  name: "benchmarks",
  platforms: [
    .macOS(.v13),
  ],
  dependencies: [
    .package(path: "../.."),
    .package(
      url: "https://github.com/ordo-one/package-benchmark",
      from: "1.12.0"),
  ],
  targets: [
    .executableTarget(
      name: "FlatbuffersBenchmarks",
      dependencies: [
        .product(name: "FlatBuffers", package: "flatbuffers"),
        .product(name: "Benchmark", package: "package-benchmark"),
      ],
      path: "Benchmarks/FlatbuffersBenchmarks",
      plugins: [
        .plugin(name: "BenchmarkPlugin", package: "package-benchmark"),
      ]),
  ])
