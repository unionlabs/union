#!/usr/bin/env python3
#
# Copyright 2021 Google Inc. All rights reserved.
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

import platform
import subprocess
from pathlib import Path

# Get the path where this script is located so we can invoke the script from
# any directory and have the paths work correctly.
script_path = Path(__file__).parent.resolve()

# Get the root path as an absolute path, so all derived paths are absolute.
root_path = script_path.parent.parent.absolute()

# Get the location of the flatc executable, reading from the first command line
# argument or defaulting to default names.
flatc_exe = Path(
    ("flatc" if not platform.system() == "Windows" else "flatc.exe")
)

# Find and assert flatc compiler is present.
if root_path in flatc_exe.parents:
    flatc_exe = flatc_exe.relative_to(root_path)
flatc_path = Path(root_path, flatc_exe)
assert flatc_path.exists(), "Cannot find the flatc compiler " + str(flatc_path)

# Specify the other paths that will be referenced
tests_path = Path(script_path, "tests")


def flatc_annotate(schema, file, cwd=script_path):
    cmd = [str(flatc_path), "--annotate", schema, file]
    result = subprocess.run(cmd, cwd=str(cwd), check=True)


test_files = [
    "annotated_binary.bin",
    "tests/invalid_root_offset.bin",
    "tests/invalid_root_table_too_short.bin",
    "tests/invalid_root_table_vtable_offset.bin",
    "tests/invalid_string_length.bin",
    "tests/invalid_string_length_cut_short.bin",
    "tests/invalid_struct_array_field_cut_short.bin",
    "tests/invalid_struct_field_cut_short.bin",
    "tests/invalid_table_field_size.bin",
    "tests/invalid_table_field_offset.bin",
    "tests/invalid_union_type_value.bin",
    "tests/invalid_vector_length_cut_short.bin",
    "tests/invalid_vector_scalars_cut_short.bin",
    "tests/invalid_vector_strings_cut_short.bin",
    "tests/invalid_vector_structs_cut_short.bin",
    "tests/invalid_vector_tables_cut_short.bin",
    "tests/invalid_vector_unions_cut_short.bin",
    "tests/invalid_vector_union_type_value.bin",
    "tests/invalid_vtable_ref_table_size_short.bin",
    "tests/invalid_vtable_ref_table_size.bin",
    "tests/invalid_vtable_size_short.bin",
    "tests/invalid_vtable_size.bin",
    "tests/invalid_vtable_field_offset.bin",
]

for test_file in test_files:
    flatc_annotate("annotated_binary.fbs", test_file)
