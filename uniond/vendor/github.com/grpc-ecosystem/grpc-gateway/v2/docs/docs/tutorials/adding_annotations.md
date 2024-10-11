---
layout: default
title: Adding gRPC-Gateway annotations to an existing proto file
nav_order: 4
parent: Tutorials
---

# Adding gRPC-Gateway annotations to an existing proto file

Now that we've got a working Go gRPC server, we need to add the gRPC-Gateway annotations.

The annotations define how gRPC services map to the JSON request and response. When using protocol buffers, each RPC must define the HTTP method and path using the `google.api.http` annotation.

So we will need to add the `google/api/http.proto` import to the proto file. We also need to add the HTTP->gRPC mapping we want. In this case, we're mapping `POST /v1/example/echo` to our `SayHello` RPC.

```protobuf
syntax = "proto3";

package helloworld;

import "google/api/annotations.proto";

// Here is the overall greeting service definition where we define all our endpoints
service Greeter {
  // Sends a greeting
  rpc SayHello (HelloRequest) returns (HelloReply) {
    option (google.api.http) = {
      post: "/v1/example/echo"
      body: "*"
    };
  }
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

See [a_bit_of_everything.proto](https://github.com/grpc-ecosystem/grpc-gateway/blob/main/examples/internal/proto/examplepb/a_bit_of_everything.proto) for examples of more annotations you can add to customize gateway behavior.

## Generating the gRPC-Gateway stubs

Now that we've got the gRPC-Gateway annotations added to the proto file, we need to use the gRPC-Gateway generator to generate the stubs.

### Using buf

We'll need to add the gRPC-Gateway generator to the generation configuration:

```yaml
version: v1
plugins:
  - plugin: go
    out: proto
    opt: paths=source_relative
  - plugin: go-grpc
    out: proto
    opt: paths=source_relative,require_unimplemented_servers=false
  - plugin: grpc-gateway
    out: proto
    opt: paths=source_relative
```

We'll also need to add the `googleapis` dependency to our `buf.yaml` file:

```yaml
version: v1
name: buf.build/myuser/myrepo
deps:
  - buf.build/googleapis/googleapis
```

Then we need to run `buf mod update` to select a version of the dependency to use.

And that's it! Now if you run:

```sh
$ buf generate
```

It should produce a `*.gw.pb.go` file.

### Using `protoc`

Before we can generate the stubs with `protoc`, we need to copy some dependencies into our proto file structure. Copy a subset of the `googleapis`
from the [official repository](https://github.com/googleapis/googleapis) to your local proto file structure. It should look like this afterwards:

```
proto
├── google
│   └── api
│       ├── annotations.proto
│       └── http.proto
└── helloworld
    └── hello_world.proto
```

Now we need to add the gRPC-Gateway generator to the `protoc` invocation:

```sh
$ protoc -I ./proto \
  --go_out ./proto --go_opt paths=source_relative \
  --go-grpc_out ./proto --go-grpc_opt paths=source_relative \
  --grpc-gateway_out ./proto --grpc-gateway_opt paths=source_relative \
  ./proto/helloworld/hello_world.proto
```

This should generate a `*.gw.pb.go` file.

We also need to add and serve the gRPC-Gateway mux in our `main.go` file.

```go
package main

import (
	"context"
	"log"
	"net"
	"net/http"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	helloworldpb "github.com/myuser/myrepo/proto/helloworld"
)

type server struct{
	helloworldpb.UnimplementedGreeterServer
}

func NewServer() *server {
	return &server{}
}

func (s *server) SayHello(ctx context.Context, in *helloworldpb.HelloRequest) (*helloworldpb.HelloReply, error) {
	return &helloworldpb.HelloReply{Message: in.Name + " world"}, nil
}

func main() {
	// Create a listener on TCP port
	lis, err := net.Listen("tcp", ":8080")
	if err != nil {
		log.Fatalln("Failed to listen:", err)
	}

	// Create a gRPC server object
	s := grpc.NewServer()
	// Attach the Greeter service to the server
	helloworldpb.RegisterGreeterServer(s, &server{})
	// Serve gRPC server
	log.Println("Serving gRPC on 0.0.0.0:8080")
	go func() {
		log.Fatalln(s.Serve(lis))
	}()

	// Create a client connection to the gRPC server we just started
	// This is where the gRPC-Gateway proxies the requests
	conn, err := grpc.DialContext(
		context.Background(),
		"0.0.0.0:8080",
		grpc.WithBlock(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatalln("Failed to dial server:", err)
	}

	gwmux := runtime.NewServeMux()
	// Register Greeter
	err = helloworldpb.RegisterGreeterHandler(context.Background(), gwmux, conn)
	if err != nil {
		log.Fatalln("Failed to register gateway:", err)
	}

	gwServer := &http.Server{
		Addr:    ":8090",
		Handler: gwmux,
	}

	log.Println("Serving gRPC-Gateway on http://0.0.0.0:8090")
	log.Fatalln(gwServer.ListenAndServe())
}
```

For more examples, please refer to [our boilerplate repository](https://github.com/johanbrandhorst/grpc-gateway-boilerplate).

## Testing the gRPC-Gateway

Now we can start the server:

```sh
$ go run main.go
```

Then we use cURL to send HTTP requests:

```sh
$ curl -X POST -k http://localhost:8090/v1/example/echo -d '{"name": " hello"}'
```

```
{"message":"hello world"}
```

Hopefully, that gives a bit of understanding of how to use the gRPC-Gateway.

Full source code of hello world program can be found here [helloworld-grpc-gateway](https://github.com/iamrajiv/helloworld-grpc-gateway).

[Next](learn_more.md){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
