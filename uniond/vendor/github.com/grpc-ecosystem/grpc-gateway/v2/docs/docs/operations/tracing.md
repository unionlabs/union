---
layout: default
title: Tracing
nav_order: 2
parent: Operations
---

# Tracing

## With [OpenCensus.io](https://opencensus.io/) and [AWS X-ray](https://aws.amazon.com/xray/)

### Adding tracing using AWS-Xray as the exporter

This example uses the AWS-Xray exporter with a global trace setting. Note that AWS X-ray exporter does not handle any metrics only tracing.

1. Add the following imports

```go
xray "contrib.go.opencensus.io/exporter/aws"
"go.opencensus.io/plugin/ocgrpc"
"go.opencensus.io/plugin/ochttp"
"go.opencensus.io/trace"
```

2. Register the AWS X-ray exporter for the GRPC server

```go
xrayExporter, err := xray.NewExporter(
    xray.WithVersion("latest"),
    // Add your AWS region.
    xray.WithRegion("ap-southeast-1"),
)
if err != nil {
    // Handle any error.
}
// Do not forget to call Flush() before the application terminates.
defer xrayExporter.Flush()

// Register the trace exporter.
trace.RegisterExporter(xrayExporter)
```

3. Add a global tracing configuration

```go
// Always trace in this example.
// In production this can be set to a trace.ProbabilitySampler.
trace.ApplyConfig(trace.Config{DefaultSampler: trace.AlwaysSample()})
```

4. Add `ocgrpc.ClientHandler` for tracing the gRPC client calls

```go
// Example using DialContext
conn, err := grpc.DialContext(
    // Other options goes here.
    // Add ocgrpc.ClientHandler for tracing the grpc client calls.
    grpc.WithStatsHandler(&ocgrpc.ClientHandler{}),
)
```

5. Wrap the gateway mux with the OpenCensus HTTP handler

```go
gwmux := runtime.NewServeMux()

openCensusHandler := &ochttp.Handler{
		Handler: gwmux,
}

gwServer := &http.Server{
    Addr: "0.0.0.0:10000",
    Handler: openCensusHandler,
    }),
}
```

### Without a global configuration

In this example we have added the [gRPC Health Checking Protocol](https://github.com/grpc/grpc/blob/master/doc/health-checking.md) and we do not wish to trace any health checks.

1. Follow step `1`, `2` and `4` from the previous section.

2. Since we are not using a global configuration we can decide what paths we want to trace.

```go
gwmux := runtime.NewServeMux()

openCensusHandler := &ochttp.Handler{
    Handler: gwmux,
    GetStartOptions: func(r *http.Request) trace.StartOptions {
        startOptions := trace.StartOptions{}
        if strings.HasPrefix(r.URL.Path, "/api") {
            // This example will always trace anything starting with /api.
            startOptions.Sampler = trace.AlwaysSample()
        }
        return startOptions
    },
}
```

4. No global configuration means we have to use the [per span sampler](https://opencensus.io/tracing/sampling/#per-span-sampler).

#### A method we want to trace

```go
func (s *service) Name(ctx context.Context, req *pb.Request) (*pb.Response, error) {
    // Here we add the span ourselves.
    ctx, span := trace.StartSpan(ctx, "name.to.use.in.trace", trace.
    // Select a sampler that fits your implementation.
    WithSampler(trace.AlwaysSample()))
    defer span.End()
    /// Other stuff goes here.
}
```

#### A method we do not wish to trace

```go
func (s *service) Check(ctx context.Context, in *health.HealthCheckRequest) (*health.HealthCheckResponse, error) {
    // Note no span here.
    return &health.HealthCheckResponse{Status: health.HealthCheckResponse_SERVING}, nil
}
```

## OpenTracing Support

If your project uses [OpenTracing](https://github.com/opentracing/opentracing-go) and you'd like spans to propagate through the gateway, you can add some middleware which parses the incoming HTTP headers to create a new span correctly.

```go
import (
	"github.com/opentracing/opentracing-go"
	"github.com/opentracing/opentracing-go/ext"
)

var grpcGatewayTag = opentracing.Tag{Key: string(ext.Component), Value: "grpc-gateway"}

func tracingWrapper(h http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		parentSpanContext, err := opentracing.GlobalTracer().Extract(
			opentracing.HTTPHeaders,
			opentracing.HTTPHeadersCarrier(r.Header))
		if err == nil || err == opentracing.ErrSpanContextNotFound {
			serverSpan := opentracing.GlobalTracer().StartSpan(
				"ServeHTTP",
				// this is magical, it attaches the new span to the parent parentSpanContext, and creates an unparented one if empty.
				ext.RPCServerOption(parentSpanContext),
				grpcGatewayTag,
			)
			r = r.WithContext(opentracing.ContextWithSpan(r.Context(), serverSpan))
			defer serverSpan.Finish()
		}
		h.ServeHTTP(w, r)
	})
}

// Then just wrap the mux returned by runtime.NewServeMux() like this
if err := http.ListenAndServe(":8080", tracingWrapper(mux)); err != nil {
	log.Fatalf("failed to start gateway server on 8080: %v", err)
}
```

Finally, don't forget to add a tracing interceptor when registering
the services. E.g.

```go
import (
	"google.golang.org/grpc"
	"github.com/grpc-ecosystem/go-grpc-middleware/tracing/opentracing"
)

opts := []grpc.DialOption{
	grpc.WithUnaryInterceptor(
		grpc_opentracing.UnaryClientInterceptor(
			grpc_opentracing.WithTracer(opentracing.GlobalTracer()),
		),
	),
}
if err := pb.RegisterMyServiceHandlerFromEndpoint(ctx, mux, serviceEndpoint, opts); err != nil {
	log.Fatalf("could not register HTTP service: %v", err)
}
```
