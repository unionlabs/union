# This is the legacy minimum version flatbuffers supported for a while.
cmake_minimum_required(VERSION 3.8...3.25.2)

# Attempt to read the current version of flatbuffers by looking at the latest tag.
include(CMake/Version.cmake)

project(FlatBuffers
        VERSION ${VERSION_MAJOR}.${VERSION_MINOR}.${VERSION_PATCH}
        LANGUAGES CXX)

# generate compile_commands.json
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# NOTE: Code coverage only works on Linux & OSX.
option(FLATBUFFERS_CODE_COVERAGE "Enable the code coverage build option." OFF)
option(FLATBUFFERS_BUILD_TESTS "Enable the build of tests and samples." ON)
option(FLATBUFFERS_INSTALL "Enable the installation of targets." ON)
option(FLATBUFFERS_BUILD_FLATLIB "Enable the build of the flatbuffers library"
       ON)
option(FLATBUFFERS_BUILD_FLATC "Enable the build of the flatbuffers compiler"
       ON)
option(FLATBUFFERS_STATIC_FLATC "Build flatbuffers compiler with -static flag"
       OFF)
option(FLATBUFFERS_BUILD_FLATHASH "Enable the build of flathash" OFF)
option(FLATBUFFERS_BUILD_BENCHMARKS "Enable the build of flatbenchmark."
       OFF)
option(FLATBUFFERS_BUILD_GRPCTEST "Enable the build of grpctest" OFF)
option(FLATBUFFERS_BUILD_SHAREDLIB
       "Enable the build of the flatbuffers shared library"
       OFF)
option(FLATBUFFERS_LIBCXX_WITH_CLANG "Force libc++ when using Clang" ON)
# NOTE: Sanitizer check only works on Linux & OSX (gcc & llvm).
option(FLATBUFFERS_CODE_SANITIZE
      "Add '-fsanitize' flags to 'flattests' and 'flatc' targets."
      OFF)
option(FLATBUFFERS_PACKAGE_REDHAT
       "Build an rpm using the 'package' target."
       OFF)
option(FLATBUFFERS_PACKAGE_DEBIAN
       "Build an deb using the 'package' target."
       OFF)
option(FLATBUFFERS_BUILD_CPP17
       "Enable the build of c++17 test target. \"
       Requirements: Clang6, GCC7, MSVC2017 (_MSC_VER >= 1914)  or higher."
       OFF)
option(FLATBUFFERS_BUILD_LEGACY
       "Run C++ code generator with '--cpp-std c++0x' switch."
       OFF)
option(FLATBUFFERS_ENABLE_PCH
       "Enable precompile headers support for 'flatbuffers' and 'flatc'. \"
        Only work if CMake supports 'target_precompile_headers'. \"
        This can speed up compilation time."
       OFF)
option(FLATBUFFERS_SKIP_MONSTER_EXTRA
      "Skip generating monster_extra.fbs that contains non-supported numerical\"
      types." OFF)
option(FLATBUFFERS_STRICT_MODE
      "Build flatbuffers with all warnings as errors (-Werror or /WX)."
      OFF)

if(NOT DEFINED FLATBUFFERS_CPP_STD)
  set(FLATBUFFERS_CPP_STD 11)
endif()

set(MSVC_LIKE OFF)
if(MSVC OR CMAKE_CXX_COMPILER_FRONTEND_VARIANT STREQUAL "MSVC")
    set(MSVC_LIKE ON)
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  set(IS_CLANG ON)
else()
  set(IS_CLANG OFF)
endif()

if(DEFINED FLATBUFFERS_COMPILATION_TIMINGS)
  message("Recording Compilation Timings to ${FLATBUFFERS_COMPILATION_TIMINGS}")
  file(REMOVE ${FLATBUFFERS_COMPILATION_TIMINGS})
  set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE "time -f 'Wall: %E User: %U Sys: %S | %C' -q -a -o ${FLATBUFFERS_COMPILATION_TIMINGS}")
  set_property(GLOBAL PROPERTY RULE_LAUNCH_CUSTOM "time -f 'Wall: %E User: %U Sys: %S | %C' -q -a -o ${FLATBUFFERS_COMPILATION_TIMINGS}")
  set_property(GLOBAL PROPERTY RULE_LAUNCH_LINK "time -f 'Wall: %E User: %U Sys: %S | %C' -q -a -o ${FLATBUFFERS_COMPILATION_TIMINGS}")
