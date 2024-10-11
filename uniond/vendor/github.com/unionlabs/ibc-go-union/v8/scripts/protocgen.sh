#!/usr/bin/env bash

set -eo pipefail

echo "Generating gogo proto code"
cd proto

buf generate --template buf.gen.gogo.yaml $file

cd ..

# move proto files to the right places
cp -r github.com/cosmos/ibc-go/v*/modules/* modules/
cp -r github.com/cosmos/ibc-go/modules/* modules/
rm -rf github.com

go mod tidy
