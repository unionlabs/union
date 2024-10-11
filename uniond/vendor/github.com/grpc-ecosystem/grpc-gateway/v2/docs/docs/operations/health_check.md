---
layout: default
title: Health check
nav_order: 1
parent: Operations
---

# Health check

## With the [gRPC Health Checking Protocol](https://github.com/grpc/grpc/blob/master/doc/health-checking.md)

To use the gRPC health checking protocol you must add the two health checking methods, `Watch` and `Check`.

## Registering the health server

1. Add `google.golang.org/grpc/health/grpc_health_v1` to your imports
2. Register the health server with `grpc_health_v1.RegisterHealthServer(grpcServer, yourService)`

## Adding the health check methods

1. Check method

```go
func (s *serviceServer) Check(ctx context.Context, in *health.HealthCheckRequest) (*health.HealthCheckResponse, error) {
	return &health.HealthCheckResponse{Status: health.HealthCheckResponse_SERVING}, nil
}
```

2. Watch method

```go
func (s *serviceServer) Watch(in *health.HealthCheckRequest, _ health.Health_WatchServer) error {
    // Example of how to register both methods but only implement the Check method.
	return status.Error(codes.Unimplemented, "unimplemented")
}
```

3. You can test the functionality with [GRPC health probe](https://github.com/grpc-ecosystem/grpc-health-probe).

## Adding `/healthz` endpoint to runtime.ServeMux

To automatically register a `/healthz` endpoint in your `ServeMux` you can use
the `ServeMuxOption` `WithHealthzEndpoint`
which takes in a connection to your registered gRPC server.

This endpoint will forward a request to the `Check` method described above to really check the health of the
whole system, not only the gateway itself. If your server doesn't implement the health checking protocol each request
to `/healthz` will result in the following:

```json
{"code":12,"message":"unknown service grpc.health.v1.Health","details":[]}
```

If you've implemented multiple services in your server you can target specific services with the `?service=<service>`
query parameter. This will then be added to the `health.HealthCheckRequest` in the `Service` property. With that you can
write your own logic to handle that in the health checking methods.

Analogously, to register an `{/endpoint/path}` endpoint in your `ServeMux` with a user-defined endpoint path, you can use
the `ServeMuxOption` `WithHealthEndpointAt`, which accepts a connection to your registered gRPC server
together with a custom `endpointPath string` parameter.