endif()

if(NOT FLATBUFFERS_BUILD_FLATC AND FLATBUFFERS_BUILD_TESTS)
    message(WARNING
    "Cannot build tests without building the compiler. Tests will be disabled.")
    set(FLATBUFFERS_BUILD_TESTS OFF)
endif()

if(DEFINED FLATBUFFERS_MAX_PARSING_DEPTH)
  # Override the default recursion depth limit.
  add_definitions(-DFLATBUFFERS_MAX_PARSING_DEPTH=${FLATBUFFERS_MAX_PARSING_DEPTH})
  message(STATUS "FLATBUFFERS_MAX_PARSING_DEPTH: ${FLATBUFFERS_MAX_PARSING_DEPTH}")
endif()

# Auto-detect locale-narrow 'strtod_l' and  'strtoull_l' functions.
if(NOT DEFINED FLATBUFFERS_LOCALE_INDEPENDENT)
  include(CheckCXXSymbolExists)

  set(FLATBUFFERS_LOCALE_INDEPENDENT 0)
  if(MSVC_LIKE)
    check_cxx_symbol_exists(_strtof_l stdlib.h FLATBUFFERS_HAS_STRTOF_L)
    check_cxx_symbol_exists(_strtoui64_l stdlib.h FLATBUFFERS_HAS_STRTOULL_L)
  else()
    check_cxx_symbol_exists(strtof_l stdlib.h FLATBUFFERS_HAS_STRTOF_L)
    check_cxx_symbol_exists(strtoull_l stdlib.h FLATBUFFERS_HAS_STRTOULL_L)
  endif()
  if(FLATBUFFERS_HAS_STRTOF_L AND FLATBUFFERS_HAS_STRTOULL_L)
    set(FLATBUFFERS_LOCALE_INDEPENDENT 1)
  endif()
endif()
add_definitions(-DFLATBUFFERS_LOCALE_INDEPENDENT=$<BOOL:${FLATBUFFERS_LOCALE_INDEPENDENT}>)

if(NOT WIN32)
  check_symbol_exists(realpath "stdlib.h" HAVE_REALPATH)
  if(NOT HAVE_REALPATH)
    add_definitions(-DFLATBUFFERS_NO_ABSOLUTE_PATH_RESOLUTION)
  endif()
endif()

set(FlatBuffers_Library_SRCS
  include/flatbuffers/allocator.h
  include/flatbuffers/array.h
  include/flatbuffers/base.h
  include/flatbuffers/buffer.h
  include/flatbuffers/buffer_ref.h
  include/flatbuffers/default_allocator.h
  include/flatbuffers/detached_buffer.h
  include/flatbuffers/code_generator.h
  include/flatbuffers/file_manager.h
  include/flatbuffers/flatbuffer_builder.h
  include/flatbuffers/flatbuffers.h
  include/flatbuffers/flexbuffers.h
  include/flatbuffers/flex_flat_util.h
  include/flatbuffers/hash.h
  include/flatbuffers/idl.h
  include/flatbuffers/minireflect.h
  include/flatbuffers/reflection.h
  include/flatbuffers/reflection_generated.h
  include/flatbuffers/registry.h
  include/flatbuffers/stl_emulation.h
  include/flatbuffers/string.h
  include/flatbuffers/struct.h
  include/flatbuffers/table.h
  include/flatbuffers/util.h
  include/flatbuffers/vector.h
  include/flatbuffers/vector_downward.h
  include/flatbuffers/verifier.h
  src/idl_parser.cpp
  src/idl_gen_text.cpp
  src/reflection.cpp
  src/util.cpp
)

