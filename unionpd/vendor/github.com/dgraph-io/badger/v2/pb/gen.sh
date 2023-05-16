#!/bin/bash

# Run this script from its directory, so that badgerpb2.proto is where it's expected to
# be.

# You might need to go get -v github.com/gogo/protobuf/...

protoc --gofast_out=plugins=grpc:. --gofast_opt=paths=source_relative -I=. badgerpb2.proto
