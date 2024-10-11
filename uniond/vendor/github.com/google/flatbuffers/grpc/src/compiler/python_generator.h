/*
 *
 * Copyright 2015 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

#ifndef GRPC_INTERNAL_COMPILER_PYTHON_GENERATOR_H
#define GRPC_INTERNAL_COMPILER_PYTHON_GENERATOR_H

#include <utility>

#include "src/compiler/schema_interface.h"

namespace grpc_python_generator {

grpc::string Generate(grpc_generator::File *file,
                      const grpc_generator::Service *service);
}  // namespace grpc_python_generator

#endif  // GRPC_INTERNAL_COMPILER_PYTHON_GENERATOR_H