set(FlatBuffers_Compiler_SRCS
  ${FlatBuffers_Library_SRCS}
  src/idl_gen_binary.cpp
  src/idl_gen_text.cpp
  src/idl_gen_cpp.cpp
  src/idl_gen_csharp.cpp
  src/idl_gen_dart.cpp
  src/idl_gen_kotlin.cpp
  src/idl_gen_kotlin_kmp.cpp
  src/idl_gen_go.cpp
  src/idl_gen_java.cpp
  src/idl_gen_ts.cpp
  src/idl_gen_php.cpp
  src/idl_gen_python.cpp
  src/idl_gen_lobster.cpp
  src/idl_gen_rust.cpp
  src/idl_gen_fbs.cpp
  src/idl_gen_grpc.cpp
  src/idl_gen_json_schema.cpp
  src/idl_gen_swift.cpp
  src/file_name_saving_file_manager.cpp
  src/file_binary_writer.cpp
  src/file_writer.cpp
  src/idl_namer.h
  src/namer.h
  src/flatc.cpp
  src/flatc_main.cpp
  src/bfbs_gen.h
  src/bfbs_gen_lua.h
  src/bfbs_gen_nim.h
  src/bfbs_namer.h
  include/flatbuffers/code_generators.h
  src/binary_annotator.h
  src/binary_annotator.cpp
  src/annotated_binary_text_gen.h
  src/annotated_binary_text_gen.cpp
  src/bfbs_gen_lua.cpp
  src/bfbs_gen_nim.cpp
  src/code_generators.cpp
  grpc/src/compiler/schema_interface.h
  grpc/src/compiler/cpp_generator.h
  grpc/src/compiler/cpp_generator.cc
  grpc/src/compiler/go_generator.h
  grpc/src/compiler/go_generator.cc
  grpc/src/compiler/java_generator.h
  grpc/src/compiler/java_generator.cc
  grpc/src/compiler/python_generator.h
  grpc/src/compiler/python_generator.cc
  grpc/src/compiler/swift_generator.h
  grpc/src/compiler/swift_generator.cc
  grpc/src/compiler/ts_generator.h
  grpc/src/compiler/ts_generator.cc
)

set(FlatHash_SRCS
  include/flatbuffers/hash.h
  src/flathash.cpp
)

set(FlatBuffers_Tests_SRCS
  ${FlatBuffers_Library_SRCS}
  src/idl_gen_fbs.cpp
  tests/evolution_test.cpp
  tests/flexbuffers_test.cpp
  tests/fuzz_test.cpp
  tests/json_test.cpp
  tests/key_field_test.cpp
  tests/monster_test.cpp
  tests/optional_scalars_test.cpp
  tests/parser_test.cpp
  tests/proto_test.cpp
  tests/reflection_test.cpp
  tests/test.cpp
  tests/test_assert.h
  tests/test_assert.cpp
  tests/test_builder.h
  tests/test_builder.cpp
  tests/util_test.cpp
  tests/native_type_test_impl.h
  tests/native_type_test_impl.cpp
  tests/alignment_test.h
  tests/alignment_test.cpp
  tests/64bit/offset64_test.h
  tests/64bit/offset64_test.cpp
  include/flatbuffers/code_generators.h
  src/code_generators.cpp
)

set(FlatBuffers_Tests_CPP17_SRCS
  ${FlatBuffers_Library_SRCS}
  tests/test_assert.h
  tests/test_assert.cpp
  tests/cpp17/test_cpp17.cpp
)

set(FlatBuffers_Sample_Binary_SRCS
  samples/sample_binary.cpp
)

set(FlatBuffers_Sample_Text_SRCS
  ${FlatBuffers_Library_SRCS}
  samples/sample_text.cpp
)

set(FlatBuffers_Sample_BFBS_SRCS
  ${FlatBuffers_Library_SRCS}
  samples/sample_bfbs.cpp
)

