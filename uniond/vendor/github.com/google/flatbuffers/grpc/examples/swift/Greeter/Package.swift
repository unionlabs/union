// swift-tools-version:5.1
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
  name: "Greeter",
  platforms: [
    .iOS(.v11),
    .macOS(.v10_14),
  ],
  dependencies: [
    .package(path: "../../../../swift"),
    .package(url: "https://github.com/grpc/grpc-swift.git", from: "1.0.0"),
  ],
  targets: [
    // Targets are the basic building blocks of a package. A target can define a module or a test suite.
    // Targets can depend on other targets in this package, and on products in packages which this package depends on.
    .target(
      name: "Model",
      dependencies: [
        "GRPC",
        "FlatBuffers",
      ],
      path: "Sources/Model"),

    // Client for the Greeter example
    .target(
      name: "Client",
      dependencies: [
        "GRPC",
        "Model",
      ],
      path: "Sources/client"),

    // Server for the Greeter example
    .target(
      name: "Server",
      dependencies: [
        "GRPC",
        "Model",
      ],
      path: "Sources/server"),
  ])
