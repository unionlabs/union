#!/bin/bash -eu
# Copyright 2022 Google LLC
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

# This sets the -coverpgk for the coverage report when the corpus is executed through go test
coverpkg="github.com/ethereum/go-ethereum/..."

function coverbuild {
  path=$1
  function=$2
  fuzzer=$3
  tags=""

  if [[ $#  -eq 4 ]]; then
    tags="-tags $4"
  fi
  cd $path
  fuzzed_package=`pwd | rev | cut -d'/' -f 1 | rev`
  cp $GOPATH/ossfuzz_coverage_runner.go ./"${function,,}"_test.go
  sed -i -e 's/FuzzFunction/'$function'/' ./"${function,,}"_test.go
  sed -i -e 's/mypackagebeingfuzzed/'$fuzzed_package'/' ./"${function,,}"_test.go
  sed -i -e 's/TestFuzzCorpus/Test'$function'Corpus/' ./"${function,,}"_test.go

cat << DOG > $OUT/$fuzzer
#/bin/sh

  cd $OUT/$path
  go test -run Test${function}Corpus -v $tags -coverprofile \$1 -coverpkg $coverpkg

DOG

  chmod +x $OUT/$fuzzer
  #echo "Built script $OUT/$fuzzer"
  #cat $OUT/$fuzzer
  cd -
}

function build_native_go_fuzzer() {
	fuzzer=$1
	function=$2
	path=$3
	tags="-tags gofuzz"

	if [[ $SANITIZER == *coverage* ]]; then
		coverbuild $path $function $fuzzer $coverpkg
	else
		go-118-fuzz-build $tags -o $fuzzer.a -func $function $path
		$CXX $CXXFLAGS $LIB_FUZZING_ENGINE $fuzzer.a -o $OUT/$fuzzer
	fi
}

function compile_fuzzer() {
  path=$GOPATH/src/github.com/ethereum/go-ethereum/$1
  function=$2
  fuzzer=$3

  echo "Building $fuzzer"
  cd $path

  # Install build dependencies
  go install github.com/AdamKorcz/go-118-fuzz-build@latest
  go get github.com/AdamKorcz/go-118-fuzz-build/testing

  # Test if file contains a line with "func $function(" and "testing.F".
  if [ $(grep -r "func $function(" $path | grep "testing.F" | wc -l) -eq 1 ]
  then
    build_native_go_fuzzer $fuzzer $function $path
  else
    echo "Could not find the function: func ${function}(f *testing.F)"
  fi

  ## Check if there exists a seed corpus file
  corpusfile="${path}/testdata/${fuzzer}_seed_corpus.zip"
  if [ -f $corpusfile ]
  then
    cp $corpusfile $OUT/
    echo "Found seed corpus: $corpusfile"
  fi
  cd -
}

compile_fuzzer tests/fuzzers/bitutil  FuzzEncoder      fuzzBitutilEncoder
compile_fuzzer tests/fuzzers/bitutil  FuzzDecoder      fuzzBitutilDecoder
compile_fuzzer tests/fuzzers/bn256    FuzzAdd   fuzzBn256Add
compile_fuzzer tests/fuzzers/bn256    FuzzMul   fuzzBn256Mul
compile_fuzzer tests/fuzzers/bn256    FuzzPair  fuzzBn256Pair
compile_fuzzer tests/fuzzers/runtime  Fuzz      fuzzVmRuntime
compile_fuzzer tests/fuzzers/keystore   Fuzz fuzzKeystore
compile_fuzzer tests/fuzzers/txfetcher  Fuzz fuzzTxfetcher
compile_fuzzer tests/fuzzers/rlp        Fuzz fuzzRlp
compile_fuzzer tests/fuzzers/trie       Fuzz fuzzTrie
compile_fuzzer tests/fuzzers/stacktrie  Fuzz fuzzStackTrie
compile_fuzzer tests/fuzzers/difficulty Fuzz fuzzDifficulty
compile_fuzzer tests/fuzzers/abi        Fuzz fuzzAbi
compile_fuzzer tests/fuzzers/les        Fuzz fuzzLes
compile_fuzzer tests/fuzzers/secp256k1  Fuzz fuzzSecp256k1
compile_fuzzer tests/fuzzers/vflux      FuzzClientPool fuzzClientPool

compile_fuzzer tests/fuzzers/bls12381  FuzzG1Add fuzz_g1_add
compile_fuzzer tests/fuzzers/bls12381  FuzzG1Mul fuzz_g1_mul
compile_fuzzer tests/fuzzers/bls12381  FuzzG1MultiExp fuzz_g1_multiexp
compile_fuzzer tests/fuzzers/bls12381  FuzzG2Add fuzz_g2_add
compile_fuzzer tests/fuzzers/bls12381  FuzzG2Mul fuzz_g2_mul
compile_fuzzer tests/fuzzers/bls12381  FuzzG2MultiExp fuzz_g2_multiexp
compile_fuzzer tests/fuzzers/bls12381  FuzzPairing fuzz_pairing
compile_fuzzer tests/fuzzers/bls12381  FuzzMapG1 fuzz_map_g1
compile_fuzzer tests/fuzzers/bls12381  FuzzMapG2 fuzz_map_g2

compile_fuzzer tests/fuzzers/bls12381  FuzzCrossG1Add fuzz_cross_g1_add
compile_fuzzer tests/fuzzers/bls12381  FuzzCrossG1MultiExp fuzz_cross_g1_multiexp
compile_fuzzer tests/fuzzers/bls12381  FuzzCrossG2Add fuzz_cross_g2_add
compile_fuzzer tests/fuzzers/bls12381  FuzzCrossPairing fuzz_cross_pairing

compile_fuzzer tests/fuzzers/snap  FuzzARange fuzz_account_range
compile_fuzzer tests/fuzzers/snap  FuzzSRange fuzz_storage_range
compile_fuzzer tests/fuzzers/snap  FuzzByteCodes fuzz_byte_codes
compile_fuzzer tests/fuzzers/snap  FuzzTrieNodes fuzz_trie_nodes

#TODO: move this to tests/fuzzers, if possible
compile_fuzzer crypto/blake2b  Fuzz      fuzzBlake2b
