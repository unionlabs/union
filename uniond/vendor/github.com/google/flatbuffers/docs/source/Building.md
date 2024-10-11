Building    {#flatbuffers_guide_building}
========

## Building with CMake

The distribution comes with a `cmake` file that should allow
you to build project/make files for any platform. For details on `cmake`, see
<https://www.cmake.org>. In brief, depending on your platform, use one of
e.g.:

    cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release
    cmake -G "Visual Studio 10" -DCMAKE_BUILD_TYPE=Release
    cmake -G "Xcode" -DCMAKE_BUILD_TYPE=Release

Then, build as normal for your platform. This should result in a `flatc`
executable, essential for the next steps.
Note that to use clang instead of gcc, you may need to set up your environment
variables, e.g.
`CC=/usr/bin/clang CXX=/usr/bin/clang++ cmake -G "Unix Makefiles"`.

Optionally, run the `flattests` executable from the root `flatbuffers/`
directory to ensure everything is working correctly on your system. If this
fails, please contact us!

Building should also produce two sample executables, `flatsamplebinary` and
`flatsampletext`, see the corresponding `.cpp` files in the
`flatbuffers/samples` directory.

*Note that you MUST be in the root of the FlatBuffers distribution when you
run 'flattests' or `flatsampletext`, or it will fail to load its files.*

### Make all warnings into errors

By default all Flatbuffers `cmake` targets are **not** built with the `-Werror` 
(or `/WX` for MSVC) flag that treats any warning as an error. This allows more 
flexibility for users of Flatbuffers to use newer compilers and toolsets that
may add new warnings that would cause a build failure.

To enable a stricter build that does treat warnings as errors, set the
`FLATBUFFERS_STRICT_MODE` `cmake` compliation flag to `ON`. 

```
cmake . -DFLATBUFFERS_STRICT_MODE=ON
```

Our CI builds run with strict mode on, ensuring the code that is committed to 
the project is as portable and warning free as possible. Thus developers
contributing to the project should enable strict mode locally before making a 
PR.

## Building with VCPKG

You can download and install flatbuffers using the [vcpkg](https://github.com/Microsoft/vcpkg/) dependency manager:

    git clone https://github.com/Microsoft/vcpkg.git
    cd vcpkg
    ./bootstrap-vcpkg.sh
    ./vcpkg integrate install
    ./vcpkg install flatbuffers

The flatbuffers port in vcpkg is kept up to date by Microsoft team members and community contributors.
If the version is out of date, please [create an issue or pull request](https://github.com/Microsoft/vcpkg) on the vcpkg repository.

## Downloading binaries
You can download the binaries from the
[GitHub release page](https://github.com/google/flatbuffers/releases).

We generate [SLSA3 signatures](slsa.dev) using the OpenSSF's [slsa-framework/slsa-github-generator](https://github.com/slsa-framework/slsa-github-generator). To verify the binaries:
1. Install the verification tool from [slsa-framework/slsa-verifier#installation](https://github.com/slsa-framework/slsa-verifier#installation)
1. Download the file named `attestation.intoto.jsonl` from the GitHub release
1. Run:
```shell
$ slsa-verifier -artifact-path <downloaded.zip> -provenance attestation.intoto.jsonl -source github.com/google/flatbuffers -tag <version>
  PASSED: Verified SLSA provenance

## Building for Android

There is a `flatbuffers/android` directory that contains all you need to build
the test executable on android (use the included `build_apk.sh` script, or use
`ndk_build` / `adb` etc. as usual). Upon running, it will output to the log
if tests succeeded or not.

You may also run an android sample from inside the `flatbuffers/samples`, by
running the `android_sample.sh` script. Optionally, you may go to the
`flatbuffers/samples/android` folder and build the sample with the
`build_apk.sh` script or `ndk_build` / `adb` etc.

## Using FlatBuffers in your own projects

For C++, there is usually no runtime to compile, as the code consists of a
single header, `include/flatbuffers/flatbuffers.h`. You should add the
`include` folder to your include paths. If you wish to be
able to load schemas and/or parse text into binary buffers at runtime,
you additionally need the other headers in `include/flatbuffers`. You must
also compile/link `src/idl_parser.cpp` (and `src/idl_gen_text.cpp` if you
also want to be able convert binary to text).

To see how to include FlatBuffers in any of our supported languages, please
view the [Tutorial](@ref flatbuffers_guide_tutorial) and select your appropriate
language using the radio buttons.

### Using in CMake-based projects
If you want to use FlatBuffers in a project which already uses CMake, then a more
robust and flexible approach is to build FlatBuffers as part of that project directly.
This is done by making the FlatBuffers source code available to the main build
and adding it using CMake's `add_subdirectory()` command. This has the
significant advantage that the same compiler and linker settings are used
between FlatBuffers and the rest of your project, so issues associated with using
incompatible libraries (eg debug/release), etc. are avoided. This is
particularly useful on Windows.

Suppose you put FlatBuffers source code in directory `${FLATBUFFERS_SRC_DIR}`.
To build it as part of your project, add following code to your `CMakeLists.txt` file:
```cmake
# Add FlatBuffers directly to our build. This defines the `flatbuffers` target.
add_subdirectory(${FLATBUFFERS_SRC_DIR}
                 ${CMAKE_CURRENT_BINARY_DIR}/flatbuffers-build
                 EXCLUDE_FROM_ALL)

# Now simply link against flatbuffers as needed to your already declared target.
# The flatbuffers target carry header search path automatically if CMake > 2.8.11.
target_link_libraries(own_project_target PRIVATE flatbuffers)
```
When build your project the `flatbuffers` library will be compiled and linked 
to a target as part of your project.

#### Override default depth limit of nested objects
To override [the depth limit of recursion](@ref flatbuffers_guide_use_cpp), 
add this directive:
```cmake
set(FLATBUFFERS_MAX_PARSING_DEPTH 16)
```
to `CMakeLists.txt` file before `add_subdirectory(${FLATBUFFERS_SRC_DIR})` line.
