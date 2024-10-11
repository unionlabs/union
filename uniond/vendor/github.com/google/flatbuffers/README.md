![logo](http://google.github.io/flatbuffers/fpl_logo_small.png) FlatBuffers
===========

![Build status](https://github.com/google/flatbuffers/actions/workflows/build.yml/badge.svg?branch=master)
[![BuildKite status](https://badge.buildkite.com/7979d93bc6279aa539971f271253c65d5e8fe2fe43c90bbb25.svg)](https://buildkite.com/bazel/flatbuffers)
[![Fuzzing Status](https://oss-fuzz-build-logs.storage.googleapis.com/badges/flatbuffers.svg)](https://bugs.chromium.org/p/oss-fuzz/issues/list?sort=-opened&can=1&q=proj:flatbuffers)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/google/flatbuffers/badge)](https://api.securityscorecards.dev/projects/github.com/google/flatbuffers)
[![Join the chat at https://gitter.im/google/flatbuffers](https://badges.gitter.im/google/flatbuffers.svg)](https://gitter.im/google/flatbuffers?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Discord Chat](https://img.shields.io/discord/656202785926152206.svg)](https:///discord.gg/6qgKs3R)
[![Twitter Follow](https://img.shields.io/twitter/follow/wvo.svg?style=social)](https://twitter.com/wvo)
[![Twitter Follow](https://img.shields.io/twitter/follow/dbaileychess.svg?style=social)](https://twitter.com/dbaileychess)


**FlatBuffers** is a cross platform serialization library architected for
maximum memory efficiency. It allows you to directly access serialized data without parsing/unpacking it first, while still having great forwards/backwards compatibility.

## Quick Start

1. Build the compiler for flatbuffers (`flatc`)

    Use `cmake` to create the build files for your platform and then perform the compliation (Linux example).

    ```
    cmake -G "Unix Makefiles"
    make -j
    ```

2. Define your flatbuffer schema (`.fbs`)

    Write the [schema](https://flatbuffers.dev/flatbuffers_guide_writing_schema.html) to define the data you want to serialize. See [monster.fbs](https://github.com/google/flatbuffers/blob/master/samples/monster.fbs) for an example.

3. Generate code for your language(s)

    Use the `flatc` compiler to take your schema and generate language-specific code:

    ```
    ./flatc --cpp --rust monster.fbs
    ```
    
    Which generates `monster_generated.h` and `monster_generated.rs` files.

4. Serialize data

    Use the generated code, as well as the `FlatBufferBuilder` to construct your serialized buffer. ([`C++` example](https://github.com/google/flatbuffers/blob/master/samples/sample_binary.cpp#L24-L56))

5. Transmit/store/save Buffer

    Use your serialized buffer however you want. Send it to someone, save it for later, etc...

6. Read the data

    Use the generated accessors to read the data from the serialized buffer.
    
    It doesn't need to be the same language/schema version, FlatBuffers ensures the data is readable across languages and schema versions. See the [`Rust` example](https://github.com/google/flatbuffers/blob/master/samples/sample_binary.rs#L92-L106) reading the data written by `C++`.

## Documentation

**Go to our [landing page][] to browse our documentation.**

## Supported operating systems
- Windows
- macOS
- Linux
- Android
- And any others with a recent C++ compiler (C++ 11 and newer)

## Supported programming languages

Code generation and runtime libraries for many popular languages.

1. C
1. C++ - [snapcraft.io](https://snapcraft.io/flatbuffers)
1. C# - [nuget.org](https://www.nuget.org/packages/Google.FlatBuffers)
1. Dart - [pub.dev](https://pub.dev/packages/flat_buffers)
1. Go - [go.dev](https://pkg.go.dev/github.com/google/flatbuffers)
1. Java - [Maven](https://search.maven.org/artifact/com.google.flatbuffers/flatbuffers-java)
1. JavaScript - [NPM](https://www.npmjs.com/package/flatbuffers)
1. Kotlin
1. Lobster
1. Lua
1. PHP
1. Python - [PyPI](https://pypi.org/project/flatbuffers/)
1. Rust - [crates.io](https://crates.io/crates/flatbuffers)
1. Swift - [swiftpackageindex](https://swiftpackageindex.com/google/flatbuffers)
1. TypeScript - [NPM](https://www.npmjs.com/package/flatbuffers)
1. Nim

## Versioning

FlatBuffers does not follow traditional SemVer versioning (see [rationale](https://github.com/google/flatbuffers/wiki/Versioning)) but rather uses a format of the date of the release.

## Contribution

* [FlatBuffers Issues Tracker][] to submit an issue.
* [stackoverflow.com][] with [`flatbuffers` tag][] for any questions regarding FlatBuffers.

*To contribute to this project,* see [CONTRIBUTING][].

## Community

* [Discord Server](https:///discord.gg/6qgKs3R)

## Security

Please see our [Security Policy](SECURITY.md) for reporting vulnerabilities.

## Licensing
*Flatbuffers* is licensed under the Apache License, Version 2.0. See [LICENSE][] for the full license text.

<br>

   [CONTRIBUTING]: http://github.com/google/flatbuffers/blob/master/CONTRIBUTING.md
   [`flatbuffers` tag]: https://stackoverflow.com/questions/tagged/flatbuffers
   [FlatBuffers Google Group]: https://groups.google.com/forum/#!forum/flatbuffers
   [FlatBuffers Issues Tracker]: http://github.com/google/flatbuffers/issues
   [stackoverflow.com]: http://stackoverflow.com/search?q=flatbuffers
   [landing page]: https://google.github.io/flatbuffers
   [LICENSE]: https://github.com/google/flatbuffers/blob/master/LICENSE
