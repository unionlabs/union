#!/bin/bash

currentSpec=$(cat ./spectests/structs_encoding.go)

# Generate the specs again
make build-spec-tests

realSpec=$(cat ./spectests/structs_encoding.go)

if [ "$currentSpec" == "$realSpec" ]; then
    echo "Specs are equal."
else
    echo "Specs are not equal."
    exit 1
fi
