#!/bin/sh

set -e

build() {
    echo finding protobuf files in "$1"
    proto_files=$(find "$1" -name "*.proto")
    for file in $proto_files; do
      echo "building proto file $file"
      protoc -I=. --plugin /usr/bin/protoc-gen-go-pulsar --go-pulsar_out=. --go-pulsar_opt=features=protoc+fast "$file"
    done
}

for dir in "$@"
do
  build "$dir"
done

cp -r github.com/cosmos/cosmos-proto/* ./
rm -rf github.com