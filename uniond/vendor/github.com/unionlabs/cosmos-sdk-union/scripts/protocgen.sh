#!/usr/bin/env bash

# How to run manually:
# docker build --pull --rm -f "contrib/devtools/Dockerfile" -t cosmossdk-proto:latest "contrib/devtools"
# docker run --rm -v $(pwd):/workspace --workdir /workspace cosmossdk-proto sh ./scripts/protocgen.sh

echo "Formatting protobuf files"
find ./ -name "*.proto" -not -path '*/vendor/*' -exec clang-format -i {} \;

set -e

home=$PWD

echo "Generating proto code"
proto_dirs=$(find ./ -name 'buf.yaml' -not -path '*/vendor/*' -print0 | xargs -0 -n1 dirname | sort | uniq)
for dir in $proto_dirs; do
  echo "Generating proto code for $dir"

  cd $dir
  echo $(pwd)
  # check if buf.gen.pulsar.yaml exists in the proto directory
  if [ -f "buf.gen.pulsar.yaml" ]; then
    buf generate --template buf.gen.pulsar.yaml
    # move generated files to the right places
    if [ -d "../cosmos" -a "$dir" != "./proto" ]; then
      cp -r ../cosmos $home/api
      rm -rf ../cosmos
    fi
  fi

  # check if buf.gen.gogo.yaml exists in the proto directory
  if [ -f "buf.gen.gogo.yaml" ]; then
      for file in $(find . -maxdepth 8 -name '*.proto' -not -path '*/vendor/*'); do
        # this regex checks if a proto file has its go_package set to cosmossdk.io/api/...
        # gogo proto files SHOULD ONLY be generated if this is false
        # we don't want gogo proto to run for proto files which are natively built for google.golang.org/protobuf
        if grep -q "option go_package" "$file" && grep -H -o -c 'option go_package.*cosmossdk.io/api' "$file" | grep -q ':0$'; then
          buf generate --template buf.gen.gogo.yaml $file
        fi
    done

    # move generated files to the right places
    if [ -d "../cosmossdk.io" ]; then
      cp -r ../cosmossdk.io/* $home
      rm -rf ../cosmossdk.io
    fi

    if [ -d "../github.com" -a "$dir" != "./proto" ]; then
      cp -r ../github.com/cosmos/cosmos-sdk/* $home
      rm -rf ../github.com
    fi
  fi

  cd $home
done

# move generated files to the right places
cp -r github.com/cosmos/cosmos-sdk/* ./
rm -rf github.com

go mod tidy

rm -rf api store depinject server/v2/streaming # remove directories that are not needed in release/v0.52.x
