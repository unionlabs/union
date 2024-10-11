#!/bin/bash
set -xe

# We use fuzzit fork until go-fuzz will support go-modules
mkdir -p /go/src/github.com/dvyukov
cd /go/src/github.com/dvyukov
git clone https://github.com/fuzzitdev/go-fuzz
cd go-fuzz
go get ./...
go build ./...

#go get -v -u ./protoc-gen-grpc-gateway/httprule
cd /src/grpc-gateway
go-fuzz-build -libfuzzer -o parse-http-rule.a ./protoc-gen-grpc-gateway/httprule
clang-9 -fsanitize=fuzzer parse-http-rule.a -o parse-http-rule

wget -q -O fuzzit https://github.com/fuzzitdev/fuzzit/releases/download/v2.4.29/fuzzit_Linux_x86_64
chmod a+x fuzzit

if [ -z "CIRCLE_PULL_REQUEST" ]; then
    TYPE="fuzzing"
else
    TYPE="local-regression"
fi
./fuzzit create job --type ${TYPE} grpc-gateway/parse-http-rule parse-http-rule
