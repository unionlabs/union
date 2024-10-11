set -e

JOBS=${JOBS:-1}
config=$1
echo ""
echo "Build 'flatc' compiler for '$config'"

cmake . -DCMAKE_BUILD_TYPE=$config \
  -DFLATBUFFERS_BUILD_FLATC=1 -DFLATBUFFERS_STATIC_FLATC=1 \
  -DFLATBUFFERS_BUILD_TESTS=0 -DFLATBUFFERS_INSTALL=0
cmake --build . --target flatc --clean-first -- -j$JOBS

echo "Checking generated code"
scripts/check_generate_code.py
echo "Done"
