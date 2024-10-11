---
layout: default
title: Generating stubs using buf
nav_order: 0
parent: Generating stubs
grand_parent: Tutorials
---

# Generating stubs using buf

[Buf](https://github.com/bufbuild/buf) is a tool that provides various protobuf utilities such as linting, breaking change detection and generation. Please find installation instructions on [https://docs.buf.build/installation/](https://docs.buf.build/installation/).

It is configured through a `buf.yaml` file that should be checked in to the root of your Protobuf file hierarchy. Buf will automatically read this file if present. Configuration can also be provided via the command-line flag `--config`, which accepts a path to a `.json` or `.yaml` file, or direct JSON or YAML data. As opposed to `protoc`, where all `.proto` files are manually specified on the command-line, buf operates by recursively discovering all `.proto` files under configuration and building them.

The following is an example of a valid configuration, and you would put it in the root of your Protobuf file hierarchy, e.g. in `proto/buf.yaml` relative to the root of your repository.

```yaml
version: v1
name: buf.build/myuser/myrepo
```

To generate type and gRPC stubs for Go, create the file `buf.gen.yaml`:

```yaml
version: v1
plugins:
  - plugin: go
    out: proto
    opt: paths=source_relative
  - plugin: go-grpc
    out: proto
    opt: paths=source_relative
```

We use the `go` and `go-grpc` plugins to generate Go types and gRPC service definitions. We're outputting the generated files relative to the `proto` folder, and we're using the `paths=source_relative` option, which means that the generated files will appear in the same directory as the source `.proto` file.

Then run

```sh
$ buf generate
```

This will have generated a `*.pb.go` and a `*_grpc.pb.go` file for each protobuf package in our `proto` file hierarchy.

[Next](../creating_main.go.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
