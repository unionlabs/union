---
title: Usage without annotations
category: documentation
order: 100
---

# gRPC API Configuration
In some situations annotating the .proto file of a service is not an option. For example, you might not have control over the .proto file, or you might want to expose the same gRPC API multiple times in completely different ways.

`grpc-gateway` supports 2 ways of dealing with these situations:

* [use the `generate_unbound_methods` option](#generate_unbound_methods)
* [provide an external configuration file](#using-an-external-configuration-file) (gRPC API Configuration)

## `generate_unbound_methods`

Providing this parameter to the protoc plugin will make it produce the HTTP mapping even for methods without any `HttpRule` annotation.
This is similar to how [Cloud Endpoints behaves](https://cloud.google.com/endpoints/docs/grpc/transcoding#where_to_configure_transcoding) and uses the way [gRPC itself](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md) maps to HTTP/2:

* HTTP method is `POST`
* URI path is built from the service's name and method: `/<fully qualified service name>/<method name>` (e.g.: `/my.package.EchoService/Echo`)
* HTTP body is the serialized protobuf message.

NOTE: the same option is also supported by the `gen-swagger` plugin.

## Using an external configuration file
Google Cloud Platform offers a way to do this for services hosted with them called ["gRPC API Configuration"](https://cloud.google.com/endpoints/docs/grpc/grpc-service-config). It can be used to define the behavior of a gRPC API service without modifications to the service itself in the form of [YAML](https://en.wikipedia.org/wiki/YAML) configuration files.

grpc-gateway generators implement the [HTTP rules part](https://cloud.google.com/endpoints/docs/grpc-service-config/reference/rpc/google.api#httprule) of this specification. This allows you to take a completely unannotated service proto file, add a YAML file describing its HTTP endpoints and use them together like a annotated proto file with the grpc-gateway generators.

### Usage of gRPC API Configuration YAML files
The following is equivalent to the basic [usage example](usage.html) but without direct annotation for grpc-gateway in the .proto file. Only some steps require minor changes to use a gRPC API Configuration YAML file instead:

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

2. Instead of annotating the .proto file in this step leave it untouched
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
    protoc -I. --go_out=plugins=grpc,paths=source_relative:./gen/go/ your/service/v1/your_service.proto
    ```
   
  It will generate a stub file with path `./gen/go/your/service/v1/your_service.pb.go`.

4. Implement your service in gRPC as usual

5. Generate the reverse-proxy. Here we have to pass the path to
    the `your_service.yaml` in addition to the .proto file:

    ```sh
    protoc -I. --grpc-gateway_out=logtostderr=true,paths=source_relative,grpc_api_configuration=path/to/your_service.yaml:./gen/go \
      your/service/v1/your_service.proto
    ```
   
   This will generate a reverse proxy `gen/go/your/service/v1/your_service.pb.gw.go` that is identical to the one produced for the annotated proto.

6. Generate the optional your_service.swagger.json

    ```sh
    protoc -I. --swagger_out=grpc_api_configuration=path/to/your_service.yaml:./gen/go \
      your/service/v1/your_service.proto
    ```
    
All other steps work as before. If you want you can remove the googleapis include path in step 3 and 4 as the unannotated proto no longer requires them.
