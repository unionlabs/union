#!/usr/bin/env python3
#
# Copyright 2022 Google Inc. All rights reserved.
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

import subprocess
import sys
import generate_grpc_examples
from pathlib import Path

# Get the path where this script is located so we can invoke the script from
# any directory and have the paths work correctly.
script_path = Path(__file__).parent.resolve()

# Get the root path as an absolute path, so all derived paths are absolute.
root_path = script_path.parent.absolute()

print("Generating GRPC code...")
generate_grpc_examples.GenerateGRPCExamples()

result = subprocess.run(["git", "diff", "--quiet", "--ignore-cr-at-eol"], cwd=root_path)

if result.returncode != 0:
    print(
        "\n"
        "ERROR: ********************************************************\n"
        "ERROR: * The following differences were found after running   *\n"
        "ERROR: * the script/generate_grpc_examples.py script. Maybe   *\n"
        "ERROR: * you forgot to run it after making changes in a       *\n"
        "ERROR: * generator or schema?                                 *\n"
        "ERROR: ********************************************************\n"
    )
    subprocess.run(["git", "diff", "--binary", "--exit-code"], cwd=root_path)
    sys.exit(result.returncode)
