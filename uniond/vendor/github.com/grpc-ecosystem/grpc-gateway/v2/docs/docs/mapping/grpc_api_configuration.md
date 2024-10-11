---
layout: default
title: gRPC API Configuration
nav_order: 3
parent: Mapping
---

# gRPC API Configuration

In some situations annotating the proto file of service is not an option. For example, you might not have control over the proto file, or you might want to expose the same gRPC API multiple times in completely different ways.

gRPC-Gateway supports 2 ways of dealing with these situations:

- [gRPC API Configuration](#grpc-api-configuration)
  - [`generate_unbound_methods`](#generate_unbound_methods)
  - [Using an external configuration file](#using-an-external-configuration-file)
    - [Usage of gRPC API Configuration YAML files](#usage-of-grpc-api-configuration-yaml-files)

## `generate_unbound_methods`

Providing this parameter to the `protoc` plugin will make it produce the HTTP mapping even for methods without any `HttpRule` annotation. This is similar to how [Cloud Endpoints behaves](https://cloud.google.com/endpoints/docs/grpc/transcoding#where_to_configure_transcoding) and uses the way [gRPC itself](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md) maps to HTTP/2:

- HTTP method is `POST`
- URI path is built from the service's name and method: `/<fully qualified service name>/<method name>` (e.g.: `/my.package.EchoService/Echo`)
- HTTP body is the serialized protobuf message.

NOTE: the same option is also supported by the `gen-openapiv2` plugin.

## Using an external configuration file

Google Cloud Platform offers a way to do this for services
hosted with them called ["gRPC API Configuration"](https://cloud.google.com/endpoints/docs/grpc/grpc-service-config). It can be used to define the behavior of a gRPC API service without modifications to the service itself in the form of [YAML](https://en.wikipedia.org/wiki/YAML) configuration files.

gRPC-Gateway generators implement the [HTTP rules part](https://cloud.google.com/endpoints/docs/grpc-service-config/reference/rpc/google.api#httprule) of this specification. This allows you to take a completely unannotated service proto file, add a YAML file describing its HTTP endpoints and use them together like an annotated proto file with the gRPC-Gateway generators.

OpenAPI options may also be configured via ["OpenAPI Configuration"](https://github.com/grpc-ecosystem/grpc-gateway/tree/main/internal/descriptor/openapiconfig/openapiconfig.proto) in the form of YAML configuration files.

### Usage of gRPC API Configuration YAML files

The following is equivalent to the basic [`README.md`](https://github.com/grpc-ecosystem/grpc-gateway/blob/main/README.md#usage) example but without direct
annotation for gRPC-Gateway in the proto file. Only some steps require minor changes to use a gRPC API Configuration YAML file instead:

1. Define your service in gRPC as usual

   your_service.proto:

   ```protobuf
   syntax = "proto3";
   package your.service.v1;
   option go_package = "github.com/yourorg/yourprotos/gen/go/your/service/v1";
   message StringMessage {
     string value = 1;
   }

   service YourService {
     rpc Echo(StringMessage) returns (StringMessage) {}
   }
   ```

2. Instead of annotating the proto file in this step leave it untouched
   and create a `your_service.yaml` with the following content:

   ```yaml
   type: google.api.Service
   config_version: 3

   http:
     rules:
       - selector: your.service.v1.YourService.Echo
         post: /v1/example/echo
         body: "*"
   ```

   Use a [linter](http://www.yamllint.com/) to validate your YAML.

3. Generate gRPC stub as before

   ```sh
   protoc -I . \
     --go_out ./gen/go/ \
     --go_opt paths=source_relative \
     --go-grpc_out ./gen/go/ \
     --go-grpc_opt paths=source_relative \
     your/service/v1/your_service.proto
   ```

It will generate a stub file with path `./gen/go/your/service/v1/your_service.pb.go`.

4. Implement your service in gRPC as usual

5. Generate the reverse-proxy. Here we have to pass the path to
   the `your_service.yaml` in addition to the proto file:

   ```sh
   protoc -I . \
     --grpc-gateway_out ./gen/go \
     --grpc-gateway_opt logtostderr=true \
     --grpc-gateway_opt paths=source_relative \
     --grpc-gateway_opt grpc_api_configuration=path/to/your_service.yaml \
     your/service/v1/your_service.proto
   ```

   This will generate a reverse proxy `gen/go/your/service/v1/your_service.pb.gw.go` that is identical to the one produced for the annotated proto.

   In situations where you only need the reverse-proxy you can use the `standalone=true` option when generating the code. This will ensure the `types` used within `your_service.pb.gw.go` reference the external source appropriately.

   ```
   protoc -I . \
     --grpc-gateway_out ./gen/go \
     --grpc-gateway_opt logtostderr=true \
     --grpc-gateway_opt paths=source_relative \
     --grpc-gateway_opt standalone=true \
     --grpc-gateway_opt grpc_api_configuration=path/to/your_service.yaml \
     your/service/v1/your_service.proto
   ```

6. Generate the optional your_service.swagger.json

   ```sh
   protoc -I . --openapiv2_out ./gen/go \
     --openapiv2_opt grpc_api_configuration=path/to/your_service.yaml \
     your/service/v1/your_service.proto
   ```

   or using an OpenAPI configuration file

   ```sh
   protoc -I . --openapiv2_out ./gen/go \
     --openapiv2_opt grpc_api_configuration=path/to/your_service.yaml \
     --openapiv2_opt openapi_configuration=path/to/your_service_swagger.yaml \
     your/service/v1/your_service.proto
   ```

   For an example of an OpenAPI configuration file, see [unannotated_echo_service.swagger.yaml](https://github.com/grpc-ecosystem/grpc-gateway/tree/main/examples/internal/proto/examplepb/unannotated_echo_service.swagger.yaml), which adds OpenAPI options to [unannotated_echo_service.proto](https://github.com/grpc-ecosystem/grpc-gateway/tree/main/examples/internal/proto/examplepb/unannotated_echo_service.proto).

   ```sh
   protoc -I . --openapiv2_out ./gen/go \
     --openapiv2_opt grpc_api_configuration=path/to/your_service.yaml \
     your/service/v1/your_service.proto
   ```

All other steps work as before. If you want you can remove the `googleapis` include path in step 3 and 4 as the unannotated proto no longer requires them.
