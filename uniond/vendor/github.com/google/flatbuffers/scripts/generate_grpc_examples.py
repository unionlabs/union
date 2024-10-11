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

from util import flatc, root_path
from pathlib import Path

grpc_examples_path = Path(root_path, "grpc/examples")

greeter_schema = str(Path(grpc_examples_path, "greeter.fbs"))

COMMON_ARGS = [
    "--grpc",
    "--bfbs-filenames",
    str(grpc_examples_path),
]

def GenerateGRPCExamples():

    flatc(
        COMMON_ARGS
        + [
            "--go",
        ],
        schema=greeter_schema,
        cwd=Path(grpc_examples_path, "go/greeter"),
    )

    flatc(
        COMMON_ARGS
        + [
            "--python",
        ],
        schema=greeter_schema,
        cwd=Path(grpc_examples_path, "python/greeter"),
    )

    flatc(
        COMMON_ARGS
        + [
            "--swift",
            "--gen-json-emit",
        ],
        schema=greeter_schema,
        cwd=Path(grpc_examples_path, "swift/Greeter/Sources/Model"),
    )

    flatc(
        COMMON_ARGS
        + [
            "--ts",
        ],
        schema=greeter_schema,
        cwd=Path(grpc_examples_path, "ts/greeter/src"),
    )

if __name__ == "__main__":
    GenerateGRPCExamples()
