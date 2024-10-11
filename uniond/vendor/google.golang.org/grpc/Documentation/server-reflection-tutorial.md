# gRPC Server Reflection Tutorial

gRPC Server Reflection provides information about publicly-accessible gRPC
services on a server, and assists clients at runtime to construct RPC requests
and responses without precompiled service information. It is used by
[gRPCurl](https://github.com/fullstorydev/grpcurl), which can be used to
introspect server protos and send/receive test RPCs.

## Enable Server Reflection

gRPC-go Server Reflection is implemented in package
[reflection](https://github.com/grpc/grpc-go/tree/master/reflection). To enable
server reflection, you need to import this package and register reflection
service on your gRPC server.

For example, to enable server reflection in `example/helloworld`, we need to
make the following changes:

```diff
--- a/examples/helloworld/greeter_server/main.go
+++ b/examples/helloworld/greeter_server/main.go
@@ -40,6 +40,7 @@ import (
        "google.golang.org/grpc"
        pb "google.golang.org/grpc/examples/helloworld/helloworld"
+       "google.golang.org/grpc/reflection"
 )

 const (
@@ -61,6 +62,8 @@ func main() {
        }
        s := grpc.NewServer()
        pb.RegisterGreeterService(s, &pb.GreeterService{SayHello: sayHello})
+       // Register reflection service on gRPC server.
+       reflection.Register(s)
        if err := s.Serve(lis); err != nil {
                log.Fatalf("failed to serve: %v", err)
        }
```

An example server with reflection registered can be found at
`examples/features/reflection/server`.

## gRPCurl

After enabling Server Reflection in a server application, you can use gRPCurl
to check its services. gRPCurl is built with Go and has packages available.
Instructions on how to install and use gRPCurl can be found at
[gRPCurl Installation](https://github.com/fullstorydev/grpcurl#installation).

## Use gRPCurl to check services

First, start the helloworld server in grpc-go directory:

```sh
$ cd <grpc-go-directory>/examples
$ go run features/reflection/server/main.go
```

output:
```sh
server listening at [::]:50051
```

After installing gRPCurl, open a new terminal and run the commands from the new
terminal.

**NOTE:** gRPCurl expects a TLS-encrypted connection by default. For all of
the commands below, use the `-plaintext` flag to use an unencrypted connection.

### List services and methods

The `list` command lists services exposed at a given port:

- List all the services exposed at a given port

  ```sh
  $ grpcurl -plaintext localhost:50051 list
  ```

  output:
  ```sh
  grpc.examples.echo.Echo
  grpc.reflection.v1alpha.ServerReflection
  helloworld.Greeter
  ```

- List all the methods of a service

  The `list` command lists methods given the full service name (in the format of
  \<package\>.\<service\>).

  ```sh
  $ grpcurl -plaintext localhost:50051 list helloworld.Greeter
  ```

  output:
  ```sh
  helloworld.Greeter.SayHello
  ```

### Describe services and methods

- Describe all services

  The `describe` command inspects a service given its full name (in the format
  of \<package\>.\<service\>). 

  ```sh
  $ grpcurl -plaintext localhost:50051 describe helloworld.Greeter
  ```

  output:
  ```sh
  helloworld.Greeter is a service:
  service Greeter {
    rpc SayHello ( .helloworld.HelloRequest ) returns ( .helloworld.HelloReply );
  }
  ```

- Describe all methods of a service

  The `describe` command inspects a method given its full name (in the format of
  \<package\>.\<service\>.\<method\>).

  ```sh
  $ grpcurl -plaintext localhost:50051 describe helloworld.Greeter.SayHello
  ```

  output:
  ```sh
  helloworld.Greeter.SayHello is a method:
  rpc SayHello ( .helloworld.HelloRequest ) returns ( .helloworld.HelloReply );
  ```

### Inspect message types

We can use the `describe` command to inspect request/response types given the
full name of the type (in the format of \<package\>.\<type\>).

- Get information about the request type

  ```sh
  $ grpcurl -plaintext localhost:50051 describe helloworld.HelloRequest
  ```

  output:
  ```sh
  helloworld.HelloRequest is a message:
  message HelloRequest {
    string name = 1;
  }
  ```

### Call a remote method

We can send RPCs to a server and get responses using the full method name (in
the format of \<package\>.\<service\>.\<method\>). The `-d <string>` flag
represents the request data and the `-format text` flag indicates that the
request data is in text format.

- Call a unary method

  ```sh
  $ grpcurl -plaintext -format text -d 'name: "gRPCurl"' \
    localhost:50051 helloworld.Greeter.SayHello
  ```

  output:
  ```sh
  message: "Hello gRPCurl"
  ```
