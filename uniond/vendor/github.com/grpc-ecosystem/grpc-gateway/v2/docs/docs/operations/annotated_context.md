---
layout: default
title: Extracting the HTTP path pattern for a request
nav_order: 4
parent: Operations
---

# Extracting the HTTP path pattern for a request

It is often interesting to know what [HTTP path pattern](https://github.com/googleapis/googleapis/blob/869d32e2f0af2748ab530646053b23a2b80d9ca5/google/api/http.proto#L61-L87) was matched for a specific request, for example for metrics. This article explains how to extract the HTTP path pattern from the request context.

## Get HTTP Path pattern
1. Define the HTTP path in the proto annotation. For example:

```proto
syntax = "proto3";
option go_package = "github.com/grpc-ecosystem/grpc-gateway/v2/examples/internal/proto/examplepb";
package grpc.gateway.examples.internal.proto.examplepb;

import "google/api/annotations.proto";

service LoginService {
  rpc Login (LoginRequest) returns (LoginReply) {
    option (google.api.http) = {
        post: "/v1/example/login"
        body: "*"
    };
  }
}

message LoginRequest {}

message LoginReply {}
```

2. At runtime, get the HTTP path pattern from the annotated context, for example using the `WithMetadata` function.
You can pass data to your backend by adding them to the gRPC metadata or push them to a metrics server.

```go
mux := runtime.NewServeMux(
	runtime.WithMetadata(func(ctx context.Context, r *http.Request) metadata.MD {
		md := make(map[string]string)
		if method, ok := runtime.RPCMethod(ctx); ok {
			md["method"] = method // /grpc.gateway.examples.internal.proto.examplepb.LoginService/Login
		}
		if pattern, ok := runtime.HTTPPathPattern(ctx); ok {
			md["pattern"] = pattern // /v1/example/login
		}
		return metadata.New(md)
	}),
)
```