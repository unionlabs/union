---
layout: default
title: Generating stubs using protoc
nav_order: 1
parent: Generating stubs
grand_parent: Tutorials
---

# Generating stubs using protoc

Here's an example of what a `protoc` command might look like to generate Go stubs, assuming that you're at the root of your repository and you have your proto files in a directory called `proto`:

```sh
$ protoc -I ./proto \
   --go_out ./proto --go_opt paths=source_relative \
   --go-grpc_out ./proto --go-grpc_opt paths=source_relative \
   ./proto/helloworld/hello_world.proto
```

We use the `go` and `go-grpc` plugins to generate Go types and gRPC service definitions. We're outputting the generated files relative to the `proto` folder, and we're using the `paths=source_relative` option, which means that the generated files will appear in the same directory as the source `.proto` file.

This will have generated a `*.pb.go` and a `*_grpc.pb.go` file for `proto/helloworld/hello_world.proto`.

[Next](../creating_main.go.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
