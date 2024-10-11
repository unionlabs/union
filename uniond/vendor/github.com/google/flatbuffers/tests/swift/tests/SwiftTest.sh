current_dir=`pwd`
cd ..
swift_dir=`pwd`
cd ..
test_dir=`pwd`
alias fbc='${test_dir}/../flatc'
shopt -s expand_aliases

cd ${current_dir}/Tests/FlatBuffers.Test.SwiftTests
fbc --swift --gen-mutable --grpc --gen-json-emit --gen-object-api -I ${test_dir}/include_test ${test_dir}/monster_test.fbs ${test_dir}/union_vector/union_vector.fbs
fbc --swift --gen-json-emit ${test_dir}/optional_scalars.fbs
fbc --swift --gen-json-emit --gen-object-api ${test_dir}/more_defaults.fbs
fbc --swift --gen-json-emit --gen-mutable --gen-object-api ${test_dir}/MutatingBool.fbs
fbc --swift --gen-json-emit ${test_dir}/vector_has_test.fbs
cd ${current_dir}

# Goes into the code generation tests
cd CodeGenerationTests
fbc --swift --gen-mutable --grpc --gen-json-emit --gen-object-api --swift-implementation-only test_import.fbs
fbc --swift --gen-mutable --grpc --gen-json-emit --gen-object-api --no-includes test_no_include.fbs
cd ..

cd ${current_dir}/Sources/SwiftFlatBuffers
# create better fuzzing test file
fbc --swift --gen-json-emit fuzzer.fbs
cd ${current_dir}

cd ${swift_dir}/Wasm.tests/Tests/FlatBuffers.Test.Swift.WasmTests
fbc --swift --gen-mutable --gen-json-emit --gen-object-api -I ${test_dir}/include_test ${test_dir}/monster_test.fbs
cd ${current_dir}

swift build --build-tests
swift test

if [ $(uname -s) != Darwin ]; then
  echo fuzzing
  swift build -c debug -Xswiftc -sanitize=fuzzer,address -Xswiftc -parse-as-library
  swift build -c release -Xswiftc -sanitize=fuzzer,address -Xswiftc -parse-as-library
fi
