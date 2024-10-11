---
layout: default
title: Creating a simple hello world with gRPC
nav_order: 1
parent: Tutorials
---

# Creating a simple hello world with gRPC

To understand the gRPC-Gateway we are going to first make a hello world gRPC service.

## Defining your gRPC service using protocol buffers

Before we create a gRPC service, we should create a proto file to define what we need, here we create a file named `hello_world.proto` in the directory `proto/helloworld/hello_world.proto`.

The gRPC service is defined using [Google Protocol Buffers](https://developers.google.com/protocol-buffers). To learn more about how to define a service in a `.proto` file see their [Basics tutorial](https://grpc.io/docs/languages/go/basics/). For now, all you need to know is that both the server and the client stub have a `SayHello()` RPC method that takes a `HelloRequest` parameter from the client and returns a `HelloReply` from the server, and that the method is defined like this:

```protobuf
syntax = "proto3";

package helloworld;

// The greeting service definition
service Greeter {
  // Sends a greeting
  rpc SayHello (HelloRequest) returns (HelloReply) {}
}

// The request message containing the user's name
message HelloRequest {
  string name = 1;
}

// The response message containing the greetings
message HelloReply {
  string message = 1;
}
```

[Next](generating_stubs/index.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
