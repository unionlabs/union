# Server

[![GoDoc](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=shield)](https://pkg.go.dev/github.com/coinbase/rosetta-sdk-go/server?tab=doc)

The Server package reduces the work required to write your own Rosetta server.
In short, this package takes care of the basics (boilerplate server code
and request validation) so that you can focus on code that is unique to your
implementation.

## Installation

```shell
go get github.com/coinbase/rosetta-sdk-go/server
```

## Components
### Router
The router is a [Mux](https://github.com/gorilla/mux) router that
routes traffic to the correct controller.

### Controller
Controllers are automatically generated code that specify an interface
that a service must implement.

### Services
Services are implemented by you to populate responses. These services
are invoked by controllers.

## Recommended Folder Structure
```
main.go
/services
  block_service.go
  network_service.go
  ...
```

## Examples
Check out the [examples](/examples) to see how easy
it is to create your own server.
