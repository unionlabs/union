#!/bin/sh
#
# Copyright 2016 Google Inc. All rights reserved.
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

pushd "$(dirname $0)" >/dev/null

command -v dart >/dev/null 2>&1 || { echo >&2 "Dart tests require dart to be in path but it's not installed.  Aborting."; exit 1; }
# output required files to the dart folder so that pub will be able to
# distribute them and more people can more easily run the dart tests
../flatc --dart --gen-object-api -I include_test -o ../dart/test monster_test.fbs
../flatc --dart --gen-object-api -I include_test/sub -o ../dart/test include_test/include_test1.fbs
../flatc --dart --gen-object-api -I include_test -o ../dart/test include_test/sub/include_test2.fbs

cp monsterdata_test.mon ../dart/test
cp monster_test.fbs ../dart/test

cd ../dart

../flatc --dart --gen-object-api -o ./test ./test/list_of_enums.fbs
../flatc --dart --gen-object-api -o ./test ./test/bool_structs.fbs

# update packages
dart pub get
# Execute the sample.
dart test
