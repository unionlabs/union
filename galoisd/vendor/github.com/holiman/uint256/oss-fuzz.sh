#!/bin/bash -eu

# This sets the -coverpgk for the coverage report when the corpus is executed through go test
coverpkg="github.com/holiman/uint256/..."

(cd /src/ && git clone https://github.com/holiman/gofuzz-shim.git )

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
  cp /src/gofuzz-shim/coverage_runner_template.txt ./"${function,,}"_test.go
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

repo=/src/uint256

function compile_fuzzer() {
  package=$1
  function=$2
  fuzzer=$3
  file=$4

  path=$repo

  echo "Building $fuzzer"
  cd $path

  # Install build dependencies
  go mod tidy
  go get github.com/holiman/gofuzz-shim/testing

	if [[ $SANITIZER == *coverage* ]]; then
		coverbuild $path $function $fuzzer $coverpkg
	else
	  gofuzz-shim --func $function --package $package -f $file -o $fuzzer.a
		$CXX $CXXFLAGS $LIB_FUZZING_ENGINE $fuzzer.a -o $OUT/$fuzzer
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

go install github.com/holiman/gofuzz-shim@latest


compile_fuzzer github.com/holiman/uint256  FuzzUnaryOperations fuzzUnary $repo/unary_test.go,$repo/shared_test.go
compile_fuzzer github.com/holiman/uint256  FuzzBinaryOperations fuzzBinary $repo/binary_test.go,$repo/shared_test.go
compile_fuzzer github.com/holiman/uint256  FuzzCompareOperations fuzzCompare $repo/binary_test.go,$repo/shared_test.go
compile_fuzzer github.com/holiman/uint256  FuzzTernaryOperations fuzzTernary $repo/ternary_test.go,$repo/shared_test.go
compile_fuzzer github.com/holiman/uint256  FuzzSetString fuzzSetString $repo/conversion_fuzz_test.go,$repo/shared_test.go
