# Pulsar

## Installing

go install github.com/cosmos/cosmos-proto/cmd/protoc-gen-go-pulsar

## Running

cd path/to/proto/files

protoc --go-pulsar_out=. --go-pulsar_opt=paths=source_relative --go-pulsar_opt=features=protoc+fast -I .
NAME_OF_FILE.proto

## Acknowledgements

Code for the generator structure/features and the functions marshal, unmarshal, and size implemented by [planetscale/vtprotobuf](https://github.com/planetscale/vtprotobuf) was used in our `ProtoMethods` implementation.

Code used to produce default code stubs found in [protobuf](https://pkg.go.dev/google.golang.org/protobuf) was copied into [features/protoc](./features/protoc).
