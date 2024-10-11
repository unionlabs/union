# Setup for running Google Benchmarks (https://github.com/google/benchmark) on
# flatbuffers. This requires both that benchmark library and its dependency gtest
# to build. Instead of including them here or doing a submodule, this uses
# FetchContent (https://cmake.org/cmake/help/latest/module/FetchContent.html) to
# grab the dependencies at config time. This requires CMake 3.14 or higher.

cmake_minimum_required(VERSION 3.14)
include(FetchContent)

# No particular reason for the specific GIT_TAGs for the following repos, they
# were just the latest releases when this was added.
FetchContent_Declare(
    googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG        e2239ee6043f73722e7aa812a459f54a28552929 # release-1.11.0
)
FetchContent_Declare(
    googlebenchmark
    GIT_REPOSITORY https://github.com/google/benchmark.git
    GIT_TAG        0d98dba29d66e93259db7daa53a9327df767a415 # v1.6.1
)

# For Windows: Prevent overriding the parent project's compiler/linker 
# settings.
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(
    googletest
    googlebenchmark
)

set(CPP_BENCH_DIR ${CMAKE_CURRENT_SOURCE_DIR}/cpp)
set(CPP_FB_BENCH_DIR ${CPP_BENCH_DIR}/flatbuffers)
set(CPP_RAW_BENCH_DIR ${CPP_BENCH_DIR}/raw)
set(CPP_BENCH_FBS ${CPP_FB_BENCH_DIR}/bench.fbs)
set(CPP_BENCH_FB_GEN ${CPP_FB_BENCH_DIR}/bench_generated.h)

set(FlatBenchmark_SRCS
    ${CPP_BENCH_DIR}/benchmark_main.cpp
    ${CPP_FB_BENCH_DIR}/fb_bench.cpp
    ${CPP_RAW_BENCH_DIR}/raw_bench.cpp
    ${CPP_BENCH_FB_GEN}
)

# Generate the flatbuffers benchmark code from the flatbuffers schema using
# flatc itself, thus it depends on flatc. This also depends on the C++ runtime
# flatbuffers and the schema file itself, so it should auto-generated at the
# correct times.
add_custom_command(
    OUTPUT ${CPP_BENCH_FB_GEN}
    COMMAND 
        "${FLATBUFFERS_FLATC_EXECUTABLE}"
        --cpp
        -o ${CPP_FB_BENCH_DIR}
        ${CPP_BENCH_FBS}
    DEPENDS 
        flatc
        flatbuffers
        ${CPP_BENCH_FBS}
    COMMENT "Run Flatbuffers Benchmark Codegen: ${CPP_BENCH_FB_GEN}"
    VERBATIM)

# The main flatbuffers benchmark executable
add_executable(flatbenchmark ${FlatBenchmark_SRCS})

# Benchmark requires C++11 
target_compile_features(flatbenchmark PRIVATE
    cxx_std_11 # requires cmake 3.8
)

target_compile_options(flatbenchmark
    PRIVATE
        -fno-aligned-new
        -Wno-deprecated-declarations
)

# Set the output directory to the root binary directory
set_target_properties(flatbenchmark 
    PROPERTIES RUNTIME_OUTPUT_DIRECTORY 
    "${CMAKE_BINARY_DIR}"
)

# The includes of the benchmark files are fully qualified from flatbuffers root.
target_include_directories(flatbenchmark PUBLIC ${CMAKE_SOURCE_DIR})

target_link_libraries(flatbenchmark PRIVATE
    benchmark::benchmark_main # _main to use their entry point 
    gtest # Link to gtest so we can also assert in the benchmarks
)