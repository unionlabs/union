# Protocol Buffers for Go with Gadgets

Cosmos's fork of [gogo/protobuf](https://github.com/gogo/protobuf).

[![Build Status](https://github.com/cosmos/gogoproto/workflows/Continuous%20Integration/badge.svg)](https://github.com/cosmos/gogoproto/actions)
[![GoDoc](https://godoc.org/github.com/cosmos/gogoproto?status.svg)](http://godoc.org/github.com/cosmos/gogoproto)

This code generation is used to achieve:

- fast marshalling and unmarshalling
- more canonical Go structures
- goprotobuf compatibility
- less typing by optionally generating extra helper code
- peace of mind by optionally generating test and benchmark code
- other serialization formats

More information in the [original readme](https://github.com/gogo/protobuf/blob/master/README).

## Getting Started

There are several ways to use gogoprotobuf, but for all you need to install go and protoc.
After that you can choose:

  - Speed
  - More Speed and more generated code
  - Most Speed and most customization

### Installation

To install it, you must first have Go (at least version 1.6.3 or 1.9 if you are using gRPC) installed (see [http://golang.org/doc/install](http://golang.org/doc/install)).
Latest patch versions of 1.12 and 1.15 are continuously tested.

Next, install the standard protocol buffer implementation from [https://github.com/google/protobuf](https://github.com/google/protobuf).
Most versions from 2.3.1 should not give any problems, but 2.6.1, 3.0.2 and 3.14.0 are continuously tested.

### Speed

Install the protoc-gen-gofast binary

    go get github.com/cosmos/gogoproto/protoc-gen-gofast

Use it to generate faster marshaling and unmarshaling go code for your protocol buffers.

    protoc --gofast_out=. myproto.proto

This does not allow you to use any of the other gogoprotobuf [extensions](https://github.com/cosmos/gogoproto/blob/master/extensions.md).

### More Speed and more generated code

Fields without pointers cause less time in the garbage collector.
More code generation results in more convenient methods.

Other binaries are also included:

    protoc-gen-gogofast (same as gofast, but imports gogoprotobuf)
    protoc-gen-gogofaster (same as gogofast, without XXX_unrecognized, less pointer fields)
    protoc-gen-gogoslick (same as gogofaster, but with generated string, gostring and equal methods)

Installing any of these binaries is easy.  Simply run:

    go get github.com/cosmos/gogoproto/proto
    go get github.com/cosmos/gogoproto/{binary}
    go get github.com/cosmos/gogoproto/gogoproto

These binaries allow you to use gogoprotobuf [extensions](https://github.com/cosmos/gogoproto/blob/master/extensions.md). You can also use your own binary.

To generate the code, you also need to set the include path properly.

    protoc -I=. -I=$GOPATH/src -I=$GOPATH/src/github.com/cosmos/gogoproto/protobuf --{binary}_out=. myproto.proto

To use proto files from "google/protobuf" you need to add additional args to protoc.

    protoc -I=. -I=$GOPATH/src -I=$GOPATH/src/github.com/cosmos/gogoproto/protobuf --{binary}_out=\
    Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types,\
    Mgoogle/protobuf/duration.proto=github.com/cosmos/gogoproto/types,\
    Mgoogle/protobuf/struct.proto=github.com/cosmos/gogoproto/types,\
    Mgoogle/protobuf/timestamp.proto=github.com/cosmos/gogoproto/types,\
    Mgoogle/protobuf/wrappers.proto=github.com/cosmos/gogoproto/types:. \
    myproto.proto

Note that in the protoc command, {binary} does not contain the initial prefix of "protoc-gen".

### Most Speed and most customization

Customizing the fields of the messages to be the fields that you actually want to use removes the need to copy between the structs you use and structs you use to serialize.
gogoprotobuf also offers more serialization formats and generation of tests and even more methods.

Please visit the [extensions](https://github.com/cosmos/gogoproto/blob/master/extensions.md) page for more documentation.

Install protoc-gen-gogo:

    go get github.com/cosmos/gogoproto/proto
    go get github.com/cosmos/gogoproto/jsonpb
    go get github.com/cosmos/gogoproto/protoc-gen-gogo
    go get github.com/cosmos/gogoproto/gogoproto
