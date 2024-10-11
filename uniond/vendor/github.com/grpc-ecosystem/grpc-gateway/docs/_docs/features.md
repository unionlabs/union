---
category: documentation
---

# Features

## Supported
* Generating JSON API handlers
* Method parameters in request body
* Method parameters in request path
* Method parameters in query string
* Enum fields in path parameter (including repeated enum fields).
* Mapping streaming APIs to newline-delimited JSON streams
* Mapping HTTP headers with `Grpc-Metadata-` prefix to gRPC metadata (prefixed with `grpcgateway-`)
* Optionally emitting API definition for [Swagger](http://swagger.io).
* Setting [gRPC timeouts](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md) through inbound HTTP `Grpc-Timeout` header.
* Partial support for [gRPC API Configuration](https://cloud.google.com/endpoints/docs/grpc/grpc-service-config) files as an alternative to annotation.

## Want to support
But not yet.
* Optionally generating the entrypoint. #8
* `import_path` parameter

## No plan to support
But patch is welcome.
* Method parameters in HTTP headers
* Handling trailer metadata
* Encoding request/response body in XML
* True bi-directional streaming. (Probably impossible?)

