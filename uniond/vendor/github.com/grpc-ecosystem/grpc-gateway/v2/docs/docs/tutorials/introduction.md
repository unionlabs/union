---
layout: default
title: Introduction to the gRPC-Gateway
nav_order: 0
parent: Tutorials
---

# Introduction to the gRPC-Gateway

We all know that gRPC is not a tool for everything. There are cases where we still want to provide a traditional HTTP/JSON API. The reasons can range from maintaining backward-compatibility to supporting programming languages or clients not well supported by gRPC. But writing another service just to expose an HTTP/JSON API is quite a time consuming and tedious task.

So is there any way to code just once, but provide APIs in both gRPC and HTTP/JSON at the same time?

The answer is Yes.

The gRPC-Gateway is a plugin of the Google protocol buffers compiler [protoc](https://github.com/protocolbuffers/protobuf). It reads protobuf service definitions and generates a reverse-proxy server which translates a RESTful HTTP API into gRPC. This server is generated according to the [`google.api.http`](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto#L46) annotations in your service definitions.

This helps you provide your APIs in both gRPC and HTTP/JSON format at the same time.

<div align="center">
<img src="../../../assets/images/architecture_introduction_diagram.svg" />
</div>

## Prerequisites

Before we start coding, we have to install some tools.

We will be using a Go gRPC server in the examples, so please install Go first from [https://golang.org/dl/](https://golang.org/dl/).

After installing Go, use `go install` to download and build the following binaries:

```sh
$ go install github.com/grpc-ecosystem/grpc-gateway/v2/protoc-gen-grpc-gateway@latest
$ go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
$ go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

This installs the `protoc` generator plugins we need to generate the stubs. Make sure to add `$GOPATH/bin` to your `$PATH` so that executables installed via `go get` are available on your `$PATH`.

We will be working in a new module for this tutorial, so go ahead and create that in a folder of your choosing now:

### Creating go.mod file

Start your module using the [go mod init command](https://golang.org/cmd/go/#hdr-Initialize_new_module_in_current_directory) to create a go.mod file.

Run the `go mod init` command, giving it the path of the module your code will be in. Here, use github.com/myuser/myrepo for the module path -- in production code, this would be the URL from which your module can be downloaded.

```sh
$ go mod init github.com/myuser/myrepo
go: creating new go.mod: module github.com/myuser/myrepo
```

The `go mod init` command creates a go.mod file that identifies your code as a module that might be used from other code. The file you just created includes only the name of your module and the Go version your code supports. But as you add dependencies -- meaning packages from other modules -- the go.mod file will list the specific module versions to use. This keeps builds reproducible and gives you direct control over which module versions to use.

[Next](simple_hello_world.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
