#!/bin/bash -eu
# Copyright 2023 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
################################################################################

# This script is meant to be run by
# https://github.com/google/oss-fuzz/tree/master/projects/compress

# In one of the Zstd fuzzers, the "dict" variable is created by reading a series of files.
# These files are not available at runtime in the OSS-FUzz environment,
# so we add the contents of these files to a variable and create a new file
# that is included when we build the fuzzers in OSS-Fuzz.
mkdir $SRC/setupdicts
cp $SRC/compress/ossfuzz/cmd/setup_dicts.go $SRC/setupdicts/main.go
cd $SRC/setupdicts
go mod init setupdicts
go mod tidy
go run main.go --dict-path=$SRC/compress/zstd/testdata/dict-tests-small.zip --output-file=$SRC/compress/zstd/fuzzDicts.go
cp $SRC/compress/zstd/fuzzDicts.go $OUT/
# Done creating "dicts" variable.

cd $SRC/compress

# Modify some files. This would be better done upstream.
printf "package compress\nimport _ \"github.com/AdamKorcz/go-118-fuzz-build/testing\"\n" > registerfuzzdependency.go
sed -i 's/zr := testCreateZipReader/\/\/zr := testCreateZipReader/g' "${SRC}"/compress/zstd/fuzz_test.go
sed -i 's/dicts = readDicts(f, zr)/dicts = fuzzDicts/g' "${SRC}"/compress/zstd/fuzz_test.go

if [ "$SANITIZER" != "coverage" ]; then
	sed -i 's/\"testing\"/\"github.com\/AdamKorcz\/go-118-fuzz-build\/testing\"/g' "${SRC}"/compress/internal/fuzz/helpers.go
	printf "\n\nreplace github.com/AdamKorcz/go-118-fuzz-build => github.com/klauspost/go-118-fuzz-build d5f2eff5a9ec105b249e0bb1a24c1725330ed424\n" >> go.mod
fi

# OSS-Fuzz uses 'go build' to build the fuzzers, so we move the tests
# we need into scope.
mv $SRC/compress/zstd/decoder_test.go $SRC/compress/zstd/decoder_test_fuzz.go
mv $SRC/compress/zstd/zstd_test.go $SRC/compress/zstd/zstd_test_fuzz.go
mv $SRC/compress/zstd/seqdec_test.go $SRC/compress/zstd/seqdec_test_fuzz.go
mv $SRC/compress/zstd/dict_test.go $SRC/compress/zstd/dict_test_fuzz.go
mv $SRC/compress/s2/s2_test.go $SRC/compress/s2/s2_test_fuzz.go
go mod tidy

# Build fuzzers
compile_native_go_fuzzer $SRC/compress/flate FuzzEncoding FuzzFlateEncoding
compile_native_go_fuzzer $SRC/compress/zstd FuzzDecodeAll FuzzDecodeAll
compile_native_go_fuzzer $SRC/compress/zstd FuzzDecoder FuzzDecoder
compile_native_go_fuzzer $SRC/compress/zstd FuzzEncoding FuzzZstdEncoding
compile_native_go_fuzzer $SRC/compress/zstd FuzzDecAllNoBMI2 FuzzDecAllNoBMI2
compile_native_go_fuzzer $SRC/compress/zstd FuzzNoBMI2Dec FuzzNoBMI2Dec
compile_native_go_fuzzer $SRC/compress/s2 FuzzDictBlocks FuzzDictBlocks
compile_native_go_fuzzer $SRC/compress/s2 FuzzEncodingBlocks FuzzEncodingBlocks
compile_native_go_fuzzer $SRC/compress/zip FuzzReader FuzzReader
compile_native_go_fuzzer $SRC/compress/snappy/xerial FuzzDecode FuzzDecode
compile_native_go_fuzzer $SRC/compress/fse FuzzCompress FuzzFSECompress
compile_native_go_fuzzer $SRC/compress/fse FuzzDecompress FuzzFSEDecompress
compile_native_go_fuzzer $SRC/compress/huff0 FuzzCompress FuzzHuff0Compress
compile_native_go_fuzzer $SRC/compress/huff0 FuzzDecompress1x FuzzHuff0Decompress1x
compile_native_go_fuzzer $SRC/compress/snappy/xerial FuzzEncode FuzzEncode

#Add corpora from compress-fuzz dir
cp $SRC/compress-fuzz/zstd/compress/fuzz/encode-corpus-raw.zip $OUT/FuzzZstdEncoding_seed_corpus.zip
cp $SRC/compress-fuzz/zstd/decompress/fuzz/decode-corpus-raw.zip $OUT/FuzzDecodeAll_seed_corpus.zip
cp $SRC/compress-fuzz/zstd/decompress/fuzz/decode-corpus-raw.zip $OUT/FuzzDecoder_seed_corpus.zip
cp $SRC/compress-fuzz/zip/fuzz/FuzzReader-raw.zip $OUT/FuzzReader_seed_corpus.zip
cp $SRC/compress-fuzz/fse/compress/fuzz/fse_compress.zip $OUT/FuzzFSECompress_seed_corpus.zip
cp $SRC/compress-fuzz/fse/decompress/fuzz/fse_decompress.zip $OUT/FuzzFSEDecompress_seed_corpus.zip
cp $SRC/compress-fuzz/huff0/compress/fuzz/huff0_compress.zip $OUT/FuzzHuff0Compress_seed_corpus.zip
cp $SRC/compress-fuzz/huff0/decompress/fuzz/huff0_decompress1x.zip $OUT/FuzzHuff0Decompress1x_seed_corpus.zip
cp $SRC/compress-fuzz/flate/flate/fuzz/encode-raw-corpus.zip $OUT/FuzzFlateEncoding_seed_corpus.zip
cp $SRC/compress-fuzz/s2/compress/fuzz/block-corpus-raw.zip $OUT/FuzzDictBlocks_seed_corpus.zip
cp $SRC/compress-fuzz/s2/compress/fuzz/block-corpus-raw.zip $OUT/FuzzEncodingBlocks_seed_corpus.zip
cp $SRC/compress-fuzz/snappy/fuzz/FuzzDecode_raw.zip $OUT/FuzzDecode_seed_corpus.zip
cp $SRC/compress-fuzz/snappy/fuzz/block-corpus-raw.zip $OUT/FuzzEncode_seed_corpus.zip

# Add missing test files to avoid errors and have test files for code coverage
cp -r $SRC/compress/zstd/testdata $OUT/
cp -r $SRC/compress/zip/testdata $OUT/
cp -r $SRC/compress/flate/testdata $OUT/
cp -r $SRC/compress/s2/testdata $OUT/
cp -r $SRC/compress/fse/testdata $OUT/
cp -r $SRC/compress/snappy/xerial/testdata $OUT/
cp -r $SRC/compress/huff0/testdata $OUT/
cp -r $SRC/compress/testdata $OUT
