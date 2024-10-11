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
import sys
from pathlib import Path

# Get the path where this script is located so we can invoke the script from
# any directory and have the paths work correctly.
script_path = Path(__file__).parent.resolve()

# Get the root path as an absolute path, so all derived paths are absolute.
root_path = script_path.parent.absolute()

result = subprocess.run(["git", "diff", "--quiet", "--ignore-cr-at-eol"], cwd=root_path)

if result.returncode != 0:
    print(
        "\n"
        "ERROR: *********************************************************\n"
        "ERROR: * The following differences were found after building.  *\n"
        "ERROR: * Perhaps there is a difference in the flags for the.   *\n"
        "ERROR: * CMakeLists.txt vs the script/generate_code.py script? *\n"
        "ERROR: *********************************************************\n"
    )
    subprocess.run(["git", "diff", "--binary", "--exit-code"], cwd=root_path)
    sys.exit(result.returncode)

# Rung the generate_code.py script, forwarding arguments
gen_cmd = ["scripts/generate_code.py"] + sys.argv[1:]
if platform.system() == "Windows":
    gen_cmd = ["py"] + gen_cmd
subprocess.run(gen_cmd, cwd=root_path)

result = subprocess.run(["git", "diff", "--quiet", "--ignore-cr-at-eol"], cwd=root_path)

if result.returncode != 0:
    print(
        "\n"
        "ERROR: ********************************************************\n"
        "ERROR: * The following differences were found after running   *\n"
        "ERROR: * the script/generate_code.py script. Maybe you forgot *\n"
        "ERROR: * to run it after making changes in a generator?       *\n"
        "ERROR: ********************************************************\n"
    )
    subprocess.run(["git", "diff", "--binary", "--exit-code"], cwd=root_path)
    sys.exit(result.returncode)

sys.exit(0)
