# Copyright 2015 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# General function to create FlatBuffer build rules for the given list of
# schemas.
#
# flatbuffers_schemas: A list of flatbuffer schema files to process.
#
# schema_include_dirs: A list of schema file include directories, which will be
# passed to flatc via the -I parameter.
#
# custom_target_name: The generated files will be added as dependencies for a
# new custom target with this name. You should add that target as a dependency
# for your main target to ensure these files are built. You can also retrieve
# various properties from this target, such as GENERATED_INCLUDES_DIR,
# BINARY_SCHEMAS_DIR, and COPY_TEXT_SCHEMAS_DIR.
#
# additional_dependencies: A list of additional dependencies that you'd like
# all generated files to depend on. Pass in a blank string if you have none.
#
# generated_includes_dir: Where to generate the C++ header files for these
# schemas. The generated includes directory will automatically be added to
# CMake's include directories, and will be where generated header files are
# placed. This parameter is optional; pass in empty string if you don't want to
# generate include files for these schemas.
#
# binary_schemas_dir: If you specify an optional binary schema directory, binary
# schemas will be generated for these schemas as well, and placed into the given
# directory.
#
# copy_text_schemas_dir: If you want all text schemas (including schemas from
# all schema include directories) copied into a directory (for example, if you
# need them within your project to build JSON files), you can specify that
# folder here. All text schemas will be copied to that folder.
#
# IMPORTANT: Make sure you quote all list arguments you pass to this function!
# Otherwise CMake will only pass in the first element.
# Example: build_flatbuffers("${fb_files}" "${include_dirs}" target_name ...)
function(build_flatbuffers flatbuffers_schemas
                           schema_include_dirs
                           custom_target_name
                           additional_dependencies
                           generated_includes_dir
                           binary_schemas_dir
                           copy_text_schemas_dir)

  # Test if including from FindFlatBuffers
  if(FLATBUFFERS_FLATC_EXECUTABLE)
    set(FLATC_TARGET "")
    set(FLATC ${FLATBUFFERS_FLATC_EXECUTABLE})
  elseif(TARGET flatbuffers::flatc)
    set(FLATC_TARGET flatbuffers::flatc)
    set(FLATC flatbuffers::flatc)
  else()
    set(FLATC_TARGET flatc)
    set(FLATC flatc)
  endif()
  set(FLATC_SCHEMA_ARGS --gen-mutable)
  if(FLATBUFFERS_FLATC_SCHEMA_EXTRA_ARGS)
    set(FLATC_SCHEMA_ARGS
      ${FLATBUFFERS_FLATC_SCHEMA_EXTRA_ARGS}
      ${FLATC_SCHEMA_ARGS}
      )
  endif()

  set(working_dir "${CMAKE_CURRENT_SOURCE_DIR}")

  set(schema_glob "*.fbs")
  # Generate the include files parameters.
  set(include_params "")
  set(all_generated_files "")
  foreach (include_dir ${schema_include_dirs})
    set(include_params -I ${include_dir} ${include_params})
    if (NOT ${copy_text_schemas_dir} STREQUAL "")
      # Copy text schemas from dependent folders.
      file(GLOB_RECURSE dependent_schemas ${include_dir}/${schema_glob})
      foreach (dependent_schema ${dependent_schemas})
        file(COPY ${dependent_schema} DESTINATION ${copy_text_schemas_dir})
      endforeach()
    endif()
  endforeach()

  foreach(schema ${flatbuffers_schemas})
    get_filename_component(filename ${schema} NAME_WE)
    # For each schema, do the things we requested.
    if (NOT ${generated_includes_dir} STREQUAL "")
      set(generated_include ${generated_includes_dir}/${filename}_generated.h)
      add_custom_command(
        OUTPUT ${generated_include}
        COMMAND ${FLATC} ${FLATC_SCHEMA_ARGS}
        -o ${generated_includes_dir}
        ${include_params}
        -c ${schema}
        DEPENDS ${FLATC_TARGET} ${schema} ${additional_dependencies}
        WORKING_DIRECTORY "${working_dir}")
      list(APPEND all_generated_files ${generated_include})
    endif()

    if (NOT ${binary_schemas_dir} STREQUAL "")
      set(binary_schema ${binary_schemas_dir}/${filename}.bfbs)
      add_custom_command(
        OUTPUT ${binary_schema}
        COMMAND ${FLATC} -b --schema
        -o ${binary_schemas_dir}
        ${include_params}
        ${schema}
        DEPENDS ${FLATC_TARGET} ${schema} ${additional_dependencies}
        WORKING_DIRECTORY "${working_dir}")
      list(APPEND all_generated_files ${binary_schema})
    endif()

    if (NOT ${copy_text_schemas_dir} STREQUAL "")
      file(COPY ${schema} DESTINATION ${copy_text_schemas_dir})
    endif()
  endforeach()

  # Create a custom target that depends on all the generated files.
  # This is the target that you can depend on to trigger all these
  # to be built.
  add_custom_target(${custom_target_name}
                    DEPENDS ${all_generated_files} ${additional_dependencies})

  # Register the include directory we are using.
  if (NOT ${generated_includes_dir} STREQUAL "")
    include_directories(${generated_includes_dir})
    set_property(TARGET ${custom_target_name}
      PROPERTY GENERATED_INCLUDES_DIR
      ${generated_includes_dir})
  endif()

  # Register the binary schemas dir we are using.
  if (NOT ${binary_schemas_dir} STREQUAL "")
    set_property(TARGET ${custom_target_name}
      PROPERTY BINARY_SCHEMAS_DIR
      ${binary_schemas_dir})
  endif()

  # Register the text schema copy dir we are using.
  if (NOT ${copy_text_schemas_dir} STREQUAL "")
    set_property(TARGET ${custom_target_name}
      PROPERTY COPY_TEXT_SCHEMAS_DIR
      ${copy_text_schemas_dir})
  endif()
