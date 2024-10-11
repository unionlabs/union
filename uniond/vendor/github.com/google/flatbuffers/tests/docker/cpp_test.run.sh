set -e

JOBS=${JOBS:-1}
export UBSAN_OPTIONS=halt_on_error=1
export ASAN_OPTIONS=halt_on_error=1
export MAKEFLAGS="-j$JOBS"

config=$1
echo ""
echo "Build Flatbuffers project for '$config' with jobs=$JOBS"

cmake . -DCMAKE_BUILD_TYPE=$config \
  -DFLATBUFFERS_BUILD_TESTS=ON -DFLATBUFFERS_CODE_SANITIZE=ON
cmake --build . --target all --clean-first -- -j$JOBS
ctest --extra-verbose --output-on-failure -j$JOBS

echo "Checking generated code"
scripts/check_generate_code.py

echo "C++ tests done"