set(FlatBuffers_GRPCTest_SRCS
  include/flatbuffers/flatbuffers.h
  include/flatbuffers/grpc.h
  include/flatbuffers/util.h
  src/util.cpp
  tests/monster_test.grpc.fb.h
  tests/test_assert.h
  tests/test_builder.h
  tests/monster_test.grpc.fb.cc
  tests/test_assert.cpp
  tests/test_builder.cpp
  grpc/tests/grpctest.cpp
  grpc/tests/message_builder_test.cpp
)

# TODO(dbaileychess): Figure out how this would now work. I posted a question on
# https://stackoverflow.com/questions/71772330/override-target-compile-options-via-cmake-command-line.
# Append FLATBUFFERS_CXX_FLAGS to CMAKE_CXX_FLAGS.
if(DEFINED FLATBUFFERS_CXX_FLAGS)
  message(STATUS "extend CXX_FLAGS with ${FLATBUFFERS_CXX_FLAGS}")
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${FLATBUFFERS_CXX_FLAGS}")
endif()
message(STATUS "CMAKE_CXX_FLAGS: ${CMAKE_CXX_FLAGS}")

function(add_fsanitize_to_target _target _sanitizer)
  if(WIN32)
    target_compile_definitions(${_target} PRIVATE FLATBUFFERS_MEMORY_LEAK_TRACKING)
    message(STATUS "Sanitizer MSVC::_CrtDumpMemoryLeaks added to ${_target}")
  else()
    # FLATBUFFERS_CODE_SANITIZE: boolean {ON,OFF,YES,NO} or string with list of sanitizer.
    # List of sanitizer is string starts with '=': "=address,undefined,thread,memory".
    if(IS_CLANG OR (CMAKE_COMPILER_IS_GNUCXX AND CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL 4.9))
      set(_sanitizer_flags "=address,undefined")
      if(_sanitizer MATCHES "=.*")
        # override default by user-defined sanitizer list
        set(_sanitizer_flags ${_sanitizer})
      endif()
      target_compile_options(${_target} PRIVATE
        -g -fsigned-char -fno-omit-frame-pointer
        "-fsanitize${_sanitizer_flags}")
      target_link_libraries(${_target} PRIVATE
        "-fsanitize${_sanitizer_flags}")
      set_target_properties(${_target} PROPERTIES POSITION_INDEPENDENT_CODE ON)
      message(STATUS "Sanitizer ${_sanitizer_flags} added to ${_target}")
    endif()
  endif()
endfunction()