endfunction()

# Creates a target that can be linked against that generates flatbuffer headers.
#
# This function takes a target name and a list of schemas. You can also specify
# other flagc flags using the FLAGS option to change the behavior of the flatc
# tool.
#
# When the target_link_libraries is done within a different directory than
# flatbuffers_generate_headers is called, then the target should also be dependent
# the custom generation target called GENERATE_<TARGET>.
#
# Arguments:
#   TARGET: The name of the target to generate.
#   SCHEMAS: The list of schema files to generate code for.
#   BINARY_SCHEMAS_DIR: Optional. The directory in which to generate binary
#       schemas. Binary schemas will only be generated if a path is provided.
#   INCLUDE: Optional. Search for includes in the specified paths. (Use this
#       instead of "-I <path>" and the FLAGS option so that CMake is aware of
#       the directories that need to be searched).
#   INCLUDE_PREFIX: Optional. The directory in which to place the generated
#       files. Use this instead of the --include-prefix option.
#   FLAGS: Optional. A list of any additional flags that you would like to pass
#       to flatc.
#
# Example:
#
#     flatbuffers_generate_headers(
#         TARGET my_generated_headers_target
#         INCLUDE_PREFIX ${MY_INCLUDE_PREFIX}"
#         SCHEMAS ${MY_SCHEMA_FILES}
#         BINARY_SCHEMAS_DIR "${MY_BINARY_SCHEMA_DIRECTORY}"
#         FLAGS --gen-object-api)
#
#     target_link_libraries(MyExecutableTarget
#         PRIVATE my_generated_headers_target
#     )
#
# Optional (only needed within different directory):
#     add_dependencies(app GENERATE_my_generated_headers_target)
function(flatbuffers_generate_headers)
  # Parse function arguments.
  set(options)
  set(one_value_args
    "TARGET"
    "INCLUDE_PREFIX"
    "BINARY_SCHEMAS_DIR")
  set(multi_value_args
    "SCHEMAS"
    "INCLUDE"
    "FLAGS")
  cmake_parse_arguments(
    PARSE_ARGV 0
    FLATBUFFERS_GENERATE_HEADERS
    "${options}"
    "${one_value_args}"
    "${multi_value_args}")

  # Test if including from FindFlatBuffers
  if(FLATBUFFERS_FLATC_EXECUTABLE)
    set(FLATC_TARGET "")
    set(FLATC ${FLATBUFFERS_FLATC_EXECUTABLE})
  elseif(TARGET flatbuffers::flatc)
    set(FLATC_TARGET flatbuffers::flatc)
    set(FLATC flatbuffers::flatc)
  else()
    set(FLATC_TARGET flatc)
    set(FLATC flatc)
  endif()

  set(working_dir "${CMAKE_CURRENT_SOURCE_DIR}")

  # Generate the include files parameters.
  set(include_params "")
  foreach (include_dir ${FLATBUFFERS_GENERATE_HEADERS_INCLUDE})
    set(include_params -I ${include_dir} ${include_params})
  endforeach()

  # Create a directory to place the generated code.
  set(generated_target_dir "${CMAKE_CURRENT_BINARY_DIR}/${FLATBUFFERS_GENERATE_HEADERS_TARGET}")
  set(generated_include_dir "${generated_target_dir}")
  if (NOT ${FLATBUFFERS_GENERATE_HEADERS_INCLUDE_PREFIX} STREQUAL "")
    set(generated_include_dir "${generated_include_dir}/${FLATBUFFERS_GENERATE_HEADERS_INCLUDE_PREFIX}")
    list(APPEND FLATBUFFERS_GENERATE_HEADERS_FLAGS 
         "--include-prefix" ${FLATBUFFERS_GENERATE_HEADERS_INCLUDE_PREFIX})
  endif()

  set(generated_custom_commands)

  # Create rules to generate the code for each schema.
  foreach(schema ${FLATBUFFERS_GENERATE_HEADERS_SCHEMAS})
    get_filename_component(filename ${schema} NAME_WE)
    set(generated_include "${generated_include_dir}/${filename}_generated.h")

    # Generate files for grpc if needed
    set(generated_source_file)
    if("${FLATBUFFERS_GENERATE_HEADERS_FLAGS}" MATCHES "--grpc")
      # Check if schema file contain a rpc_service definition
      file(STRINGS ${schema} has_grpc REGEX "rpc_service")
      if(has_grpc)
        list(APPEND generated_include "${generated_include_dir}/${filename}.grpc.fb.h")
        set(generated_source_file "${generated_include_dir}/${filename}.grpc.fb.cc")
      endif()
    endif()

    add_custom_command(
      OUTPUT ${generated_include} ${generated_source_file}
      COMMAND ${FLATC} ${FLATC_ARGS}
      -o ${generated_include_dir}
      ${include_params}
      -c ${schema}
      ${FLATBUFFERS_GENERATE_HEADERS_FLAGS}
      DEPENDS ${FLATC_TARGET} ${schema}
      WORKING_DIRECTORY "${working_dir}"
      COMMENT "Building ${schema} flatbuffers...")
    list(APPEND all_generated_header_files ${generated_include})
    list(APPEND all_generated_source_files ${generated_source_file})
    list(APPEND generated_custom_commands "${generated_include}" "${generated_source_file}")

    # Geneate the binary flatbuffers schemas if instructed to.
    if (NOT ${FLATBUFFERS_GENERATE_HEADERS_BINARY_SCHEMAS_DIR} STREQUAL "")
      set(binary_schema
          "${FLATBUFFERS_GENERATE_HEADERS_BINARY_SCHEMAS_DIR}/${filename}.bfbs")
      add_custom_command(
        OUTPUT ${binary_schema}
        COMMAND ${FLATC} -b --schema
        -o ${FLATBUFFERS_GENERATE_HEADERS_BINARY_SCHEMAS_DIR}
        ${include_params}
        ${schema}
        DEPENDS ${FLATC_TARGET} ${schema}
        WORKING_DIRECTORY "${working_dir}")
      list(APPEND generated_custom_commands "${binary_schema}")
      list(APPEND all_generated_binary_files ${binary_schema})
    endif()
  endforeach()

  # Create an additional target as add_custom_command scope is only within same directory (CMakeFile.txt)
  set(generate_target GENERATE_${FLATBUFFERS_GENERATE_HEADERS_TARGET})
  add_custom_target(${generate_target} ALL
                    DEPENDS ${generated_custom_commands}
                    COMMENT "Generating flatbuffer target ${FLATBUFFERS_GENERATE_HEADERS_TARGET}")

  # Set up interface library
  add_library(${FLATBUFFERS_GENERATE_HEADERS_TARGET} INTERFACE)
  target_sources(
    ${FLATBUFFERS_GENERATE_HEADERS_TARGET}
    INTERFACE
      ${all_generated_header_files}
      ${all_generated_binary_files}
      ${all_generated_source_files}
      ${FLATBUFFERS_GENERATE_HEADERS_SCHEMAS})
  add_dependencies(
    ${FLATBUFFERS_GENERATE_HEADERS_TARGET}
    ${FLATC}
    ${FLATBUFFERS_GENERATE_HEADERS_SCHEMAS})
  target_include_directories(
    ${FLATBUFFERS_GENERATE_HEADERS_TARGET}
    INTERFACE ${generated_target_dir})

  # Organize file layout for IDEs.
  source_group(
    TREE "${generated_target_dir}"
    PREFIX "Flatbuffers/Generated/Headers Files"
    FILES ${all_generated_header_files})
  source_group(
    TREE "${generated_target_dir}"
    PREFIX "Flatbuffers/Generated/Source Files"
    FILES ${all_generated_source_files})
  source_group(
    TREE ${working_dir}
    PREFIX "Flatbuffers/Schemas"
    FILES ${FLATBUFFERS_GENERATE_HEADERS_SCHEMAS})
  if (NOT ${FLATBUFFERS_GENERATE_HEADERS_BINARY_SCHEMAS_DIR} STREQUAL "")
    source_group(
      TREE "${FLATBUFFERS_GENERATE_HEADERS_BINARY_SCHEMAS_DIR}"
      PREFIX "Flatbuffers/Generated/Binary Schemas"
      FILES ${all_generated_binary_files})
  endif()
