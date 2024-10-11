# For more information about using CMake with Android Studio, read the
# documentation: https://d.android.com/studio/projects/add-native-code.html

# Sets the minimum version of CMake required to build the native library.

cmake_minimum_required(VERSION 3.4.1)

include_directories(${FLATBUFFERS_SRC}/include)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=gnu++11 -fexceptions -Wall -DFLATBUFFERS_TRACK_VERIFIER_BUFFER_SIZE")

# Certain platforms such as ARM do not use signed chars by default
# which causes issues with certain bounds checks.
set(CMAKE_CXX_FLAGS
   "${CMAKE_CXX_FLAGS} -fsigned-char")

set(FlatBuffers_Library_SRCS
        ${FLATBUFFERS_SRC}/include/flatbuffers/allocator.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/array.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/base.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/buffer.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/buffer_ref.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/default_allocator.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/detached_buffer.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/flatbuffer_builder.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/flatbuffers.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/flexbuffers.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/flex_flat_util.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/hash.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/idl.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/minireflect.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/reflection.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/reflection_generated.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/registry.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/stl_emulation.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/string.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/struct.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/table.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/util.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/vector.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/vector_downward.h
        ${FLATBUFFERS_SRC}/include/flatbuffers/verifier.h
        ${FLATBUFFERS_SRC}/src/idl_parser.cpp
        ${FLATBUFFERS_SRC}/src/idl_gen_text.cpp
        ${FLATBUFFERS_SRC}/src/reflection.cpp
        ${FLATBUFFERS_SRC}/src/util.cpp
        ${FLATBUFFERS_SRC}/src/idl_gen_fbs.cpp
        ${FLATBUFFERS_SRC}/src/code_generators.cpp
        )

add_library( # Sets the name of the library.
             flatbuffers

             ${FlatBuffers_Library_SRCS}
             ${Generated_SRCS}
)