function(add_pch_to_target _target _pch_header)
  # the command is available since cmake 3.16
  if(COMMAND target_precompile_headers)
    target_precompile_headers(${_target} PRIVATE ${_pch_header})
    if(NOT MSVC)
      set_source_files_properties(src/util.cpp PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
    endif()
  endif()
endfunction()

include_directories(include)
include_directories(grpc)

# Creates an interface library that stores the configuration settings that each
# target links too. This is a compromise between setting configuration globally
# with add_compile_options() and the more targetted target_compile_options().
# This way each target in this file can share settings and override them if
# needed.
add_library(ProjectConfig INTERFACE)
target_compile_features(ProjectConfig
  INTERFACE
    cxx_std_${FLATBUFFERS_CPP_STD}
)

# Force the standard to be met.
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# We shouldn't rely on any compiler-extensions to make things work.
set(CMAKE_CXX_EXTENSIONS OFF)

if(MSVC_LIKE)
  target_compile_options(ProjectConfig
    INTERFACE
      /W4
      $<$<BOOL:${FLATBUFFERS_STRICT_MODE}>:
        /WX       # Treat all compiler warnings as errors
      >
      /wd4512   # C4512: assignment operator could not be generated
      /wd4316   # C4316: object allocated on the heap may not be aligned
      /wd4456   # C4456: hides previous local declaration
      $<$<CXX_COMPILER_ID:Clang>:
        /D_CRT_SECURE_NO_WARNINGS
      >
  )
else()
  target_compile_options(ProjectConfig
    INTERFACE
      -Wall
      $<$<BOOL:${FLATBUFFERS_STRICT_MODE}>:
        -Werror   # Treat all compiler warnings as errors

        -fno-rtti # Disable runtime type information

        $<$<CXX_COMPILER_ID:GNU>:
          # False positive string overflow
          # https://github.com/google/flatbuffers/issues/7366
          -Wno-error=stringop-overflow
        >
      >
      -pedantic
      -Wextra
      -Wno-unused-parameter
      -Wold-style-cast
      -fsigned-char
      -Wnon-virtual-dtor

      # This isn't working for some reason: $<$<CXX_COMPILER_ID:CLANG>:
      $<$<BOOL:${IS_CLANG}>:
        -Wnewline-eof
        -Wno-unknown-warning-option
        -Wmissing-declarations
        -Wzero-as-null-pointer-constant
        $<$<VERSION_GREATER:$<CXX_COMPILER_VERSION>,3.8>:
          -Wimplicit-fallthrough
          -Wextra-semi
          $<$<BOOL:${FLATBUFFERS_STRICT_MODE}>:
            -Werror=unused-private-field
          >
        >
      >

      $<$<CXX_COMPILER_ID:GNU>:
        $<$<VERSION_GREATER:$<CXX_COMPILER_VERSION>,4.4>:
          -Wunused-result
          -Wunused-parameter
          -Werror=unused-parameter
          -Wmissing-declarations
        >
        $<$<VERSION_GREATER:$<CXX_COMPILER_VERSION>,4.7>:
          -Wzero-as-null-pointer-constant
        >
        $<$<VERSION_GREATER:$<CXX_COMPILER_VERSION>,7.0>:
          -faligned-new
          $<$<BOOL:${FLATBUFFERS_STRICT_MODE}>:
            -Werror=implicit-fallthrough=2
          >
        >
        $<$<VERSION_GREATER:$<CXX_COMPILER_VERSION>,8.0>:
          -Wextra-semi
        >
      >

      $<$<BOOL:${FLATBUFFERS_CODE_COVERAGE}>:
        -g
        -fprofile-arcs
        -ftest-coverage
      >
    )

  if(FLATBUFFERS_CODE_COVERAGE)
    target_link_options(ProjectConfig
      INTERFACE
        -fprofile-arcs
        -ftest-coverage
    )
  endif()
endif()

if(FLATBUFFERS_BUILD_FLATLIB)
  add_library(flatbuffers STATIC ${FlatBuffers_Library_SRCS})

  # Attach header directory for when build via add_subdirectory().
  target_include_directories(flatbuffers
    INTERFACE
      $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  )
  target_link_libraries(flatbuffers PRIVATE $<BUILD_INTERFACE:ProjectConfig>)

  if(FLATBUFFERS_ENABLE_PCH)
    add_pch_to_target(flatbuffers include/flatbuffers/pch/pch.h)
  endif()
endif()

if(FLATBUFFERS_BUILD_FLATC)
  add_executable(flatc ${FlatBuffers_Compiler_SRCS})
  if(FLATBUFFERS_ENABLE_PCH)
    add_pch_to_target(flatc include/flatbuffers/pch/flatc_pch.h)
  endif()

  target_link_libraries(flatc PRIVATE $<BUILD_INTERFACE:ProjectConfig>)
  target_compile_options(flatc
    PRIVATE
      $<$<AND:$<BOOL:${MSVC_LIKE}>,$<CONFIG:Release>>:
        /MT
      >
  )

  if(FLATBUFFERS_CODE_SANITIZE AND NOT WIN32)
    add_fsanitize_to_target(flatc ${FLATBUFFERS_CODE_SANITIZE})
  endif()
  if(NOT FLATBUFFERS_FLATC_EXECUTABLE)
    set(FLATBUFFERS_FLATC_EXECUTABLE $<TARGET_FILE:flatc>)
  endif()
  if(FLATBUFFERS_STATIC_FLATC AND NOT MSVC)
    target_link_libraries(flatc PRIVATE -static)
  endif()
endif()

if(FLATBUFFERS_BUILD_FLATHASH)
  add_executable(flathash ${FlatHash_SRCS})
  target_link_libraries(flathash PRIVATE $<BUILD_INTERFACE:ProjectConfig>)
endif()

if(FLATBUFFERS_BUILD_SHAREDLIB)
  add_library(flatbuffers_shared SHARED ${FlatBuffers_Library_SRCS})
  target_link_libraries(flatbuffers_shared PRIVATE $<BUILD_INTERFACE:ProjectConfig>)
  # FlatBuffers use calendar-based versioning and do not provide any ABI
  # stability guarantees. Therefore, always use the full version as SOVERSION
  # in order to avoid breaking reverse dependencies on upgrades.
  set(FlatBuffers_Library_SONAME_FULL "${PROJECT_VERSION}")
  set_target_properties(flatbuffers_shared PROPERTIES
                        OUTPUT_NAME flatbuffers
                        SOVERSION "${FlatBuffers_Library_SONAME_FULL}"
                        VERSION "${FlatBuffers_Library_SONAME_FULL}")
  if(FLATBUFFERS_ENABLE_PCH)
    add_pch_to_target(flatbuffers_shared include/flatbuffers/pch/pch.h)
  endif()
endif()

function(compile_schema SRC_FBS OPT OUT_GEN_FILE) 
  get_filename_component(SRC_FBS_DIR ${SRC_FBS} PATH)
  string(REGEX REPLACE "\\.fbs$" "_generated.h" GEN_HEADER ${SRC_FBS})
  add_custom_command(
    OUTPUT ${GEN_HEADER}
    COMMAND "${FLATBUFFERS_FLATC_EXECUTABLE}"
      ${OPT}
      -o "${SRC_FBS_DIR}"
      "${CMAKE_CURRENT_SOURCE_DIR}/${SRC_FBS}"
    DEPENDS flatc ${SRC_FBS}
    COMMENT "flatc generation: `${SRC_FBS}` -> `${GEN_HEADER}`"
    )
  set(${OUT_GEN_FILE} ${GEN_HEADER} PARENT_SCOPE)
endfunction()

function(compile_schema_for_test SRC_FBS OPT)
  compile_schema("${SRC_FBS}" "${OPT}" GEN_FILE)
  target_sources(flattests PRIVATE ${GEN_FILE})
endfunction()

function(compile_schema_for_samples SRC_FBS OPT)
  compile_schema("${SRC_FBS}" "${OPT}" GEN_FILE)
  target_sources(flatsample PRIVATE ${GEN_FILE})
endfunction()

if(FLATBUFFERS_BUILD_TESTS)
  add_executable(flattests ${FlatBuffers_Tests_SRCS})
  target_link_libraries(flattests PRIVATE $<BUILD_INTERFACE:ProjectConfig>)
  target_include_directories(flattests PUBLIC 
    # Ideally everything is fully qualified from the root directories
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_BINARY_DIR}
    # TODO(derekbailey): update includes to fully qualify src/ and tests/
    src 
    tests 
    ${CMAKE_CURRENT_BINARY_DIR}/tests 
  )

  # Have tests load data from the source directory, not the build directory.
  add_definitions(-DFLATBUFFERS_TEST_PATH_PREFIX=${CMAKE_CURRENT_SOURCE_DIR}/)

  # The flattest target needs some generated files
  SET(FLATC_OPT --cpp --gen-mutable --gen-object-api --reflect-names)
  SET(FLATC_OPT_COMP ${FLATC_OPT};--gen-compare)
  SET(FLATC_OPT_SCOPED_ENUMS ${FLATC_OPT_COMP};--scoped-enums)

  compile_schema_for_test(tests/alignment_test.fbs "${FLATC_OPT_COMP}")
  compile_schema_for_test(tests/arrays_test.fbs "${FLATC_OPT_SCOPED_ENUMS}")
  compile_schema_for_test(tests/native_inline_table_test.fbs "${FLATC_OPT_COMP}")
  compile_schema_for_test(tests/native_type_test.fbs "${FLATC_OPT}")
  compile_schema_for_test(tests/key_field/key_field_sample.fbs "${FLATC_OPT_COMP}")
  compile_schema_for_test(tests/64bit/test_64bit.fbs "${FLATC_OPT_COMP};--bfbs-gen-embed")
  compile_schema_for_test(tests/64bit/evolution/v1.fbs "${FLATC_OPT_COMP}")
  compile_schema_for_test(tests/64bit/evolution/v2.fbs "${FLATC_OPT_COMP}")
  compile_schema_for_test(tests/union_underlying_type_test.fbs "${FLATC_OPT_SCOPED_ENUMS}")

  if(FLATBUFFERS_CODE_SANITIZE)
    add_fsanitize_to_target(flattests ${FLATBUFFERS_CODE_SANITIZE})
  endif()
  
  include_directories(${CMAKE_CURRENT_BINARY_DIR}/samples)

  add_executable(flatsamplebinary ${FlatBuffers_Sample_Binary_SRCS})
  add_executable(flatsampletext ${FlatBuffers_Sample_Text_SRCS})
  add_executable(flatsamplebfbs ${FlatBuffers_Sample_BFBS_SRCS})

  # Add a library so there is a single target that the generated samples can 
  # link too.
  if(MSVC OR ${CMAKE_VERSION} VERSION_GREATER_EQUAL "3.20.0")
    add_library(flatsample INTERFACE)
  else()
    add_library(flatsample STATIC)
  endif()

  # Since flatsample has no sources, we have to explicitly set the linker lang.
  set_target_properties(flatsample PROPERTIES LINKER_LANGUAGE CXX)
  
  compile_schema_for_samples(samples/monster.fbs "${FLATC_OPT_COMP}")

  target_link_libraries(flatsamplebinary PRIVATE $<BUILD_INTERFACE:ProjectConfig> flatsample)
  target_link_libraries(flatsampletext PRIVATE $<BUILD_INTERFACE:ProjectConfig> flatsample)
  target_link_libraries(flatsamplebfbs PRIVATE $<BUILD_INTERFACE:ProjectConfig> flatsample)

  if(FLATBUFFERS_BUILD_CPP17)
    add_executable(flattests_cpp17 ${FlatBuffers_Tests_CPP17_SRCS})
    target_link_libraries(flattests_cpp17 PRIVATE $<BUILD_INTERFACE:ProjectConfig>)
    target_include_directories(flattests_cpp17 PUBLIC src tests)
    target_compile_features(flattests_cpp17 PRIVATE cxx_std_17) # requires cmake 3.8

    if(FLATBUFFERS_CODE_SANITIZE)
      add_fsanitize_to_target(flattests_cpp17 ${FLATBUFFERS_CODE_SANITIZE})
    endif()
  endif(FLATBUFFERS_BUILD_CPP17)