endfunction()

# Creates a target that can be linked against that generates flatbuffer binaries
# from json files.
#
# This function takes a target name and a list of schemas and Json files. You
# can also specify other flagc flags and options to change the behavior of the
# flatc compiler.
#
# Adding this target to your executable ensurses that the flatbuffer binaries
# are compiled before your executable is run.
#
# Arguments:
#   TARGET: The name of the target to generate.
#   JSON_FILES: The list of json files to compile to flatbuffers binaries.
#   SCHEMA: The flatbuffers schema of the Json files to be compiled.
#   INCLUDE: Optional. Search for includes in the specified paths. (Use this 
#       instead of "-I <path>" and the FLAGS option so that CMake is aware of 
#       the directories that need to be searched).
#   OUTPUT_DIR: The directly where the generated flatbuffers binaries should be
#       placed.
#   FLAGS: Optional. A list of any additional flags that you would like to pass
#       to flatc.
#
# Example:
#
#     flatbuffers_generate_binary_files(
#         TARGET my_binary_data
#         SCHEMA "${MY_SCHEMA_DIR}/my_example_schema.fbs"
#         JSON_FILES ${MY_JSON_FILES}
#         OUTPUT_DIR "${MY_BINARY_DATA_DIRECTORY}"
#         FLAGS --strict-json)
#
#     target_link_libraries(MyExecutableTarget
#         PRIVATE my_binary_data
#     )
function(flatbuffers_generate_binary_files)
  # Parse function arguments.
  set(options)
  set(one_value_args
    "TARGET"
    "SCHEMA"
    "OUTPUT_DIR")
  set(multi_value_args
    "JSON_FILES"
    "INCLUDE"
    "FLAGS")
  cmake_parse_arguments(
    PARSE_ARGV 0
    FLATBUFFERS_GENERATE_BINARY_FILES
    "${options}"
    "${one_value_args}"
    "${multi_value_args}")

  # Test if including from FindFlatBuffers
  if(FLATBUFFERS_FLATC_EXECUTABLE)
    set(FLATC_TARGET "")
    set(FLATC ${FLATBUFFERS_FLATC_EXECUTABLE})
  elseif(TARGET flatbuffers::flatc)
    set(FLATC_TARGET flatbuffers::flatc)
    set(FLATC flatbuffers::flatc)
  else()
    set(FLATC_TARGET flatc)
    set(FLATC flatc)
  endif()

  set(working_dir "${CMAKE_CURRENT_SOURCE_DIR}")

  # Generate the include files parameters.
  set(include_params "")
  foreach (include_dir ${FLATBUFFERS_GENERATE_BINARY_FILES_INCLUDE})
    set(include_params -I ${include_dir} ${include_params})
  endforeach()

  # Create rules to generate the flatbuffers binary for each json file.
  foreach(json_file ${FLATBUFFERS_GENERATE_BINARY_FILES_JSON_FILES})
    get_filename_component(filename ${json_file} NAME_WE)
    set(generated_binary_file "${FLATBUFFERS_GENERATE_BINARY_FILES_OUTPUT_DIR}/${filename}.bin")
    add_custom_command(
      OUTPUT ${generated_binary_file}
      COMMAND ${FLATC} ${FLATC_ARGS}
      -o ${FLATBUFFERS_GENERATE_BINARY_FILES_OUTPUT_DIR}
      ${include_params}
      -b ${FLATBUFFERS_GENERATE_BINARY_FILES_SCHEMA} ${json_file}
      ${FLATBUFFERS_GENERATE_BINARY_FILES_FLAGS}
      DEPENDS ${FLATC_TARGET} ${json_file}
      WORKING_DIRECTORY "${working_dir}"
      COMMENT "Building ${json_file} binary flatbuffers...")
      list(APPEND all_generated_binary_files ${generated_binary_file})
  endforeach()

  # Set up interface library
  add_library(${FLATBUFFERS_GENERATE_BINARY_FILES_TARGET} INTERFACE)
  target_sources(
    ${FLATBUFFERS_GENERATE_BINARY_FILES_TARGET}
    INTERFACE
      ${all_generated_binary_files}
      ${FLATBUFFERS_GENERATE_BINARY_FILES_JSON_FILES}
      ${FLATBUFFERS_GENERATE_BINARY_FILES_SCHEMA})
  add_dependencies(
    ${FLATBUFFERS_GENERATE_BINARY_FILES_TARGET}
    ${FLATC})

  # Organize file layout for IDEs.
  source_group(
    TREE ${working_dir}
    PREFIX "Flatbuffers/JSON Files"
    FILES ${FLATBUFFERS_GENERATE_BINARY_FILES_JSON_FILES})
  source_group(
    TREE ${working_dir}
    PREFIX "Flatbuffers/Schemas"
    FILES ${FLATBUFFERS_GENERATE_BINARY_FILES_SCHEMA})
  source_group(
    TREE ${FLATBUFFERS_GENERATE_BINARY_FILES_OUTPUT_DIR}
    PREFIX "Flatbuffers/Generated/Binary Files"
    FILES ${all_generated_binary_files})
endfunction()