endif()

if(FLATBUFFERS_BUILD_GRPCTEST)
  if(NOT GRPC_INSTALL_PATH)
    message(SEND_ERROR "GRPC_INSTALL_PATH variable is not defined. See grpc/README.md")
  endif()
  if(NOT PROTOBUF_DOWNLOAD_PATH)
    message(SEND_ERROR "PROTOBUF_DOWNLOAD_PATH variable is not defined. See grpc/README.md")
  endif()
  INCLUDE_DIRECTORIES(${GRPC_INSTALL_PATH}/include)
  INCLUDE_DIRECTORIES(${PROTOBUF_DOWNLOAD_PATH}/src)
  find_package(Threads REQUIRED)
  list(APPEND CMAKE_PREFIX_PATH ${GRPC_INSTALL_PATH})
  find_package(absl CONFIG REQUIRED)
  find_package(protobuf CONFIG REQUIRED)
  find_package(gRPC CONFIG REQUIRED)
  add_executable(grpctest ${FlatBuffers_GRPCTest_SRCS})
  target_link_libraries(grpctest
    PRIVATE
      $<BUILD_INTERFACE:ProjectConfig>
      gRPC::grpc++_unsecure
      gRPC::gpr
      pthread
      dl
  )
endif()

if(FLATBUFFERS_INSTALL)
  include(GNUInstallDirs)

  install(DIRECTORY include/flatbuffers DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

  set(FB_CMAKE_DIR "${CMAKE_INSTALL_LIBDIR}/cmake/flatbuffers")

  configure_file(CMake/flatbuffers-config-version.cmake.in flatbuffers-config-version.cmake @ONLY)
  install(
      FILES
        "CMake/flatbuffers-config.cmake"
        "CMake/BuildFlatBuffers.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/flatbuffers-config-version.cmake"
      DESTINATION ${FB_CMAKE_DIR}
  )

  if(FLATBUFFERS_BUILD_FLATLIB)
    install(
      TARGETS flatbuffers EXPORT FlatBuffersTargets
      ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
      INCLUDES DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )

    install(EXPORT FlatBuffersTargets
      FILE FlatBuffersTargets.cmake
      NAMESPACE flatbuffers::
      DESTINATION ${FB_CMAKE_DIR}
    )
  endif()

  if(FLATBUFFERS_BUILD_FLATC)
    install(
      TARGETS flatc EXPORT FlatcTargets
      RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )

    install(
      EXPORT FlatcTargets
      FILE FlatcTargets.cmake
      NAMESPACE flatbuffers::
      DESTINATION ${FB_CMAKE_DIR}
    )
  endif()

  if(FLATBUFFERS_BUILD_SHAREDLIB)
    install(
      TARGETS flatbuffers_shared EXPORT FlatBuffersSharedTargets
      ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
      RUNTIME DESTINATION ${CMAKE_INSTALL_LIBDIR}
      LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
      INCLUDES DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )

    install(
      EXPORT FlatBuffersSharedTargets
      FILE FlatBuffersSharedTargets.cmake
      NAMESPACE flatbuffers::
      DESTINATION ${FB_CMAKE_DIR}
    )
  endif()

  if(FLATBUFFERS_BUILD_SHAREDLIB OR FLATBUFFERS_BUILD_FLATLIB)
      configure_file(CMake/flatbuffers.pc.in flatbuffers.pc @ONLY)
      install(
        FILES "${CMAKE_CURRENT_BINARY_DIR}/flatbuffers.pc"
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig
      )
  endif()
endif()

if(FLATBUFFERS_BUILD_TESTS)
  enable_testing()

  add_test(NAME flattests COMMAND flattests)
  if(FLATBUFFERS_BUILD_CPP17)
    add_test(NAME flattests_cpp17 COMMAND flattests_cpp17)
  endif()
  if(FLATBUFFERS_BUILD_GRPCTEST)
    add_test(NAME grpctest COMMAND grpctest)
  endif()
endif()

include(CMake/BuildFlatBuffers.cmake)

if(UNIX)
    # Use of CPack only supported on Linux systems.
    if(FLATBUFFERS_PACKAGE_DEBIAN)
        include(CMake/PackageDebian.cmake)
        include(CPack)
    endif()
    if (FLATBUFFERS_PACKAGE_REDHAT)
        include(CMake/PackageRedhat.cmake)
        include(CPack)
    endif()
endif()

# Include for running Google Benchmarks.
if(FLATBUFFERS_BUILD_BENCHMARKS)
  add_subdirectory(benchmarks)
endif()

# Add FlatBuffers::FlatBuffers interface, needed for FetchContent_Declare
add_library(FlatBuffers INTERFACE)
add_library(FlatBuffers::FlatBuffers ALIAS FlatBuffers)
target_include_directories(
  FlatBuffers
  INTERFACE $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
            $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/include>)
